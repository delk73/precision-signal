// SAFETY POLICY:
// This crate is embedded-only (cfg arm/none).
// Unsafe usage is limited to:
// - NVIC unmask
// - direct PAC peripheral pointer access in the latency ISR and trigger pulse helper
// - direct volatile peripheral writes in the timing-capture latency ISR
// - PAC register .bits() writes
// - USART DR write
// These are required by current PAC APIs and are quarantined to this firmware crate.

use core::cell::RefCell;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use cortex_m::interrupt::Mutex;
use panic_halt as _;
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
use replay_core::artifact::{
    encode_event_frame0_le, encode_header1_le, EventFrame0, Header1, FRAME_SIZE, MAGIC,
    V1_MIN_HEADER_SIZE, VERSION1,
};
use stm32f4::stm32f446::{self as pac};

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
use crate::artifact_metadata::{BUILD_HASH, CONFIG_HASH, RPL0_SCHEMA, SCHEMA_HASH};
#[cfg(all(
    feature = "demo-persistent-divergence",
    not(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))
))]
use crate::signal_model::persistent_divergence_state;
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
use crate::signal_model::{
    advance_state_for_model, sample_for_model, SELECTED_SIGNAL_MODEL, SIGNAL_INITIAL_STATE,
};

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
const FRAME_COUNT: usize = 10_000;
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
const IRQ_ID_TIM2: u8 = 0x02;
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
const TIMER_DELTA_NOMINAL: u32 = 1_000;
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
const CAPTURE_BOUNDARY_ISR: u16 = 0;
#[cfg(not(any(
    feature = "sync_trigger_out",
    feature = "sync_trigger_in",
    feature = "sync_timing_observer"
)))]
const DEFAULT_APB1_HZ: u32 = 16_000_000;
#[cfg(any(
    feature = "sync_trigger_out",
    feature = "sync_trigger_in",
    feature = "sync_timing_observer"
))]
const SYNC_APB1_HZ: u32 = 45_000_000;
#[cfg(any(feature = "sync_trigger_out", feature = "sync_trigger_in"))]
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
const SYNC_TIM2_HZ: u32 = 90_000_000;

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
const BOARD_ID: [u8; 16] = *b"NUCLEO-F446RE\0\0\0";
#[cfg(not(any(
    feature = "sync_trigger_out",
    feature = "sync_trigger_in",
    feature = "sync_timing_observer"
)))]
const CLOCK_PROFILE: [u8; 16] = *b"reset-16mhz-apb1";
#[cfg(all(
    not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")),
    any(
        feature = "sync_trigger_out",
        feature = "sync_trigger_in",
        feature = "sync_timing_observer"
    )
))]
const CLOCK_PROFILE: [u8; 16] = *b"hse-pll-180mhz\0\0";
#[cfg(feature = "demo-divergence")]
const DEMO_DIVERGENCE_FRAME: usize = 4_096;
#[cfg(feature = "demo-persistent-divergence")]
const DEMO_PERSISTENT_DIVERGENCE_FRAME: usize = 4_096;

#[cfg(all(feature = "demo-divergence", feature = "demo-persistent-divergence"))]
compile_error!("demo-divergence and demo-persistent-divergence are mutually exclusive");
#[cfg(all(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
compile_error!("sync_timing_capture and sync_timing_observer are mutually exclusive");
#[cfg(all(
    feature = "sync_timing_capture",
    not(feature = "sync_timing_observer"),
    not(all(feature = "sync_trigger_out", feature = "sync_trigger_in"))
))]
compile_error!("sync_timing_capture requires sync_trigger_out and sync_trigger_in");
#[cfg(all(
    feature = "sync_timing_observer",
    any(feature = "sync_trigger_out", feature = "sync_trigger_in")
))]
compile_error!(
    "sync_timing_observer must not be combined with sync_trigger_out or sync_trigger_in"
);

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const SYNC_TIMING_TRIGGER_TARGET: u32 = 10_000;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const SYNC_TIMING_EVIDENCE_WINDOW_START_TRIGGER: u32 = 8;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const SYNC_TIMING_TIMER_HZ: u32 = 90_000_000;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const SYNC_TIMING_THRESHOLD_TICKS: u32 = 9;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const SYNC_TIMING_ACK_GRACE_POLLS: u32 = 10_000;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const SYNC_TIMING_ACK_GRACE_DELAY_CYCLES: u32 = 180;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const TIM_SR_CC3IF: u32 = 1 << 3;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const TIM_SR_CC4IF: u32 = 1 << 4;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const TIM_SR_CC3OF: u32 = 1 << 11;
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
const TIM_SR_CC4OF: u32 = 1 << 12;
#[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
const SYNC_TIM2_ACK_PULSE_TICKS: u32 = 8;
#[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
const SYNC_TIM2_ACK_ARR: u32 = 0xffff;
#[cfg(all(
    feature = "sync_trigger_in",
    feature = "sync_timing_capture",
    not(feature = "sync_timing_observer")
))]
const GPIOA_BSRR_ADDR: usize = 0x4002_0018;
#[cfg(all(
    feature = "sync_trigger_in",
    feature = "sync_timing_capture",
    not(feature = "sync_timing_observer")
))]
const EXTI_PR_ADDR: usize = 0x4001_3c14;
#[cfg(all(
    feature = "sync_trigger_in",
    feature = "sync_timing_capture",
    not(feature = "sync_timing_observer")
))]
const GPIO_BSRR_BS1: u32 = 1 << 1;
#[cfg(all(
    feature = "sync_trigger_in",
    feature = "sync_timing_capture",
    not(feature = "sync_timing_observer")
))]
const GPIO_BSRR_BR1: u32 = 1 << 17;
#[cfg(all(
    feature = "sync_trigger_in",
    feature = "sync_timing_capture",
    not(feature = "sync_timing_observer")
))]
const EXTI_PR_PR0: u32 = 1;

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
static CAPTURE_DONE: AtomicBool = AtomicBool::new(false);
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
static WRITE_IDX: AtomicU32 = AtomicU32::new(0);
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
static SIGNAL_STATE: AtomicU32 = AtomicU32::new(SIGNAL_INITIAL_STATE);
#[cfg(feature = "debug-irq-count")]
#[used]
#[no_mangle]
#[link_section = ".bss.irq_probe"]
pub static mut IRQ_COUNT_PROBE: u32 = 0;
#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
static SAMPLES: Mutex<RefCell<[i32; FRAME_COUNT]>> = Mutex::new(RefCell::new([0; FRAME_COUNT]));
#[cfg(not(feature = "sync_timing_observer"))]
static TIM2_DEV: Mutex<RefCell<Option<pac::TIM2>>> = Mutex::new(RefCell::new(None));
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIM4_DEV: Mutex<RefCell<Option<pac::TIM4>>> = Mutex::new(RefCell::new(None));
static USART2_DEV: Mutex<RefCell<Option<pac::USART2>>> = Mutex::new(RefCell::new(None));
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_REPORT_READY: AtomicBool = AtomicBool::new(false);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_PAIRED_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_MISSED_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_UNEXPECTED_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_PRE_FIRST_TRIGGER_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_IN_WINDOW_UNEXPECTED_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_FIRST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_LAST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_POST_FINAL_TRIGGER_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_CAPTURE_ERROR_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_MAX_DELTA_TICKS: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_EVIDENCE_WINDOW_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_EVIDENCE_WINDOW_PAIRED_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_EVIDENCE_WINDOW_MISSED_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_EVIDENCE_WINDOW_UNEXPECTED_ACK_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_EVIDENCE_WINDOW_CAPTURE_ERROR_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_EVIDENCE_WINDOW_MAX_DELTA_TICKS: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_LATEST_TRIGGER_VALID: AtomicBool = AtomicBool::new(false);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_LATEST_TRIGGER_TS: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_LATEST_TRIGGER_COUNT: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_STARTUP_TRIGGER_INPUT_LEVEL: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_STARTUP_ACK_INPUT_LEVEL: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_CAPTURE_CLEAR_ATTEMPTED: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_CAPTURE_SR_BEFORE_CLEAR: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_CAPTURE_SR_AFTER_CLEAR: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
static TIMING_CAPTURE_SR_AFTER_ARM: AtomicU32 = AtomicU32::new(0);

