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

    // Configure the system clock:
    // - Uses external 25 MHz crystal (`HSE`)
    // - Sets system clock to 100 MHz (`sysclk`)
    // - Sets bus clock (`hclk`) to 25 MHz
    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.MHz()) // Use 25 MHz external oscillator
        .sysclk(100.MHz()) // Set system clock to 100 MHz
        .hclk(25.MHz()) // Set AHB clock to 25 MHz
        .freeze(); // Apply clock configuration

    // Create a delay abstraction using TIM5 (a 32-bit timer)
    let mut delay = dp.TIM5.delay_us(&clocks);

    // Configure PC13 (LED) as push-pull output
    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    loop {
        // Turn LED ON
        led.set_high();
        delay.delay_ms(500); // Wait for 500ms

        // Turn LED OFF
        led.set_low();
        delay.delay_ms(500); // Wait for 500ms
    }
}
