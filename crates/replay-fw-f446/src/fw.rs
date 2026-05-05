// SAFETY POLICY:
// This crate is embedded-only (cfg arm/none).
// Unsafe usage is limited to:
// - NVIC unmask
// - PAC register .bits() writes
// - USART DR write
// These are required by current PAC APIs and are quarantined to this firmware crate.

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use cortex_m::interrupt::Mutex;
use panic_halt as _;
use replay_core::artifact::{
    encode_header1_le, EventFrame0, Header1, EVENTFRAME0_SIZE, FRAME_SIZE, MAGIC,
    V1_MIN_HEADER_SIZE, VERSION1,
};
use stm32f4::stm32f446::{self as pac};

const FRAME_COUNT: usize = 10_000;
const IRQ_ID_TIM2: u8 = 0x02;
const TIMER_DELTA_NOMINAL: u32 = 1_000;
const STEP: u32 = 0x0100_0000;
const CAPTURE_BOUNDARY_ISR: u16 = 0;
use crate::artifact_metadata::{BUILD_HASH, CONFIG_HASH, RPL0_SCHEMA, SCHEMA_HASH};

const BOARD_ID: [u8; 16] = *b"NUCLEO-F446RE\0\0\0";
const CLOCK_PROFILE: [u8; 16] = *b"reset-16mhz-apb1";
#[cfg(feature = "demo-divergence")]
const DEMO_DIVERGENCE_FRAME: usize = 4_096;
#[cfg(feature = "demo-persistent-divergence")]
const DEMO_PERSISTENT_DIVERGENCE_FRAME: usize = 4_096;

#[cfg(all(feature = "demo-divergence", feature = "demo-persistent-divergence"))]
compile_error!("demo-divergence and demo-persistent-divergence are mutually exclusive");

static CAPTURE_DONE: AtomicBool = AtomicBool::new(false);
static WRITE_IDX: AtomicU32 = AtomicU32::new(0);
static SIGNAL_PHASE: AtomicU32 = AtomicU32::new(0);
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

    init_gpioa_for_usart2_tx(&dp);
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
    }

    while !CAPTURE_DONE.load(Ordering::Acquire) {
        cortex_m::asm::wfi();
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

    #[cfg(feature = "demo-persistent-divergence")]
    let phase = {
        let mut p = SIGNAL_PHASE.load(Ordering::Relaxed);
        if idx == DEMO_PERSISTENT_DIVERGENCE_FRAME {
            // One-time state perturbation: shift phase trajectory by one output step.
            p = p.wrapping_add(STEP);
        }
        p
    };
    #[cfg(not(feature = "demo-persistent-divergence"))]
    let phase = SIGNAL_PHASE.load(Ordering::Relaxed);

    #[cfg(not(feature = "demo-divergence"))]
    let sample = (phase >> 24) as i32;
    #[cfg(feature = "demo-divergence")]
    let sample = {
        let mut s = (phase >> 24) as i32;
        if idx == DEMO_DIVERGENCE_FRAME {
            s = s.wrapping_add(1);
        }
        s
    };

    let next = phase.wrapping_add(STEP);
    SIGNAL_PHASE.store(next, Ordering::Relaxed);
    cortex_m::interrupt::free(|cs| {
        SAMPLES.borrow(cs).borrow_mut()[idx] = sample;
    });

    let next = idx + 1;
    WRITE_IDX.store(next as u32, Ordering::Release);
    if next >= FRAME_COUNT {
        CAPTURE_DONE.store(true, Ordering::Release);
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

    // Reset clocks: APB1 = 16 MHz, oversampling by 16.
    // USARTDIV = 16_000_000 / (16 * 115200) = 8.6805
    // Mantissa = 8, Fraction = 11.
    dp.USART2.cr1().modify(|_, w| w.ue().clear_bit());
    dp.USART2
        .brr()
        .write(|w| unsafe { w.div_mantissa().bits(8).div_fraction().bits(11) });
    dp.USART2.cr2().reset();
    dp.USART2.cr3().reset();
    dp.USART2
        .cr1()
        .modify(|_, w| w.te().set_bit().re().clear_bit().ue().set_bit());
}

fn init_tim2_1khz() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            // Timer clock assumed 16 MHz (reset clocks):
            // update_hz = 16_000_000 / (PSC + 1) / (ARR + 1)
            // PSC=15 and ARR=999 => 1,000 Hz update interrupt.
            // Force a known base state: disabled, continuous mode.
            tim2.cr1()
                .modify(|_, w| w.cen().clear_bit().opm().clear_bit());
            tim2.psc().write(|w| unsafe { w.psc().bits(15) });
            tim2.arr().write(|w| unsafe { w.arr().bits(999) });
            tim2.egr().write(|w| w.ug().set_bit());
            tim2.sr().modify(|_, w| w.uif().clear_bit());
            tim2.dier().modify(|_, w| w.uie().set_bit());
            // Start last.
            tim2.cr1()
                .modify(|_, w| w.opm().clear_bit().cen().set_bit());
        }
    });
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
    debug_assert_eq!(EVENTFRAME0_SIZE, 16);
    write_bytes(usart2, &frame.frame_idx.to_le_bytes());
    write_bytes(usart2, &[frame.irq_id]);
    write_bytes(usart2, &[frame.flags]);
    write_bytes(usart2, &frame.rsv.to_le_bytes());
    write_bytes(usart2, &frame.timer_delta.to_le_bytes());
    write_bytes(usart2, &frame.input_sample.to_le_bytes());
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
