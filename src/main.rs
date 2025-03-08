#![no_std]
#![no_main]
use cortex_m_rt::entry;
#[allow(unused_imports)]
use panic_halt as _;
#[allow(unused_imports)]
use stm32f4xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    panic!()
}