pub fn fw_main() -> ! {
    let dp = loop {
        if let Some(p) = pac::Peripherals::take() {
            break p;
        }
    };

    #[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
    let mut cp = loop {
        if let Some(p) = cortex_m::Peripherals::take() {
            break p;
        }
    };
    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    let _cp = loop {
        if let Some(p) = cortex_m::Peripherals::take() {
            break p;
        }
    };

    #[cfg(any(
        feature = "sync_trigger_out",
        feature = "sync_trigger_in",
        feature = "sync_timing_observer"
    ))]
    init_hse_pll_180mhz_or_fault(&dp);

    init_gpioa_for_usart2_tx(&dp);
    #[cfg(feature = "sync_trigger_out")]
    init_sync_trigger_output(&dp);
    #[cfg(all(feature = "sync_trigger_in", not(feature = "sync_timing_observer")))]
    init_trigger_boundary(&dp);
    #[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
    init_sync_timing_capture_gpio(&dp);
    init_usart2(&dp);
    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    dp.RCC.apb1enr().modify(|_, w| w.tim2en().set_bit());
    #[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
    dp.RCC
        .apb1enr()
        .modify(|_, w| w.tim2en().set_bit().tim4en().set_bit());
    #[cfg(feature = "sync_timing_observer")]
    dp.RCC.apb1enr().modify(|_, w| w.tim4en().set_bit());

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    cortex_m::interrupt::free(|cs| {
        TIM2_DEV.borrow(cs).replace(Some(dp.TIM2));
        USART2_DEV.borrow(cs).replace(Some(dp.USART2));
    });
    #[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
    cortex_m::interrupt::free(|cs| {
        TIM2_DEV.borrow(cs).replace(Some(dp.TIM2));
        TIM4_DEV.borrow(cs).replace(Some(dp.TIM4));
        USART2_DEV.borrow(cs).replace(Some(dp.USART2));
    });
    #[cfg(feature = "sync_timing_observer")]
    cortex_m::interrupt::free(|cs| {
        TIM4_DEV.borrow(cs).replace(Some(dp.TIM4));
        USART2_DEV.borrow(cs).replace(Some(dp.USART2));
    });

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    init_tim2_1khz();
    #[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
    {
        reset_sync_timing_state();
        init_tim2_sync_hardware_ack();
        init_tim4_sync_timing_capture();
    }
    #[cfg(feature = "sync_timing_observer")]
    {
        reset_sync_timing_state();
        init_tim4_sync_timing_capture();
    }

    // Enable IRQs at NVIC. IRQs are globally enabled after reset.
    unsafe {
        #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
        #[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
        cp.NVIC.set_priority(pac::Interrupt::TIM4, 0x10);
        #[cfg(all(
            feature = "sync_trigger_in",
            not(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))
        ))]
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI0);
        #[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM4);
    }

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    #[cfg(feature = "sync_trigger_out")]
    let mut sync_trigger_out_div: u32 = 0;

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
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

    #[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
    while TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.load(Ordering::Acquire) < SYNC_TIMING_TRIGGER_TARGET
    {
        pulse_sync_trigger_output();
        let next = TIMING_TRIGGER_COUNT.fetch_add(1, Ordering::AcqRel) + 1;
        process_sync_timing_accepted_trigger(next);
        if TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.load(Ordering::Acquire)
            >= SYNC_TIMING_TRIGGER_TARGET
        {
            break;
        }
        cortex_m::asm::delay(18_000);
    }

    #[cfg(feature = "sync_timing_observer")]
    while TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.load(Ordering::Acquire) < SYNC_TIMING_TRIGGER_TARGET
    {
        cortex_m::asm::wfi();
        drain_tim4_sync_timing_capture();
    }

    #[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
    wait_for_final_sync_timing_ack();

    // Halt capture and enter dump-only phase.
    cortex_m::interrupt::disable();
    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    cortex_m::peripheral::NVIC::mask(pac::Interrupt::TIM2);
    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    stop_tim2();
    #[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
    {
        drain_tim4_sync_timing_capture();
        cortex_m::peripheral::NVIC::mask(pac::Interrupt::TIM4);
        #[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
        stop_tim2_sync_hardware_ack();
        stop_tim4_sync_timing_capture();
        finalize_sync_timing_capture();
        dump_sync_timing_report();
        TIMING_REPORT_READY.store(true, Ordering::Release);
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    #[cfg(feature = "debug-repeat-dump")]
    loop {
        dump_artifact();
    }

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    #[cfg(not(feature = "debug-repeat-dump"))]
    {
        dump_artifact();
        loop {
            cortex_m::asm::wfi();
        }
    }
}

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
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

