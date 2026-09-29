//! 05 — `add` wraps without a word; Rust's `+` checks.
//!
//! The `add` inside `span_end` (program 04) keeps the low 64 bits of the sum
//! and drops the carry. Nothing traps, and no flag is left behind:
//!
//!     0xFFFF_FFFF_FFFF_FFFF + 1  =  1_0000_0000_0000_0000
//!                                   ^ the 65th bit: gone
//!                                →  0
//!
//! Rust in a debug build checks every `+` and panics on overflow (program 13
//! shows the panic). When wrapping is what you mean, say so:
//! `wrapping_add`, or `checked_add` to get `None` instead.
//!
//! Run:  cargo run --bin 05_add_wraps
#![no_std]
#![no_main]

use core::arch::global_asm;
use core::hint::black_box;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

global_asm!(
    r#"
.globl span_end
span_end:                # a0 = *const Span
    ld   t0, 0(a0)       # t0 = start
    ld   t1, 8(a0)       # t1 = len
    add  a0, t0, t1      # a0 = start + len
    ret
"#
);

#[repr(C)]
pub struct Span {
    pub start: u64,
    pub len: u64,
}

extern "C" {
    fn span_end(s: *const Span) -> u64;
}

fn main() {
    let big = black_box(u64::MAX);

    println!("== the hardware add ==");
    let near = Span { start: big - 0x10, len: 0x10 };
    let over = Span { start: big, len: 1 };
    println!("span_end(start {:#x}, len {:#x}) = {:#x}", near.start, near.len, unsafe { span_end(&near) });
    println!("span_end(start {:#x}, len {:#x}) = {:#x}", over.start, over.len, unsafe { span_end(&over) });
    println!("no trap, no flag: the carry out of bit 63 is simply dropped");

    println!("\n== Rust, saying what it means ==");
    println!("big.wrapping_add(1) = {:#x}", big.wrapping_add(1));
    println!("big.checked_add(1)  = {:?}", big.checked_add(1));
    println!("big.checked_add(0)  = {:?}", big.checked_add(0));
    println!("big + 1 would panic here: 'attempt to add with overflow'");

    println!("\n== the same bits, signed ==");
    let edge = Span { start: black_box(i64::MAX as u64), len: 1 };
    let sum = unsafe { span_end(&edge) };
    println!("span_end(start i64::MAX, len 1) = {:#x} = {} as i64", sum, sum as i64);
    println!("one add for both: signed or not is how you read the bits");
}
