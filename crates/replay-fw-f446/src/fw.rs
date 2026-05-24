// SAFETY POLICY:
// This crate is embedded-only (cfg arm/none).
// Unsafe usage is limited to:
// - NVIC unmask
// - direct PAC peripheral pointer access in the latency ISR and trigger pulse helper
// - PAC register .bits() writes
// - USART DR write
// These are required by current PAC APIs and are quarantined to this firmware crate.

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use cortex_m::interrupt::Mutex;
use panic_halt as _;
use replay_core::artifact::{
    encode_event_frame0_le, encode_header1_le, EventFrame0, Header1, FRAME_SIZE, MAGIC,
    V1_MIN_HEADER_SIZE, VERSION1,
};
use stm32f4::stm32f446::{self as pac};

use crate::artifact_metadata::{BUILD_HASH, CONFIG_HASH, RPL0_SCHEMA, SCHEMA_HASH};
#[cfg(feature = "demo-persistent-divergence")]
use crate::signal_model::persistent_divergence_state;
use crate::signal_model::{
    advance_state_for_model, sample_for_model, SELECTED_SIGNAL_MODEL, SIGNAL_INITIAL_STATE,
};

const FRAME_COUNT: usize = 10_000;
const IRQ_ID_TIM2: u8 = 0x02;
const TIMER_DELTA_NOMINAL: u32 = 1_000;
const CAPTURE_BOUNDARY_ISR: u16 = 0;
#[cfg(not(any(feature = "sync_trigger_out", feature = "sync_trigger_in")))]
const DEFAULT_APB1_HZ: u32 = 16_000_000;
#[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
const SYNC_APB1_HZ: u32 = 45_000_000;
#[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
const SYNC_TIM2_HZ: u32 = 90_000_000;

const BOARD_ID: [u8; 16] = *b"NUCLEO-F446RE\0\0\0";
#[cfg(not(any(feature = "sync_trigger_out", feature = "sync_trigger_in")))]
const CLOCK_PROFILE: [u8; 16] = *b"reset-16mhz-apb1";
#[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
const CLOCK_PROFILE: [u8; 16] = *b"hse-pll-180mhz\0\0";
#[cfg(feature = "demo-divergence")]
const DEMO_DIVERGENCE_FRAME: usize = 4_096;
#[cfg(feature = "demo-persistent-divergence")]
const DEMO_PERSISTENT_DIVERGENCE_FRAME: usize = 4_096;

#[cfg(all(feature = "demo-divergence", feature = "demo-persistent-divergence"))]
compile_error!("demo-divergence and demo-persistent-divergence are mutually exclusive");

static CAPTURE_DONE: AtomicBool = AtomicBool::new(false);
static WRITE_IDX: AtomicU32 = AtomicU32::new(0);
static SIGNAL_STATE: AtomicU32 = AtomicU32::new(SIGNAL_INITIAL_STATE);
#[cfg(feature = "debug-irq-count")]
#[used]
#[no_mangle]
#[link_section = ".bss.irq_probe"]
pub static mut IRQ_COUNT_PROBE: u32 = 0;
static SAMPLES: Mutex<RefCell<[i32; FRAME_COUNT]>> = Mutex::new(RefCell::new([0; FRAME_COUNT]));
static TIM2_DEV: Mutex<RefCell<Option<pac::TIM2>>> = Mutex::new(RefCell::new(None));
static USART2_DEV: Mutex<RefCell<Option<pac::USART2>>> = Mutex::new(RefCell::new(None));

