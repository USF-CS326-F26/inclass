//! 07 — `ret` goes wherever `ra` says, on whatever stack `sp` says.
//!
//! `resume` loads a new `sp` and a new `ra`, then returns. It returns into a
//! function that never called it, running on a stack main never used:
//!
//!     main ── resume(&r) ──►  ld sp, 0(a0)     sp = top of the second stack
//!                             ld ra, 8(a0)     ra = landing
//!                             ret         ──►  landing() on the second stack
//!
//! It is a one-way door: nothing recorded where main was, so there is no
//! coming back. `landing` powers the machine off itself.
//!
//! Run:  cargo run --bin 07_a_ret_that_lands_elsewhere
#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::addr_of_mut;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

global_asm!(
    r#"
.globl resume
resume:                  # a0 = *const Resume: sp at 0, ra at 8
    ld   sp, 0(a0)       # the saved stack
    ld   ra, 8(a0)       # the saved return address
    ret                  # "return" into it
"#
);

#[repr(C)]
struct Resume {
    sp: usize,
    ra: usize,
}

extern "C" {
    fn resume(r: *const Resume) -> !;
}

/// A second stack: 4 KiB, 16-byte aligned, as a RISC-V `sp` must be. It is
/// `static mut` because it will be written: `landing`'s frames live in it.
#[repr(C, align(16))]
struct Stack([u8; 4096]);
static mut SECOND: Stack = Stack([0; 4096]);

/// The second stack's lowest address and the address just past its top.
fn second_stack() -> (usize, usize) {
    let lo = addr_of_mut!(SECOND) as usize;
    (lo, lo + 4096)
}

extern "C" fn landing() -> ! {
    let sp: usize;
    unsafe { core::arch::asm!("mv {}, sp", out(reg) sp) };
    let (lo, hi) = second_stack();
    println!("landing: nobody called me, and sp = {sp:#x}");
    println!("landing: that is inside the second stack: {}", lo <= sp && sp < hi);
    println!("landing: main's frame is still there, but nothing points at it");
    week06::exit(0)
}

fn main() {
    let (lo, hi) = second_stack();
    let r = Resume { sp: hi, ra: landing as *const () as usize };

    println!("== two stacks ==");
    let sp: usize;
    unsafe { core::arch::asm!("mv {}, sp", out(reg) sp) };
    println!("main runs with sp = {sp:#x}");
    println!("the second stack is {lo:#x}..{hi:#x}");

    println!("\n== resume: load sp, load ra, ret ==");
    println!("Resume {{ sp: {:#x}, ra: {:#x} = landing }}", r.sp, r.ra);
    unsafe { resume(&r) }
}
