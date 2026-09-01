//! 02 — Compound types: tuples, arrays, structs, enums.
//!
//! All of these are *plain layouts*. A struct is its fields laid end to end
//! (with padding); an array is n elements back to back. No pointers appear
//! unless you write one.
//!
//!     struct Frame { addr: usize, free: bool }
//!
//!     stack: [ addr: 8 bytes ][ free: 1 ][ 7 bytes padding ]   = 16 bytes
//!
//! Run:  cargo run --bin 02_compound

use std::mem::size_of;

/// A fixed-size record. Copy: 16 bytes of plain data, no owned resource.
#[derive(Debug, Clone, Copy)]
struct Frame {
    addr: usize,
    free: bool,
}

/// An enum is a tag plus the largest variant's payload.
#[derive(Debug)]
enum Request {
    Read { block: u32 },
    Write { block: u32, len: u32 },
    Flush,
}

fn main() {
    println!("== tuples: a fixed group, positional ==");
    let split: (usize, usize) = (0x8000_0000, 0x8800_0000);
    let (start, end) = split; // destructuring
    println!("ram {start:#x}..{end:#x} = {} MiB", (end - start) / (1024 * 1024));
    println!("tuple.0 = {:#x}, size_of = {} bytes", split.0, size_of::<(usize, usize)>());

    println!("\n== arrays: [T; N], N is part of the type ==");
    // Fixed size, known at compile time, lives right here on the stack.
    let mut ptable: [Frame; 4] = [Frame { addr: 0, free: true }; 4];
    for (i, f) in ptable.iter_mut().enumerate() {
        f.addr = 0x8000_0000 + i * 4096;
    }
    ptable[2].free = false;
    println!("size_of::<[Frame; 4]>() = {} bytes (4 x {})",
             size_of::<[Frame; 4]>(), size_of::<Frame>());
    for f in &ptable {
        println!("  {:#x}  {}", f.addr, if f.free { "free" } else { "in use" });
    }
    // Bounds are checked. ptable[9] would panic, not read the neighbor.
    println!("get(9) = {:?}  <- no out-of-bounds read", ptable.get(9).map(|f| f.addr));

    println!("\n== structs: named fields, still just a layout ==");
    let f = ptable[0];
    println!("{f:?}\n  addr in hex = {:#x}, size_of::<Frame>() = {} bytes", f.addr, size_of::<Frame>());

    println!("\n== enums: one of several shapes, with the tag included ==");
    let queue = [
        Request::Read { block: 7 },
        Request::Write { block: 7, len: 512 },
        Request::Flush,
    ];
    for r in &queue {
        // `match` must cover every variant — the compiler checks that.
        let described = match r {
            Request::Read { block } => format!("read block {block}"),
            Request::Write { block, len } => format!("write {len}B to block {block}"),
            Request::Flush => "flush".to_string(),
        };
        println!("  {described}");
    }
    println!("size_of::<Request>() = {} bytes", size_of::<Request>());

    println!("\n== Option: the null pointer, made a type you must open ==");
    // There is no null. "Maybe a value" is spelled in the type system.
    let head: Option<usize> = Some(0x8000_5000);
    let empty: Option<usize> = None;
    for slot in [head, empty] {
        match slot {
            Some(addr) => println!("  free list head at {addr:#x}"),
            None => println!("  free list is empty"),
        }
    }
    // And it can cost nothing extra: the niche optimization uses the
    // impossible value 0 as the None tag for a reference.
    println!("size_of::<&u8>()         = {}", size_of::<&u8>());
    println!("size_of::<Option<&u8>>() = {}  <- same! None is the 0 bit pattern",
             size_of::<Option<&u8>>());
}
