//! 10 — `unsafe` unlocks five operations and turns nothing off.
//!
//! Inside an `unsafe` block you may dereference a raw pointer, call an
//! `unsafe fn` (every `extern "C"` function is one), touch a `static mut`,
//! implement an `unsafe trait`, and access a union field. That is the whole list. The
//! borrow checker, the type checker and bounds checks all still run.
//!
//! The usual shape is a small `unsafe` core inside a safe function whose
//! signature makes the promise the caller can no longer get wrong:
//!
//!     pub fn end_of(s: &Span) -> u64      a &Span is live, aligned, 16 bytes
//!         unsafe { span_end(s) }          …so this call is always fine
//!
//! Run:  cargo run --bin 10_unsafe_does_not_turn_off
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
    ld   t0, 0(a0)
    ld   t1, 8(a0)
    add  a0, t0, t1
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

/// Safe to call: the reference is the proof the assembly needs.
pub fn end_of(s: &Span) -> u64 {
    // promise: s is live, aligned, and 16 bytes long
    unsafe { span_end(s) }
}

static mut BOOTS: u32 = 0;

fn main() {
    println!("== a safe wrapper ==");
    let s = Span { start: 0x8020_0000, len: 0x2000 };
    println!("end_of(&s) = {:#x}   no unsafe at the call site", end_of(&s));
    println!("the one unsafe block is inside end_of, where the promise is written");

    println!("\n== bounds checks still run ==");
    let table = [10u32, 20, 30];
    let i = black_box(7);
    let got = unsafe { table.get(i) };
    println!("inside unsafe, table.get({i}) = {got:?}");
    println!("rustc even warns that this block is unnecessary: press e to see it");
    println!("table[{i}] would still panic; get_unchecked({i}) is the unsafe way to skip");
    println!("the check, and with 7 it would read past the end: undefined behavior");

    println!("\n== a static mut needs unsafe for every touch ==");
    unsafe {
        BOOTS += 1;
        BOOTS += 1;
    }
    let n = unsafe { BOOTS };
    println!("BOOTS = {n}");
    println!("the compiler cannot see who else touches a global, so each access is yours to justify");

    println!("\n== types and borrows still apply ==");
    println!("broken/e0308_unsafe_keeps_types.rs: a u64 is still not a u32 inside unsafe");
    println!("broken/e0133_raw_deref_needs_unsafe.rs: and outside it, *p does not compile at all");
}
