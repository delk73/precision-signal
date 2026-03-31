// SAFETY POLICY:
// This crate is embedded-only (cfg arm/none).
// Unsafe usage is limited to:
// - NVIC unmask
// - PAC register .bits() writes
// - USART DR write
// These are required by current PAC APIs and are quarantined to this firmware crate.

use core::cell::RefCell;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use cortex_m::interrupt::Mutex;
use panic_halt as _;
use stm32f4::stm32f446::{self as pac};

const INTERVAL_COUNT: usize = 138;
const CSV_HEADER: &str = "index,interval_us\n";
const CAPTURE_WAIT_POLL_CYCLES: u32 = 16_000;
const CAPTURE_WAIT_POLLS: u32 = 5_000;
const STIM_TIM_PSC: u16 = 15_999;
const STIM_TIM_ARR: u16 = 18;
const STIM_TIM_CCR1: u16 = 9;

static CAPTURE_DONE: AtomicBool = AtomicBool::new(false);
static HAVE_PREV: AtomicBool = AtomicBool::new(false);
static LAST_CAPTURE: AtomicU32 = AtomicU32::new(0);
static WRITE_IDX: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "debug-irq-count")]
#[used]
#[no_mangle]
#[link_section = ".bss.irq_probe"]
pub static mut IRQ_COUNT_PROBE: u32 = 0;
static INTERVALS: Mutex<RefCell<[u32; INTERVAL_COUNT]>> =
    Mutex::new(RefCell::new([0; INTERVAL_COUNT]));
static TIM2_DEV: Mutex<RefCell<Option<pac::TIM2>>> = Mutex::new(RefCell::new(None));
static TIM3_DEV: Mutex<RefCell<Option<pac::TIM3>>> = Mutex::new(RefCell::new(None));
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

    init_gpioa_for_tim2_ch1(&dp);
    init_gpioa_for_tim3_ch1(&dp);
    init_gpioa_for_usart2_tx(&dp);
    dp.RCC.apb1enr().modify(|_, w| {
        w.tim2en().set_bit();
        w.tim3en().set_bit();
        w.usart2en().set_bit()
    });
    init_usart2(&dp);
    reset_capture_state();

    cortex_m::interrupt::free(|cs| {
        TIM2_DEV.borrow(cs).replace(Some(dp.TIM2));
        TIM3_DEV.borrow(cs).replace(Some(dp.TIM3));
        USART2_DEV.borrow(cs).replace(Some(dp.USART2));
    });

    init_tim2_capture();
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
    }
    init_tim3_stimulus();

    wait_for_capture_progress();

    cortex_m::peripheral::NVIC::mask(pac::Interrupt::TIM2);
    stop_tim2_capture();
    stop_tim3_stimulus();
    let write_idx = WRITE_IDX.load(Ordering::Acquire).min(INTERVAL_COUNT);

    cortex_m::interrupt::disable();
    emit_capture_state(write_idx);
    if write_idx == INTERVAL_COUNT {
        dump_csv();
    }

    loop {
        cortex_m::asm::wfi();
    }
}

pub fn tim2_isr() {
    #[cfg(feature = "debug-irq-count")]
    unsafe {
        let p = core::ptr::addr_of_mut!(IRQ_COUNT_PROBE);
        let v = core::ptr::read_volatile(p);
        core::ptr::write_volatile(p, v.wrapping_add(1));
    }

    let mut maybe_now = None;
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            let sr = tim2.sr().read();
            if sr.cc1if().is_match() {
                let now = tim2.ccr1().read().ccr().bits();
                tim2.sr().write(|w| {
                    w.cc1if().clear();
                    w.cc1of().clear()
                });
                maybe_now = Some(now);
            }
        }
    });

    let Some(now) = maybe_now else {
        return;
    };

    if !HAVE_PREV.load(Ordering::Acquire) {
        LAST_CAPTURE.store(now, Ordering::Release);
        HAVE_PREV.store(true, Ordering::Release);
        return;
    }

    let interval = now.wrapping_sub(LAST_CAPTURE.load(Ordering::Relaxed));
    LAST_CAPTURE.store(now, Ordering::Release);

    let idx = WRITE_IDX.load(Ordering::Relaxed);
    if idx >= INTERVAL_COUNT {
        CAPTURE_DONE.store(true, Ordering::Release);
        return;
    }

    cortex_m::interrupt::free(|cs| {
        INTERVALS.borrow(cs).borrow_mut()[idx] = interval;
    });

    let next = idx + 1;
    WRITE_IDX.store(next, Ordering::Release);
    if next >= INTERVAL_COUNT {
        CAPTURE_DONE.store(true, Ordering::Release);
        stop_tim2_capture();
    }
}

