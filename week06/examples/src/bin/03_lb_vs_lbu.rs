//! 03 — `lb` sign-extends, `lbu` zero-extends, and below 0x80 you cannot tell.
//!
//! A register is 64 bits; a byte is 8. Loading one byte has to decide what
//! goes in the other 56:
//!
//!     byte 0xE9 = 1110_1001
//!     lb   → 0xFFFF_FFFF_FFFF_FFE9   top bit copied up: −23
//!     lbu  → 0x0000_0000_0000_00E9   zeros:             233
//!
//! Characters, sizes and lengths are unsigned, so they load with `lbu`.
//!
//! Run:  cargo run --bin 03_lb_vs_lbu
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
.globl load_lb
load_lb:                 # a0 = address; returns the byte, sign-extended
    lb   a0, 0(a0)
    ret

.globl load_lbu
load_lbu:                # a0 = address; returns the byte, zero-extended
    lbu  a0, 0(a0)
    ret
"#
);

extern "C" {
    fn load_lb(p: *const u8) -> u64;
    fn load_lbu(p: *const u8) -> u64;
}

fn main() {
    let bytes: [u8; 4] = [0x05, 0x60, 0xE9, 0xFF];

    println!("== the same byte, loaded two ways ==");
    for b in &bytes {
        let (s, u) = unsafe { (load_lb(b), load_lbu(b)) };
        println!("{:#04x}:  lb {:#018x} = {:>4}   lbu {:#018x} = {:>3}", b, s, s as i64, u, u);
    }
    println!("0x05 and 0x60 agree; 0xE9 and 0xFF do not: their top bit is 1");

    println!("\n== a length byte, read with the wrong one ==");
    let len: u8 = 200;
    let wrong = unsafe { load_lb(&len) };
    let right = unsafe { load_lbu(&len) };
    println!("lbu says {} bytes", right);
    println!("lb  says {} as i64, which is {} as a usize", wrong as i64, wrong as usize);
    println!("every byte of 0x80 or more comes out 256 too small");

    println!("\n== Rust makes the same choice, by type ==");
    let b = 0xE9u8;
    println!("{:#x} as u8 -> i8 -> i64 = {}", b, b as i8 as i64);
    println!("{:#x} as u8 -> u64       = {}", b, b as u64);
}
