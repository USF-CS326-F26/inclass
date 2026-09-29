//! 01 — `sp` and `ra` are ordinary registers, and you can read them.
//!
//! This program has no OS under it. QEMU jumped to 0x8000_0000, the runtime
//! pointed `sp` at a stack, and `main` was called like any function. So
//! what `main` sees in its registers is the whole story:
//!
//!     sp   the stack pointer: grows down, a multiple of 16 at every call
//!     ra   the return address: `call f` writes it, `ret` jumps to it
//!     zero always 0: writes to it vanish
//!
//! `where_from` below is a leaf in assembly that hands back its own `ra`,
//! which is the address of the instruction right after the `call` in main.
//!
//! Run:  cargo run --bin 01_registers
#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

global_asm!(
    r#"
.globl where_from
where_from:              # returns its own ra: where the call came from
    mv   a0, ra
    ret

.globl long_way_back
long_way_back:           # the same `ret`, spelled out
    li   a0, 326
    jalr zero, 0(ra)
"#
);

extern "C" {
    fn where_from() -> usize;
    fn long_way_back() -> u64;
    static __stack_top: u8;
}

fn main() {
    println!("== sp: the stack ==");
    let sp: usize;
    unsafe { asm!("mv {}, sp", out(reg) sp) };
    let top = core::ptr::addr_of!(__stack_top) as usize;
    println!("sp          = {sp:#x}");
    println!("stack top   = {top:#x}   (from link.ld)");
    println!("in use      = {} bytes, by the runtime and main", top - sp);
    println!("sp % 16     = {}", sp % 16);

    println!("\n== ra: where a ret will go ==");
    let main_at = main as fn() as usize;
    let ra = unsafe { where_from() };
    println!("main        = {main_at:#x}");
    println!("ra          = {ra:#x}   = main + {:#x}", ra - main_at);
    println!("the call wrote ra; the leaf's ret jumped back to it");

    println!("\n== zero: writes vanish ==");
    let z: u64;
    unsafe { asm!("li zero, 99", "mv {}, zero", out(reg) z) };
    println!("li zero, 99 then mv from zero: {z}");

    println!("\n== ret is jalr zero, 0(ra) ==");
    println!("long_way_back() = {}", unsafe { long_way_back() });
    println!("it returned: jalr jumped to ra and threw the link away in zero");
}
