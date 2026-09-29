//! 03 — The buffer's size sets how many times you trap.
//!
//! Every `read` is a trip into the kernel: a trap, a table lookup, a copy,
//! a return. That fixed cost does not shrink when the request does, so what
//! a bigger buffer buys is FEWER CALLS:
//!
//!     copying 1 MiB (1,048,576 bytes), not counting the final read of 0
//!       1-byte buffer    1,048,576 reads
//!     512-byte buffer        2,048 reads   0.2% as many
//!       4 KiB buffer           256 reads   and your whole stack
//!
//! `Source` below is a 1 MiB file that fills the buffer whenever it can, the
//! way the host does, and counts every call. Then it is a file on rv6, whose
//! kernel never returns more than 128 bytes per read, whatever you ask for.
//!
//! Run:  cargo run --bin 03_buffer_size_counts_calls

const MIB: usize = 1024 * 1024;

/// A file of `left` bytes. Each read returns at most `cap` of them.
struct Source {
    left: usize,
    cap: usize,
    calls: usize,
}

impl Source {
    fn new(size: usize, cap: usize) -> Source {
        Source { left: size, cap, calls: 0 }
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        self.calls += 1;
        let n = self.left.min(buf.len()).min(self.cap);
        buf[..n].fill(b'x');
        self.left -= n;
        n
    }
}

/// Read to the end through `buf`. Returns (reads that returned data, all calls).
fn drain(src: &mut Source, buf: &mut [u8]) -> (usize, usize) {
    let mut data = 0;
    loop {
        if src.read(buf) == 0 {
            return (data, src.calls);
        }
        data += 1;
    }
}

/// 1048576 -> "1,048,576"
fn commas(n: usize) -> String {
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
    // One array. A buffer of B bytes is its first B bytes.
    let mut big = [0u8; 65536];

    println!("== 1 MiB through three buffers ==");
    let mut counts = Vec::new();
    for size in [1, 512, 4096] {
        let (data, calls) = drain(&mut Source::new(MIB, usize::MAX), &mut big[..size]);
        println!("{:>5}-byte buffer  {:>9} reads, then the 0: {} calls",
                 size, commas(data), commas(calls));
        counts.push(data);
    }
    println!("512 bytes is {:.1}% of the calls a 1-byte buffer makes",
             100.0 * counts[1] as f64 / counts[0] as f64);

    println!("\n== the first factor of 512 is nearly all of it ==");
    let (big_reads, _) = drain(&mut Source::new(MIB, usize::MAX), &mut big[..65536]);
    println!("     1 -> 512   saves {} calls", commas(counts[0] - counts[1]));
    println!("   512 -> 4096  saves {} calls", commas(counts[1] - counts[2]));
    println!("  4096 -> 65536 saves {} calls   ({} left)", commas(counts[2] - big_reads), big_reads);
    println!("512 to 64 KiB removes {} of the last {} calls, at 128 times the memory",
             commas(counts[1] - big_reads), commas(counts[1]));

    println!("\n== what the buffer costs: one 4 KiB stack page ==");
    for size in [512, 1024, 4096, 8192] {
        let verdict = match size {
            s if s < 4096 => "fits, with room for everything else",
            4096 => "the whole page: nothing left for any other local",
            _ => "does not fit: sp runs off the page",
        };
        println!("{:>5} bytes = {:>5.1}% of the stack   {verdict}",
                 size, 100.0 * size as f64 / 4096.0);
    }
    println!("that is why cat and wc use 512 bytes and grep uses 1,024.");

    println!("\n== on rv6: a kernel that returns at most 128 bytes ==");
    for size in [512, 4096] {
        let (data, _) = drain(&mut Source::new(MIB, 128), &mut big[..size]);
        println!("{:>5}-byte buffer  {:>9} reads", size, commas(data));
    }
    println!("past 128 the count stops falling: the kernel, not the buffer, sets it.");
    println!("the loop is the same on both. Only the number of trips changes.");
}
