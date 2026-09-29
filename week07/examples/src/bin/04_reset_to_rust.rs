//! 04 — Reset to Rust: each step of the boot needs the one before it.
//!
//! By the time `main` runs, all five steps are over. Each one left something
//! behind that `main` can still read:
//!
//!     1. QEMU copies the ELF into RAM    .data holds its value; .bss is zeros
//!     2. reset: the ROM at 0x1000 runs   its a0, a1 and a2 are still in registers
//!     3. it jumps to 0x8000_0000         _entry is there: link.ld put .entry first
//!     4. _entry points sp at the top     the first frame starts right at the top
//!     5. only then does Rust run         its first store is 8 bytes below the top
//!
//! Here step 5 is a jump, not a call. `_entry` jumps to `__week07_start`, the
//! first Rust function. Its prologue is the first store through `sp`. Then it
//! calls `main`.
//!
//! Run:  cargo run --bin 04_reset_to_rust
#![no_std]
#![no_main]

use core::arch::asm;
use core::ptr::{addr_of, read_volatile};
use week07::println;

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

extern "C" {
    fn _entry();
    static __stack_top: u8;
}

static mut SEEDED: u64 = 326; // .data: the ELF carries these 8 bytes
static mut CLEARED: [u64; 256] = [0; 256]; // .bss: the ELF carries only its size

const ROM: usize = 0x1000;

fn main() {
    // Read before anything else: the first call would overwrite all four.
    // Naming a0-a2 as outputs reads what they hold; the asm changes nothing.
    let (a0, a1, a2, ra): (usize, usize, usize, usize);
    unsafe { asm!("mv {ra}, ra", ra = out(reg) ra, out("a0") a0, out("a1") a1, out("a2") a2) };

    println!("== 1. QEMU loaded the image before the first instruction ==");
    let first = unsafe { read_volatile(0x8000_0000 as *const u32) };
    let seeded = unsafe { read_volatile(addr_of!(SEEDED)) };
    let cleared = addr_of!(CLEARED) as *const u64;
    let zeros = (0..256).all(|i| unsafe { read_volatile(cleared.add(i)) } == 0);
    println!("[0x80000000] = {first:#010x}   _entry's first instruction, loaded before any ran");
    println!("SEEDED       = {seeded}          .data: the value came in the ELF");
    println!("CLEARED      all zero: {zeros}   .bss: the ELF records only its size");
    println!("nothing in this program stores to SEEDED or CLEARED: QEMU's loader did both");

    println!("\n== 2, 3. reset: the ROM ran, then jumped to _entry ==");
    let mhartid: usize;
    unsafe { asm!("csrr {}, mhartid", out(reg) mhartid) };
    let dtb = unsafe { read_volatile((ROM + 32) as *const u64) } as usize;
    println!("a0 = {a0}            from csrr a0, mhartid: mhartid is {mhartid}");
    println!("a1 = {a1:#x}   from ld a1, 32(t0): the ROM's word there is {dtb:#x}");
    println!("a2 = {a2:#x}       from addi a2, t0, 40: an address inside the ROM itself");
    let same = a0 == mhartid && a1 == dtb && a2 == ROM + 40;
    println!("all three match: {same}. Nothing between the ROM and main wrote them");
    println!("a1 and a2 are the proof: QEMU zeroes every register at reset, so a0 is 0 either way");
    let target = unsafe { read_volatile((ROM + 24) as *const u64) } as usize;
    let entry = _entry as *const () as usize;
    println!("jump target {target:#x} == _entry {entry:#x}: {}", target == entry);
    println!("step 3 needs step 1: the ROM jumps there whether or not anything was loaded");

    println!("\n== 4, 5. sp at the top, then the first Rust frame ==");
    let top = addr_of!(__stack_top) as usize;
    let word = |below: usize| unsafe { read_volatile((top - below) as *const usize) };
    let (w8, w16, w24) = (word(8), word(16), word(24));
    let start = __week07_start as *const () as usize;
    println!("__stack_top       {top:<#12x} where _entry pointed sp, then jumped to __week07_start");
    println!("__stack_top - 8   {w8:<#12x} __week07_start's saved ra: reset's 0; a jump sets no ra");
    println!("__stack_top - 16  {w16:<#12x} padding: its frame is 16 bytes, to keep sp aligned");
    println!("__stack_top - 24  {w24:<#12x} main's saved ra: __week07_start + {:#x}", w24.wrapping_sub(start));
    println!("matches main's ra: {}. The frames start right at __stack_top: _entry pushed nothing", w24 == ra);
    println!("step 5 needs step 4: Rust's very first store went through sp, 8 bytes below the top");
}
