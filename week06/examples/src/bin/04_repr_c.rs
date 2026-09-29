//! 04 — `#[repr(C)]` is what makes the assembly's offsets true.
//!
//! `span_end` is handed a pointer in a0 and reads two fields out of memory at
//! fixed offsets. Nothing checks those offsets against the struct:
//!
//!     #[repr(C)] struct Span { start: u64, len: u64 }
//!
//!     a0 ─► +0  start   ld t0, 0(a0)
//!           +8  len     ld t1, 8(a0)
//!
//! `repr(C)` promises declaration order and C's padding rules. Rust's default
//! layout promises nothing and may reorder fields to save space.
//!
//! Run:  cargo run --bin 04_repr_c
#![no_std]
#![no_main]

use core::arch::global_asm;
use core::mem::{offset_of, size_of};
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

/// Three fields that do not fit neatly: C order, then Rust's.
#[repr(C)]
struct SlotC {
    busy: u8,
    addr: u64,
    owner: u16,
}

struct SlotRust {
    busy: u8,
    addr: u64,
    owner: u16,
}

/// A field inserted ahead of `len`. Still 8-byte aligned, still compiles.
#[repr(C)]
struct Tagged {
    start: u64,
    tag: u8,
    len: u64,
}

fn main() {
    println!("== span_end reads offsets 0 and 8 ==");
    let s = Span { start: 0x8000_0000, len: 0x1000 };
    println!("span_end(&Span {{ start: {:#x}, len: {:#x} }}) = {:#x}", s.start, s.len,
             unsafe { span_end(&s) });
    println!("offset_of!(Span, start) = {}", offset_of!(Span, start));
    println!("offset_of!(Span, len)   = {}", offset_of!(Span, len));

    println!("\n== repr(C) keeps order; the default may not ==");
    println!("repr(C): busy at {}, addr at {}, owner at {}, size {}",
             offset_of!(SlotC, busy), offset_of!(SlotC, addr), offset_of!(SlotC, owner),
             size_of::<SlotC>());
    println!("default: busy at {}, addr at {}, owner at {}, size {}",
             offset_of!(SlotRust, busy), offset_of!(SlotRust, addr), offset_of!(SlotRust, owner),
             size_of::<SlotRust>());
    println!("C pads after busy so addr lands on 8; Rust moved busy to the end instead");

    println!("\n== insert a field ahead of len ==");
    // Zeroed first, so the padding after `tag` is 0 and the result is the same every run.
    let mut t: Tagged = unsafe { core::mem::zeroed() };
    (t.start, t.tag, t.len) = (0x8000_0000, 7, 0x1000);
    let wrong = unsafe { span_end(&t as *const Tagged as *const Span) };
    println!("offset_of!(Tagged, len) = {}, but span_end still reads 8", offset_of!(Tagged, len));
    println!("span_end = {wrong:#x}; it added the tag byte and its padding, not len");
    println!("the cast compiled, because a raw-pointer cast checks nothing");
}
