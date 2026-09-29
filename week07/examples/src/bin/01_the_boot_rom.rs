//! 01 — At reset, six instructions in ROM hand the hart to 0x8000_0000.
//!
//! QEMU's `virt` machine starts one hart at pc = 0x1000, in machine mode,
//! with paging off. With `-bios none` there is no firmware. A boot ROM runs
//! six instructions, and the last one jumps to RAM, where QEMU copied this
//! program. The ROM is memory like any other, so a load can read it:
//!
//!     0x1000   six instructions    auipc, addi, csrr, ld, ld, jr
//!     0x1018   a word              where the jr goes: 0x8000_0000
//!     0x1020   a word              the device tree, a description of the board
//!     0x1028   a table             for firmware, which -bios none leaves out
//!
//! QEMU writes those words into the ROM at startup. The last section follows
//! the device-tree word and finds the lecture's memory map there.
//!
//! Run:  cargo run --bin 01_the_boot_rom
#![no_std]
#![no_main]

use core::arch::asm;
use core::ptr::read_volatile;
use week07::{print, println};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

const ROM: usize = 0x1000;

fn rom_word(offset: usize) -> u32 {
    // SAFETY: `virt` maps its boot ROM at 0x1000, and a load cannot change it.
    unsafe { read_volatile((ROM + offset) as *const u32) }
}

fn rom_dword(offset: usize) -> u64 {
    // SAFETY: as above; the ROM's data words are 8-byte aligned.
    unsafe { read_volatile((ROM + offset) as *const u64) }
}

// fold: decodes just the five instruction shapes the ROM uses
fn decode(at: usize, w: u32) {
    const REG: [&str; 32] = [
        "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3",
        "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
        "t3", "t4", "t5", "t6",
    ];
    let rd = REG[(w >> 7 & 31) as usize];
    let rs1 = REG[(w >> 15 & 31) as usize];
    let imm = (w as i32) >> 20;
    print!("{at:#06x}  {w:08x}  ");
    match (w & 0x7f, w >> 12 & 7) {
        (0x17, _) => println!("auipc {rd}, {:#x}", w >> 12),
        (0x13, 0) => println!("addi  {rd}, {rs1}, {imm}"),
        (0x73, 2) if w >> 20 == 0xf14 => println!("csrr  {rd}, mhartid"),
        (0x03, 3) => println!("ld    {rd}, {imm}({rs1})"),
        (0x67, 0) if rd == "zero" && imm == 0 => println!("jr    {rs1}"),
        _ => println!("(not one of the five)"),
    }
}

/// Four bytes of the device tree. Everything in it is big-endian.
fn be32(at: usize) -> u32 {
    // SAFETY: `at` is inside the device tree the ROM points at.
    u32::from_be(unsafe { read_volatile(at as *const u32) })
}

// fold: finds the `reg` property (base, size) of the first node named prefix@…
fn reg_of(dtb: usize, prefix: &str) -> (u64, u64) {
    let cstr = |at: usize| unsafe { core::ffi::CStr::from_ptr(at as *const _) }.to_str().unwrap();
    let be64 = |at: usize| (be32(at) as u64) << 32 | be32(at + 4) as u64;
    let strings = dtb + be32(dtb + 12) as usize;
    let mut p = dtb + be32(dtb + 8) as usize;
    let mut here = false;
    loop {
        let token = be32(p);
        p += 4;
        match token {
            1 => {
                // FDT_BEGIN_NODE: the node's name, NUL-terminated, padded to 4
                let name = cstr(p);
                here = name.starts_with(prefix) && name[prefix.len()..].starts_with('@');
                p = (p + name.len() + 4) & !3;
            }
            3 => {
                // FDT_PROP: the value's length, where its name is, the value
                let len = be32(p) as usize;
                if here && cstr(strings + be32(p + 4) as usize) == "reg" {
                    return (be64(p + 8), be64(p + 16));
                }
                p = (p + 8 + len + 3) & !3;
            }
            2 => here = false, // FDT_END_NODE
            4 => {}            // FDT_NOP
            _ => panic!("no node named {prefix}@ in the device tree"),
        }
    }
}

fn main() {
    println!("== the hart, as reset left it ==");
    let (hart, misa, mtvec): (usize, usize, usize);
    unsafe {
        asm!("csrr {}, mhartid", out(reg) hart);
        asm!("csrr {}, misa", out(reg) misa);
        asm!("csrr {}, mtvec", out(reg) mtvec);
    }
    println!("mhartid = {hart}      this hart's ID; -smp 1 starts only this one");
    println!("mtvec   = {mtvec:#x}    a fault would jump to address 0: no handler is set");
    print!("misa    = {misa:#x}: RV{} with ", 16 << (misa >> 62));
    for bit in 0..26 {
        if misa >> bit & 1 == 1 {
            print!("{}", (b'A' + bit as u8) as char);
        }
    }
    println!();
    println!("IMAFD (plus Zicsr, Zifencei) is the g in riscv64gc, and C is the c");
    println!("S and U: supervisor and user mode. H: hypervisor");
    println!("reading mhartid, misa and mtvec did not trap: this is machine mode");

    println!("\n== the six instructions at 0x1000 ==");
    for i in 0..6 {
        decode(ROM + 4 * i, rom_word(4 * i));
    }
    println!("auipc t0, 0x0 sets t0 to its own address, 0x1000; the rest count from there");

    println!("\n== the words the ROM loads ==");
    let target = rom_dword(24);
    let dtb = rom_dword(32) as usize;
    println!("[t0 + 24] = {target:#x}   loaded into t0, then jr t0");
    println!("            the RAM base, where QEMU copied this program");
    println!("[t0 + 32] = {dtb:#x}   loaded into a1: the device tree");
    println!("            magic {:#x}, {} bytes long", be32(dtb), be32(dtb + 4));
    let magic = rom_word(40).to_le_bytes();
    let magic = core::str::from_utf8(&magic).unwrap();
    println!("[t0 + 40] = {magic:?}       where a2 points: a table for OpenSBI, unused here");

    println!("\n== the board, as its device tree describes it ==");
    let map = [
        ("test", "the test finisher"),
        ("clint", "the CLINT: the timer"),
        ("plic", "the PLIC: routes device interrupts"),
        ("serial", "the UART"),
        ("memory", "RAM"),
    ];
    for (node, what) in map {
        let (base, size) = reg_of(dtb, node);
        println!("{node:<7} {base:#010x}  {size:#10x} bytes   {what}");
    }
    let (ram, ram_size) = reg_of(dtb, "memory");
    let phystop = (ram + ram_size) as usize;
    println!("RAM is {} MiB and ends at {phystop:#x}", ram_size >> 20);
    println!("the tree itself sits {} MiB below that end", (phystop - dtb) >> 20);
    println!("rv6 never reads the tree: it hardcodes these addresses, so it boots on virt only");
}
