//! 02 — The linker script gives every byte an address, and `end` names the first one past the image.
//!
//! link.ld starts the location counter at the RAM base and lays the image out
//! in order. Each item below lands in one of these, by what kind of item it is:
//!
//!     0x8000_0000  .entry   _entry, listed first so the ROM's jump lands on it
//!                  .text    the rest of the code, then padding to a page: etext
//!                  .rodata  string literals and read-only statics
//!                  .data    writable statics that start with a value
//!                  .bss     writable statics that start as zeros
//!     end                   the first address past the image
//!     __stack_top           16 KiB higher: this runtime's stack
//!
//! Rust sees a linker symbol as an extern static. Its byte means nothing; its
//! address is the answer.
//!
//! Run:  cargo run --bin 02_where_the_linker_put_it
#![no_std]
#![no_main]

use core::ptr::addr_of;
use week07::println;

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

extern "C" {
    fn _entry();
    static etext: u8;
    static end: u8;
    static __stack_top: u8;
}

const GREETING: &str = "hello from .rodata";
static mut COUNT: u64 = 326; // mut, so it needs writable memory: .data
static mut TABLE: [u64; 512] = [0; 512]; // all zeros: .bss

const PHYSTOP: usize = 0x8800_0000; // the end of RAM, from program 01

fn main() {
    println!("== one item of each kind, in address order ==");
    let entry = _entry as *const () as usize;
    let etext_at = addr_of!(etext) as usize;
    let table = addr_of!(TABLE) as usize;
    let end_at = addr_of!(end) as usize;
    let top = addr_of!(__stack_top) as usize;
    let items = [
        ("_entry", entry, ".entry: first, where the ROM jumps"),
        ("main", main as fn() as usize, ".text"),
        ("etext", etext_at, "the end of the code, rounded up to a page"),
        ("GREETING", GREETING.as_ptr() as usize, ".rodata: a string literal's bytes"),
        ("COUNT", addr_of!(COUNT) as usize, ".data: a static mut that starts at 326"),
        ("TABLE", table, ".bss: 512 u64s that start as zeros"),
        ("end", end_at, "the first address past the image"),
        ("__stack_top", top, "the top of the 16 KiB stack"),
    ];
    for (name, at, what) in items {
        println!("{name:<12} {at:#x}   {what}");
    }
    let in_order = items.windows(2).all(|w| w[0].1 < w[1].1);
    println!("in increasing order: {in_order}, as link.ld lists them");
    println!("_entry is first because link.ld lists *(.entry) first");
    println!("ENTRY(_entry) only fills in the ELF header, which the ROM never reads");

    println!("\n== how big each part is ==");
    let code = etext_at - entry;
    println!("code   etext - _entry        {code:>9} bytes: {} pages, the last one padded", code / 4096);
    println!("data   end - etext           {:>9} bytes: everything after the code", end_at - etext_at);
    println!("stack  __stack_top - end     {:>9} bytes: reserved by link.ld", top - end_at);
    println!("free   PHYSTOP - __stack_top {:>9} bytes: this program never touches them", PHYSTOP - top);

    println!("\n== end is an address, not a value ==");
    let first_free = unsafe { &end as *const u8 as usize };
    let byte = unsafe { end };
    println!("&end as *const u8 as usize = {first_free:#x}");
    println!("in rv6 free memory starts here. This runtime puts its stack here instead,");
    println!("so here free memory starts at __stack_top (program 08)");
    println!("the byte at end holds {byte}: it means nothing (here, the stack's lowest byte)");
    let size = core::mem::size_of::<[u64; 512]>();
    println!("end - TABLE = {}, and TABLE is {size} bytes: nothing follows it", first_free - table);
    println!("make TABLE bigger and end moves up with it: a symbol, not a constant");
}
