//! 11 — Arrays, slices, and Vec: three ways to hold N things.
//!
//!     [T; N]   the values ARE the variable.  N is in the type; never grows.
//!     Vec<T>   OWNS a heap buffer.          ptr + len + cap, 24 bytes.
//!     &[T]     POINTS AT data you do not own. ptr + len, 16 bytes.
//!
//!       [u8; 4]
//!       +----+----+----+----+
//!       | 10 | 20 | 30 | 40 |     stack or .bss, size fixed at compile time
//!       +----+----+----+----+
//!        ^
//!        |  &arr[1..3]
//!        |  +-------+-------+
//!        +--|  ptr  | len=2 |     a slice: 16 bytes, borrowing the middle two
//!           +-------+-------+
//!
//! `[T]` on its own is legal but UNSIZED -- you can never hold one, because
//! the compiler cannot say how much stack to reserve. It is usable only behind
//! a pointer, and that pointer carries the missing length: a FAT POINTER.
//!
//! Run:  cargo run --bin 11_slices

use std::mem::size_of;

fn main() {
    println!("== the three types ==");
    let arr: [u8; 8] = [b'k', b'e', b'r', b'n', b'e', b'l', 0, 0];
    let owned: Vec<u8> = vec![b'k', b'e', b'r', b'n'];
    let view: &[u8] = &arr[..6];
    println!("array  {:?}  at {:p}", &arr[..], arr.as_ptr());
    println!("vec    {:?}      at {:p}   <- heap", owned, owned.as_ptr());
    println!("slice  {:?}  at {:p}   <- points into the array",
             view, view.as_ptr());

    println!("\n== a slice is a fat pointer ==");
    println!("size_of::<&u8>()        = {:>2}   thin", size_of::<&u8>());
    println!("size_of::<&[u8; 8]>()   = {:>2}   thin: the 8 is in the type",
             size_of::<&[u8; 8]>());
    println!("size_of::<&[u8]>()      = {:>2}   FAT: address + length",
             size_of::<&[u8]>());
    println!("size_of::<[u8; 8]>()    = {:>2}   the data itself", size_of::<[u8; 8]>());
    println!("size_of::<Vec<u8>>()    = {:>2}   ptr + len + cap", size_of::<Vec<u8>>());
    println!("&[u8] could point at three elements or three million");

    println!("\n== making slices from ranges ==");
    println!("&arr[..]     {:?}", &arr[..]);
    println!("&arr[..3]    {:?}", &arr[..3]);
    println!("&arr[3..]    {:?}", &arr[3..]);
    println!("&arr[2..5]   {:?}", &arr[2..5]);

    println!("\n== the prefix that holds real data ==");
    let mut buf = [0u8; 16];
    let n = read_into(&mut buf);
    println!("read_into filled {n} of {} bytes", buf.len());
    println!("&buf[..]   -> {} bytes, most of them stale", buf.len());
    println!("&buf[..n]  -> {} bytes, all of them real", buf[..n].len());
    println!("text = {:?}", core::str::from_utf8(&buf[..n]).unwrap());
    println!("the slice says `this many of those` without a second variable");
    println!("that can drift out of sync");

    println!("\n== one function, every container ==");
    println!("checksum(&arr)      = {}", checksum(&arr));
    println!("checksum(&owned)    = {}", checksum(&owned));
    println!("checksum(&arr[2..5]) = {}", checksum(&arr[2..5]));
    println!("arrays and Vecs convert to slices automatically");
    println!("write `&[T]` to read and `&mut [T]` to write, and one signature");
    println!("serves a table of any size");

    println!("\n== &mut [T] is an EXCLUSIVE borrow of the whole run ==");
    let mut page = [0u8; 8];
    fill(&mut page, 0xff);
    println!("after fill: {page:?}");
    // let a = &mut page[0];
    // let b = &mut page[1];        // error[E0499]: the compiler cannot prove 0 != 1
    let (left, right) = page.split_at_mut(4);
    left[0] = 1;
    right[0] = 2;
    println!("split_at_mut: two writers, provably disjoint -> {page:?}");

    println!("\n== indexing is checked ==");
    println!("table[i]      panics on a bad index      -- i is yours, provably in range");
    println!("table.get(i)  returns None               -- out of range is normal");
    println!("arr.get(3)  = {:?}", arr.get(3));
    println!("arr.get(99) = {:?}", arr.get(99));
    println!("in RISC-V terms `table[i]` is three extra instructions:");
    println!("    bgeu  a1, t0, .Lpanic     unsigned, so it catches both ends");
    println!("    slli  t1, a1, 3");
    println!("    add   t1, a0, t1");
    println!("a branch that is never taken, and so predicted perfectly");

    println!("\n== validate untrusted indices at the boundary ==");
    for fd in [1usize, 99] {
        println!("  get_file({fd}) = {:?}", get_file(fd));
    }
    println!("A panic in a kernel is not a failed test: the handler prints and");
    println!("halts the machine. A user program that can panic the kernel with");
    println!("one bad system-call argument owns a denial of service. Check");
    println!("where the number ENTERS, once, and index freely after that.");
    println!("In C the missing check reads into a neighbouring process.");
    println!("\n**RUN** ../examples/show-errors.sh e0080");
}

pub const NOFILE: usize = 4;

/// Reads to a slice and reports how many bytes it actually delivered.
fn read_into(buf: &mut [u8]) -> usize {
    let msg = b"kernel";
    buf[..msg.len()].copy_from_slice(msg);
    msg.len()
}

/// One signature, every container: `&[T]` to read.
fn checksum(bytes: &[u8]) -> u32 {
    let mut sum = 0u32;
    for &b in bytes {
        sum = sum.wrapping_add(b as u32);
    }
    sum
}

/// `&mut [T]` to write.
fn fill(bytes: &mut [u8], value: u8) {
    for b in bytes.iter_mut() {
        *b = value;
    }
}

/// The kernel shape: refuse the index before it ever reaches the array.
fn get_file(fd: usize) -> Option<&'static str> {
    if fd >= NOFILE {
        return None;
    }
    let ofile = ["stdin", "stdout", "stderr", "disk"];
    Some(ofile[fd])
}
