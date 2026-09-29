//! 05 — An address is a page number and an offset.
//!
//! A page is 4096 = 2^12 bytes. So the low 12 bits of an address say where a
//! byte sits inside its page, and the bits above them say which page it is:
//!
//!     0x8000_2B6C
//!       >> 12       0x8_0002        the page number: drop three hex digits
//!       & 0xFFF     0xB6C           the offset: keep only those three
//!       & !0xFFF    0x8000_2000     the page it falls in: zero those three
//!
//! An address is page-aligned when its offset is 0: in hex it ends in 000.
//! The addresses below are this program's own, as QEMU runs it, plus the
//! lecture's 0x8002_3D40.
//!
//! Run:  cargo run --bin 05_page_number_and_offset
#![no_std]
#![no_main]

use core::ptr::addr_of;
use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

const PGSIZE: usize = 4096;
const KERNBASE: usize = 0x8000_0000; // RAM starts here
const PHYSTOP: usize = 0x8800_0000; // and ends here: 128 MiB
const UART: usize = 0x1000_0000; // a device, not RAM

extern "C" {
    static etext: u8;
    static __stack_top: u8;
}

fn main() {
    let local = 0u64;
    let here = [
        ("main", main as fn() as usize),
        ("etext", addr_of!(etext) as usize),
        ("a local", addr_of!(local) as usize),
        ("__stack_top", addr_of!(__stack_top) as usize),
        ("the UART", UART),
    ];

    println!("== page number and offset ==");
    for (name, a) in here {
        println!("{name:<12} {:>12}   number {:>8}   offset {:>5}", Hex(a), Hex(a >> 12), Hex(a & 0xFFF));
    }
    let same = here.iter().all(|&(_, a)| a >> 12 == a / PGSIZE && (a & 0xFFF) == a % PGSIZE);
    let back = here.iter().all(|&(_, a)| (a >> 12) * PGSIZE + (a & 0xFFF) == a);
    println!(">> 12 is / 4096 and & 0xFFF is % 4096, for every one: {same}");
    println!("number * 4096 + offset gives each address back: {back}");
    println!("the shift and the mask work only because 4096 is a power of two, 2^12");

    println!("\n== the page it falls in ==");
    for (name, a) in here {
        let page = a & !(PGSIZE - 1);
        match a.checked_sub(KERNBASE) {
            Some(d) => println!("{name:<12} in page {:>12}   RAM page {}", Hex(page), d >> 12),
            None => println!("{name:<12} in page {:>12}   below 0x8000_0000: not RAM", Hex(page)),
        }
    }
    let pages = (PHYSTOP - KERNBASE) >> 12;
    println!("RAM has {pages} pages: page 0 is {}, and the last, {}, is {}",
             Hex(KERNBASE), pages - 1, Hex(KERNBASE + (pages - 1) * PGSIZE));
    let below = addr_of!(__stack_top) as usize - addr_of!(local) as usize;
    println!("a local is {below} bytes below __stack_top: main's frame is near the top of the stack");

    println!("\n== aligned means low zero bits ==");
    let some = [
        ("the lecture", 0x8002_3D40),
        ("etext", addr_of!(etext) as usize),
        ("a local", addr_of!(local) as usize),
        ("KERNBASE", KERNBASE),
    ];
    for (name, a) in some {
        let k = a.trailing_zeros();
        println!("{name:<12} {:>12}   {k:>2} low zero bits   {:>10}-byte aligned   page-aligned: {}",
                 Hex(a), 1usize << k, k >= 12);
    }
    println!("0x8002_3D40 ends in 40: six zero bits, so 64-byte aligned, not page-aligned");
    println!("etext is page-aligned because link.ld says ALIGN(0x1000) just before it");
}
