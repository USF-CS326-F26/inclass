//! 08 — A line that straddles two reads, in a buffer that never grows.
//!
//! Reads know nothing about lines, so a read can stop halfway through one.
//! With a heap you would grow the buffer until the line fits. Without one,
//! you slide the unfinished part to the front and read more BEHIND it. With
//! a 16-byte buffer, after `abc` and `def` came back:
//!
//!     before   [a b c \n d e f \n g h i j k l . .]   start 8, len 14
//!     compact  [g h i j k l . . . . . . . . . .]    start 0, len 6
//!     refill   read(fd, &mut buf[6..]) -> 10
//!              [g h i j k l m n \n o p q r s t u]   len 16
//!     return   "ghijklmn"                           start 9
//!
//! `Lines` below is `ulib::Lines` cut down to those three cases, and it
//! prints each row as it goes: a buffered `\n` returns a line with no read;
//! at end of input the leftover bytes are the last line; otherwise compact
//! and refill. (A line longer than the buffer is left out here. ulib cuts it
//! to the buffer's length and `truncated()` says so.)
//!
//! Run:  cargo run --bin 08_lines_in_a_fixed_buffer

use kernel::Source;

// fold: ---- the kernel's side: reads of the sizes it chooses -------------
mod kernel {
    /// Hands out `data` in reads of `sizes[0]`, `sizes[1]`, ... bytes, then
    /// whatever fits, then 0. Counts every call.
    pub struct Source {
        data: &'static [u8],
        sizes: &'static [usize],
        pub calls: usize,
    }

    impl Source {
        pub fn new(data: &'static [u8], sizes: &'static [usize]) -> Source {
            Source { data, sizes, calls: 0 }
        }

        pub fn read(&mut self, buf: &mut [u8]) -> usize {
            self.calls += 1;
            let want = match self.sizes.split_first() {
                Some((&size, rest)) => {
                    self.sizes = rest;
                    size
                }
                None => usize::MAX,
            };
            let n = want.min(buf.len()).min(self.data.len());
            buf[..n].copy_from_slice(&self.data[..n]);
            self.data = &self.data[n..];
            n
        }
    }
}

/// Lines out of a buffer the caller owns. It never allocates.
struct Lines<'b> {
    src: Source,
    buf: &'b mut [u8],
    start: usize, // where the next line begins
    len: usize,   // bytes held in buf
    eof: bool,
}

impl<'b> Lines<'b> {
    fn new(src: Source, buf: &'b mut [u8]) -> Lines<'b> {
        Lines { src, buf, start: 0, len: 0, eof: false }
    }

    fn next_line(&mut self) -> Option<&[u8]> {
        loop {
            // 1. A whole line is already buffered: no read at all.
            if let Some(nl) = self.buf[self.start..self.len].iter().position(|&b| b == b'\n') {
                let from = self.start;
                self.start += nl + 1;
                println!("return   {:<35}   start {}", shown(&self.buf[from..from + nl]), self.start);
                return Some(&self.buf[from..from + nl]);
            }
            // 2. End of input: whatever is left is the last line.
            if self.eof {
                if self.start == self.len {
                    println!("return   None");
                    return None;
                }
                let from = self.start;
                self.start = self.len;
                println!("return   {:<35}   no \\n, and still a line", shown(&self.buf[from..self.len]));
                return Some(&self.buf[from..self.len]);
            }
            // 3. Part of a line: slide it to the front, then read behind it.
            if self.start > 0 {
                println!("before   {}   start {}, len {}", grid(self.buf, self.len), self.start, self.len);
                self.buf.copy_within(self.start..self.len, 0);
                self.len -= self.start;
                self.start = 0;
                println!("compact  {}   start 0, len {}", grid(self.buf, self.len), self.len);
            }
            debug_assert!(self.len < self.buf.len(), "a line longer than the buffer");
            let n = self.src.read(&mut self.buf[self.len..]);
            println!("refill   read(fd, &mut buf[{}..]) -> {n}", self.len);
            if n == 0 {
                self.eof = true;
            } else {
                self.len += n;
                println!("         {}   len {}", grid(self.buf, self.len), self.len);
            }
        }
    }
}

/// The buffer as the lecture draws it: held bytes, then dots.
fn grid(buf: &[u8], len: usize) -> String {
    let cells: Vec<String> = buf.iter().enumerate().map(|(i, &b)| match b {
        _ if i >= len => ".".to_string(),
        b'\n' => "\\n".to_string(),
        _ => (b as char).to_string(),
    }).collect();
    format!("[{}]", cells.join(" "))
}

/// Bytes as Rust would write them in a string literal.
fn shown(bytes: &[u8]) -> String {
    format!("{:?}", String::from_utf8_lossy(bytes))
}

const INPUT: &[u8] = b"abc\ndef\nghijklmn\nopqrstuvw\nxyz";

fn main() {
    println!("== the input, and a 16-byte buffer ==");
    println!("{}   {} bytes, five lines, the last with no \\n", shown(INPUT), INPUT.len());
    println!("the kernel hands it over in reads of 14, then 10, then whatever is left.");
    let mut buf = [0u8; 16];
    let mut lines = Lines::new(Source::new(INPUT, &[14, 10]), &mut buf);

    println!("\n== abc and def: one read, two lines ==");
    lines.next_line(); // it prints its own rows
    lines.next_line();
    println!("the second call made no read. Its \\n was already in the buffer.");

    println!("\n== ghijklmn: the tail moves to the front, the read goes behind it ==");
    lines.next_line();
    println!("`ghijkl` came in one read and `mn` in the next. The caller sees one line.");

    println!("\n== the rest, and a last line with no \\n ==");
    while lines.next_line().is_some() {}
    println!("the read that returned 0 did not end the lines. The call after the last one did.");

    println!("\n== what it cost ==");
    println!("{} reads for {} bytes, through 16 bytes of storage.", lines.src.calls, INPUT.len());
    println!("each line was a slice of that one array. Nothing was copied out, and nothing grew.");
}
