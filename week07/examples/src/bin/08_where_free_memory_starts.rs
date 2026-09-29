//! 08 — Free memory starts at the first whole page above the stack.
//!
//! In rv6 the boot stack is an array inside the image, so free memory starts
//! at `end`. The link.ld these programs share puts the stack after `end`
//! instead, so here the first byte nobody owns is `__stack_top`:
//!
//!     0x8000_0000   _entry, then code       .text
//!     etext         constants and data      .rodata .data .bss
//!     end           16 KiB of stack         grows down from __stack_top
//!     __stack_top   round up to a page
//!     first page    free, 4096 bytes at a time, up to 0x8800_0000
//!
//! Round up: add 0xFFF, then clear the low 12 bits. The clear alone rounds
//! down, into memory that is already in use.
//!
//! Run:  cargo run --bin 08_where_free_memory_starts
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
const KERNBASE: usize = 0x8000_0000;
const PHYSTOP: usize = 0x8800_0000;

extern "C" {
    static etext: u8;
    static end: u8;
    static __stack_top: u8;
}

fn main() {
    let local = 0u64;
    let here = addr_of!(local) as usize;
    let text = addr_of!(etext) as usize;
    let image = addr_of!(end) as usize;
    let top = addr_of!(__stack_top) as usize;

    println!("== the stack is already in use ==");
    println!("end          {:>12}   the image stops here", Hex(image));
    println!("__stack_top  {:>12}   end + {} bytes of stack", Hex(top), top - image);
    println!("a local      {:>12}   in [end, __stack_top): {}", Hex(here), (image..top).contains(&here));
    println!("program 02 prints the whole layout; here only the stack matters");
    println!("sp started at __stack_top before main ran, so free memory starts there, not at end");

    println!("\n== round __stack_top up to a page ==");
    let plus = top + (PGSIZE - 1);
    let first = plus & !(PGSIZE - 1);
    println!("__stack_top             {:>12}", Hex(top));
    let crossed = plus >> 12 != top >> 12;
    println!("add 0xFFF               {:>12}   in the next page now: {crossed}", Hex(plus));
    println!("clear the low 12 bits   {:>12}   the first whole page nobody uses", Hex(first));
    let t = (text + 0xFFF) & !0xFFF;
    println!("etext, the same way     {:>12}   already aligned, so it stays put: {}", Hex(t), t == text);
    println!("0xFFF = {:b}: twelve ones, because 4096 is a power of two", PGSIZE - 1);
    println!("so clearing the low 12 bits is zeroing the last three hex digits");

    println!("\n== two ways to get it wrong ==");
    let down = top & !(PGSIZE - 1);
    println!("round down instead      {:>12}   the clear step alone", Hex(down));
    println!("  that page starts inside the stack: {}", (image..top).contains(&down));
    println!("  its first {} bytes are the top of the stack, where the first frames go", top - down);
    let from_end = (image + 0xFFF) & !0xFFF;
    println!("round up end instead    {:>12}   right for rv6, whose stack is below end", Hex(from_end));
    println!("  inside this program's stack: {}, {} bytes below __stack_top",
             (image..top).contains(&from_end), top - from_end);
    println!("hand out a page of the stack, and the page's new owner");
    println!("and the next function call write the same bytes");

    println!("\n== whole pages up to 0x8800_0000 ==");
    let pages = (PHYSTOP - first) >> 12;
    let below = (first - KERNBASE) >> 12;
    println!("first page   {:>12}", Hex(first));
    println!("last page    {:>12}   0x8800_0000 - 0x1000", Hex(PHYSTOP - PGSIZE));
    println!("pages        (0x8800_0000 - {}) >> 12 = {pages}", Hex(first));
    println!("check        32768 in RAM - {below} below {} = {}", Hex(first), 32768 - below);
    println!("count from both ends, and the two must agree: {}", pages == 32768 - below);
}
