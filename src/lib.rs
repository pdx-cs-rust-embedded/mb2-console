#![no_std]

pub use embedded_hal;
use embedded_hal::blocking::serial as bserial;
use embedded_hal::serial;
use microbit::{
    board::UartPins,
    hal::{
        pac::UARTE0,
        uarte::{self, Baudrate, Error, Instance, Parity, Uarte, UarteRx, UarteTx},
    },
};

use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};
pub use nb::{self};

#[macro_export]
macro_rules! print {
    ($fmt:literal $(, $args:expr)* $(,)?) => {
        $crate::with_console(|console| {
            use core::fmt::Write;
            use $crate::embedded_hal::prelude::_embedded_hal_serial_Write;
            write!(console, $fmt, $($args),*).unwrap();
            $crate::nb::block!(console.flush()).unwrap();
        });
    };
}

#[macro_export]
macro_rules! println {
    ($fmt:literal $(, $args:expr)* $(,)?) => {
        $crate::print!($fmt, $($args),*);
        $crate::print!("\r\n");
    };
}

pub type ConsolePort = UartePort<UARTE0>;

static mut CONSOLE: Mutex<RefCell<Option<ConsolePort>>> = Mutex::new(RefCell::new(None));

fn with_maybe_console<F: FnOnce(&mut Option<ConsolePort>)>(f: F) {
    unsafe {
        interrupt::free(|cs| {
            let mut maybe_port = CONSOLE.borrow(cs).borrow_mut();
            f(&mut maybe_port);
        });
    }
}

pub fn with_console<F: FnOnce(&mut ConsolePort)>(f: F) {
    unsafe {
        interrupt::free(|cs| {
            let mut port = CONSOLE.borrow(cs).borrow_mut();
            f(port.as_mut().unwrap());
        });
    }
}

pub fn init_serial(uarte0: UARTE0, uart: UartPins) {
    with_maybe_console(|c| {
        let serial = uarte::Uarte::new(
            uarte0,
            uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        let port = UartePort::new(serial);
        *c = Some(port);
    });
}

static mut TX_BUF: [u8; 1] = [0; 1];
static mut RX_BUF: [u8; 1] = [0; 1];

pub struct UartePort<T: Instance>(UarteTx<T>, UarteRx<T>);

impl<T: Instance> UartePort<T> {
    fn new(serial: Uarte<T>) -> UartePort<T> {
        let (tx, rx) = serial
            .split(unsafe { &mut TX_BUF }, unsafe { &mut RX_BUF })
            .unwrap();
        UartePort(tx, rx)
    }
}

impl<T: Instance> core::fmt::Write for UartePort<T> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_str(s)
    }
}

impl<T: Instance> serial::Write<u8> for UartePort<T> {
    type Error = Error;

    fn write(&mut self, b: u8) -> nb::Result<(), Self::Error> {
        self.0.write(b)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0.flush()
    }
}

impl<T: Instance> bserial::write::Default<u8> for UartePort<T> {}

impl<T: Instance> serial::Read<u8> for UartePort<T> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.1.read()
    }
}

#[cfg(feature = "panic_handler")]
#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    with_maybe_console(|maybe_port| {
        if let Some(console) = maybe_port {
            use core::fmt::Write;
            use embedded_hal::prelude::_embedded_hal_serial_Write;
            write!(console, "{}\r\n", info).ok();
            nb::block!(console.flush()).unwrap();
        }
        *maybe_port = None;
    });
    loop {
        cortex_m::asm::wfi();
    }
}