pub fn fw_main() -> ! {
    let dp = loop {
        if let Some(p) = pac::Peripherals::take() {
            break p;
        }
    };

    let _cp = loop {
        if let Some(p) = cortex_m::Peripherals::take() {
            break p;
        }
    };

    #[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
    init_hse_pll_180mhz_or_fault(&dp);

    init_gpioa_for_usart2_tx(&dp);
    #[cfg(feature = "sync_trigger_in")]
    init_trigger_boundary(&dp);
    #[cfg(feature = "sync_trigger_out")]
    init_sync_trigger_output(&dp);
    init_usart2(&dp);
    dp.RCC.apb1enr().modify(|_, w| w.tim2en().set_bit());

    cortex_m::interrupt::free(|cs| {
        TIM2_DEV.borrow(cs).replace(Some(dp.TIM2));
        USART2_DEV.borrow(cs).replace(Some(dp.USART2));
    });

    init_tim2_1khz();

    // Enable TIM2 IRQ at NVIC. IRQs are globally enabled after reset.
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
        #[cfg(feature = "sync_trigger_in")]
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI0);
    }

    #[cfg(feature = "sync_trigger_out")]
    let mut sync_trigger_out_div: u32 = 0;

    while !CAPTURE_DONE.load(Ordering::Acquire) {
        cortex_m::asm::wfi();

        #[cfg(feature = "sync_trigger_out")]
        {
            sync_trigger_out_div = sync_trigger_out_div.wrapping_add(1);
            if sync_trigger_out_div >= 100 {
                sync_trigger_out_div = 0;
                pulse_sync_trigger_output();
            }
        }
    }

    // Halt capture and enter dump-only phase.
    cortex_m::interrupt::disable();
    cortex_m::peripheral::NVIC::mask(pac::Interrupt::TIM2);
    stop_tim2();

    #[cfg(feature = "debug-repeat-dump")]
    loop {
        dump_artifact();
    }

    #[cfg(not(feature = "debug-repeat-dump"))]
    {
        dump_artifact();
        loop {
            cortex_m::asm::wfi();
        }
    }
}

pub fn tim2_isr() {
    #[cfg(feature = "debug-irq-count")]
    unsafe {
        let p = core::ptr::addr_of_mut!(IRQ_COUNT_PROBE);
        let v = core::ptr::read_volatile(p);
        core::ptr::write_volatile(p, v.wrapping_add(1));
    }

    if CAPTURE_DONE.load(Ordering::Acquire) {
        clear_tim2_update_flag();
        return;
    }

    clear_tim2_update_flag();

    let idx = WRITE_IDX.load(Ordering::Relaxed) as usize;
    if idx >= FRAME_COUNT {
        CAPTURE_DONE.store(true, Ordering::Release);
        return;
    }

    let state = SIGNAL_STATE.load(Ordering::Relaxed);
    #[cfg(feature = "demo-persistent-divergence")]
    let state = {
        let mut s = state;
        if idx == DEMO_PERSISTENT_DIVERGENCE_FRAME {
            // One-time state perturbation: shift the selected model trajectory.
            s = persistent_divergence_state(SELECTED_SIGNAL_MODEL, s)
                .expect("demo-persistent-divergence is unsupported for selected signal model");
        }
        s
    };

    #[cfg(not(feature = "demo-divergence"))]
    let sample = sample_for_model(SELECTED_SIGNAL_MODEL, idx as u32, state);
    #[cfg(feature = "demo-divergence")]
    let sample = {
        let mut s = sample_for_model(SELECTED_SIGNAL_MODEL, idx as u32, state);
        if idx == DEMO_DIVERGENCE_FRAME {
            s = s.wrapping_add(1);
        }
        s
    };

    let next_state = advance_state_for_model(SELECTED_SIGNAL_MODEL, state);
    SIGNAL_STATE.store(next_state, Ordering::Relaxed);
    cortex_m::interrupt::free(|cs| {
        SAMPLES.borrow(cs).borrow_mut()[idx] = sample;
    });

    let next = idx + 1;
    WRITE_IDX.store(next as u32, Ordering::Release);
    if next >= FRAME_COUNT {
        CAPTURE_DONE.store(true, Ordering::Release);
    }
}

