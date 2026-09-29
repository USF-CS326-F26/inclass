//! 13 — W⊕X: a page may be writable or executable, never both.
//!
//! With paging off, nothing marks code as code. This program runs in
//! machine mode on bare RAM, so a store can rewrite one of its instructions,
//! and the next call runs the new one. A page table stops that with flags:
//!
//!     code            R X     fetch and read, never write
//!     stack, data     R W     read and write, never fetch
//!     R W X           the violation: whoever can write it can run anything
//!
//! W is 4 and X is 8, so a violation shows in one hex digit of the entry.
//!
//! Run:  cargo run --bin 13_w_xor_x
#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::fmt;
use core::hint::black_box;
use core::ptr::addr_of;
use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

global_asm!(
    r#"
.pushsection .text.answer, "ax"
.option push
.option norvc            # 4-byte instructions, so one word store replaces one
.p2align 2
.globl answer
answer:
    addi a0, zero, 7     # a0 = 7
    ret
.option pop
.popsection
"#
);

extern "C" {
    fn answer() -> u64;
    fn _entry();
    static etext: u8;
    static end: u8;
    static __stack_top: u8;
}

const V: u64 = 1 << 0;
const R: u64 = 1 << 1;
const W: u64 = 1 << 2;
const X: u64 = 1 << 3;

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

/// What a valid entry is, from its R, W and X bits.
fn kind(flags: u64) -> &'static str {
    match (flags & R != 0, flags & W != 0, flags & X != 0) {
        (false, false, false) => "branch: the next table",
        (false, true, _) => "reserved: W without R",
        (true, false, false) => "leaf: read-only data",
        (true, true, false) => "leaf: data",
        (false, false, true) => "leaf: execute-only code",
        (true, false, true) => "leaf: code",
        (true, true, true) => "leaf: breaks W xor X",
    }
}

fn main() {
    println!("== with paging off, code is data ==");
    let code = answer as *const () as *mut u32;
    println!("answer() = {}", unsafe { answer() });
    let first = unsafe { code.read_volatile() };
    println!("its first instruction, at {}: {:.8}   addi a0, zero, 7",
             Hex(code as u64), Hex(first as u64));
    let patched = (black_box(42u32) << 20) | (10 << 7) | 0x13;
    println!("addi a0, zero, 42 packs as (42 << 20) | (10 << 7) | 0x13 = {:.8}",
             Hex(patched as u64));
    unsafe {
        code.write_volatile(patched);
        asm!("fence.i"); // the next fetch sees the store
    }
    println!("stored it over the first instruction, then fence.i");
    println!("answer() = {}", unsafe { answer() });
    println!("machine mode, no page table: every byte of RAM is readable, writable and executable");
    println!("in supervisor mode, with this page mapped R X, that store would be a page fault");

    println!("\n== X W R: the eight combinations with V set ==");
    println!("{:<6} {:<6} {:<8} kind", "flags", "X W R", "letters");
    for xwr in 0..8u64 {
        let flags = V | xwr << 1;
        println!("{:<6.2} {:03b}    {:<8} {}", Hex(flags), xwr, Flags(flags), kind(flags));
    }
    println!("a leaf breaks W xor X when W (4) and X (8) are both set: a low hex digit of F");
    println!("W without R, X W R = 010 or 110, is reserved: the hardware faults on it");

    println!("\n== this program, split at etext ==");
    let (text, top) = (addr_of!(etext) as u64, addr_of!(__stack_top) as u64);
    println!("_entry        {:>11}   code", Hex(_entry as *const () as u64));
    println!("answer        {:>11}   code, and rewritten above", Hex(answer as *const () as u64));
    println!("etext         {:>11}   the end of code, rounded up to a page", Hex(text));
    println!("end           {:>11}   the end of the data", Hex(addr_of!(end) as u64));
    println!("__stack_top   {:>11}   the top of the stack", Hex(top));
    println!("etext & 0xFFF = {}: link.ld aligns it, so no page holds both code and data",
             text & 0xFFF);
    println!("[0x8000_0000, etext) is {} pages to map R X", (text - 0x8000_0000) >> 12);
    println!("[etext, __stack_top) is {} pages to map R W", (top - text + 0xFFF) >> 12);
    println!("xv6 splits its kernel's map of RAM at etext this way");

    println!("\n== the top pages ==");
    let maxva = black_box(1u64 << 38);
    let top_pages = [
        ("TRAMPOLINE", maxva - 0x1000, V | R | X),
        ("TRAPFRAME", maxva - 0x2000, V | R | W),
    ];
    for (name, va, flags) in top_pages {
        println!("{name:<11} {}   {:.2}  {:<6} {}", Hex(va), Hex(flags), Flags(flags), kind(flags));
    }
    println!("code in one, data in the other: each keeps W xor X");
    println!("neither has U: supervisor mode may use both, and a user-mode access faults");
}
