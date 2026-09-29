//! 02 — Rust calls assembly through `extern "C"`, and the signature is a promise.
//!
//! `global_asm!` puts `find_byte` in the program, `.globl` makes its name
//! visible to the linker, and the `extern "C"` block tells Rust how to call
//! it: arguments in a0, a1, …, the result in a0.
//!
//!     Rust                          find_byte
//!     a0 = s ──────────────────►   1: lbu  t0, 0(a0)     load a byte
//!     a1 = b ──────────────────►      beq  t0, a1, 2f    found
//!                                     beqz t0, 2f        the NUL
//!                                     addi a0, a0, 1
//!                                     j    1b
//!     a0 ◄──────────────────────   2: ret
//!
//! A leaf: it calls nothing, so `ra` survives, and it uses only `a` and `t`
//! registers, so it needs no stack frame.
//!
//! Run:  cargo run --bin 02_calling_assembly
#![no_std]
#![no_main]

use core::arch::global_asm;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

global_asm!(
    r#"
.globl find_byte
find_byte:               # a0 = string, a1 = byte wanted
1:  lbu  t0, 0(a0)       # load a byte
    beq  t0, a1, 2f      # found: exit
    beqz t0, 2f          # the NUL: exit too
    addi a0, a0, 1       # next byte
    j    1b              # back to the top
2:  ret                  # a0 = match or NUL
"#
);

extern "C" {
    /// SAFETY, for the caller: `s` points at readable bytes that end in a 0.
    fn find_byte(s: *const u8, b: u8) -> *const u8;
}

/// How far into `s` find_byte stopped.
fn stop(s: &[u8], b: u8) -> usize {
    let at = unsafe { find_byte(s.as_ptr(), b) };
    at as usize - s.as_ptr() as usize
}

fn main() {
    let s = b"kernel, meet user\0";

    println!("== found ==");
    println!("find_byte(s, b',') stopped at {}", stop(s, b','));
    println!("find_byte(s, b'u') stopped at {}", stop(s, b'u'));
    println!("the first match wins: s[{}] is the 'e' in \"kernel\"", stop(s, b'e'));

    println!("\n== not there: it stops at the NUL ==");
    let at = stop(s, b'z');
    println!("find_byte(s, b'z') stopped at {at}, and s[{at}] = {}", s[at]);
    println!("the 0 is the only way it knows where the bytes end");

    println!("\n== what crossed the boundary ==");
    println!("a0 in:  {:p}   the pointer, 8 bytes", s.as_ptr());
    println!("a1 in:  {:#x}         the byte, zero-extended into a register", b'u');
    println!("a0 out: a pointer again, and Rust trusts that it is one");
}