#[cfg(feature = "sync_trigger_in")]
pub fn exti0_isr() {
    unsafe {
        let gpioa = &*pac::GPIOA::ptr();
        let exti = &*pac::EXTI::ptr();

        gpioa.bsrr().write(|w| w.bs1().set_bit());
        // EXTI PR is write-one-to-clear; bit 0 clears pending EXTI0.
        exti.pr().write(|w| w.bits(1));
        gpioa.bsrr().write(|w| w.br1().set_bit());
    }
}

#[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
fn init_hse_pll_180mhz_or_fault(dp: &pac::Peripherals) {
    dp.RCC.apb1enr().modify(|_, w| w.pwren().set_bit());
    dp.PWR.cr().modify(|_, w| w.vos().scale1());

    dp.FLASH.acr().modify(|_, w| {
        w.icen().set_bit();
        w.dcen().set_bit();
        w.prften().set_bit();
        w.latency().ws5()
    });

    dp.RCC.cr().modify(|_, w| w.hsebyp().set_bit());
    dp.RCC.cr().modify(|_, w| w.hseon().set_bit());
    wait_until_or_clock_fault(|| dp.RCC.cr().read().hserdy().bit_is_set());

    dp.RCC.pllcfgr().write(|w| unsafe {
        w.pllm().bits(4);
        w.plln().bits(180);
        w.pllp().div2();
        w.pllsrc().hse();
        w.pllq().bits(7)
    });

    dp.PWR.cr().modify(|_, w| w.oden().set_bit());
    wait_until_or_clock_fault(|| dp.PWR.csr().read().odrdy().bit_is_set());
    dp.PWR.cr().modify(|_, w| w.odswen().set_bit());
    wait_until_or_clock_fault(|| dp.PWR.csr().read().odswrdy().bit_is_set());

    dp.RCC.cfgr().modify(|_, w| {
        w.hpre().div1();
        w.ppre1().div4();
        w.ppre2().div2()
    });

    dp.RCC.cr().modify(|_, w| w.pllon().set_bit());
    wait_until_or_clock_fault(|| dp.RCC.cr().read().pllrdy().bit_is_set());

    dp.RCC.cfgr().modify(|_, w| w.sw().pll());
    wait_until_or_clock_fault(|| dp.RCC.cfgr().read().sws().is_pll());

    if !dp.RCC.cfgr().read().sws().is_pll() {
        clock_fault_loop();
    }
}

#[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
fn wait_until_or_clock_fault(mut ready: impl FnMut() -> bool) {
    for _ in 0..2_000_000 {
        if ready() {
            return;
        }
        cortex_m::asm::nop();
    }
    clock_fault_loop();
}

#[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
fn clock_fault_loop() -> ! {
    // In sync trigger builds, repeating PA1 blink means clock initialization fault
    // before HIL operation; with sync_trigger_in, a short PA1 pulse from
    // exti0_isr() means EXTI acknowledgment during validation.
    unsafe {
        let rcc = &*pac::RCC::ptr();
        let gpioa = &*pac::GPIOA::ptr();

        rcc.ahb1enr().modify(|_, w| w.gpioaen().set_bit());
        gpioa.moder().modify(|_, w| w.moder1().output());
        gpioa.otyper().modify(|_, w| w.ot1().clear_bit());
        gpioa
            .ospeedr()
            .modify(|_, w| w.ospeedr1().very_high_speed());
        gpioa.pupdr().modify(|_, w| w.pupdr1().floating());

        loop {
            gpioa.bsrr().write(|w| w.bs1().set_bit());
            cortex_m::asm::delay(180_000);
            gpioa.bsrr().write(|w| w.br1().set_bit());
            cortex_m::asm::delay(180_000);
        }
    }
}

