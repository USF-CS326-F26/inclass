//! 12 — `#![no_main]`: nobody generates a `main`, so something else must be first.
//!
//! On a host, the OS starts a program at a C runtime entry that std provides,
//! which calls `main`. Here there is no OS and no runtime from std, so the
//! program says where to begin itself:
//!
//!     link.ld       ENTRY(_entry), and .entry first at 0x8000_0000
//!     entry!(main)  emits _entry: sp = __stack_top, then __week06_start
//!     __week06_start  calls main(), then powers off
//!
//! `main` is an ordinary function here. It could be called anything.
//!
//! Run:  cargo run --bin 12_who_calls_main
#![no_std]
#![no_main]

use core::arch::asm;
use core::ptr::addr_of;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

extern "C" {
    fn _entry();
    static etext: u8;
    static end: u8;
    static __stack_top: u8;
}

fn main() {
    // Read before main calls anything: a call would overwrite ra.
    let ra: usize;
    unsafe { asm!("mv {}, ra", out(reg) ra) };

    println!("== where this program lives ==");
    let start = __week06_start as *const () as usize;
    println!("_entry          {:#x}   the first instruction QEMU ran", _entry as *const () as usize);
    println!("__week06_start  {start:#x}");
    println!("main            {:#x}", main as fn() as usize);
    println!("etext           {:p}   end of code", addr_of!(etext));
    println!("end             {:p}   end of data", addr_of!(end));
    println!("__stack_top     {:p}   16 KiB above end", addr_of!(__stack_top));

    println!("\n== who called main ==");
    println!("main's ra = {ra:#x} = __week06_start + {:#x}", ra - start);
    println!("no generated main, no C runtime: a function in this program called it");

    println!("\n== after main returns ==");
    println!("__week06_start calls week06::exit(0), and the test finisher ends QEMU");
    println!("without it there is nothing to return to: the CPU would run into whatever is next");
}
