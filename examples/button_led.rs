#![no_std]
#![no_main]

use cortex_m_rt::entry;
#[allow(unused_imports)]
use panic_halt as _;
use stm32f4xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // Acquire STM32 peripheral handles:
    // `dp` -> Device Peripherals (hardware registers)
    // `cp` -> Core Peripherals (CPU-specific registers)
    let dp = pac::Peripherals::take().unwrap();
    let _cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Configure PC13 (LED) as push-pull output
    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    // Configure PA0 ('Key' Button) as input with pull-up
    let gpioa = dp.GPIOA.split();
    let button = gpioa.pa0.into_pull_up_input();

    loop {
        // If button is pressed
        if button.is_high() {
            led.set_high(); // LED ON
        } else {
            led.set_low(); // LED OFF
        }
    }
}
