//! 12 — argv in, bytes out, and the slice that is re-pointed, not copied.
//!
//! Everything below the line `// ---- the command ----` is 10c's `echo`, as
//! it will be on rv6. Everything above it is a 40-line stand-in for ulib: an
//! `Args` over `&[&[u8]]`, a `write` that may accept FEWER bytes than it was
//! given, and the `write_all` that copes:
//!
//!     while !buf.is_empty() {
//!         let n = write(fd, buf)?;        maybe fewer than buf.len()
//!         if n == 0 { return Err(..) }    a stalled descriptor, not progress
//!         buf = &buf[n..];                <- re-point the slice past what went
//!     }
//!
//! `buf = &buf[n..]` copies nothing. It moves the pointer forward and shrinks
//! the length. The stand-in can be told to accept at most CHUNK bytes per
//! call, which is what a real UART, pipe, or socket does to you.
//!
//! Run:  cargo run --bin 12_argv_and_write_all
//!       cargo run --bin 12_argv_and_write_all -- one two three

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

// ---- a 40-line ulib ------------------------------------------------------

pub type Fd = i32;
pub const STDOUT: Fd = 1;

/// A failed call. rv6 says -1 and nothing more, so neither does this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error(pub i32);

/// The most bytes one `write` will accept. usize::MAX = "all of them".
static CHUNK: AtomicUsize = AtomicUsize::new(usize::MAX);
static CALLS: AtomicUsize = AtomicUsize::new(0);

/// May write fewer bytes than asked. Says how many.
pub fn write(fd: Fd, buf: &[u8]) -> Result<usize, Error> {
    if fd != STDOUT {
        return Err(Error(-1));
    }
    let n = buf.len().min(CHUNK.load(Ordering::Relaxed));
    let _ = std::io::stdout().write_all(&buf[..n]);
    let _ = std::io::stdout().flush();
    CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(n)
}

/// Every byte, or an error. ulib/src/lib.rs, verbatim.
pub fn write_all(fd: Fd, mut buf: &[u8]) -> Result<(), Error> {
    while !buf.is_empty() {
        let n = write(fd, buf)?;
        if n == 0 {
            return Err(Error(-1));
        }
        buf = &buf[n..];
    }
    Ok(())
}

/// The command line: a borrowed slice of borrowed byte slices.
#[derive(Clone, Copy)]
pub struct Args<'a> {
    argv: &'a [&'a [u8]],
}

impl<'a> Args<'a> {
    pub fn len(&self) -> usize {
        self.argv.len()
    }
    pub fn is_empty(&self) -> bool {
        self.argv.is_empty()
    }
    pub fn get(&self, i: usize) -> Option<&'a [u8]> {
        self.argv.get(i).copied()
    }
}

// ---- the command ---------------------------------------------------------

/// 10c's `echo`. This function would be byte-identical on rv6.
fn run(args: Args) -> i32 {
    for i in 1..args.len() {
        if i > 1 {
            let _ = write_all(STDOUT, b" ");
        }
        let _ = write_all(STDOUT, args.get(i).unwrap());
    }
    let _ = write_all(STDOUT, b"\n");
    0
}

// ---- what ulib::main! expands to on the host -----------------------------

fn main() {
    // argv as owned bytes, then as borrowed slices. No UTF-8 check anywhere.
    let owned: Vec<Vec<u8>> = {
        let real: Vec<Vec<u8>> = std::env::args_os().map(|a| a.into_encoded_bytes()).collect();
        if real.len() > 1 {
            real
        } else {
            vec![b"echo".to_vec(), b"hello".to_vec(), b"world".to_vec()]
        }
    };
    let refs: Vec<&[u8]> = owned.iter().map(|v| &v[..]).collect();
    let args = Args { argv: &refs };

    println!("== argv, as the program sees it ==");
    println!("args.len() = {}   that is argc, and argv[0] is the name", args.len());
    for i in 0..args.len() {
        println!("  args.get({i}) = {:?}", args.get(i).map(String::from_utf8_lossy));
    }
    println!("  args.get({}) = {:?}   past the end: None, not a crash",
             args.len(), args.get(args.len()));

    println!("\n== the loop, narrated ==");
    for i in 1..args.len() {
        let sep = if i > 1 { "write b\" \" first, then" } else { "no space before it;" };
        println!("  i = {i}: {sep} write args.get({i}).unwrap()");
    }
    println!("  then exactly one b\"\\n\"");
    println!("the space is a SEPARATOR: n words, n - 1 spaces. `i > 1` is that rule.");

    println!("\n== run(args) -> ");
    let code = run(args);
    println!("run returned {code}. On the host, main() now calls process::exit({code}).");

    println!("\n== a console that takes three bytes at a time ==");
    CHUNK.store(3, Ordering::Relaxed);
    CALLS.store(0, Ordering::Relaxed);
    print!("bare write(STDOUT, b\"hello world\\n\") -> ");
    let n = write(STDOUT, b"hello world\n");
    println!("   <- returned {n:?}: three bytes went, and nothing complained");
    println!("a short write is not an error. It is a number you have to read.");

    CALLS.store(0, Ordering::Relaxed);
    print!("write_all(STDOUT, b\"hello world\\n\")  -> ");
    let r = write_all(STDOUT, b"hello world\n");
    println!("   <- {r:?} after {} calls to write", CALLS.load(Ordering::Relaxed));
    println!("same twelve bytes, four calls, and `buf = &buf[n..]` between them.");
    CHUNK.store(usize::MAX, Ordering::Relaxed);

    println!("\n== the Result from write_all ==");
    println!("three things you can do with it inside `fn run(..) -> i32`:");
    println!("    write_all(STDOUT, b\"\\n\");            warning: unused Result that must be used");
    println!("    write_all(STDOUT, b\"\\n\")?;           error[E0277]: run returns i32, not Result");
    println!("    let _ = write_all(STDOUT, b\"\\n\");    the honest discard");
    println!("there is nothing an echo can do if the console is gone. Say so.");

    println!("\n== then the number ==");
    println!("process::exit({code}) -- the shell reads it as $?");
    std::process::exit(code);
}
