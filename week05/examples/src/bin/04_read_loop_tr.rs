//! 04 — Read until 0, write every byte: `tr a-z A-Z` on a stingy kernel.
//!
//! `tr a-z A-Z` copies its input to its output with each lowercase letter
//! made uppercase. The work is one line inside the loop every filter has:
//!
//!     loop {
//!         let n = read(src, &mut buf)?;       maybe fewer than buf.len()
//!         if n == 0 { break }                 the only end
//!         buf[..n].make_ascii_uppercase();    this chunk, and only its n bytes
//!         write_all(dst, &buf[..n])?;         maybe several writes
//!     }
//!
//! The folded `mod ulib` below is a 40-line stand-in: one input file on fd 3,
//! a `read` that returns at most READ_MAX bytes, and a `write` to fd 1 that
//! takes at most WRITE_MAX. Both return how many. Neither complains. Every
//! byte still arrives, because the loop listens to both numbers.
//!
//! Run:  cargo run --bin 04_read_loop_tr

use ulib::{read, write, write_all, Error, Fd, INPUT, STDOUT};

// fold: ---- a 40-line ulib: one input file, one output, both stingy -----
mod ulib {
    use std::sync::Mutex;

    pub type Fd = i32;
    pub const STDOUT: Fd = 1;
    pub const INPUT: Fd = 3; // a file somebody already opened

    /// A failed call. rv6 says -1 and nothing more, so neither does this.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Error(pub i32);

    struct State {
        input: &'static [u8],
        read_max: usize,
        write_max: usize,
        out: Vec<u8>,
        reads: Vec<usize>,
        writes: Vec<usize>,
    }

    static S: Mutex<State> = Mutex::new(State {
        input: b"", read_max: 0, write_max: 0,
        out: Vec::new(), reads: Vec::new(), writes: Vec::new(),
    });

    /// A fresh input, fresh limits, an empty output, no calls yet.
    pub fn reset(input: &'static [u8], read_max: usize, write_max: usize) {
        let mut s = S.lock().unwrap();
        *s = State { input, read_max, write_max, out: Vec::new(), reads: Vec::new(), writes: Vec::new() };
    }

    /// Hands back at most READ_MAX bytes, and says how many. 0: used up.
    pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize, Error> {
        let mut s = S.lock().unwrap();
        if fd != INPUT {
            return Err(Error(-1));
        }
        let n = s.input.len().min(buf.len()).min(s.read_max);
        buf[..n].copy_from_slice(&s.input[..n]);
        s.input = &s.input[n..];
        s.reads.push(n);
        Ok(n)
    }

    /// Keeps at most WRITE_MAX bytes, and says how many.
    pub fn write(fd: Fd, buf: &[u8]) -> Result<usize, Error> {
        let mut s = S.lock().unwrap();
        if fd != STDOUT {
            return Err(Error(-1));
        }
        let n = buf.len().min(s.write_max);
        s.out.extend_from_slice(&buf[..n]);
        s.writes.push(n);
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

    /// What reached fd 1, and what each call returned.
    pub fn report() -> (String, Vec<usize>, Vec<usize>) {
        let s = S.lock().unwrap();
        (String::from_utf8_lossy(&s.out).into_owned(), s.reads.clone(), s.writes.clone())
    }
}

/// `tr a-z A-Z`: returns how many bytes went through.
fn upcase(src: Fd, dst: Fd) -> Result<usize, Error> {
    let mut buf = [0u8; 16];
    let mut total = 0;
    loop {
        let n = read(src, &mut buf)?;
        if n == 0 {
            return Ok(total);
        }
        buf[..n].make_ascii_uppercase();
        write_all(dst, &buf[..n])?;
        total += n;
    }
}

/// The same loop with a bare `write`. It compiles, and nothing warns.
fn upcase_bare(src: Fd, dst: Fd) -> Result<usize, Error> {
    let mut buf = [0u8; 16];
    let mut total = 0;
    loop {
        let n = read(src, &mut buf)?;
        if n == 0 {
            return Ok(total);
        }
        buf[..n].make_ascii_uppercase();
        write(dst, &buf[..n])?; // the count it returns goes nowhere
        total += n;
    }
}

const TEXT: &[u8] = b"mount /\nstart the shell\nready\n";

fn main() {
    println!("== the input: {} bytes in a file on fd {INPUT} ==", TEXT.len());
    println!("input = {:?}", String::from_utf8_lossy(TEXT));
    println!("the buffer is 16 bytes. read hands back at most 8; write takes at most 5.");

    println!("\n== the loop, on the stingy kernel ==");
    ulib::reset(TEXT, 8, 5);
    let r = upcase(INPUT, STDOUT);
    let (out, reads, writes) = ulib::report();
    println!("upcase(..) = {r:?}");
    println!("read  returned {reads:?}");
    println!("write returned {writes:?}");
    println!("fd 1 got {out:?}");
    println!("8 bytes in, then write took 5 and write_all went back for the 3.");

    println!("\n== the same loop, on a generous host ==");
    ulib::reset(TEXT, usize::MAX, usize::MAX);
    let r = upcase(INPUT, STDOUT);
    let (out2, reads, writes) = ulib::report();
    println!("upcase(..) = {r:?}");
    println!("read  returned {reads:?}");
    println!("write returned {writes:?}");
    println!("same output: {}. The kernel chose the calls; the loop chose nothing.", out2 == out);

    println!("\n== bare write: every call says how many, and nobody listens ==");
    ulib::reset(TEXT, 8, 5);
    let r = upcase_bare(INPUT, STDOUT);
    let (out, _, writes) = ulib::report();
    println!("upcase_bare(..) = {r:?}");
    println!("write returned {writes:?}");
    println!("fd 1 got {out:?}");
    println!("{} of {} bytes arrived, and the function still said Ok.", out.len(), TEXT.len());
    println!("on the host every write is taken whole, so every test of this passes.");

    println!("\n== a write that takes 0 is an error, not a loop ==");
    ulib::reset(TEXT, 8, 0);
    let r = upcase(INPUT, STDOUT);
    let (_, reads, writes) = ulib::report();
    println!("upcase(..) = {r:?}");
    println!("read  returned {reads:?}");
    println!("write returned {writes:?}");
    println!("without the `n == 0` check, write_all would ask again forever.");
}
