//! 09 — Without `volatile`, the compiler may merge, drop, or hoist device accesses.
//!
//! To the optimizer, a plain `*p = x` is a store to memory, and memory only
//! remembers the last store. A plain `*p` is a load, and memory does not
//! change unless this program changes it. Both are false for a device:
//!
//!     plain:     *UART = b'A'; *UART = b'B';   →  one store: "B"
//!     volatile:  write_volatile ×2             →  two stores: "AB"
//!
//!     plain:     loop { t = *MTIME; … }        →  one load, hoisted out
//!     volatile:  loop { read_volatile(MTIME) } →  a load every time around
//!
//! This crate builds its dev profile at opt-level 1 (see Cargo.toml), so what
//! you see is what an optimized kernel would do. At opt-level 0 the plain
//! versions happen to work, and the bug waits for a release build.
//!
//! Run:  cargo run --bin 09_volatile_matters
#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};
use week06::{print, println};

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

const UART: *mut u8 = 0x1000_0000 as *mut u8;
const MTIME: *const u64 = 0x0200_BFF8 as *const u64;

fn plain_pair() {
    unsafe {
        *UART = b'A';
        *UART = b'B';
    }
}

fn volatile_pair() {
    unsafe {
        write_volatile(UART, b'A');
        write_volatile(UART, b'B');
    }
}

/// How many different values `n` plain loads of the clock saw.
fn plain_distinct(n: u32) -> u32 {
    let (mut seen, mut last) = (0, u64::MAX);
    for _ in 0..n {
        let t = unsafe { *MTIME };
        if t != last {
            seen += 1;
            last = t;
        }
    }
    seen
}

/// The same loop, with every load kept.
fn volatile_distinct(n: u32) -> u32 {
    let (mut seen, mut last) = (0, u64::MAX);
    for _ in 0..n {
        let t = unsafe { read_volatile(MTIME) };
        if t != last {
            seen += 1;
            last = t;
        }
    }
    seen
}

fn main() {
    println!("== two plain stores to the UART ==");
    print!("the UART received: ");
    plain_pair();
    println!();
    println!("the first store was dead to the optimizer: the second overwrites it");

    println!("\n== two volatile stores ==");
    print!("the UART received: ");
    volatile_pair();
    println!();
    println!("both bytes, in order: each volatile store happens, exactly as written");

    println!("\n== 100,000 plain loads of mtime ==");
    let n = plain_distinct(100_000);
    println!("distinct values seen: {n}");
    println!("the load was hoisted out of the loop: the clock was read once");

    println!("\n== 100,000 volatile loads ==");
    let n = volatile_distinct(100_000);
    println!("distinct values seen: {n}");
    println!("every trip around the loop asked the device again");
}
