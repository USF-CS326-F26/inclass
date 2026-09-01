//! 09 — Slices: the systems shape.
//!
//! A slice is a borrowed window: pointer + length, two words, no ownership.
//!
//!     buf: [ 10 11 12 13 14 15 16 17 ]        (owned, on the stack or heap)
//!            ▲     ▲
//!     &buf[2..5] = [ ptr ][ len=3 ] ──────────┘
//!
//! This is what a kernel actually passes around: "these bytes, this many",
//! with the length checked instead of trusted. Compare with C:
//!
//!     void write(char *buf, int n)     // two arguments, no relationship
//!     fn write(buf: &[u8])             // one value, length can't disagree
//!
//! Run:  cargo run --bin 09_slices

fn main() {
    println!("== a window into an array ==");
    let buf: [u8; 8] = [10, 11, 12, 13, 14, 15, 16, 17];
    let middle: &[u8] = &buf[2..5];
    println!("buf     {:?} at {:p}", buf, buf.as_ptr());
    println!("&buf[2..5] {:?} at {:p}  <- same memory, offset 2, len {}",
             middle, middle.as_ptr(), middle.len());
    println!("size_of::<&[u8]>() = {} bytes (ptr + len)",
             std::mem::size_of::<&[u8]>());

    println!("\n== one function, any backing storage ==");
    let heap: Vec<u8> = vec![1, 2, 3, 4];
    let stack: [u8; 3] = [9, 9, 9];
    println!("checksum(array)   = {}", checksum(&buf));
    println!("checksum(vec)     = {}", checksum(&heap));
    println!("checksum(array)   = {}", checksum(&stack));
    println!("checksum(slice)   = {}", checksum(middle));

    println!("\n== &str is a slice of UTF-8 bytes ==");
    let line = String::from("kalloc: 0x8000_5000");
    let (tag, addr) = line.split_at(7);
    println!("tag  = {tag:?}");
    println!("addr = {addr:?}");
    println!("both point into the ONE buffer at {:p}: {:p} and {:p}",
             line.as_ptr(), tag.as_ptr(), addr.as_ptr());

    println!("\n== mutable slices write through ==");
    let mut page = [0u8; 16];
    fill(&mut page[..8], 0xAA);
    fill(&mut page[8..], 0x55);
    println!("page = {page:02x?}");

    println!("\n== length is checked, not trusted ==");
    println!("buf.get(20) = {:?}   <- None, not a wild read", buf.get(20));
    // buf[20] would panic with a clear message instead of reading a neighbor.
    match buf.get(2..5) {
        Some(s) => println!("buf.get(2..5) = {s:?}"),
        None => println!("out of range"),
    }

    println!("\n== chunks: how you walk a page table or a disk block ==");
    for (i, chunk) in page.chunks(4).enumerate() {
        println!("  entry {i}: {chunk:02x?}");
    }
}

/// Takes a *borrowed* window. Works for arrays, Vecs, and other slices.
fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().map(|&b| b as u32).sum()
}

fn fill(bytes: &mut [u8], value: u8) {
    for b in bytes.iter_mut() {
        *b = value;
    }
}
