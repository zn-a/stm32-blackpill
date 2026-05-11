#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{otg_fs::{USB, UsbBus}, pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.MHz())
        .sysclk(100.MHz())
        .require_pll48clk()
        .freeze();

    let gpioa = dp.GPIOA.split();

    static mut EP_MEMORY: [u32; 256] = [0; 256];

    let usb = USB::new(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (gpioa.pa11, gpioa.pa12),
        &clocks,
    );

    // synopsys-usb-otg returns an allocator already.
    let _usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    loop {}
}