#[cfg(feature = "sync_trigger_in")]
fn init_trigger_boundary(dp: &pac::Peripherals) {
    dp.RCC.ahb1enr().modify(|_, w| w.gpioaen().set_bit());
    dp.RCC.apb2enr().modify(|_, w| w.syscfgen().set_bit());

    dp.GPIOA.moder().modify(|_, w| {
        w.moder0().input();
        w.moder1().output()
    });
    dp.GPIOA.otyper().modify(|_, w| w.ot1().clear_bit());
    dp.GPIOA
        .ospeedr()
        .modify(|_, w| w.ospeedr1().very_high_speed());
    dp.GPIOA.pupdr().modify(|_, w| {
        w.pupdr0().floating();
        w.pupdr1().floating()
    });
    dp.GPIOA.bsrr().write(|w| w.br1().set_bit());

    dp.SYSCFG.exticr1().modify(|_, w| w.exti0().pa());
    dp.EXTI.imr().modify(|_, w| w.mr0().clear_bit());
    dp.EXTI.rtsr().modify(|_, w| w.tr0().set_bit());
    dp.EXTI.ftsr().modify(|_, w| w.tr0().clear_bit());
    // EXTI PR is write-one-to-clear; bit 0 clears pending EXTI0.
    dp.EXTI.pr().write(|w| unsafe { w.bits(1) });
    dp.EXTI.imr().modify(|_, w| w.mr0().set_bit());
}

#[cfg(feature = "sync_trigger_out")]
fn init_sync_trigger_output(dp: &pac::Peripherals) {
    dp.RCC.ahb1enr().modify(|_, w| w.gpioaen().set_bit());
    dp.GPIOA.bsrr().write(|w| w.br6().set_bit());
    dp.GPIOA.moder().modify(|_, w| w.moder6().output());
    dp.GPIOA.otyper().modify(|_, w| w.ot6().clear_bit());
    dp.GPIOA
        .ospeedr()
        .modify(|_, w| w.ospeedr6().very_high_speed());
    dp.GPIOA.pupdr().modify(|_, w| w.pupdr6().floating());
    dp.GPIOA.bsrr().write(|w| w.br6().set_bit());
}

#[cfg(feature = "sync_trigger_out")]
fn pulse_sync_trigger_output() {
    unsafe {
        let gpioa = &*pac::GPIOA::ptr();

        gpioa.bsrr().write(|w| w.bs6().set_bit());
        cortex_m::asm::delay(180);
        gpioa.bsrr().write(|w| w.br6().set_bit());
    }
}

fn init_gpioa_for_usart2_tx(dp: &pac::Peripherals) {
    dp.RCC.ahb1enr().modify(|_, w| w.gpioaen().set_bit());

    // PA2 -> USART2_TX (AF7)
    dp.GPIOA.moder().modify(|_, w| w.moder2().alternate());
    dp.GPIOA.afrl().modify(|_, w| w.afrl2().af7());
    dp.GPIOA
        .ospeedr()
        .modify(|_, w| w.ospeedr2().very_high_speed());
    dp.GPIOA.otyper().modify(|_, w| w.ot2().clear_bit());
    dp.GPIOA.pupdr().modify(|_, w| w.pupdr2().floating());
}

fn init_usart2(dp: &pac::Peripherals) {
    dp.RCC.apb1enr().modify(|_, w| w.usart2en().set_bit());

    let apb1_hz = usart2_apb1_hz();
    let usartdiv_x16 = (apb1_hz + 57_600) / 115_200;
    let mantissa = (usartdiv_x16 / 16) as u16;
    let fraction = (usartdiv_x16 % 16) as u8;

    dp.USART2.cr1().modify(|_, w| w.ue().clear_bit());
    dp.USART2.brr().write(|w| unsafe {
        w.div_mantissa()
            .bits(mantissa)
            .div_fraction()
            .bits(fraction)
    });
    dp.USART2.cr2().reset();
    dp.USART2.cr3().reset();
    dp.USART2
        .cr1()
        .modify(|_, w| w.te().set_bit().re().clear_bit().ue().set_bit());
}

