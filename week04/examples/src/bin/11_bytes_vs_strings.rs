//! 11 — A string is bytes plus a promise the kernel cannot afford to check.
//!
//! Four types that all look like "text" and are not the same thing:
//!
//!     u8        one byte                            1 byte
//!     char      one Unicode scalar value            4 bytes
//!     &str      pointer + length, UTF-8 GUARANTEED  16 bytes
//!     &[u8]     pointer + length, no promise at all  16 bytes
//!
//! The kernel boundary is defined in bytes: a UART sends bytes, a disk block
//! is bytes, and the argv `exec` builds is NUL-terminated bytes. Turning bytes
//! into a `&str` is a CHECK, it costs a validation table, and it can fail --
//! so `str::from_utf8` returns a Result, and `ulib::Args::get` hands you the
//! bytes and lets you decide.
//!
//! Run:  cargo run --bin 11_bytes_vs_strings

use std::mem::size_of;
use std::str;

fn main() {
    println!("== four sizes ==");
    println!("size_of::<u8>()    = {}", size_of::<u8>());
    println!("size_of::<char>()  = {}   a Unicode scalar, not a byte", size_of::<char>());
    println!("size_of::<&str>()  = {}  pointer + length, plus a promise", size_of::<&str>());
    println!("size_of::<&[u8]>() = {}  pointer + length, no promise", size_of::<&[u8]>());

    println!("\n== a byte-string literal ==");
    let flag: &[u8; 2] = b"-n";
    let s: &[u8] = flag; // an array reference coerces to a slice
    println!("b\"-n\" is a &[u8; 2] = {flag:?}, and coerces to &[u8] = {s:?}");
    let arg: &[u8] = b"-n";
    println!("arg == b\"-n\"  -> {}   slices compare byte by byte", arg == b"-n");
    println!("arg == \"-n\"   -> error[E0277]: can't compare `[u8]` with `str`");

    println!("\n== a &str is bytes underneath, and len() counts the bytes ==");
    let word = "café";
    println!("\"café\".len()           = {}   bytes", word.len());
    println!("\"café\".chars().count() = {}   characters", word.chars().count());
    print!("as bytes:");
    for b in word.as_bytes() {
        print!(" {b:#04x}");
    }
    println!();
    println!("é is two bytes, 0xc3 0xa9, and both have the high bit set.");
    println!("every byte of a multi-byte character is >= 0x80, so a byte-wise");
    println!("search for b'\\n' (0x0a) can never land inside one. Lines are free.");

    println!("\n== from_utf8 is a check, and it can fail ==");
    println!("from_utf8(b\"hello\")            = {:?}", str::from_utf8(b"hello"));
    // Built at run time: rustc can see through a literal here and warns,
    // which is itself the point -- the check is real, and it is not free.
    let bad: Vec<u8> = [0xffu8, 0x41].to_vec();
    println!("from_utf8(&[0xff, 0x41])        = {:?}", str::from_utf8(&bad));
    let cut = &word.as_bytes()[..4]; // "caf" + the first byte of é
    println!("from_utf8(&\"café\".as_bytes()[..4]) = {:?}", str::from_utf8(cut));
    println!("a read() that stops mid-character hands you exactly that last case.");
    println!("Args::get gives bytes. Args::str runs this check for you and says None.");

    println!("\n== matching on bytes ==");
    let line = b"pid 42\tstate: run\n";
    let (mut digits, mut blanks, mut other) = (0, 0, 0);
    for &b in line.iter() {
        match b {
            b'\n' => break,
            b'0'..=b'9' => digits += 1,
            b' ' | b'\t' => blanks += 1,
            _ => other += 1,
        }
    }
    println!("{:?}", str::from_utf8(line).unwrap());
    println!("digits = {digits}, blanks = {blanks}, other = {other}   (stopped at the newline)");
    println!("b'0'..=b'9' is a range of u8. No char, no str, no table.");

    println!("\n== parsing a number from bytes, by hand ==");
    println!("b'7' - b'0' = {}   digits are consecutive in ASCII", b'7' - b'0');
    for arg in [b"512".as_slice(), b"4096", b"12x", b"", b"99999999999999999999"] {
        println!("parse_usize({:?}) = {:?}", String::from_utf8_lossy(arg), parse_usize(arg));
    }
    println!("`head -n 12x` is a usage error, not a panic, and overflow is not wrap.");

    println!("\n== where the newline is ==");
    let buf = b"first line\nsecond\n";
    println!("buf.iter().position(|&b| b == b'\\n') = {:?}", buf.iter().position(|&b| b == b'\n'));
    println!("that Option<usize> is the whole of what ulib::Lines does per line:");
    println!("find the newline, hand out &buf[..i], carry on from i + 1.");
}

/// Decimal bytes to a number. `None` on empty, on a non-digit, on overflow.
fn parse_usize(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }
    let mut n: usize = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        n = n.checked_mul(10)?.checked_add((b - b'0') as usize)?;
    }
    Some(n)
}
