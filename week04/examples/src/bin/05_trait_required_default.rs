//! 05 — A trait: one required method, and defaults written once.
//!
//! The kernel writes bytes to a UART. The tests write the same bytes to a
//! buffer they can inspect. A trait is the name for what those two have in
//! common:
//!
//!     trait Uart {
//!         fn putc(&mut self, b: u8);                 REQUIRED: no body
//!         fn puts(&mut self, s: &[u8]) { ... }       DEFAULT: body in the trait
//!     }
//!
//! Every implementer writes `putc`. Nobody writes `puts`; it was written once,
//! here, against a `self` whose type does not exist yet -- and it works for
//! every type that ever implements the trait, including the ones written next
//! year.
//!
//!     impl Console          methods of Console (last week's impl)
//!     impl Uart for Console the SECOND kind of impl: Console promises Uart
//!
//! Run:  cargo run --bin 05_trait_required_default

use std::fmt::Write as _; // for the last section; the `as _` imports only the methods
use std::io::Write;

/// The promise. Holds no data. You never build one of these.
pub trait Uart {
    /// Send one byte. Every implementer must write this.
    fn putc(&mut self, b: u8);

    /// Send a run of bytes. Default: one `putc` per byte.
    fn puts(&mut self, s: &[u8]) {
        for &b in s {
            self.putc(b);
        }
    }

    /// Send bytes, then a newline. Default, built on the default.
    fn putln(&mut self, s: &[u8]) {
        self.puts(s);
        self.putc(b'\n');
    }
}

/// The real thing: bytes go to the terminal. A UNIT struct -- no fields,
/// zero bytes -- exists only so there is a type to hang the impl on.
pub struct Console;

impl Uart for Console {
    fn putc(&mut self, b: u8) {
        let _ = std::io::stdout().write_all(&[b]);
    }
}

/// The test's version: keep every byte so the test can read it back.
pub struct Recorder {
    pub buf: Vec<u8>,
}

impl Uart for Recorder {
    fn putc(&mut self, b: u8) {
        self.buf.push(b);
    }
}

/// Measure, and throw the bytes away.
#[derive(Debug)] // <- this line is `impl Debug for Tally`, written by the compiler
pub struct Tally {
    pub bytes: usize,
}

impl Uart for Tally {
    fn putc(&mut self, _b: u8) {
        self.bytes += 1;
    }
}

/// core::fmt::Write has one required method with 07r's exact signature, plus
/// a Result. Implement it and `write!` works on your type.
impl std::fmt::Write for Recorder {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

/// An implementer with a faster way to do `puts`, so it overrides the default.
pub struct Fast {
    pub bytes: usize,
    pub calls: usize,
}

impl Uart for Fast {
    fn putc(&mut self, _b: u8) {
        self.bytes += 1;
        self.calls += 1;
    }
    // Overriding is allowed. The contract -- "all the bytes arrive" -- is the
    // same; only the cost changed: one call instead of s.len() calls.
    fn puts(&mut self, s: &[u8]) {
        self.bytes += s.len();
        self.calls += 1;
    }
}

fn main() {
    println!("== three implementers, each wrote ONLY putc ==");
    let mut console = Console;
    let mut rec = Recorder { buf: Vec::new() };
    let mut tally = Tally { bytes: 0 };

    print!("Console.putln(b\"rv6 boot\")  -> ");
    console.putln(b"rv6 boot");
    let _ = std::io::stdout().flush();
    rec.putln(b"rv6 boot");
    tally.putln(b"rv6 boot");
    println!("Recorder.putln(b\"rv6 boot\") -> buf = {:?}", String::from_utf8_lossy(&rec.buf));
    println!("Tally.putln(b\"rv6 boot\")    -> bytes = {}   8 + the newline", tally.bytes);
    println!("putln and puts have no impl block anywhere. They are the defaults.");

    println!("\n== a default is written once, against a self that does not exist yet ==");
    println!("    fn putln(&mut self, s: &[u8]) {{ self.puts(s); self.putc(b'\\n'); }}");
    println!("when that line was compiled, Console/Recorder/Tally were not in scope.");
    println!("it only assumed what the trait promises: that self has putc and puts.");

    println!("\n== overriding a default ==");
    let mut fast = Fast { bytes: 0, calls: 0 };
    fast.putln(b"rv6 boot");
    println!("Fast.putln(b\"rv6 boot\") -> bytes = {}, calls = {}   (puts was one call)",
             fast.bytes, fast.calls);
    println!("Tally would have made 9 calls. Same bytes, same contract, cheaper.");

    println!("\n== a unit struct is zero bytes ==");
    println!("size_of::<Console>() = {}", std::mem::size_of::<Console>());
    println!("it exists to be a TYPE, so that `impl Uart for Console` has a home");

    println!("\n== you have been using this since week 03 ==");
    println!("#[derive(Debug)] on Tally is `impl Debug for Tally`: {tally:?}");
    println!("`impl Uart for Recorder` is the same shape, written by hand");

    println!("\n== and the standard library's version ==");
    let mut log = Recorder { buf: Vec::new() };
    let _ = write!(log, "pid {} exited {}", 3, -1);
    println!("write!(log, \"pid {{}} exited {{}}\", 3, -1) -> {:?}",
             String::from_utf8_lossy(&log.buf));
    println!("    trait Write {{ fn write_str(&mut self, s: &str) -> fmt::Result; ... }}");
    println!("07r's `Out::write_str` is this signature without the Result.");
    println!("the kernel's println! is this impl on the UART. That is the whole trick.");
}