fn reset_capture_state() {
    CAPTURE_DONE.store(false, Ordering::Release);
    HAVE_PREV.store(false, Ordering::Release);
    LAST_CAPTURE.store(0, Ordering::Release);
    WRITE_IDX.store(0, Ordering::Release);
}

fn wait_for_capture_progress() {
    for _ in 0..CAPTURE_WAIT_POLLS {
        let write_idx = WRITE_IDX.load(Ordering::Acquire);
        if write_idx >= INTERVAL_COUNT {
            return;
        }
        cortex_m::asm::delay(CAPTURE_WAIT_POLL_CYCLES);
    }
}

fn init_gpioa_for_tim2_ch1(dp: &pac::Peripherals) {
    dp.RCC.ahb1enr().modify(|_, w| w.gpioaen().set_bit());

    dp.GPIOA.moder().modify(|_, w| w.moder0().alternate());
    dp.GPIOA.afrl().modify(|_, w| w.afrl0().af1());
    dp.GPIOA
        .ospeedr()
        .modify(|_, w| w.ospeedr0().very_high_speed());
    dp.GPIOA.otyper().modify(|_, w| w.ot0().clear_bit());
    dp.GPIOA.pupdr().modify(|_, w| w.pupdr0().floating());
}

fn init_gpioa_for_tim3_ch1(dp: &pac::Peripherals) {
    dp.RCC.ahb1enr().modify(|_, w| w.gpioaen().set_bit());

    dp.GPIOA.moder().modify(|_, w| w.moder6().alternate());
    dp.GPIOA.afrl().modify(|_, w| w.afrl6().af2());
    dp.GPIOA
        .ospeedr()
        .modify(|_, w| w.ospeedr6().very_high_speed());
    dp.GPIOA.otyper().modify(|_, w| w.ot6().clear_bit());
    dp.GPIOA.pupdr().modify(|_, w| w.pupdr6().floating());
}

fn init_gpioa_for_usart2_tx(dp: &pac::Peripherals) {
    dp.RCC.ahb1enr().modify(|_, w| w.gpioaen().set_bit());

    dp.GPIOA.moder().modify(|_, w| w.moder2().alternate());
    dp.GPIOA.afrl().modify(|_, w| w.afrl2().af7());
    dp.GPIOA
        .ospeedr()
        .modify(|_, w| w.ospeedr2().very_high_speed());
    dp.GPIOA.otyper().modify(|_, w| w.ot2().clear_bit());
    dp.GPIOA.pupdr().modify(|_, w| w.pupdr2().floating());
}

fn init_usart2(dp: &pac::Peripherals) {
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

fn init_tim3_stimulus() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim3) = TIM3_DEV.borrow(cs).borrow_mut().as_mut() {
            tim3.cr1().modify(|_, w| w.cen().disabled().opm().clear_bit());
            tim3.psc().write(|w| unsafe { w.psc().bits(STIM_TIM_PSC) });
            tim3.arr().write(|w| unsafe { w.arr().bits(STIM_TIM_ARR) });
            tim3.ccr1().write(|w| unsafe { w.ccr().bits(STIM_TIM_CCR1) });
            tim3.cnt().write(|w| unsafe { w.cnt().bits(0) });
            tim3.smcr().write(|w| w.sms().disabled());
            tim3.ccmr1_output().write(|w| {
                w.cc1s().output();
                w.oc1fe().disabled();
                w.oc1pe().enabled();
                w.oc1m().pwm_mode1()
            });
            tim3.ccer().write(|w| {
                w.cc1np().clear_bit();
                w.cc1p().rising_edge();
                w.cc1e().enabled()
            });
            tim3.egr().write(|w| w.ug().update());
            tim3.sr().write(|w| w.uif().clear());
            tim3.cr1().modify(|_, w| w.arpe().enabled().cen().enabled());
        }
    });
}

