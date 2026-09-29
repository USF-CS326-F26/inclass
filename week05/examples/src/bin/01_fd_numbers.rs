//! 01 — A descriptor is the lowest free slot in a table.
//!
//! `open` does not hand back a name or a pointer. It hands back an index into
//! a per-process table in the kernel, and it always picks the LOWEST free
//! slot. Three slots are taken before `main` runs:
//!
//!     fd    0       1       2       3     4     5
//!         [stdin ][stdout][stderr][    ][    ][    ] ...
//!
//!     open -> 3     open -> 4     open -> 5
//!     close(4)                    slot 4 is free again
//!     open -> 4                   not 6: the lowest free slot wins
//!
//! These are REAL descriptors from the real kernel, the one running this
//! program. `File::open` makes the `open` system call, `as_raw_fd()` shows
//! the number it returned, and dropping the `File` makes the `close`.
//!
//! Run:  cargo run --bin 01_fd_numbers

use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;

/// rv6 gives each process 16 slots: fds 0 through 15.
const NOFILE: i32 = 16;

fn main() {
    println!("== three are open before main runs ==");
    println!("stdin  is fd {}", std::io::stdin().as_raw_fd());
    println!("stdout is fd {}", std::io::stdout().as_raw_fd());
    println!("stderr is fd {}", std::io::stderr().as_raw_fd());
    println!("nobody in this program opened them. They came with the process.");

    println!("\n== open spends the lowest free slot ==");
    let mut a = File::open("/dev/null").expect("/dev/null is always there");
    let b = File::open("/dev/null").expect("/dev/null is always there");
    let c = File::open("/dev/null").expect("/dev/null is always there");
    let (fa, fb, fc) = (a.as_raw_fd(), b.as_raw_fd(), c.as_raw_fd());
    println!("open(\"/dev/null\") -> {fa}");
    println!("open(\"/dev/null\") -> {fb}");
    println!("open(\"/dev/null\") -> {fc}");
    println!("the same file three times, three different slots");

    println!("\n== close refunds the slot, and the next open takes it ==");
    drop(b); // File's Drop is the close system call
    println!("close({fb})");
    let d = File::open("/dev/null").expect("/dev/null is always there");
    let fd = d.as_raw_fd();
    println!("open(\"/dev/null\") -> {fd}");
    println!("{fd} again, not {}: the slot close gave back is the lowest free one", fc + 1);

    println!("\n== a leak: slots nobody refunds ==");
    let mut held = Vec::new();
    while let Ok(f) = File::open("/dev/null") {
        let n = f.as_raw_fd();
        held.push(f);
        if n >= NOFILE - 1 {
            break;
        }
    }
    let last = held.last().map_or(-1, |f| f.as_raw_fd());
    println!("opened {} more without closing any; the last one is fd {last}", held.len());
    println!("on rv6 that is all {NOFILE} slots. The next open fails, although /dev/null");
    println!("is right there. Nothing failed where the close was forgotten.");
    let leaked = held.len();
    drop(held); // closes every one of them
    let e = File::open("/dev/null").expect("/dev/null is always there");
    println!("close those {leaked}, then open(\"/dev/null\") -> {}", e.as_raw_fd());

    println!("\n== the number says nothing about what is behind it ==");
    let mut buf = [0u8; 64];
    println!("read(fd {fa}, 64 bytes)  -> {:?}   /dev/null is always at its end",
             a.read(&mut buf));
    let mut w = File::options().write(true).open("/dev/null").expect("writable");
    println!("write(fd {}, 13 bytes) -> {:?}   and swallows every write whole",
             w.as_raw_fd(), w.write(b"into the void"));
    println!("a file, a terminal, a pipe, /dev/null: all of them are read(fd, buf).");
}
