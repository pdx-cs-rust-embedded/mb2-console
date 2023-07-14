#![no_std]
#![no_main]

use mb2_console::{init_serial, println};

use cortex_m_rt::entry;
use microbit::board::Board;

#[cfg(not(feature = "panic_handler"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    init_serial(board.UARTE0, board.uart);
    for i in 0..1000 {
        println!("ding: {}", i);
    }
    panic!("deliberate panic");
}
