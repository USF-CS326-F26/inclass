//! 11 — `#![no_std]` takes away the OS, not the language.
//!
//! `std` is three layers, and a kernel keeps the bottom one:
//!
//!     std     needs an OS     println!, files, threads, HashMap
//!     alloc   needs a heap    Box, Vec, String
//!     core    needs nothing   Option, Result, slices, iterators, fmt, ptr
//!
//! Everything below runs with no OS, no heap and no allocator, on a stack
//! of 16 KiB. Even this program's `println!` is `core::fmt` writing bytes to
//! the UART; the runtime supplies the last step, and the rest is core's.
//!
//! Run:  cargo run --bin 11_what_core_still_has
#![no_std]
#![no_main]

use core::fmt::{self, Write};
use core::mem::size_of;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

/// Text formatted into a fixed array on the stack: `fmt::Write` for 48 bytes.
struct Line {
    buf: [u8; 48],
    len: usize,
}

impl Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let room = &mut self.buf[self.len..];
        if s.len() > room.len() {
            return Err(fmt::Error);
        }
        room[..s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

fn main() {
    println!("== slices, iterators, Option ==");
    let ticks = [7u32, 3, 9, 3, 5];
    println!("max       = {:?}", ticks.iter().max());
    println!("position  = {:?}", ticks.iter().position(|&t| t == 9));
    println!("sum of odd = {}", ticks.iter().filter(|&&t| t % 2 == 1).sum::<u32>());
    println!("first > 8 = {:?}", ticks.iter().find(|&&t| t > 8));
    println!("10 / 0    = {:?}", 10u32.checked_div(0));

    println!("\n== formatting, with no heap ==");
    let mut line = Line { buf: [0; 48], len: 0 };
    let r = write!(line, "hart {} at {:#010x}, {:>5} ticks", 0, 0x8000_0000u32, 1234);
    println!("{:?}: {:?}", r, core::str::from_utf8(&line.buf[..line.len]).unwrap());
    println!("{} of 48 bytes used", line.len);
    let r = write!(line, " and this sentence does not fit in what is left");
    println!("{r:?}: a full buffer is an error value, not a reallocation");

    println!("\n== sizes are compile-time facts ==");
    println!("size_of::<Option<&u8>>() = {}   the null pointer is None", size_of::<Option<&u8>>());
    println!("size_of::<Option<u8>>()  = {}   a u8 has no spare value", size_of::<Option<u8>>());
    println!("size_of::<&[u8]>()       = {}  pointer + length", size_of::<&[u8]>());

    println!("\n== what is not here ==");
    println!("Vec, String and Box live in alloc, which needs an allocator: this program has none");
    println!("broken/e0433_vec_needs_alloc.rs is what rustc says when you reach for one anyway");
}
