//! 01 — Scalar types: what a value *is* in memory.
//!
//! Every scalar has a size the compiler knows at compile time. Nothing is
//! boxed, nothing has a header, and nothing is scanned at run time.
//!
//!     let x: u8 = 200;      stack:  [ 11001000 ]                 1 byte
//!     let y: u32 = 200;     stack:  [ 11001000 00000000 x2 ]     4 bytes
//!
//! Run:  cargo run --bin 01_scalars

use std::mem::size_of;

fn main() {
    println!("== integers ==");
    // Signed and unsigned, 8 through 64 bits, plus the pointer-sized pair.
    println!("u8    {} byte  0 ..= {}", size_of::<u8>(), u8::MAX);
    println!("i32   {} bytes {} ..= {}", size_of::<i32>(), i32::MIN, i32::MAX);
    println!("u64   {} bytes 0 ..= {}", size_of::<u64>(), u64::MAX);
    // usize is as wide as an address on this machine: 8 bytes on x86-64 and
    // on riscv64. This is the type an OS uses for addresses and lengths.
    println!("usize {} bytes (one machine address)", size_of::<usize>());

    println!("\n== literals are the same bits, written differently ==");
    let pgsize_dec = 4096;
    let pgsize_hex = 0x1000;
    let pgsize_bin = 0b1_0000_0000_0000;
    let pgsize_shift = 1 << 12;
    println!(
        "{pgsize_dec} == {pgsize_hex} == {pgsize_bin} == {pgsize_shift}  -> {}",
        pgsize_dec == pgsize_hex && pgsize_hex == pgsize_bin && pgsize_bin == pgsize_shift
    );
    // The underscore is nothing at all — just a reading aid.
    let big = 0x8000_0000usize;
    println!("0x8000_0000 = {big} = {big:#x} = {big:#b}");

    println!("\n== no implicit conversion, ever ==");
    let small: u8 = 200;
    let wide: u32 = small as u32 + 100; // `as` is the explicit request
    println!("200u8 as u32 + 100 = {wide}");
    // The reverse truncates, and Rust makes you say so out loud:
    println!("300u32 as u8 = {}  (bits above 8 are gone)", 300u32 as u8);

    println!("\n== overflow is not undefined behavior ==");
    let x: u8 = 255;
    // Debug builds panic on overflow; release builds wrap. Neither is silent
    // corruption, and neither is UB. Say which one you want:
    println!("255u8.checked_add(1)   = {:?}", x.checked_add(1));
    println!("255u8.wrapping_add(1)  = {}", x.wrapping_add(1));
    println!("255u8.saturating_add(1)= {}", x.saturating_add(1));
    println!("255u8.overflowing_add(1)= {:?}", x.overflowing_add(1));

    println!("\n== the other scalars ==");
    let ready: bool = true;                 // 1 byte
    let mode: char = 'S';                   // 4 bytes: a Unicode scalar value
    let ratio: f64 = 4096.0 / 3.0;          // 8 bytes, IEEE-754
    println!("bool {} ({} byte)   char {mode:?} ({} bytes)   f64 {ratio:.3} ({} bytes)",
             ready, size_of::<bool>(), size_of::<char>(), size_of::<f64>());

    println!("\n== a u64 that is really 64 flags: an SV39 page-table entry ==");
    // V=1 R=1 W=1, PPN = physical page 0x8_0005
    let pte: u64 = (0x8_0005 << 10) | 0b111;
    println!("pte      = {pte:#018x}");
    println!("valid    = {}", pte & 1 == 1);
    println!("readable = {}", (pte >> 1) & 1 == 1);
    println!("writable = {}", (pte >> 2) & 1 == 1);
    println!("ppn      = {:#x} -> phys addr {:#x}", pte >> 10, (pte >> 10) << 12);
}
