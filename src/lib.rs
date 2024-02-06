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

use critical_section_lock_mut::LockMut;
pub use nb::{self};

#[macro_export]
macro_rules! print {
    ($fmt:literal $(, $args:expr)* $(,)?) => {
        $crate::CONSOLE.with_lock(|console| {
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

pub static CONSOLE: LockMut<ConsolePort> = LockMut::new();

pub fn init_serial(uarte0: UARTE0, uart: UartPins) {
    CONSOLE.init({
        let serial = uarte::Uarte::new(
            uarte0,
            uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
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
    CONSOLE.try_with_lock(|console| {
        use core::fmt::Write;
        use embedded_hal::prelude::_embedded_hal_serial_Write;
        write!(console, "{}\r\n", info).ok();
        nb::block!(console.flush()).unwrap();
    });
    loop {
        cortex_m::asm::wfi();
    }
}
