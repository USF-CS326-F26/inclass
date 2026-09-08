//! 06 — repr(C): when something other than Rust reads your struct.
//!
//! By default a struct has representation `repr(Rust)`, and the language
//! promises NOTHING about layout: not field order, not padding, not that two
//! identical declarations agree. The compiler may sort fields to reduce
//! padding, and it does.
//!
//!     #[repr(C)]  gives that freedom up and uses C's rules: fields in source
//!     order, each at the next offset satisfying its alignment.
//!
//! You need it exactly when something that is not the Rust compiler reads the
//! bytes -- assembly, hardware, a C library, or a saved disk image.
//!
//!     repr(C) Header             the assembly that indexes it
//!     +----------------------+
//!   0 | flag  u8             |   sb  a1, 0(a0)
//!     | (7 bytes of padding) |
//!   8 | addr  u64            |   sd  a2, 8(a0)
//!  16 | count u16            |   sh  a3, 16(a0)
//!     | (6 bytes of padding) |
//!     +----------------------+   24 bytes
//!
//! Run:  cargo run --bin 06_repr_c

use std::mem::{align_of, offset_of, size_of};

fn main() {
    println!("== the same three fields, two representations ==");
    println!("size_of::<RustHeader>() = {:>2}   align {}",
             size_of::<RustHeader>(), align_of::<RustHeader>());
    println!("size_of::<CHeader>()    = {:>2}   align {}",
             size_of::<CHeader>(), align_of::<CHeader>());
    println!("declared identically; only the attribute differs");

    println!("\n== where the fields actually landed ==");
    println!("            repr(Rust)   #[repr(C)]");
    println!("  flag       offset {:>2}   offset {:>2}",
             offset_of!(RustHeader, flag), offset_of!(CHeader, flag));
    println!("  addr       offset {:>2}   offset {:>2}",
             offset_of!(RustHeader, addr), offset_of!(CHeader, addr));
    println!("  count      offset {:>2}   offset {:>2}",
             offset_of!(RustHeader, count), offset_of!(CHeader, count));
    println!("the left column is today's answer on this compiler, and a");
    println!("promise about nothing; the right column is a contract");

    println!("\n== why the C rules cost bytes ==");
    println!("`addr` is 8-aligned, so it cannot start at offset 1:");
    println!("  flag at 0, then 7 bytes of padding, then addr at 8,");
    println!("  count at 16, then 6 more bytes so the whole struct is 8-aligned");
    println!("repr(Rust) is free to put the most-aligned field first instead");

    println!("\n== order the fields yourself and the padding goes away ==");
    println!("size_of::<PackedWell>() = {}   (#[repr(C)], big fields first)",
             size_of::<PackedWell>());
    println!("  addr  offset {}", offset_of!(PackedWell, addr));
    println!("  count offset {}", offset_of!(PackedWell, count));
    println!("  flag  offset {}", offset_of!(PackedWell, flag));

    println!("\n== a struct assembly reads by numeric offset ==");
    let ctx = SavedRegs { ra: 0x8000_1234, sp: 0x8010_0000, s0: 0, s1: 0 };
    println!("size_of::<SavedRegs>() = {}   = 4 x 8", size_of::<SavedRegs>());
    println!("  ra offset {}   <- `ld ra, 0(a1)`", offset_of!(SavedRegs, ra));
    println!("  sp offset {}   <- `ld sp, 8(a1)`", offset_of!(SavedRegs, sp));
    println!("  s0 offset {}", offset_of!(SavedRegs, s0));
    println!("  s1 offset {}", offset_of!(SavedRegs, s1));
    println!("ra = {:#x}, sp = {:#x}", ctx.ra, ctx.sp);

    println!("\n== this failure is silent at compile time ==");
    println!("Delete #[repr(C)] from SavedRegs and NOTHING breaks in the build.");
    println!("The assembly is still valid; it may simply store the return");
    println!("address into whatever field now sits at offset 0. The first");
    println!("symptom is a jump to a garbage address on the next switch.");
    println!("Worse: all four fields are u64, so today's compiler has no");
    println!("reason to reorder them and it would probably keep working.");
    println!("The bug is not that it breaks -- it is that nothing PROMISES");
    println!("it will not.");

    println!("\n== the three you will meet ==");
    println!("  repr(Rust)             none; optimize freely     everything internal");
    println!("  #[repr(C)]             source order, C alignment assembly, hardware, disk");
    println!("  #[repr(transparent)]   identical to the one field newtypes (program 04)");
    println!("size_of::<Wrapped>() = {}   == size_of::<u64>() = {}",
             size_of::<Wrapped>(), size_of::<u64>());
}

/// Rust may lay these out in any order it likes.
pub struct RustHeader {
    pub flag: u8,
    pub addr: u64,
    pub count: u16,
}

/// Exactly the same fields, in source order, with C's padding.
#[repr(C)]
pub struct CHeader {
    pub flag: u8,
    pub addr: u64,
    pub count: u16,
}

/// `#[repr(C)]` fixes the order -- so pick a good one.
#[repr(C)]
pub struct PackedWell {
    pub addr: u64,
    pub count: u16,
    pub flag: u8,
}

/// The registers a context switch saves. Assembly reads these by byte offset.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SavedRegs {
    pub ra: u64, // return address  -- offset 0
    pub sp: u64, // stack pointer   -- offset 8
    pub s0: u64, // saved register  -- offset 16
    pub s1: u64, // saved register  -- offset 24
}

/// The third representation: a newtype with the exact layout of its field.
#[repr(transparent)]
pub struct Wrapped(pub u64);
