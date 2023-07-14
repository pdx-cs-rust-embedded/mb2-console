# mb2-serial: serial console for the Micro::Bit v2 in Rust
Bart Massey 2023

This is a Rust "serial console" for the BBC Micro::Bit v2. It
allows `print!` and `println!` statements and prints panic
messages from `no_std` Micro::Bit v2 Rust code.

Use this crate with

```rust
use mb2_console::{init_serial, print, println};
```

and initialize at start of execution with

```rust
init_serial(board.UARTE0, board.uart);
```

You can then `print!()`, `println!()` and take `panic!()`
and the output will go to the USB serial port on the
Micro::Bit v2, where you can observe it with a terminal
program.

# License

This work is licensed under the "MIT License". Please see the file
`LICENSE.txt` in this distribution for license terms.
