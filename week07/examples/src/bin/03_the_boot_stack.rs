//! 03 — The stack is 16 KiB of plain RAM: `sp` starts at the top, and nothing guards the bottom.
//!
//! Before any Rust ran, the runtime's `_entry` pointed `sp` at `__stack_top`.
//! Every call since has moved `sp` down, and every return has moved it back:
//!
//!     __stack_top   ← sp started here, the high end
//!         │          the runtime's frame, then main's, then each callee's
//!         ▼
//!     end           ← the low end: the stack's own lowest byte
//!     .bss          ← below it, somebody else's data
//!
//! rv6 keeps its boot stack as a static array in the image, below `end`,
//! because 32k hands out the pages above `end`. This runtime hands out
//! nothing, so link.ld puts the stack above `end`. Either way `sp` starts at
//! the high end.
//!
//! Run:  cargo run --bin 03_the_boot_stack
#![no_std]
#![no_main]

use core::arch::asm;
use core::hint::black_box;
use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
use week07::println;

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

extern "C" {
    static end: u8;
    static __stack_top: u8;
}

const PAINT: u64 = 0x5a5a_5a5a_5a5a_5a5a;

/// The last thing in .bss, so it ends where the stack begins: at `end`.
static mut FLOOR: [u64; 64] = [0; 64];

#[inline(always)]
fn sp() -> usize {
    let sp;
    unsafe { asm!("mv {}, sp", out(reg) sp, options(nomem, nostack)) };
    sp
}

/// Records `sp` at each depth. The work after the call keeps every frame alive.
#[inline(never)]
fn descend(depth: usize, trail: &mut [usize]) {
    trail[depth] = sp();
    if depth + 1 < trail.len() {
        descend(depth + 1, trail);
    }
    black_box(depth);
}

/// Calls itself until `sp` is below `target`, then reports the deepest `sp`.
#[inline(never)]
fn dig(target: usize) -> usize {
    let here = sp();
    if here < target {
        return here;
    }
    black_box(dig(target))
}

// fold: fills the unused stack, from `end` up to just below the caller, with PAINT
#[inline(never)]
fn paint() {
    let mut at = addr_of!(end) as usize;
    while at < sp() - 64 {
        unsafe { write_volatile(at as *mut u64, PAINT) };
        at += 8;
    }
}

// fold: the lowest stack address written since paint(): the first word that changed
#[inline(never)]
fn lowest_touched() -> usize {
    let mut at = addr_of!(end) as usize;
    while unsafe { read_volatile(at as *const u64) } == PAINT {
        at += 8;
    }
    at
}

fn main() {
    println!("== where sp starts ==");
    let top = addr_of!(__stack_top) as usize;
    let bottom = addr_of!(end) as usize;
    let here = sp();
    println!("__stack_top = {top:#x}   the high end, where _entry pointed sp");
    println!("end         = {bottom:#x}   the low end, {} bytes below", top - bottom);
    println!("sp in main  = {here:#x}   {} bytes down: the runtime's frame and main's", top - here);
    println!("sp % 16     = {}   the calling convention keeps sp 16-byte aligned", here % 16);

    println!("\n== each call moves it down ==");
    let mut trail = [0; 5];
    descend(0, &mut trail);
    for (depth, at) in trail.iter().enumerate() {
        println!("descend depth {depth}: sp = {at:#x}   {} bytes below main's", here - at);
    }
    let frame = trail[0] - trail[1];
    println!("each call claimed {frame} bytes for its frame; each return gave them back");

    println!("\n== what 16 KiB buys ==");
    paint();
    println!("the free stack below main is painted with {PAINT:#x}");
    let deepest = lowest_touched();
    println!("printing that line wrote as deep as {} bytes below main's sp", here - deepest);
    let room = here - bottom;
    println!("room left below main: {room} bytes = {} more calls of descend", room / frame);

    println!("\n== nothing guards the low end ==");
    let floor = addr_of_mut!(FLOOR) as *mut u64;
    for i in 0..64 {
        unsafe { write_volatile(floor.add(i), PAINT) };
    }
    println!("FLOOR, 512 bytes of .bss, ends at end: {}", floor as usize + 512 == bottom);
    let low = dig(bottom - 128);
    println!("dig called itself until sp = {low:#x}, {} bytes below end", bottom - low);
    let changed = (0..64).filter(|&i| unsafe { read_volatile(floor.add(i)) } != PAINT).count();
    println!("words of FLOOR the frames overwrote: {changed} of 64");
    let dig_at = dig as fn(usize) -> usize as u64;
    let word = unsafe { read_volatile(floor.add(63)) };
    println!("FLOOR[63], just below end, now holds {word:#x} = dig + {:#x}", word.wrapping_sub(dig_at));
    println!("that is a return address: a frame of dig saved its ra there");
    println!("no fault and no message: in machine mode, nothing marks the memory below end off-limits");
    println!("a stub that started sp at end, the low end, would do this from the first prologue");
}
