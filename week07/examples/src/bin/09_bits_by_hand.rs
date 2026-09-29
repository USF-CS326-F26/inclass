//! 09 — Shift, mask, extract, pack: four moves do every translation.
//!
//! A hex digit is four bits, so a shift by 4, 8 or 12 moves whole digits.
//! Friday's checks and the midterm's translations are all these four:
//!
//!     shift     0x8_1234 << 12               = 0x8123_4000
//!     mask      (1 << 9) - 1                 = 0x1FF, nine ones
//!     extract   (0x8123_4ABC >> 12) & 0x1FF  = 0x34
//!     pack      (0x5 << 4) | 0x3             = 0x53
//!
//! FAT file systems pack a date into 16 bits the same way. Packing trusts
//! each field to fit in its bits. Nothing checks, and a field that is too
//! big spills into its neighbor or off the top. Rust's operators are C's
//! with two changes: NOT is `!`, and `&` binds tighter than `==` and `!=`.
//!
//! Run:  cargo run --bin 09_bits_by_hand
#![no_std]
#![no_main]

use core::hint::black_box;
use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

// years since 1980 in bits 15..9, month in 8..5, day in 4..0
const fn pack_date(y: u16, m: u16, d: u16) -> u16 {
    (y << 9) | (m << 5) | d
}

const fn month(date: u16) -> u16 {
    (date >> 5) & 0xF
}

fn main() {
    let ppn = black_box(0x8_1234u64);
    let a = black_box(0x8123_4ABCu64);

    println!("== four moves ==");
    println!("shift    {} << 12              = {}", Hex(ppn), Hex(ppn << 12));
    println!("mask     (1 << 12) - 1 = {}   (1 << 10) - 1 = {}   (1 << 9) - 1 = {}",
             Hex((1 << 12) - 1), Hex((1 << 10) - 1), Hex((1 << 9) - 1));
    println!("extract  ({} >> 12) & 0x1FF = {}", Hex(a), Hex((a >> 12) & 0x1FF));
    println!("pack     (0x5 << 4) | 0x3            = {}", Hex((black_box(0x5) << 4) | 0x3));
    println!("clear    {} & !0xFFF        = {}", Hex(a), Hex(a & !0xFFF));
    println!("a shift by 4 moves one hex digit, and by 12 moves three: {} << 4 = {}",
             Hex(ppn), Hex(ppn << 4));

    println!("\n== a FAT date ==");
    let date = pack_date(black_box(46), 10, 9);
    println!("pack_date(46, 10, 9) = {}", Hex(date as u64));
    println!("in binary: {:07b} {:04b} {:05b}   year, month, day",
             date >> 9, month(date), date & 0x1F);
    println!("month({}) = {}", Hex(date as u64), month(date));
    println!("unmasked:  {} >> 5 = {}, the year still attached",
             Hex(date as u64), Hex((date >> 5) as u64));
    println!("year: date >> 9 = {}, so {}", date >> 9, 1980 + (date >> 9));
    println!("day:  date & 0x1F = {}", date & 0x1F);
    println!("slide the field down to bit 0, then keep only its width");

    println!("\n== nothing checks that a field fits ==");
    let spill = pack_date(46, 10, black_box(32));
    println!("pack_date(46, 10, 32) = {}: month {}, day {}",
             Hex(spill as u64), month(spill), spill & 0x1F);
    println!("32 is 0b10_0000, six bits: the sixth landed in the month");
    println!("October became November, and the day became 0");
    let late = pack_date(black_box(128), 1, 1);
    println!("pack_date(128, 1, 1)  = {}: year {}, so {}",
             Hex(late as u64), late >> 9, 1980 + (late >> 9));
    println!("128 needs eight bits: its top one went past bit 15 and off the u16");
    println!("a FAT date cannot say 2108; the last year it holds is 1980 + 127 = {}", 1980 + 127);

    println!("\n== Rust's operators are not C's ==");
    println!("!0xFFF = {}: Rust's bitwise NOT is `!`, and `~` does not exist in Rust", Hex(!0xFFF));
    let flags = black_box(0x6u64); // R and W set, V clear
    println!("flags = {}, so W (0x4) is set", Hex(flags));
    println!("Rust: flags & 0x4 != 0 is {}: it reads (flags & 0x4) != 0", flags & 0x4 != 0);
    println!("C:    flags & 0x4 != 0 is {}, false: it reads flags & (0x4 != 0), so {} & 1",
             flags & (0x4 != 0) as u64, Hex(flags));
    println!("C's != binds tighter than &, so C misses the W bit; with parentheses both agree");
    let one = black_box(1u16);
    println!("1u16.checked_shl(15) = {:?}, 1u16.checked_shl(16) = {:?}",
             one.checked_shl(15), one.checked_shl(16));
    println!("with overflow checks on, a u16 `<<` panics only for a shift of 16 or more");
    println!("bits pushed off the top, as in 2108 above, vanish without a word");
}
