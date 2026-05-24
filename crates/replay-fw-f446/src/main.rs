#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_main)]

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    not(feature = "sync_timing_capture")
))]
mod artifact_metadata;

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod fw;

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    not(feature = "sync_timing_capture")
))]
mod signal_model;

#[cfg(all(target_arch = "arm", target_os = "none"))]
use cortex_m_rt::entry;
#[cfg(all(target_arch = "arm", target_os = "none"))]
use stm32f4::stm32f446::interrupt;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[entry]
fn main() -> ! {
    fw::fw_main()
}

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    not(feature = "sync_timing_capture")
))]
#[interrupt]
fn TIM2() {
    fw::tim2_isr()
}

#[cfg(all(
    target_arch = "arm",
    target_os = "none",
    feature = "sync_timing_capture"
))]
#[interrupt]
fn TIM4() {
    fw::tim4_isr()
}

#[cfg(all(target_arch = "arm", target_os = "none", feature = "sync_trigger_in"))]
#[interrupt]
fn EXTI0() {
    fw::exti0_isr()
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}
