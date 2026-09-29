//! 09 — `while let`: call, match, and stop at the first miss.
//!
//!     while let Some(top) = stack.pop() {
//!         ...                          runs once for each Some
//!     }                                the first None ends the loop
//!
//! is exactly this, written shorter:
//!
//!     loop {
//!         match stack.pop() {
//!             Some(top) => { ... }
//!             _ => break,
//!         }
//!     }
//!
//! A `for` loop needs an `Iterator`, and `ulib::Lines` is not one. Each line
//! it returns borrows the buffer that the NEXT call may overwrite, and
//! `Iterator` has no way to say "this item dies when you ask for another".
//! So a program that reads lines drives them by hand, with `while let`.
//!
//! Run:  cargo run --bin 09_while_let

/// Lines out of a byte slice, with ulib's signature: each line borrows
/// `self`, so nothing else may touch `self` while a line is in use.
struct Lines<'a> {
    rest: &'a [u8],
    calls: usize,
}

impl<'a> Lines<'a> {
    fn new(text: &'a [u8]) -> Lines<'a> {
        Lines { rest: text, calls: 0 }
    }

    fn next_line(&mut self) -> Option<&[u8]> {
        self.calls += 1;
        if self.rest.is_empty() {
            return None;
        }
        let end = self.rest.iter().position(|&b| b == b'\n').unwrap_or(self.rest.len());
        let (line, after) = self.rest.split_at(end);
        self.rest = after.get(1..).unwrap_or(b""); // step over the \n
        Some(line)
    }
}

fn main() {
    println!("== pop until None ==");
    let mut stack = vec![10, 20, 30];
    while let Some(top) = stack.pop() {
        println!("  top = {top}");
    }
    println!("pop() answered None, so the loop ended. stack = {stack:?}");

    println!("\n== the same loop, spelled out ==");
    let mut stack = vec!["init", "sh", "grep"];
    loop {
        match stack.pop() {
            Some(top) => println!("  top = {top:?}"),
            _ => break,
        }
    }
    println!("one call, one pattern, per turn. The first value that fails it ends the loop.");

    println!("\n== any pattern will do, not just Some ==");
    let mut rest: &[u8] = b"rv6";
    while let [first, tail @ ..] = rest {
        println!("  first = {:?}, {} byte(s) after it", *first as char, tail.len());
        rest = tail;
    }
    println!("`[first, tail @ ..]` fails on an empty slice, and that ended the loop.");

    println!("\n== a line source, driven by hand ==");
    let mut lines = Lines::new(b"ls\ncd /\n\nexit\n");
    while let Some(line) = lines.next_line() {
        println!("  {:<8} {} byte(s)", format!("{:?}", String::from_utf8_lossy(line)), line.len());
    }
    println!("{} calls for 4 lines: the last call is the None that ended the loop.", lines.calls);
    println!("the empty line came back as Some(\"\"). Empty is a line; None is the end.");

    println!("\n== leaving early: break ==");
    let mut lines = Lines::new(b"Host: rv6\nAccept: */*\n\nthe body, which nobody asks for\n");
    while let Some(line) = lines.next_line() {
        if line.is_empty() {
            break; // the blank line ends the header block
        }
        println!("  header {:?}", String::from_utf8_lossy(line));
    }
    println!("{} calls, and the body was never asked for.", lines.calls);
    println!("`break` and `return` leave a `while let` like any other loop.");
}
