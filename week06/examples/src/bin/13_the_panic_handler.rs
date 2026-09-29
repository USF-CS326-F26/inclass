//! 13 — Every `no_std` program says what a panic does, exactly once.
//!
//! With std, a panic prints a message and unwinds or aborts, and std wrote
//! that code. Without it, the program must supply one function with this
//! signature, marked `#[panic_handler]`:
//!
//!     fn(&PanicInfo) -> !       ! : it never returns, because there is
//!                               nothing sensible to return to
//!
//! Every other program here hands the report to `week06::report_panic`. This
//! one writes the handler out: print where and why, then power off with a
//! failure status. rv6's does the same with `OSLINGS:FAIL (panic)`.
//!
//! Run:  cargo run --bin 13_the_panic_handler
//! Exit: 1 (the handler powers QEMU off with a failure status, on purpose)
#![no_std]
#![no_main]

use core::hint::black_box;
use core::panic::PanicInfo;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[panic handler] a panic, and this function is all that is left");
    if let Some(at) = info.location() {
        println!("[panic handler] at   {}:{}", at.file(), at.line());
    }
    println!("[panic handler] why  {}", info.message());
    week06::exit(1)
}

fn add_ticks(a: u8, b: u8) -> u8 {
    a + b // overflow checks are on in a dev build
}

fn main() {
    println!("== an ordinary call ==");
    println!("add_ticks(200, 50) = {}", add_ticks(200, black_box(50)));

    println!("\n== one that overflows ==");
    println!("add_ticks(200, 60): 260 does not fit in a u8");
    let n = add_ticks(200, black_box(60));
    println!("unreachable: {n}");
}
