//! 07 — State that must survive the end of a read.
//!
//! A stream arrives in chunks, and the kernel picks where each one ends.
//! Anything that spans two bytes can span two reads. This counter of
//! Windows line endings, `\r\n`, keeps the one fact about the past that the
//! next byte might need, in a variable the CALLER owns:
//!
//!     fn feed(chunk: &[u8], after_cr: &mut bool, pairs: &mut usize)
//!
//!     chunk 1   o n e \r            after_cr = true    pairs = 0
//!     chunk 2   \n t w o \r \n      after_cr = false   pairs = 2
//!               ^ pairs with the \r at the end of chunk 1
//!
//! `feed_forgetful` declares `after_cr` inside itself instead, so it starts
//! false on every call. It is right whenever no pair straddles two chunks,
//! and the program does not choose where the chunks end.
//!
//! Run:  cargo run --bin 07_state_across_chunks

/// The lecture's counter. Called once per chunk; the caller keeps `after_cr`.
fn feed(chunk: &[u8], after_cr: &mut bool, pairs: &mut usize) {
    for &b in chunk {
        if *after_cr && b == b'\n' {
            *pairs += 1;
        }
        *after_cr = b == b'\r';
    }
}

/// The same loop, with the state declared in the wrong place.
fn feed_forgetful(chunk: &[u8], pairs: &mut usize) {
    let mut after_cr = false; // reset on every call: the past is gone
    for &b in chunk {
        if after_cr && b == b'\n' {
            *pairs += 1;
        }
        after_cr = b == b'\r';
    }
}

/// Bytes as Rust would write them in a string literal: "one\r".
fn shown(bytes: &[u8]) -> String {
    format!("{:?}", String::from_utf8_lossy(bytes))
}

fn main() {
    let input: &[u8] = b"one\r\ntwo\r\n";

    println!("== one feed of the whole input ==");
    let (mut after_cr, mut pairs) = (false, 0);
    feed(input, &mut after_cr, &mut pairs);
    println!("feed({}) -> pairs = {pairs}", shown(input));

    println!("\n== the same bytes in two chunks, state carried ==");
    let (mut after_cr, mut pairs) = (false, 0);
    for chunk in [&input[..4], &input[4..]] {
        feed(chunk, &mut after_cr, &mut pairs);
        println!("after {:<14} after_cr = {after_cr:<5}  pairs = {pairs}", shown(chunk));
    }
    println!("2, the same as one feed. One row per chunk, each starting from the row above.");

    println!("\n== the same chunks, state forgotten ==");
    let mut pairs = 0;
    for chunk in [&input[..4], &input[4..]] {
        feed_forgetful(chunk, &mut pairs);
        println!("after {:<14} pairs = {pairs}", shown(chunk));
    }
    println!("1. The \\n that opens chunk 2 met a fresh `false`, and that pair vanished.");

    println!("\n== every place a read could end ==");
    for k in 0..=input.len() {
        let (a, b) = input.split_at(k);
        let (mut after_cr, mut carried) = (false, 0);
        feed(a, &mut after_cr, &mut carried);
        feed(b, &mut after_cr, &mut carried);
        let mut forgot = 0;
        feed_forgetful(a, &mut forgot);
        feed_forgetful(b, &mut forgot);
        let mark = if forgot == carried { "" } else { "   <- wrong" };
        println!("  split at {k:>2}: {:>16} | {:<16}  carried {carried}  forgetful {forgot}{mark}",
                 shown(a), shown(b));
    }
    println!("the carried count is 2 at every split. The forgetful one is wrong only where");
    println!("a read ends between \\r and \\n, and the kernel, not the program, picks that.");

    println!("\n== what the state costs ==");
    println!("size_of::<bool>() + size_of::<usize>() = {} bytes, for a line or a terabyte",
             std::mem::size_of::<bool>() + std::mem::size_of::<usize>());
    println!("that is O(1) state: the same variables whatever the input's length.");
}
