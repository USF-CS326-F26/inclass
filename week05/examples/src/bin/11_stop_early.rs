//! 11 — What a program does not read costs nothing.
//!
//! A `read` hands over one buffer's worth. Nothing makes the kernel fetch
//! the rest of a file until somebody asks for it. So the first line of a
//! 10 GB log costs one read, not ten billion bytes:
//!
//!     the log     |line 1|line 2| ... |line 51| ........................ 10 GB |
//!     one read    |<------ 1,024 bytes ------>|
//!     the rest    never asked for, so never read
//!
//! `head` is judged by exactly this: its cost is set by how many lines it
//! wants, not by how big the file is. The log below does not exist. Each
//! `read` makes up the bytes it returns and counts them, so the program can
//! say exactly how much it read before it stopped.
//!
//! Run:  cargo run --bin 11_stop_early

use log::Log;

// fold: ---- a 10 GB log that exists only when read ------------------------
mod log {
    /// Fixed-width lines: "line 0000000001 ok.\n" is 20 bytes.
    pub const LINE: u64 = 20;

    pub struct Log {
        pub size: u64,
        offset: u64,
        pub reads: u64,
        pub served: u64,
    }

    impl Log {
        pub fn new(size: u64) -> Log {
            Log { size, offset: 0, reads: 0, served: 0 }
        }

        /// The byte at `pos`, made up on the spot.
        fn byte_at(pos: u64) -> u8 {
            let (line, col) = (pos / LINE + 1, (pos % LINE) as usize);
            match col {
                0..=4 => b"line "[col],
                5..=14 => b'0' + (line / 10u64.pow(14 - col as u32) % 10) as u8,
                _ => b" ok.\n"[col - 15],
            }
        }

        pub fn read(&mut self, buf: &mut [u8]) -> usize {
            self.reads += 1;
            let n = (self.size - self.offset).min(buf.len() as u64) as usize;
            for (i, b) in buf[..n].iter_mut().enumerate() {
                *b = Log::byte_at(self.offset + i as u64);
            }
            self.offset += n as u64;
            self.served += n as u64;
            n
        }
    }
}

/// `yes`: "y\n" forever. Its read never returns 0.
struct Yes {
    reads: u64,
}

impl Yes {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        self.reads += 1;
        for (i, b) in buf.iter_mut().enumerate() {
            *b = if i % 2 == 0 { b'y' } else { b'\n' };
        }
        buf.len()
    }
}

/// 10000000000 -> "10,000,000,000"
fn commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn main() {
    let mut log = Log::new(10_000_000_000);
    let mut buf = [0u8; 1024];

    println!("== a 10 GB log, made up on demand ==");
    println!("size  {} bytes", commas(log.size));
    println!("lines {} of {} bytes each", commas(log.size / log::LINE), log::LINE);
    println!("read so far: {} bytes, in {} reads", log.served, log.reads);

    println!("\n== the first line: one read ==");
    let n = log.read(&mut buf);
    let end = buf[..n].iter().position(|&b| b == b'\n').unwrap_or(n);
    println!("read -> {n}");
    println!("first line: {:?}", String::from_utf8_lossy(&buf[..end]));
    println!("read so far: {} bytes, in {} read", commas(log.served), log.reads);
    println!("the same read holds lines 2 through {} whole, so they would cost no more.",
             n as u64 / log::LINE);
    println!("that is {:.7}% of the file. The other {} bytes were never made.",
             100.0 * log.served as f64 / log.size as f64, commas(log.size - log.served));

    println!("\n== what reading to the end would cost ==");
    let reads = log.size.div_ceil(buf.len() as u64);
    println!("{} reads of 1,024 bytes, then one more that returns 0", commas(reads));
    println!("a program that reads everything and prints only the first line gives the");
    println!("same output as the one above, and pays for all {} of them.", commas(reads));

    println!("\n== a source that never ends ==");
    let mut yes = Yes { reads: 0 };
    let n = yes.read(&mut buf);
    println!("yes: read -> {n}, first line {:?}", String::from_utf8_lossy(&buf[..1]));
    let mut zero_seen = false;
    for _ in 0..1000 {
        if yes.read(&mut buf) == 0 {
            zero_seen = true;
            break;
        }
    }
    println!("after {} reads, has read ever said 0? {zero_seen}", yes.reads);
    println!("`yes | head -n 1` finishes only because head stops asking.");
    println!("a reader that waits for 0 here waits forever.");
}
