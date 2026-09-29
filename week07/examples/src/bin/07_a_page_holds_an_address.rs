//! 07 — A free page has room for the address of another page.
//!
//! Nobody uses a free page, so all 4096 of its bytes are spare. Its first 8
//! are exactly one usize, and a usize holds an address. So a free page can
//! hold the record that it is free, at no cost outside the page. That is what
//! makes the free list intrusive: its links live in the free pages.
//!
//!     page P   +0   the address of page Q    one usize, 8 bytes
//!              +8   0 0 0 ... 0              4088 bytes nobody reads
//!
//! This program builds no list. It does one store through a cast, then some
//! loads to see what the store did.
//!
//! Run:  cargo run --bin 07_a_page_holds_an_address
#![no_std]
#![no_main]

use core::mem::size_of;
use core::ptr::addr_of;
use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

const PGSIZE: usize = 4096;
const PHYSTOP: usize = 0x8800_0000;
const P: usize = 0x8400_6000; // two pages 64 MiB into RAM, far above this program
const Q: usize = 0x8400_A000;

extern "C" {
    static __stack_top: u8;
}

fn main() {
    let top = addr_of!(__stack_top) as usize;

    println!("== two pages nobody uses ==");
    println!("__stack_top  {:>12}   the stack grows down from here", Hex(top));
    println!("page P       {:>12}   above __stack_top: {}   below 0x8800_0000: {}",
             Hex(P), P >= top, P + PGSIZE <= PHYSTOP);
    println!("page Q       {:>12}   above __stack_top: {}   page-aligned: {}",
             Hex(Q), Q >= top, Q % PGSIZE == 0);
    println!("this program keeps nothing above __stack_top, so both are free");
    let before = unsafe { *(P as *const usize) };
    println!("P's first usize before the store: {}", Hex(before));
    println!("QEMU starts RAM zeroed; real hardware makes no such promise");

    println!("\n== one store, through a cast ==");
    let slot = P as *mut usize; // P's first 8 bytes, seen as one usize
    unsafe { *slot = Q };
    let back = unsafe { *(P as *const usize) };
    println!("*(P as *mut usize) = Q      stored {}", Hex(Q));
    println!("P's first usize, read back: {}", Hex(back));
    let q_first = unsafe { *(Q as *const usize) };
    println!("Q's own first usize is still {}", Hex(q_first));
    println!("P holds Q's address, and Q itself was never touched");

    println!("\n== the same bytes, one at a time ==");
    let bytes = P as *const u8;
    let first = unsafe { *(bytes as *const [u8; 8]) };
    println!("P + 0 .. P + 8:  {first:02X?}");
    println!("low byte first, because RISC-V is little-endian");
    println!("from_le_bytes gives Q back: {}", usize::from_le_bytes(first) == Q);
    let rest = (8..PGSIZE).filter(|&i| unsafe { *bytes.add(i) } != 0).count();
    println!("nonzero bytes in P + 8 .. P + 4096: {rest}   the store wrote 8 bytes and no more");

    println!("\n== what the record costs ==");
    println!("size_of::<usize>() = {}, and size_of::<*mut u8>() = {}: an address is 8 bytes",
             size_of::<usize>(), size_of::<*mut u8>());
    println!("the page gives up 8 of its 4096 bytes, and only while it is free");
    println!("once a page is handed out, nothing reads that word, and its owner may overwrite all 4096");
    println!("so taking a page back needs no memory of its own, and cannot fail for lack of it");
}
