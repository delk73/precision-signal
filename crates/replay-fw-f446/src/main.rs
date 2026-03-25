#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_main)]

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod artifact_metadata;

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod fw;

#[cfg(all(target_arch = "arm", target_os = "none"))]
use cortex_m_rt::entry;
#[cfg(all(target_arch = "arm", target_os = "none"))]
use stm32f4::stm32f446::interrupt;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[entry]
fn main() -> ! {
    fw::fw_main()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[interrupt]
fn TIM2() {
    fw::tim2_isr()
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}
