//! 06 — One block size: every request costs whole pages.
//!
//! The page allocator hands out 4096 bytes or nothing, so a caller that needs
//! n bytes must take whole pages. The bytes it did not need are spent anyway:
//!
//!      100 bytes  ->  1 page    3996 bytes nobody uses
//!     5000 bytes  ->  2 pages   3192 bytes nobody uses, and the two must touch
//!
//! Space lost inside a block is internal fragmentation. What one size buys is
//! that any free page fits any one-page request. No hole between blocks is
//! ever too small to use, and there is nothing to search.
//!
//! Run:  cargo run --bin 06_why_one_page_size
#![no_std]
#![no_main]

use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

const PGSIZE: usize = 4096;

fn main() {
    let asks = [8usize, 100, 4000, 4096, 4097, 5000, 12288];

    println!("== a request, in whole pages ==");
    for n in asks {
        let pages = n.div_ceil(PGSIZE);
        println!("{n:>6} bytes  ->  {pages} page{}   {:>6} bytes given   {:>5} unused",
                 if pages == 1 { " " } else { "s" }, pages * PGSIZE, pages * PGSIZE - n);
    }
    println!("n.div_ceil(4096) rounds the count up");
    println!("4096 and 12288 fit exactly, and 4097 costs a second page");

    println!("\n== the waste stays inside the block ==");
    let asked: usize = asks.iter().sum();
    let given: usize = asks.iter().map(|n| n.div_ceil(PGSIZE) * PGSIZE).sum();
    println!("asked for {asked} bytes, given {given}: {} unused", given - asked);
    // integers only: QEMU starts with the FPU off (mstatus.FS = 0)
    let permille = 100 * 1000 / PGSIZE;
    println!("a 100-byte request uses {}.{}% of its page", permille / 10, permille % 10);
    println!("one request can waste at most 4095 bytes: ask for 1, or for 4097");
    println!("that loss is internal fragmentation");
    println!("week 10's first heap gives every request a page of its own");
    println!("so an 8-byte Box there leaves 4088 bytes idle, like the first row above");

    println!("\n== two pages are not one buffer ==");
    let (a, b) = (0x8710_3000usize, 0x8710_5000usize);
    println!("the lecture's trace hands out {}, then {}", Hex(a), Hex(b));
    println!("page numbers {} and {}: {} apart", Hex(a >> 12), Hex(b >> 12), (b >> 12) - (a >> 12));
    println!("a 5000-byte buffer needs page numbers p and p + 1; these are not");
    println!("the free list gives one page per call and cannot promise neighbors (Going deeper)");
}
