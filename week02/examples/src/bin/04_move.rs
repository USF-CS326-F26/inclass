//! 04 — Move: the owner changes, the bytes do not.
//!
//!     BEFORE   a: [ ptr | 6 | 6 ] ────▶ [ k e r n e l ]
//!              b: (does not exist)
//!
//!     let b = a;
//!
//!     AFTER    a: struck off by rustc — using it is a compile error
//!              b: [ ptr | 6 | 6 ] ────▶ [ k e r n e l ]   (same buffer!)
//!
//! Run time: three words copied, usually optimized away.
//! Compile time: `a` is removed from the set of usable names.
//!
//! Run:  cargo run --bin 04_move

fn main() {
    println!("== assignment moves ==");
    let a = String::from("kernel");
    let a_buf = a.as_ptr();
    let b = a;                       // `a` is moved out of, right here
    println!("a's buffer was {a_buf:p}");
    println!("b's buffer is  {:p}   <- identical: nothing was copied", b.as_ptr());
    // println!("{a}");              // error[E0382]: borrow of moved value: `a`
    println!("but `a` is no longer a usable name (uncomment the line to see E0382)");

    println!("\n== passing to a function moves too ==");
    let page = String::from("page 0x8000_5000");
    println!("caller owns {:p}", page.as_ptr());
    consume(page);                   // ownership transfers to the parameter
    // page is gone here; the value was dropped inside `consume`.
    println!("caller no longer owns anything");

    println!("\n== if the caller still needs it, hand it back ==");
    let mut log = String::from("boot");
    log = annotate(log, "kalloc ready");   // move in, move out
    log = annotate(log, "mmu on");
    println!("returned to caller: {log:?}");

    println!("\n== moves out of collections are blocked, not silent ==");
    #[allow(clippy::useless_vec)] // a Vec on purpose: the heap is the point
    let v = vec![String::from("a"), String::from("b")];
    // let first = v[0];             // error[E0507]: cannot move out of index
    let first = &v[0];               // borrow instead
    println!("borrowed v[0] = {first:?}; v is still whole with {} items", v.len());
    let owned_first = v[0].clone();  // or pay for a copy, explicitly
    println!("cloned v[0] = {owned_first:?} at a different buffer {:p} vs {:p}",
             owned_first.as_ptr(), v[0].as_ptr());

    println!("\n== a move is not a memcpy of the data ==");
    let big = vec![0u8; 8 * 1024 * 1024];   // 8 MiB on the heap
    let buf = big.as_ptr();
    let moved = big;                         // 3 words move; 8 MiB stays put
    assert_eq!(buf, moved.as_ptr());
    println!("moved an 8 MiB Vec by copying 24 bytes of stack: buffer still {:p}",
             moved.as_ptr());
}

/// Takes ownership. The value dies at this closing brace.
fn consume(text: String) {
    println!("  consume() now owns {:p} ({text:?})", text.as_ptr());
} // <- dropped here

/// Move in, move out: the classic "I need it back" signature.
fn annotate(mut text: String, note: &str) -> String {
    text.push_str(" | ");
    text.push_str(note);
    text
}
