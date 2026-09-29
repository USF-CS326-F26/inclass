//! 06 — A function that calls saves `ra`; a function that uses `s0` saves it.
//!
//! `framed` keeps `x` in `s0` across a call, so it owes its caller two
//! things: the caller's `s0`, and its own way home in `ra`. It claims 16
//! bytes of stack for them and hands `visit` a pointer to see them:
//!
//!               ┌──────────────┐  ← sp before the prologue
//!     sp + 8    │ saved ra     │    the way back into main
//!     sp + 0    │ saved s0     │    main's s0, owed back
//!               └──────────────┘  ← sp after `addi sp, sp, -16`
//!
//! The last section runs the rule's failure: a callee that borrows `s1`
//! without saving it works fine, until its caller keeps something there.
//!
//! Run:  cargo run --bin 06_prologue_and_s0
#![no_std]
#![no_main]

use core::arch::global_asm;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

global_asm!(
    r#"
.globl framed
framed:                  # a0 = x, a1 = visit; returns visit(x, frame) + x
    addi sp, sp, -16     # claim 16 bytes; sp stays 16-byte aligned
    sd   ra, 8(sp)       # we call, so save ra
    sd   s0, 0(sp)       # we use s0, so save the caller's s0
    mv   s0, a0          # x lives in s0 across the call
    mv   t0, a1
    mv   a1, sp          # let visit see this frame
    jalr t0              # visit(x, frame): ra = the next instruction
    add  a0, a0, s0      # visit's answer + x
    ld   s0, 0(sp)       # the epilogue: the prologue, backwards
    ld   ra, 8(sp)
    addi sp, sp, 16
    ret

.globl keeper
keeper:                  # a0 = callee; keeps 0x5151 in s1 across calling it
    addi sp, sp, -16
    sd   ra, 8(sp)
    sd   s1, 0(sp)
    li   s1, 0x5151
    jalr a0
    mv   a0, s1          # what s1 holds after the call
    ld   s1, 0(sp)
    ld   ra, 8(sp)
    addi sp, sp, 16
    ret

.globl polite
polite:                  # borrows s1, and gives it back
    addi sp, sp, -16
    sd   s1, 0(sp)
    li   s1, 99
    ld   s1, 0(sp)
    addi sp, sp, 16
    ret

.globl rude
rude:                    # borrows s1, and keeps it
    li   s1, 99
    ret
"#
);

extern "C" {
    fn framed(x: u64, visit: extern "C" fn(u64, *const u64) -> u64) -> u64;
    fn keeper(callee: unsafe extern "C" fn()) -> u64;
    fn polite();
    fn rude();
}

/// Called from inside `framed`'s frame; reads the two saved words.
extern "C" fn visit(x: u64, frame: *const u64) -> u64 {
    let main_at = main as fn() as u64;
    let (s0, ra) = unsafe { (*frame, *frame.add(1)) };
    println!("frame at {frame:p}");
    println!("  [sp + 0] saved s0 = {s0:#x}");
    println!("  [sp + 8] saved ra = {ra:#x}   = main + {:#x}", ra - main_at);
    x * 10
}

fn main() {
    println!("== the frame framed built ==");
    let sp: u64;
    unsafe { core::arch::asm!("mv {}, sp", out(reg) sp) };
    println!("main's sp = {sp:#x}");
    let r = unsafe { framed(4, visit) };

    println!("\n== what the saves bought ==");
    println!("framed(4, visit) = {r} = visit's 40 + the 4 kept in s0");
    println!("visit ran Rust code with its own frames, and s0 still held 4");

    println!("\n== a callee that skips the save ==");
    println!("keeper(polite) -> s1 = {:#x}", unsafe { keeper(polite) });
    println!("keeper(rude)   -> s1 = {:#x}", unsafe { keeper(rude) });
    println!("rude returns fine on its own. Only a caller keeping something in s1 notices,");
    println!("and it notices later, somewhere else");
}