fn init_tim2_capture() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.cr1().modify(|_, w| w.cen().clear_bit().opm().clear_bit());
            tim2.psc().write(|w| unsafe { w.psc().bits(15) });
            tim2.arr().write(|w| unsafe { w.arr().bits(u32::MAX) });
            tim2.cnt().write(|w| unsafe { w.cnt().bits(0) });
            tim2.smcr().write(|w| w.sms().disabled());
            tim2.ccmr1_input().write(|w| {
                w.cc1s().ti1();
                w.ic1psc().no_prescaler();
                w.ic1f().no_filter()
            });
            tim2.ccer().write(|w| {
                w.cc1np().clear_bit();
                w.cc1p().rising_edge();
                w.cc1e().enabled()
            });
            tim2.sr().write(|w| {
                w.cc1if().clear();
                w.cc1of().clear();
                w.uif().clear()
            });
            tim2.dier().write(|w| w.cc1ie().enabled());
            tim2.cr1().modify(|_, w| w.cen().set_bit());
        }
    });
}

fn stop_tim3_stimulus() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim3) = TIM3_DEV.borrow(cs).borrow_mut().as_mut() {
            tim3.ccer().write(|w| w.cc1e().disabled());
            tim3.cr1().modify(|_, w| w.cen().clear_bit());
            tim3.sr().write(|w| w.uif().clear());
        }
    });
}

fn stop_tim2_capture() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim2) = TIM2_DEV.borrow(cs).borrow_mut().as_mut() {
            tim2.dier().write(|w| w.cc1ie().disabled());
            tim2.ccer().write(|w| w.cc1e().disabled());
            tim2.cr1().modify(|_, w| w.cen().clear_bit());
            tim2.sr().write(|w| {
                w.cc1if().clear();
                w.cc1of().clear();
                w.uif().clear()
            });
        }
    });
}

fn dump_csv() {
    cortex_m::interrupt::free(|cs| {
        if let Some(usart2) = USART2_DEV.borrow(cs).borrow().as_ref() {
            write_bytes(usart2, CSV_HEADER.as_bytes());

            let intervals = INTERVALS.borrow(cs).borrow();
            for (idx, interval) in intervals.iter().copied().enumerate() {
                let mut line = LineBuf::new();
                let _ = writeln!(&mut line, "{idx},{interval}");
                write_bytes(usart2, line.as_bytes());
            }

            wait_tc(usart2);
        }
    });
}

fn emit_capture_state(write_idx: usize) {
    cortex_m::interrupt::free(|cs| {
        if let Some(usart2) = USART2_DEV.borrow(cs).borrow().as_ref() {
            let mut line = LineBuf::new();
            if write_idx == INTERVAL_COUNT {
                let _ = writeln!(&mut line, "STATE,CAPTURE_DONE,{INTERVAL_COUNT}");
            } else {
                let _ = writeln!(&mut line, "STATE,CAPTURE_INCOMPLETE,{write_idx}");
            }
            write_bytes(usart2, line.as_bytes());
            wait_tc(usart2);
        }
    });
}

struct LineBuf {
    buf: [u8; 32],
    len: usize,
}

impl LineBuf {
    const fn new() -> Self {
        Self {
            buf: [0; 32],
            len: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

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
    for &byte in bytes {
        while usart2.sr().read().txe().bit_is_clear() {}
        usart2.dr().write(|w| unsafe { w.dr().bits(u16::from(byte)) });
    }
}

fn wait_tc(usart2: &pac::USART2) {
    while usart2.sr().read().tc().bit_is_clear() {}
}
