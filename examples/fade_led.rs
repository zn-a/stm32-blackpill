#![no_std]
#![no_main]

use cortex_m_rt::entry;
#[allow(unused_imports)]
use panic_halt as _;
use stm32f4xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // Acquire STM32 peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure system clocks
    let rcc = dp.RCC.constrain();

    // Configure the system clock:
    // - Uses external 25 MHz crystal (`HSE`)
    // - Sets system clock to 100 MHz (`sysclk`)
    // - Sets bus clock (`hclk`) to 25 MHz
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

    const FADE_CONST: u32 = 100;
    loop {
        // Fade in (Increase brightness)
        for duty in 0..=FADE_CONST {
            led.set_high(); // Turn LED on
            delay.delay_us(duty * FADE_CONST); // ON time
            led.set_low(); // Turn LED off
            delay.delay_us((FADE_CONST - duty) * FADE_CONST); // OFF time
        }

        // Fade out (Decrease brightness)
        for duty in (0..=FADE_CONST).rev() {
            led.set_high();
            delay.delay_us(duty * FADE_CONST);
            led.set_low();
            delay.delay_us((FADE_CONST - duty) * FADE_CONST);
        }
    }
}
