#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    otg_fs::{USB, UsbBus},
    pac,
    prelude::*,
};
use usb_device::prelude::*;

use smoltcp::{
    iface::{Config as IfaceConfig, Interface, SocketSet, SocketStorage},
    socket::tcp::{Socket as TcpSocket, SocketBuffer as TcpSocketBuffer},
    time::Instant,
    wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address},
};

use usbd_ethernet::{Ethernet, USB_CLASS_CDC};
#[derive(Clone, Copy, PartialEq, Eq)]

enum ResponsePart {
    Header,
    Body,
    Done,
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Configure clocks with PLL48 for USB
    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.MHz())
        .sysclk(100.MHz())
        .require_pll48clk()
        .freeze();

    // Enable DWT cycle counter so we can produce a monotonic smoltcp `Instant`.
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    // Allocate static endpoint memory for USB (required by synopsys-usb-otg).
    static mut EP_MEMORY: [u32; 256] = [0; 256];

    // usbd-ethernet requires 2x 2048-byte NTB buffers.
    static mut USB_IN_NTB: [u8; 2048] = [0; 2048];
    static mut USB_OUT_NTB: [u8; 2048] = [0; 2048];

    // smoltcp TCP socket buffers.
    static mut TCP_RX: [u8; 2048] = [0; 2048];
    static mut TCP_TX: [u8; 2048] = [0; 2048];

    // smoltcp socket storage (1 socket is enough for a tiny demo web server).
    static mut SOCKET_STORAGE: [SocketStorage; 1] = [SocketStorage::EMPTY; 1];

    // Build USB peripheral.
    // NOTE: use `USB::new` so PA11/PA12 are converted into the strongly-typed OTG FS pins.
    let usb_peripheral = USB::new(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (gpioa.pa11, gpioa.pa12),
        &clocks,
    );

    // Create USB bus allocator (synopsys-usb-otg returns an allocator already!).
    let usb_bus = UsbBus::new(usb_peripheral, unsafe { &mut EP_MEMORY });

    let mac = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);
    let mut usb_eth = Ethernet::new(&usb_bus, mac.0, 64, unsafe { &mut USB_IN_NTB }, unsafe {
        &mut USB_OUT_NTB
    });

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .device_class(USB_CLASS_CDC)
        .composite_with_iads()
        .strings(&[StringDescriptors::new(LangID::EN_US)
            .manufacturer("BlackPill")
            .product("USB Ethernet")
            .serial_number("0001")])
        .unwrap()
        .max_packet_size_0(64)
        .unwrap()
        .build();

    // Build smoltcp interface over the USB Ethernet device.
    let mut iface = {
        let mut cfg = IfaceConfig::new(HardwareAddress::Ethernet(mac));
        // Any non-constant seed is better than 0; DWT is monotonic after enabling.
        cfg.random_seed = cortex_m::peripheral::DWT::cycle_count() as u64;
        Interface::new(cfg, &mut usb_eth, Instant::ZERO)
    };

    // Static IPv4 configuration. Configure your Windows USB Ethernet adapter to match.
    iface.update_ip_addrs(|addrs| {
        let _ = addrs.push(IpCidr::new(
            IpAddress::Ipv4(Ipv4Address::new(192, 168, 7, 2)),
            24,
        ));
    });

    let mut sockets = SocketSet::new(unsafe { &mut SOCKET_STORAGE[..] });
    let tcp_socket = TcpSocket::new(
        TcpSocketBuffer::new(unsafe { &mut TCP_RX[..] }),
        TcpSocketBuffer::new(unsafe { &mut TCP_TX[..] }),
    );
    let tcp_handle = sockets.add(tcp_socket);

    // DWT wrap tracking (DWT cycle counter is 32-bit, wraps roughly every ~42s @ 100MHz).
    let mut dwt_prev = cortex_m::peripheral::DWT::cycle_count();
    let mut dwt_hi: u32 = 0;
    let cycles_per_us: u64 = (clocks.sysclk().raw() / 1_000_000) as u64;

    // Minimal HTTP/1.0 response. No Content-Length is used; the socket is closed after sending.
    // const HTTP_RESPONSE: &[u8] = b"HTTP/1.0 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n<html><body><h1>Hello from STM32 Black Pill</h1></body></html>\r\n";
    const HTTP_HEADER: &[u8] =
        b"HTTP/1.0 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n";

    const INDEX_HTML: &[u8] = include_bytes!("web/index.html");

    let mut reply_offset: usize = 0;
    let mut reply_part = ResponsePart::Done;

    loop {
        // Poll USB. This must run frequently for enumeration and transfers.
        let _ = usb_dev.poll(&mut [&mut usb_eth]);

        // If the host enabled the data interface, attempt to notify "link up".
        // (Safe to call repeatedly; it returns WouldBlock when it can't proceed.)
        let _ = usb_eth.connect();
        let _ = usb_eth.set_connection_speed(12_000_000, 12_000_000);

        // Monotonic timestamp for smoltcp.
        let dwt_now = cortex_m::peripheral::DWT::cycle_count();
        if dwt_now < dwt_prev {
            dwt_hi = dwt_hi.wrapping_add(1);
        }
        dwt_prev = dwt_now;
        let total_cycles: u64 = ((dwt_hi as u64) << 32) | (dwt_now as u64);
        let now = Instant::from_micros((total_cycles / cycles_per_us) as i64);

        iface.poll(now, &mut usb_eth, &mut sockets);

        // Very small HTTP server: listen on :80, reply with a fixed page, close.
        {
            let socket = sockets.get_mut::<TcpSocket>(tcp_handle);

            if !socket.is_listening() && !socket.is_active() {
                let _ = socket.listen(80);
                reply_offset = 0;
                reply_part = ResponsePart::Done;
            }

            if socket.can_recv() {
                let _ = socket.recv(|buf| {
                    if buf.starts_with(b"GET /led/on ") {
                        // On many BlackPill boards PC13 LED is active-low.
                        led.set_low();
                    } else if buf.starts_with(b"GET /led/off ") {
                        led.set_high();
                    }

                    let len = buf.len();
                    (len, ())
                });

                reply_part = ResponsePart::Header;
                reply_offset = 0;
            }

            if reply_part != ResponsePart::Done && socket.can_send() {
                let current: &[u8] = match reply_part {
                    ResponsePart::Header => HTTP_HEADER,
                    ResponsePart::Body => INDEX_HTML,
                    ResponsePart::Done => &[],
                };

                let remaining = &current[reply_offset..];

                match socket.send_slice(remaining) {
                    Ok(sent) => {
                        reply_offset += sent;

                        if reply_offset >= current.len() {
                            reply_offset = 0;

                            match reply_part {
                                ResponsePart::Header => {
                                    reply_part = ResponsePart::Body;
                                }
                                ResponsePart::Body => {
                                    socket.close();
                                    reply_part = ResponsePart::Done;
                                }
                                ResponsePart::Done => {}
                            }
                        }
                    }
                    Err(_) => {
                        // Can't send right now.
                    }
                }
            }

            // If the peer closed early, ensure we don't keep stale state.
            if !socket.is_open() {
                reply_offset = 0;
                reply_part = ResponsePart::Done;
            }
        }
    }
}
