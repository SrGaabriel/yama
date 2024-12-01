#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write;

mod panic;
mod vga;
mod gpu;
mod mem;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print!("> ");

    let texts = vec!["Hello", "World", "Yama", "OS"];
    println!("{:?}", texts);

    loop {}
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}