//! 11 — A PTE is a page number at bit 10, above ten bits of flags.
//!
//! An entry stores the physical page number, not the address. A page's low
//! 12 bits are always 0, so the entry drops them and spends the room on flags:
//!
//!      63    54 53             10 9 8 7 6 5 4 3 2 1 0
//!     +--------+-----------------+---+-+-+-+-+-+-+-+-+
//!     |reserved|  PPN (44 bits)  |RSW|D|A|G|U|X|W|R|V|
//!     +--------+-----------------+---+-+-+-+-+-+-+-+-+
//!
//! Valid with R, W and X all clear is a branch: its PPN names the next
//! table. Valid with any of them set is a leaf: its PPN names the page.
//! Packing shifts right by 12 and left by 10, and the two do not cancel.
//!
//! rv6 wraps an entry in a `#[repr(transparent)]` newtype: its own type, and
//! only its integer in memory. The last section shows that on the lecture's
//! framebuffer Pixel.
//!
//! Run:  cargo run --bin 11_pte_by_hand
#![no_std]
#![no_main]

use core::fmt;
use core::hint::black_box;
use core::mem::{align_of, size_of};
use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

const V: u64 = 1 << 0;
const R: u64 = 1 << 1;
const W: u64 = 1 << 2;
const X: u64 = 1 << 3;

/// The lecture's framebuffer pixel: its own type, laid out exactly as a u32.
#[repr(transparent)]
struct Pixel(u32);

/// An entry's low eight bits, shown as the letters of the flags that are set.
struct Flags(u64);

// fold: prints the letters V R W X U G A D, V first, for each bit that is set
impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [b' '; 16];
        let mut n = 0;
        for (bit, &name) in b"VRWXUGAD".iter().enumerate() {
            if self.0 >> bit & 1 == 1 {
                buf[n] = name;
                n += 2;
            }
        }
        f.pad(if n == 0 { "none" } else { core::str::from_utf8(&buf[..n - 1]).unwrap() })
    }
}

/// Leaf or branch comes from V and R, W, X alone, never from the level.
fn kind(entry: u64) -> &'static str {
    if entry & V == 0 {
        "invalid: V is clear, and no other bit means anything"
    } else if entry & (R | W | X) == 0 {
        "a branch: the address is the next table"
    } else {
        "a leaf: the address is the mapped page"
    }
}

fn main() {
    println!("== encode 0x8123_4000 as V R W ==");
    let pa = black_box(0x8123_4000u64);
    let ppn = pa >> 12;
    println!("pa          {:.8}", Hex(pa));
    println!("pa >> 12    {:.8}   the PPN: three hex digits dropped", Hex(ppn));
    println!("PPN << 8    {:.8}   two zero digits appended", Hex(ppn << 8));
    println!("times 4     {:.8}   that makes << 10", Hex((ppn << 8) * 4));
    println!("| V R W     {:.8}   the flags, 1 + 2 + 4 = {}",
             Hex((ppn << 10) | (V | R | W)), V | R | W);
    println!("the PPN now starts at bit 10, and the flags fill bits 9..0");

    println!("\n== decode it, reading upward ==");
    let e = black_box(0x2048_D007u64);
    println!("entry          {:.8}", Hex(e));
    println!("entry & 0x3FF  {:.3}         flags {}", Hex(e & 0x3FF), Flags(e));
    println!("entry >> 10    {:.8}   the PPN", Hex(e >> 10));
    println!("PPN << 12      {:.8}   the address", Hex((e >> 10) << 12));
    println!("R and W are set, so this is {}", kind(e));
    println!("{:<12} {:<6} {:<12} kind", "entry", "flags", "address");
    for entry in [(e & !0x3FF) | V, 0x21FF_F401, 0x2014_8C07, 0] {
        let to = (entry >> 10) << 12;
        println!("{:<12.8} {:<6} {:<12.8} {}", Hex(entry), Flags(entry), Hex(to), kind(entry));
    }
    println!("low digit 1, V alone: the same PPN is now a table, not a page");
    println!("the middle two are from the lecture's walk: level 1's entry 5, level 0's entry 3");

    println!("\n== two slips ==");
    println!("(entry >> 12) << 12 = {:.8}   page-aligned, plausible, and wrong",
             Hex((e >> 12) << 12));
    println!("(entry >> 10) << 12 = {:.8}   right: the PPN starts at bit 10", Hex((e >> 10) << 12));
    let odd = black_box(0x8123_4ABCu64);
    println!("{:<12} {:>18} {:>13}", "pa", "(pa >> 12) << 10", "pa >> 2");
    for p in [pa, odd] {
        println!("{:<12.8} {:>18.8} {:>13.8}", Hex(p), Hex((p >> 12) << 10), Hex(p >> 2));
    }
    println!("for an aligned pa the two agree, so pa >> 2 looks like a shortcut");
    let low = (odd >> 2) & 0x3FF;
    println!("unaligned, (pa >> 2) & 0x3FF = {:.3}: flags {}, and RSW = {}",
             Hex(low), Flags(low), low >> 8);
    println!("the offset turned into flags, W and X among them");
    println!(">> 12 drops the offset first, and only then does << 10 make room for flags");

    println!("\n== a newtype is only its integer ==");
    let fb = 0x8600_0000 as *mut Pixel; // free RAM, far above this program
    let row1 = unsafe { fb.add(640) };
    println!("fb = {}, fb.add(640) = {}: 640 pixels along is {} bytes",
             Hex(fb as u64), Hex(row1 as u64), row1 as usize - fb as usize);
    unsafe { row1.write_volatile(Pixel(0x00C0_FFEE)) };
    let as_u32 = unsafe { (row1 as *const u32).read_volatile() };
    let as_pixel = unsafe { row1.read_volatile() };
    println!("stored Pixel(0x00C0_FFEE) at fb.add(640)");
    println!("read back as a u32:   {:.8}", Hex(as_u32 as u64));
    println!("read back as a Pixel: {:.8}, its .0", Hex(as_pixel.0 as u64));
    println!("size_of::<Pixel>() = {}, align_of::<Pixel>() = {}: exactly a u32's",
             size_of::<Pixel>(), align_of::<Pixel>());
    println!("on 8-byte entries, .add(511) is 511 x 8 = {} bytes in: a table's last entry",
             511 * 8);
}