fn init_tim2_1khz() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            let psc = (tim2_clock_hz() / 1_000_000) - 1;
            let arr = 999;

            // Force a known base state: disabled, continuous mode.
            tim2.cr1()
                .modify(|_, w| w.cen().clear_bit().opm().clear_bit());
            tim2.psc().write(|w| unsafe { w.psc().bits(psc as u16) });
            tim2.arr().write(|w| unsafe { w.arr().bits(arr) });
            tim2.egr().write(|w| w.ug().set_bit());
            tim2.sr().modify(|_, w| w.uif().clear_bit());
            tim2.dier().modify(|_, w| w.uie().set_bit());
            // Start last.
            tim2.cr1()
                .modify(|_, w| w.opm().clear_bit().cen().set_bit());
        }
    });
}

fn usart2_apb1_hz() -> u32 {
    #[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
    {
        SYNC_APB1_HZ
    }
    #[cfg(not(any(feature = "sync_trigger_out", feature = "sync_trigger_in")))]
    {
        DEFAULT_APB1_HZ
    }
}

fn tim2_clock_hz() -> u32 {
    #[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
    {
        SYNC_TIM2_HZ
    }
    #[cfg(not(any(feature = "sync_trigger_out", feature = "sync_trigger_in")))]
    {
        DEFAULT_APB1_HZ
    }
}

fn stop_tim2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.dier().modify(|_, w| w.uie().clear_bit());
            tim2.cr1().modify(|_, w| w.cen().clear_bit());
            tim2.sr().modify(|_, w| w.uif().clear_bit());
        }
    });
}

fn clear_tim2_update_flag() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.sr().write(|w| w.uif().clear_bit());
        }
    });
}

fn dump_artifact() {
    cortex_m::interrupt::free(|cs| {
        if let Some(usart2) = USART2_DEV.borrow(cs).borrow().as_ref() {
            let header = Header1 {
                magic: MAGIC,
                version: VERSION1,
                header_len: V1_MIN_HEADER_SIZE as u16,
                frame_count: FRAME_COUNT as u32,
                frame_size: FRAME_SIZE as u16,
                flags: 0,
                schema_len: RPL0_SCHEMA.len() as u32,
                schema_hash: SCHEMA_HASH,
                build_hash: BUILD_HASH,
                config_hash: CONFIG_HASH,
                board_id: BOARD_ID,
                clock_profile: CLOCK_PROFILE,
                capture_boundary: CAPTURE_BOUNDARY_ISR,
                reserved: 0,
            };

            write_header1(usart2, &header);
            write_bytes(usart2, RPL0_SCHEMA);

            let samples = SAMPLES.borrow(cs).borrow();
            for (idx, sample) in samples.iter().copied().enumerate() {
                let frame = EventFrame0 {
                    frame_idx: idx as u32,
                    irq_id: IRQ_ID_TIM2,
                    flags: 0,
                    rsv: 0,
                    timer_delta: TIMER_DELTA_NOMINAL,
                    input_sample: sample,
                };
                write_event_frame0(usart2, &frame);
            }

            wait_tc(usart2);
        }
    });
}

fn write_header1(usart2: &pac::USART2, header: &Header1) {
    let bytes = encode_header1_le(header);
    debug_assert_eq!(bytes.len(), V1_MIN_HEADER_SIZE);
    write_bytes(usart2, &bytes);
}

fn write_event_frame0(usart2: &pac::USART2, frame: &EventFrame0) {
    write_bytes(usart2, &encode_event_frame0_le(frame));
}

fn write_bytes(usart2: &pac::USART2, bytes: &[u8]) {
    for byte in bytes {
        write_u8(usart2, *byte);
    }
}

fn write_u8(usart2: &pac::USART2, byte: u8) {
    while usart2.sr().read().txe().bit_is_clear() {}
    usart2
        .dr()
        .write(|w| unsafe { w.dr().bits(u16::from(byte)) });
}

fn wait_tc(usart2: &pac::USART2) {
    while usart2.sr().read().tc().bit_is_clear() {}
}
