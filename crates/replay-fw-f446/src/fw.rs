use panic_halt as _;
use stm32f4::stm32f446 as pac;

const PA6_TIM_PSC: u16 = 15_999;
const PA6_TIM_ARR: u16 = 999;
const PA6_TIM_CCR1: u16 = 500;

pub fn fw_main() -> ! {
    let dp = loop {
        if let Some(p) = pac::Peripherals::take() {
            break p;
        }
    };

    init_gpioa_for_tim3_ch1(&dp);
    init_tim3_free_running(&dp);

    loop {
        cortex_m::asm::wfi();
    }
}

pub fn tim2_isr() {}

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

fn init_tim3_free_running(dp: &pac::Peripherals) {
    dp.RCC.apb1enr().modify(|_, w| w.tim3en().set_bit());

    dp.TIM3.cr1().modify(|_, w| w.cen().disabled().opm().clear_bit());
    dp.TIM3.psc().write(|w| unsafe { w.psc().bits(PA6_TIM_PSC) });
    dp.TIM3.arr().write(|w| unsafe { w.arr().bits(PA6_TIM_ARR) });
    dp.TIM3.ccr1().write(|w| unsafe { w.ccr().bits(PA6_TIM_CCR1) });
    dp.TIM3.cnt().write(|w| unsafe { w.cnt().bits(0) });
    dp.TIM3.smcr().write(|w| w.sms().disabled());
    dp.TIM3.ccmr1_output().write(|w| {
        w.cc1s().output();
        w.oc1fe().disabled();
        w.oc1pe().enabled();
        w.oc1m().pwm_mode1()
    });
    dp.TIM3.ccer().write(|w| {
        w.cc1np().clear_bit();
        w.cc1p().rising_edge();
        w.cc1e().enabled()
    });
    dp.TIM3.egr().write(|w| w.ug().update());
    dp.TIM3.sr().write(|w| w.uif().clear());
    dp.TIM3.cr1().modify(|_, w| w.arpe().enabled().cen().enabled());
}