#[cfg(all(feature = "sync_trigger_in", not(feature = "sync_timing_observer")))]
pub fn exti0_isr() {
    #[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
    unsafe {
        core::ptr::write_volatile(GPIOA_BSRR_ADDR as *mut u32, GPIO_BSRR_BS1);
        core::ptr::write_volatile(EXTI_PR_ADDR as *mut u32, EXTI_PR_PR0);
        core::ptr::write_volatile(GPIOA_BSRR_ADDR as *mut u32, GPIO_BSRR_BR1);
    }

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    unsafe {
        let gpioa = &*pac::GPIOA::ptr();
        let exti = &*pac::EXTI::ptr();

        gpioa.bsrr().write(|w| w.bs1().set_bit());
        // EXTI PR is write-one-to-clear; bit 0 clears pending EXTI0.
        exti.pr().write(|w| w.bits(1));
        gpioa.bsrr().write(|w| w.br1().set_bit());
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
pub fn tim4_isr() {
    drain_tim4_sync_timing_capture();
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn drain_tim4_sync_timing_capture() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim4) = TIM4_DEV.borrow(cs).borrow_mut().as_mut() {
            let sr_bits = tim4.sr().read().bits();
            let have_trigger = (sr_bits & TIM_SR_CC3IF) != 0;
            let have_ack = (sr_bits & TIM_SR_CC4IF) != 0;
            let mut clear_mask = 0u32;

            if have_ack {
                if have_trigger {
                    process_sync_timing_passive_trigger(tim4.ccr3().read().ccr().bits() as u16);
                    clear_mask |= TIM_SR_CC3IF;
                }
                process_sync_timing_ack(tim4.ccr4().read().ccr().bits() as u16);
                clear_mask |= TIM_SR_CC4IF;
            } else if have_trigger {
                process_sync_timing_passive_trigger(tim4.ccr3().read().ccr().bits() as u16);
                clear_mask |= TIM_SR_CC3IF;
            }

            if (sr_bits & TIM_SR_CC3OF) != 0 {
                clear_mask |= TIM_SR_CC3OF;
                TIMING_CAPTURE_ERROR_COUNT.fetch_add(1, Ordering::AcqRel);
                process_sync_timing_evidence_window_capture_error();
            }
            if (sr_bits & TIM_SR_CC4OF) != 0 {
                clear_mask |= TIM_SR_CC4OF;
                TIMING_CAPTURE_ERROR_COUNT.fetch_add(1, Ordering::AcqRel);
                process_sync_timing_evidence_window_capture_error();
            }
            if clear_mask != 0 {
                // TIMx_SR capture flags are rc_w0: write 0 to handled flags and
                // 1 elsewhere so unrelated pending flags are preserved.
                tim4.sr().write(|w| unsafe { w.bits(!clear_mask) });
            }
        }
    });
}

#[cfg(any(
    feature = "sync_trigger_out",
    feature = "sync_trigger_in",
    feature = "sync_timing_observer"
))]
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

#[cfg(any(
    feature = "sync_trigger_out",
    feature = "sync_trigger_in",
    feature = "sync_timing_observer"
))]
fn wait_until_or_clock_fault(mut ready: impl FnMut() -> bool) {
    for _ in 0..2_000_000 {
        if ready() {
            return;
        }
        cortex_m::asm::nop();
    }
    clock_fault_loop();
}

#[cfg(any(
    feature = "sync_trigger_out",
    feature = "sync_trigger_in",
    feature = "sync_timing_observer"
))]
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

