//! 02 — A short read is normal. Only 0 means the end.
//!
//! `read(fd, buf)` is a request with a ceiling. It puts SOME bytes in `buf`,
//! anywhere from 0 to `buf.len()`, and returns how many. A terminal hands
//! over only what has been typed, one line per Enter:
//!
//!     you type hello, Enter   read -> 6   buf[..6] is "hello\n"
//!     you type ok, Enter      read -> 3   buf[..3] is "ok\n"; buf[3..6] still says "lo\n"
//!     you press Ctrl-D        read -> 0   end of input: three reads, two lines
//!
//! `Terminal` below replays what somebody typed, the way the real one
//! answers `cat` with no arguments. The buffer is 512 bytes and is never
//! full. 6 is not the end, 3 is not the end, and the bytes past 3 belong to
//! the line before.
//!
//! Run:  cargo run --bin 02_short_reads

/// What a terminal does with `read`: one typed line per call, never more
/// than fits, the rest of a long line on the next call, and 0 after Ctrl-D.
struct Terminal {
    typed: &'static [&'static [u8]], // lines still to come, each with its \n
    pending: &'static [u8],          // the part of a line that did not fit yet
}

impl Terminal {
    fn new(typed: &'static [&'static [u8]]) -> Terminal {
        Terminal { typed, pending: b"" }
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        if self.pending.is_empty() {
            match self.typed.split_first() {
                Some((line, rest)) => {
                    self.pending = line;
                    self.typed = rest;
                }
                None => return 0, // Ctrl-D: end of input
            }
        }
        let n = self.pending.len().min(buf.len());
        buf[..n].copy_from_slice(&self.pending[..n]);
        self.pending = &self.pending[n..];
        n
    }
}

/// Bytes as Rust would write them in a string literal: "ok\n".
fn shown(bytes: &[u8]) -> String {
    format!("{:?}", String::from_utf8_lossy(bytes))
}

/// A 1,100-byte file: everything was "typed" before the first read.
static FILE: [&[u8]; 1] = [&[b'x'; 1100]];

fn main() {
    println!("== three reads, two lines ==");
    let mut term = Terminal::new(&[b"hello\n", b"ok\n"]);
    let mut buf = [0u8; 512];
    loop {
        let n = term.read(&mut buf);
        println!("read -> {n}   buf[..{n}] is {}", shown(&buf[..n]));
        if n == 0 {
            break;
        }
    }
    println!("6 and 3 are both short of 512, and neither one is the end. 0 is.");

    println!("\n== the bytes past n are left over ==");
    println!("buf[..3]  is {:<12} the second read", shown(&buf[..3]));
    println!("buf[3..6] is {:<12} the first read's tail: nothing cleared it", shown(&buf[3..6]));
    println!("buf[..6]  is {:<12} what a reader that ignores n passes on", shown(&buf[..6]));
    println!("`lo` was never typed on the second line. Only n says which bytes are real.");

    println!("\n== the mistake: treat a short read as the end ==");
    let mut term = Terminal::new(&[b"hello\n", b"ok\n"]);
    let mut buf = [0u8; 512];
    let mut reads = 0;
    loop {
        let n = term.read(&mut buf);
        reads += 1;
        println!("read -> {n}   got {}", shown(&buf[..n]));
        if n < buf.len() {
            break; // WRONG: short is not the end
        }
    }
    println!("{reads} read, then it quit. `ok` was typed and never read.");

    println!("\n== a buffer smaller than the line ==");
    let mut term = Terminal::new(&[b"hello\n", b"ok\n"]);
    let mut small = [0u8; 4];
    loop {
        let n = term.read(&mut small);
        println!("read -> {n}   small[..{n}] is {}", shown(&small[..n]));
        if n == 0 {
            break;
        }
    }
    println!("the terminal kept `o\\n` for the next read. A full buffer is not a line either.");

    println!("\n== a file ends with a short read too ==");
    let mut file = Terminal::new(&FILE);
    let mut buf = [0u8; 512];
    let mut total = 0;
    loop {
        let n = file.read(&mut buf);
        println!("read -> {n}");
        if n == 0 {
            break;
        }
        total += n;
    }
    let last = total % buf.len();
    println!("{total} bytes. The read that got the last {last} said {last}, not 0.");
    println!("Only the read after it, the one that returned 0, said the file was over.");
}
