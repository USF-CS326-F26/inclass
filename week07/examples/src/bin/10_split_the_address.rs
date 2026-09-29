//! 10 — An Sv39 address is three 9-bit indices over a 12-bit offset.
//!
//! Sv39 reads the low 39 bits, and the bits above must copy bit 38. The
//! offset is copied across untranslated; the 27 bits above it pick one entry
//! at each level of the tree:
//!
//!      38      30 29      21 20      12 11        0
//!     +----------+----------+----------+----------+
//!     |  VPN[2]  |  VPN[1]  |  VPN[0]  |  offset  |
//!     +----------+----------+----------+----------+
//!        1 GiB      2 MiB      4 KiB   byte in page  what one entry covers
//!
//! VPN[i] is `(va >> (12 + 9 * i)) & 0x1FF`: shift first, then mask. Nine
//! bits, because one table is one page: 512 entries of 8 bytes.
//!
//! Run:  cargo run --bin 10_split_the_address
#![no_std]
#![no_main]

use core::hint::black_box;
use core::ptr::addr_of;
use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

extern "C" {
    static end: u8;
}

static GREETING: [u8; 8] = *b"hello, 7";

/// One row of a table: an address, then its three indices and its offset.
fn row(what: &str, va: u64) {
    println!("{what:<14} {:>19}   {:>6} {:>6} {:>6}   {:.3}",
             Hex(va), (va >> 30) & 0x1FF, (va >> 21) & 0x1FF, (va >> 12) & 0x1FF, Hex(va & 0xFFF));
}

fn main() {
    let va = black_box(0x20_00A0_35F0u64);

    println!("== the lecture's address, by hand ==");
    println!("va         = {}", Hex(va));
    println!("va >> 12   = {:>11}   & 0x1FF = {:>3}   VPN[0]", Hex(va >> 12), (va >> 12) & 0x1FF);
    println!("va >> 21   = {:>11}   & 0x1FF = {:>3}   VPN[1]", Hex(va >> 21), (va >> 21) & 0x1FF);
    println!("va >> 30   = {:>11}   & 0x1FF = {:>3}   VPN[2]", Hex(va >> 30), (va >> 30) & 0x1FF);
    println!("va & 0xFFF = {:>11}                   offset", Hex(va & 0xFFF));
    let (v2, v1, v0) = ((va >> 30) & 0x1FF, (va >> 21) & 0x1FF, (va >> 12) & 0x1FF);
    let off = va & 0xFFF;
    let back = (v2 << 30) | (v1 << 21) | (v0 << 12) | off;
    println!("({v2} << 30) | ({v1} << 21) | ({v0} << 12) | {} = {}", Hex(off), Hex(back));
    println!("each index is nine bits, 0 to 511; the pieces add back up to the address");

    println!("\n== what one entry covers ==");
    println!("an entry at level 0: 1 << 12 = {:>10} bytes, 4 KiB", 1u64 << 12);
    println!("an entry at level 1: 1 << 21 = {:>10} bytes, 2 MiB = 512 x 4 KiB", 1u64 << 21);
    println!("an entry at level 2: 1 << 30 = {:>10} bytes, 1 GiB = 512 x 2 MiB", 1u64 << 30);
    println!("{:<14} {:>19}   VPN[2] VPN[1] VPN[0]   offset", "", "address");
    row("va", va);
    row("va + 4 KiB", va + (1 << 12));
    row("va + 2 MiB", va + (1 << 21));
    row("va + 1 GiB", va + (1 << 30));
    println!("each step moves one index by one, and the offset never moves");
    let flat = (1u64 << 27) * 8;
    println!("a flat table for all 2^27 pages: 2^27 x 8 bytes = {} GiB, per process", flat >> 30);
    println!("a tree has tables only where something is mapped: 12 KiB for a first page");

    println!("\n== this program's own addresses ==");
    let local = black_box(0u64);
    println!("{:<14} {:>19}   VPN[2] VPN[1] VPN[0]   offset", "", "address");
    row("main", main as fn() as usize as u64);
    row("a static", addr_of!(GREETING) as u64);
    row("end", addr_of!(end) as u64);
    row("a local", &local as *const u64 as u64);
    row("last RAM byte", 0x87FF_FFFF);
    println!("RAM starts at 0x8000_0000, which is 2 GiB: all of it is root entry 2");
    println!("128 MiB of RAM is 64 x 2 MiB, so VPN[1] runs from 0 to 63 across it");
    let leaves = (128u64 << 20) >> 12;
    println!("mapped with 4 KiB pages: {leaves} leaves, in {} level-0 tables of 512", leaves / 512);
    println!("add one level-1 table and the root: {} pages of tables", leaves / 512 + 2);
    println!("rv6's kernel will map RAM to itself, so these are its virtual addresses too");

    println!("\n== the top: MAXVA ==");
    let maxva = black_box(1u64 << 38);
    println!("MAXVA = 1 << 38 = {}: one bit short of 39", Hex(maxva));
    println!("{:<14} {:>19}   VPN[2] VPN[1] VPN[0]   offset", "", "address");
    row("TRAMPOLINE", maxva - 0x1000);
    row("MAXVA", maxva);
    println!("TRAMPOLINE is the top page below MAXVA: root entry 255, the last below bit 38");
    println!("MAXVA itself would be root entry 256: entries 256 to 511 all have bit 38 set");
    let copied = ((maxva << 25) as i64 >> 25) as u64;
    println!("Sv39 wants bits 63..39 to copy bit 38, so 1 << 38 is legal only as {}", Hex(copied));
    println!("rv6 stays below MAXVA, so no address it uses needs sign extension");
}