#[cfg(all(feature = "sync_trigger_in", not(feature = "sync_timing_observer")))]
fn init_trigger_boundary(dp: &pac::Peripherals) {
    #[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
    {
        dp.RCC.ahb1enr().modify(|_, w| w.gpioaen().set_bit());
        dp.GPIOA.bsrr().write(|w| w.br1().set_bit());
        dp.GPIOA.moder().modify(|_, w| {
            w.moder0().alternate();
            w.moder1().output()
        });
        dp.GPIOA.afrl().modify(|_, w| {
            w.afrl0().af1();
            w.afrl1().af1()
        });
        dp.GPIOA.otyper().modify(|_, w| w.ot1().clear_bit());
        dp.GPIOA.ospeedr().modify(|_, w| {
            w.ospeedr0().very_high_speed();
            w.ospeedr1().very_high_speed()
        });
        dp.GPIOA.pupdr().modify(|_, w| {
            w.pupdr0().floating();
            w.pupdr1().floating()
        });
    }

    #[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
    {
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
}

#[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
fn wait_for_sync_trigger_input_idle() {
    unsafe {
        let gpioa = &*pac::GPIOA::ptr();
        while gpioa.idr().read().idr0().bit_is_set() {
            cortex_m::asm::nop();
        }
    }
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

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn init_sync_timing_capture_gpio(dp: &pac::Peripherals) {
    dp.RCC.ahb1enr().modify(|_, w| w.gpioben().set_bit());

    let startup_idr = dp.GPIOB.idr().read();
    TIMING_STARTUP_TRIGGER_INPUT_LEVEL.store(
        if startup_idr.idr8().bit_is_set() { 1 } else { 0 },
        Ordering::Release,
    );
    TIMING_STARTUP_ACK_INPUT_LEVEL.store(
        if startup_idr.idr9().bit_is_set() { 1 } else { 0 },
        Ordering::Release,
    );

    dp.GPIOB.moder().modify(|_, w| {
        w.moder8().alternate();
        w.moder9().alternate()
    });
    dp.GPIOB.afrh().modify(|_, w| {
        w.afrh8().af2();
        w.afrh9().af2()
    });
    dp.GPIOB.ospeedr().modify(|_, w| {
        w.ospeedr8().very_high_speed();
        w.ospeedr9().very_high_speed()
    });
    dp.GPIOB.otyper().modify(|_, w| {
        w.ot8().clear_bit();
        w.ot9().clear_bit()
    });
    dp.GPIOB.pupdr().modify(|_, w| {
        w.pupdr8().floating();
        w.pupdr9().floating()
    });
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

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
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

#[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
// TIM2 hardware-ack invariant:
// - TIM2 remains continuously armed; OPM is disabled and CEN remains set.
// - PA0/TIM2_CH1 is the slave reset trigger source.
// - Before a trigger, PA1/TIM2_CH2 must be inactive/low.
// - A PA0/TIM2_CH1 rising edge resets TIM2 CNT to 0.
// - TIM2_CH2 PWM mode 1 drives PA1 active/high while CNT < CCR2.
// - PA1 returns inactive/low after CCR2 ticks.
// - ARR is kept large enough to avoid free-running rollover pulses between normal PA6 triggers.
fn init_tim2_sync_hardware_ack() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.cr1()
                .modify(|_, w| w.cen().clear_bit().opm().clear_bit());
            tim2.cr2().reset();
            tim2.dier().reset();
            tim2.psc().write(|w| unsafe { w.psc().bits(0) });
            tim2.arr()
                .write(|w| unsafe { w.arr().bits(SYNC_TIM2_ACK_ARR) });
            tim2.ccr2()
                .write(|w| unsafe { w.ccr().bits(SYNC_TIM2_ACK_PULSE_TICKS) });
            tim2.cnt()
                .write(|w| unsafe { w.cnt().bits(SYNC_TIM2_ACK_PULSE_TICKS + 1) });
            tim2.ccmr1_output().write(|w| {
                w.cc1s().output();
                w.cc2s().output();
                w.oc2fe().enabled();
                w.oc2pe().disabled();
                w.oc2m().pwm_mode1();
                w.oc2ce().disabled()
            });
            tim2.ccmr1_input().modify(|_, w| {
                w.cc1s().ti1();
                w.ic1psc().no_prescaler();
                w.ic1f().no_filter()
            });
            tim2.ccer().write(|w| {
                w.cc1np().clear_bit();
                w.cc1p().rising_edge();
                w.cc1e().enabled();
                w.cc2np().clear_bit();
                w.cc2p().rising_edge();
                w.cc2e().enabled()
            });
            wait_for_sync_trigger_input_idle();
            tim2.smcr().write(|w| {
                w.ts().ti1fp1();
                w.sms().reset_mode()
            });
            tim2.egr().write(|w| w.ug().set_bit());
            tim2.cnt()
                .write(|w| unsafe { w.cnt().bits(SYNC_TIM2_ACK_PULSE_TICKS + 1) });
            tim2.sr().write(|w| unsafe { w.bits(0) });
            tim2.cr1().write(|w| {
                w.opm().disabled();
                w.dir().up();
                w.urs().any_event();
                w.cen().set_bit()
            });
            unsafe {
                let gpioa = &*pac::GPIOA::ptr();
                gpioa.moder().modify(|_, w| w.moder1().alternate());
            }
        }
    });
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn reset_sync_timing_state() {
    TIMING_REPORT_READY.store(false, Ordering::Release);
    TIMING_TRIGGER_COUNT.store(0, Ordering::Release);
    TIMING_ACK_COUNT.store(0, Ordering::Release);
    TIMING_PAIRED_ACK_COUNT.store(0, Ordering::Release);
    TIMING_MISSED_ACK_COUNT.store(0, Ordering::Release);
    TIMING_UNEXPECTED_ACK_COUNT.store(0, Ordering::Release);
    TIMING_PRE_FIRST_TRIGGER_ACK_COUNT.store(0, Ordering::Release);
    TIMING_IN_WINDOW_UNEXPECTED_ACK_COUNT.store(0, Ordering::Release);
    TIMING_FIRST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT.store(0, Ordering::Release);
    TIMING_LAST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT.store(0, Ordering::Release);
    TIMING_POST_FINAL_TRIGGER_ACK_COUNT.store(0, Ordering::Release);
    TIMING_CAPTURE_ERROR_COUNT.store(0, Ordering::Release);
    TIMING_MAX_DELTA_TICKS.store(0, Ordering::Release);
    TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.store(0, Ordering::Release);
    TIMING_EVIDENCE_WINDOW_ACK_COUNT.store(0, Ordering::Release);
    TIMING_EVIDENCE_WINDOW_PAIRED_ACK_COUNT.store(0, Ordering::Release);
    TIMING_EVIDENCE_WINDOW_MISSED_ACK_COUNT.store(0, Ordering::Release);
    TIMING_EVIDENCE_WINDOW_UNEXPECTED_ACK_COUNT.store(0, Ordering::Release);
    TIMING_EVIDENCE_WINDOW_CAPTURE_ERROR_COUNT.store(0, Ordering::Release);
    TIMING_EVIDENCE_WINDOW_MAX_DELTA_TICKS.store(0, Ordering::Release);
    TIMING_LATEST_TRIGGER_VALID.store(false, Ordering::Release);
    TIMING_LATEST_TRIGGER_TS.store(0, Ordering::Release);
    TIMING_LATEST_TRIGGER_COUNT.store(0, Ordering::Release);
    TIMING_CAPTURE_CLEAR_ATTEMPTED.store(0, Ordering::Release);
    TIMING_CAPTURE_SR_BEFORE_CLEAR.store(0, Ordering::Release);
    TIMING_CAPTURE_SR_AFTER_CLEAR.store(0, Ordering::Release);
    TIMING_CAPTURE_SR_AFTER_ARM.store(0, Ordering::Release);
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn init_tim4_sync_timing_capture() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim4) = TIM4_DEV.borrow(cs).borrow_mut().as_mut() {
            tim4.cr1()
                .modify(|_, w| w.cen().clear_bit().opm().clear_bit());
            tim4.psc().write(|w| unsafe { w.psc().bits(0) });
            tim4.arr().write(|w| unsafe { w.arr().bits(0xffff) });
            tim4.cnt().write(|w| unsafe { w.cnt().bits(0) });
            tim4.smcr().write(|w| w.sms().disabled());
            tim4.ccmr2_input().write(|w| {
                w.cc3s().ti3();
                w.ic3psc().no_prescaler();
                w.ic3f().no_filter();
                w.cc4s().ti4();
                w.ic4psc().no_prescaler();
                w.ic4f().no_filter()
            });
            tim4.ccer().write(|w| {
                w.cc3np().clear_bit();
                w.cc3p().rising_edge();
                w.cc3e().enabled();
                w.cc4np().clear_bit();
                w.cc4p().rising_edge();
                w.cc4e().enabled()
            });
            let sr_before_clear = tim4.sr().read().bits();
            TIMING_CAPTURE_CLEAR_ATTEMPTED.store(1, Ordering::Release);
            TIMING_CAPTURE_SR_BEFORE_CLEAR.store(sr_before_clear, Ordering::Release);
            tim4.sr().write(|w| unsafe {
                w.bits(!(TIM_SR_CC3IF | TIM_SR_CC4IF | TIM_SR_CC3OF | TIM_SR_CC4OF))
            });
            TIMING_CAPTURE_SR_AFTER_CLEAR.store(tim4.sr().read().bits(), Ordering::Release);
            tim4.dier().write(|w| {
                #[cfg(all(
                    feature = "sync_timing_capture",
                    not(feature = "sync_timing_observer")
                ))]
                w.cc3ie().disabled();
                #[cfg(feature = "sync_timing_observer")]
                w.cc3ie().enabled();
                w.cc4ie().enabled()
            });
            tim4.cr1().modify(|_, w| w.cen().set_bit());
            TIMING_CAPTURE_SR_AFTER_ARM.store(tim4.sr().read().bits(), Ordering::Release);
        }
    });
}

