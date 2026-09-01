//! 03 — Where values live: the stack handle and the heap buffer.
//!
//! A `String` is three words on the stack that describe bytes on the heap:
//!
//!     stack                                  heap
//!     s: [ ptr | len=6 | cap=6 ] ─────────▶  [ k e r n e l ]
//!
//! Ownership is about that arrow. The three words are cheap to copy; the
//! buffer they point at must be freed exactly once, by exactly one owner.
//!
//! Run:  cargo run --bin 03_stack_and_heap

use std::mem::size_of;

fn main() {
    println!("== a fixed-size value is the whole value ==");
    let n: u64 = 4096;
    println!("n = {n}   lives at stack address {:p}   size {} bytes",
             &n, size_of::<u64>());

    println!("\n== a String is a handle to somewhere else ==");
    let s = String::from("kernel");
    println!("handle on the stack at {:p}, {} bytes wide (ptr, len, cap)",
             &s, size_of::<String>());
    println!("  ptr -> {:p}   len = {}   cap = {}", s.as_ptr(), s.len(), s.capacity());
    println!("  the bytes {:?} are NOT at {:p}; they are at {:p}",
             s.as_bytes(), &s, s.as_ptr());

    println!("\n== growth moves the buffer, not the handle ==");
    let mut v: Vec<u32> = Vec::with_capacity(2);
    println!("handle stays at {:p} for the whole loop", &v);
    for i in 0..6 {
        let before = v.as_ptr();
        v.push(i);
        let after = v.as_ptr();
        let note = if before == after { "" } else { "  <- reallocated: new buffer, old one freed" };
        println!("  push({i}) len={} cap={} buf={:p}{note}", v.len(), v.capacity(), after);
    }

    println!("\n== &str: a borrowed view, no ownership at all ==");
    let owned: String = String::from("0x8000_0000");
    let view: &str = &owned[2..];            // ptr + len, points into `owned`
    println!("owned buffer {:p}, view {:p}  <- same memory, offset by 2",
             owned.as_ptr(), view.as_ptr());
    println!("size_of::<String>() = {}   size_of::<&str>() = {}",
             size_of::<String>(), size_of::<&str>());

    println!("\n== a literal is not on the heap either ==");
    // "..." is baked into the binary; the &str just points at it.
    let lit: &'static str = "compiled into .rodata";
    println!("{lit:?} at {:p}", lit.as_ptr());

    println!("\n== the release point is visible ==");
    {
        let scoped = String::from("freed at the closing brace");
        println!("  inside the block: {:p} holds {scoped:?}", scoped.as_ptr());
    } // <- the buffer is freed HERE. No free(), no collector, no delay.
    println!("  after the block: that allocation is already gone");
}