fn usart2_apb1_hz() -> u32 {
    #[cfg(any(
        feature = "sync_trigger_out",
        feature = "sync_trigger_in",
        feature = "sync_timing_observer"
    ))]
    {
        SYNC_APB1_HZ
    }
    #[cfg(not(any(
        feature = "sync_trigger_out",
        feature = "sync_trigger_in",
        feature = "sync_timing_observer"
    )))]
    {
        DEFAULT_APB1_HZ
    }
}

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
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

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
fn stop_tim2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.dier().modify(|_, w| w.uie().clear_bit());
            tim2.cr1().modify(|_, w| w.cen().clear_bit());
            tim2.sr().modify(|_, w| w.uif().clear_bit());
        }
    });
}

#[cfg(all(feature = "sync_timing_capture", not(feature = "sync_timing_observer")))]
fn stop_tim2_sync_hardware_ack() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.cr1().modify(|_, w| w.cen().clear_bit());
            tim2.ccer().modify(|_, w| w.cc2e().disabled());
            tim2.smcr().write(|w| w.sms().disabled());
            tim2.sr().write(|w| unsafe { w.bits(0) });
        }
    });
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn stop_tim4_sync_timing_capture() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim4) = TIM4_DEV.borrow(cs).borrow_mut().as_mut() {
            tim4.dier().write(|w| {
                w.cc3ie().disabled();
                w.cc4ie().disabled()
            });
            tim4.ccer().write(|w| {
                w.cc3e().disabled();
                w.cc4e().disabled()
            });
            tim4.cr1().modify(|_, w| w.cen().clear_bit());
            tim4.sr().write(|w| unsafe {
                w.bits(!(TIM_SR_CC3IF | TIM_SR_CC4IF | TIM_SR_CC3OF | TIM_SR_CC4OF))
            });
        }
    });
}

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
fn clear_tim2_update_flag() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.sr().write(|w| w.uif().clear_bit());
        }
    });
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn wait_for_final_sync_timing_ack() {
    let mut polls = 0;
    while TIMING_PAIRED_ACK_COUNT.load(Ordering::Acquire)
        < TIMING_TRIGGER_COUNT.load(Ordering::Acquire)
        && polls < SYNC_TIMING_ACK_GRACE_POLLS
    {
        drain_tim4_sync_timing_capture();
        cortex_m::asm::delay(SYNC_TIMING_ACK_GRACE_DELAY_CYCLES);
        polls += 1;
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn process_sync_timing_passive_trigger(timestamp: u16) {
    #[cfg(feature = "sync_timing_observer")]
    let trigger_count = {
        let trigger_count = TIMING_TRIGGER_COUNT.fetch_add(1, Ordering::AcqRel) + 1;
        process_sync_timing_accepted_trigger(trigger_count);
        trigger_count
    };
    #[cfg(not(feature = "sync_timing_observer"))]
    let trigger_count = TIMING_TRIGGER_COUNT.load(Ordering::Acquire);
    // Replacing an unpaired trigger does not increment missed_ack_count here;
    // finalization derives misses as trigger_count - paired_ack_count.
    TIMING_LATEST_TRIGGER_TS.store(u32::from(timestamp), Ordering::Release);
    TIMING_LATEST_TRIGGER_COUNT.store(trigger_count, Ordering::Release);
    TIMING_LATEST_TRIGGER_VALID.store(true, Ordering::Release);
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn process_sync_timing_accepted_trigger(trigger_count: u32) {
    if trigger_count >= SYNC_TIMING_EVIDENCE_WINDOW_START_TRIGGER {
        TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.fetch_add(1, Ordering::AcqRel);
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn process_sync_timing_ack(timestamp: u16) {
    TIMING_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
    if !TIMING_LATEST_TRIGGER_VALID.swap(false, Ordering::AcqRel) {
        process_sync_timing_unexpected_ack();
        return;
    }

    let trigger_ts = TIMING_LATEST_TRIGGER_TS.load(Ordering::Acquire) as u16;
    let trigger_count = TIMING_LATEST_TRIGGER_COUNT.load(Ordering::Acquire);
    let delta_ticks = u32::from(timestamp.wrapping_sub(trigger_ts));
    TIMING_PAIRED_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
    update_sync_timing_max_delta(delta_ticks);
    if trigger_count >= SYNC_TIMING_EVIDENCE_WINDOW_START_TRIGGER {
        TIMING_EVIDENCE_WINDOW_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
        TIMING_EVIDENCE_WINDOW_PAIRED_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
        update_sync_timing_evidence_window_max_delta(delta_ticks);
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn process_sync_timing_unexpected_ack() {
    let trigger_count = TIMING_TRIGGER_COUNT.load(Ordering::Acquire);
    TIMING_UNEXPECTED_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
    if trigger_count == 0 {
        TIMING_PRE_FIRST_TRIGGER_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
    } else if TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.load(Ordering::Acquire)
        >= SYNC_TIMING_TRIGGER_TARGET
    {
        TIMING_POST_FINAL_TRIGGER_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
    } else {
        let previous = TIMING_IN_WINDOW_UNEXPECTED_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
        if previous == 0 {
            TIMING_FIRST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT
                .store(trigger_count, Ordering::Release);
        }
        TIMING_LAST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT.store(trigger_count, Ordering::Release);
        if trigger_count >= SYNC_TIMING_EVIDENCE_WINDOW_START_TRIGGER {
            TIMING_EVIDENCE_WINDOW_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
            TIMING_EVIDENCE_WINDOW_UNEXPECTED_ACK_COUNT.fetch_add(1, Ordering::AcqRel);
        }
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn update_sync_timing_max_delta(delta_ticks: u32) {
    let mut current = TIMING_MAX_DELTA_TICKS.load(Ordering::Acquire);
    while delta_ticks > current {
        match TIMING_MAX_DELTA_TICKS.compare_exchange(
            current,
            delta_ticks,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return,
            Err(next) => current = next,
        }
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn update_sync_timing_evidence_window_max_delta(delta_ticks: u32) {
    let mut current = TIMING_EVIDENCE_WINDOW_MAX_DELTA_TICKS.load(Ordering::Acquire);
    while delta_ticks > current {
        match TIMING_EVIDENCE_WINDOW_MAX_DELTA_TICKS.compare_exchange(
            current,
            delta_ticks,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return,
            Err(next) => current = next,
        }
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn process_sync_timing_evidence_window_capture_error() {
    let trigger_count = TIMING_TRIGGER_COUNT.load(Ordering::Acquire);
    let evidence_window_trigger_count =
        TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.load(Ordering::Acquire);
    if trigger_count >= SYNC_TIMING_EVIDENCE_WINDOW_START_TRIGGER
        && evidence_window_trigger_count < SYNC_TIMING_TRIGGER_TARGET
    {
        TIMING_EVIDENCE_WINDOW_CAPTURE_ERROR_COUNT.fetch_add(1, Ordering::AcqRel);
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn finalize_sync_timing_capture() {
    let trigger_count = TIMING_TRIGGER_COUNT.load(Ordering::Acquire);
    let paired_ack_count = TIMING_PAIRED_ACK_COUNT.load(Ordering::Acquire);
    TIMING_MISSED_ACK_COUNT.store(
        trigger_count.saturating_sub(paired_ack_count),
        Ordering::Release,
    );
    let evidence_window_trigger_count =
        TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.load(Ordering::Acquire);
    let evidence_window_paired_ack_count =
        TIMING_EVIDENCE_WINDOW_PAIRED_ACK_COUNT.load(Ordering::Acquire);
    TIMING_EVIDENCE_WINDOW_MISSED_ACK_COUNT.store(
        evidence_window_trigger_count.saturating_sub(evidence_window_paired_ack_count),
        Ordering::Release,
    );
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn dump_sync_timing_report() {
    cortex_m::interrupt::free(|cs| {
        if let Some(usart2) = USART2_DEV.borrow(cs).borrow().as_ref() {
            let trigger_count = TIMING_TRIGGER_COUNT.load(Ordering::Acquire);
            let ack_count = TIMING_ACK_COUNT.load(Ordering::Acquire);
            let missed_ack_count = TIMING_MISSED_ACK_COUNT.load(Ordering::Acquire);
            let unexpected_ack_count = TIMING_UNEXPECTED_ACK_COUNT.load(Ordering::Acquire);
            let pre_first_trigger_ack_count =
                TIMING_PRE_FIRST_TRIGGER_ACK_COUNT.load(Ordering::Acquire);
            let in_window_unexpected_ack_count =
                TIMING_IN_WINDOW_UNEXPECTED_ACK_COUNT.load(Ordering::Acquire);
            let first_in_window_unexpected_ack_trigger_count =
                TIMING_FIRST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT.load(Ordering::Acquire);
            let last_in_window_unexpected_ack_trigger_count =
                TIMING_LAST_IN_WINDOW_UNEXPECTED_ACK_TRIGGER_COUNT.load(Ordering::Acquire);
            let post_final_trigger_ack_count =
                TIMING_POST_FINAL_TRIGGER_ACK_COUNT.load(Ordering::Acquire);
            let capture_error_count = TIMING_CAPTURE_ERROR_COUNT.load(Ordering::Acquire);
            let max_delta_ticks = TIMING_MAX_DELTA_TICKS.load(Ordering::Acquire);
            let max_delta_ns =
                (u64::from(max_delta_ticks) * 1_000_000_000u64) / u64::from(SYNC_TIMING_TIMER_HZ);
            let evidence_window_trigger_count =
                TIMING_EVIDENCE_WINDOW_TRIGGER_COUNT.load(Ordering::Acquire);
            let evidence_window_ack_count =
                TIMING_EVIDENCE_WINDOW_ACK_COUNT.load(Ordering::Acquire);
            let evidence_window_missed_ack_count =
                TIMING_EVIDENCE_WINDOW_MISSED_ACK_COUNT.load(Ordering::Acquire);
            let evidence_window_unexpected_ack_count =
                TIMING_EVIDENCE_WINDOW_UNEXPECTED_ACK_COUNT.load(Ordering::Acquire);
            let evidence_window_capture_error_count =
                TIMING_EVIDENCE_WINDOW_CAPTURE_ERROR_COUNT.load(Ordering::Acquire);
            let evidence_window_max_delta_ticks =
                TIMING_EVIDENCE_WINDOW_MAX_DELTA_TICKS.load(Ordering::Acquire);
            let evidence_window_max_delta_ns = (u64::from(evidence_window_max_delta_ticks)
                * 1_000_000_000u64)
                / u64::from(SYNC_TIMING_TIMER_HZ);
            let startup_trigger_input_level =
                TIMING_STARTUP_TRIGGER_INPUT_LEVEL.load(Ordering::Acquire);
            let startup_ack_input_level = TIMING_STARTUP_ACK_INPUT_LEVEL.load(Ordering::Acquire);
            let capture_clear_attempted =
                TIMING_CAPTURE_CLEAR_ATTEMPTED.load(Ordering::Acquire);
            let capture_sr_before_clear =
                TIMING_CAPTURE_SR_BEFORE_CLEAR.load(Ordering::Acquire);
            let capture_sr_after_clear = TIMING_CAPTURE_SR_AFTER_CLEAR.load(Ordering::Acquire);
            let capture_sr_after_arm = TIMING_CAPTURE_SR_AFTER_ARM.load(Ordering::Acquire);
            let capture_trigger_pending_after_arm =
                if (capture_sr_after_arm & TIM_SR_CC3IF) != 0 { 1 } else { 0 };
            let capture_ack_pending_after_arm =
                if (capture_sr_after_arm & TIM_SR_CC4IF) != 0 { 1 } else { 0 };
            let capture_event_pending_after_arm =
                if capture_trigger_pending_after_arm != 0 || capture_ack_pending_after_arm != 0 {
                    1
                } else {
                    0
                };
            let result = if missed_ack_count == 0
                && unexpected_ack_count == 0
                && capture_error_count == 0
                && max_delta_ticks < SYNC_TIMING_THRESHOLD_TICKS
            {
                "PASS"
            } else {
                "FAIL"
            };
            let evidence_window_result = if evidence_window_trigger_count
                == SYNC_TIMING_TRIGGER_TARGET
                && evidence_window_ack_count == SYNC_TIMING_TRIGGER_TARGET
                && evidence_window_missed_ack_count == 0
                && evidence_window_unexpected_ack_count == 0
                && evidence_window_capture_error_count == 0
                && evidence_window_max_delta_ticks <= SYNC_TIMING_THRESHOLD_TICKS
            {
                "PASS"
            } else {
                "FAIL"
            };

            write_bytes(usart2, b"SYNC_TIMING_CAPTURE_V1\n");
            write_report_u32(usart2, "timer_hz", SYNC_TIMING_TIMER_HZ);
            write_report_u32(usart2, "threshold_ticks", SYNC_TIMING_THRESHOLD_TICKS);
            write_report_u32(usart2, "trigger_count", trigger_count);
            write_report_u32(usart2, "ack_count", ack_count);
            write_report_u32(usart2, "missed_ack_count", missed_ack_count);
            write_report_u32(usart2, "unexpected_ack_count", unexpected_ack_count);
            write_report_u32(
                usart2,
                "pre_first_trigger_ack_count",
                pre_first_trigger_ack_count,
            );
            write_report_u32(
                usart2,
                "in_window_unexpected_ack_count",
                in_window_unexpected_ack_count,
            );
            write_report_u32(
                usart2,
                "first_in_window_unexpected_ack_trigger_count",
                first_in_window_unexpected_ack_trigger_count,
            );
            write_report_u32(
                usart2,
                "last_in_window_unexpected_ack_trigger_count",
                last_in_window_unexpected_ack_trigger_count,
            );
            write_report_u32(
                usart2,
                "post_final_trigger_ack_count",
                post_final_trigger_ack_count,
            );
            write_report_u32(usart2, "capture_error_count", capture_error_count);
            write_report_u32(usart2, "max_delta_ticks", max_delta_ticks);
            write_report_u64(usart2, "max_delta_ns", max_delta_ns);
            write_report_str(usart2, "result", result);
            write_report_u32(
                usart2,
                "evidence_window_start_trigger_count",
                SYNC_TIMING_EVIDENCE_WINDOW_START_TRIGGER,
            );
            write_report_u32(
                usart2,
                "evidence_window_trigger_count",
                evidence_window_trigger_count,
            );
            write_report_u32(
                usart2,
                "evidence_window_ack_count",
                evidence_window_ack_count,
            );
            write_report_u32(
                usart2,
                "evidence_window_unexpected_ack_count",
                evidence_window_unexpected_ack_count,
            );
            write_report_u32(
                usart2,
                "evidence_window_missed_ack_count",
                evidence_window_missed_ack_count,
            );
            write_report_u32(
                usart2,
                "evidence_window_capture_error_count",
                evidence_window_capture_error_count,
            );
            write_report_u32(
                usart2,
                "evidence_window_max_delta_ticks",
                evidence_window_max_delta_ticks,
            );
            write_report_u64(
                usart2,
                "evidence_window_max_delta_ns",
                evidence_window_max_delta_ns,
            );
            write_report_str(usart2, "evidence_window_result", evidence_window_result);
            write_report_str(usart2, "capture_trigger", "PB8_TIM4_CH3");
            write_report_str(usart2, "capture_ack", "PB9_TIM4_CH4");
            #[cfg(feature = "sync_timing_capture")]
            write_report_str(usart2, "wiring_profile", "single_board_split_capture_v1");
            #[cfg(feature = "sync_timing_observer")]
            write_report_str(usart2, "wiring_profile", "dual_edge_observer_v1");
            write_report_u32(
                usart2,
                "diagnostic_startup_trigger_input_level",
                startup_trigger_input_level,
            );
            write_report_u32(
                usart2,
                "diagnostic_startup_ack_input_level",
                startup_ack_input_level,
            );
            write_report_u32(
                usart2,
                "diagnostic_capture_clear_attempted",
                capture_clear_attempted,
            );
            write_report_u32(
                usart2,
                "diagnostic_capture_sr_before_clear",
                capture_sr_before_clear,
            );
            write_report_u32(
                usart2,
                "diagnostic_capture_sr_after_clear",
                capture_sr_after_clear,
            );
            write_report_u32(
                usart2,
                "diagnostic_capture_sr_after_arm",
                capture_sr_after_arm,
            );
            write_report_u32(
                usart2,
                "diagnostic_capture_event_pending_after_arm",
                capture_event_pending_after_arm,
            );
            write_report_u32(
                usart2,
                "diagnostic_capture_trigger_pending_after_arm",
                capture_trigger_pending_after_arm,
            );
            write_report_u32(
                usart2,
                "diagnostic_capture_ack_pending_after_arm",
                capture_ack_pending_after_arm,
            );
            write_report_str(usart2, "measured_path", "PB9_PA1_minus_PB8_PA6");
            wait_tc(usart2);
        }
    });
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn write_report_u32(usart2: &pac::USART2, key: &str, value: u32) {
    let mut line = LineBuf::new();
    let _ = writeln!(&mut line, "{key}={value}");
    write_bytes(usart2, line.as_bytes());
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn write_report_u64(usart2: &pac::USART2, key: &str, value: u64) {
    let mut line = LineBuf::new();
    let _ = writeln!(&mut line, "{key}={value}");
    write_bytes(usart2, line.as_bytes());
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
fn write_report_str(usart2: &pac::USART2, key: &str, value: &str) {
    let mut line = LineBuf::new();
    let _ = writeln!(&mut line, "{key}={value}");
    write_bytes(usart2, line.as_bytes());
}

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
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

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
fn write_header1(usart2: &pac::USART2, header: &Header1) {
    let bytes = encode_header1_le(header);
    debug_assert_eq!(bytes.len(), V1_MIN_HEADER_SIZE);
    write_bytes(usart2, &bytes);
}

#[cfg(not(any(feature = "sync_timing_capture", feature = "sync_timing_observer")))]
fn write_event_frame0(usart2: &pac::USART2, frame: &EventFrame0) {
    write_bytes(usart2, &encode_event_frame0_le(frame));
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
struct LineBuf {
    buf: [u8; 96],
    len: usize,
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
impl LineBuf {
    const fn new() -> Self {
        Self {
            buf: [0; 96],
            len: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

#[cfg(any(feature = "sync_timing_capture", feature = "sync_timing_observer"))]
impl Write for LineBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let end = self.len + bytes.len();
        if end > self.buf.len() {
            return Err(fmt::Error);
        }

        self.buf[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }
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
