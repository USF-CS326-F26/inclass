//! 02 — `iter()` lends, `iter_mut()` lends exclusively.
//!
//! xv6's `wakeup(chan)` walks the whole process table and moves every process
//! sleeping on `chan` to runnable. That loop has to WRITE into the table, and
//! it has to know WHICH slot it is writing. Two things, two tools:
//!
//!     for c in chan.iter()        c: &Option<usize>       read only
//!     for c in chan.iter_mut()    c: &mut Option<usize>   one exclusive borrow at a time
//!     .enumerate()                (index, item)           the slot number comes along
//!
//! The write is `*c = None`. The star is the whole lesson: `c` is a reference
//! INTO the array, and assigning through it changes the array. Without the
//! star you would be trying to overwrite the reference itself.
//!
//! Run:  cargo run --bin 02_iter_mut_and_enumerate

/// Which channel each slot is sleeping on, or `None` if it is not asleep.
/// (In the kernel the channel is an address: the lock, the buffer, the pipe.)
const NPROC: usize = 8;

fn main() {
    let mut chan: [Option<usize>; NPROC] =
        [None, Some(0x10), None, Some(0x20), Some(0x10), None, None, Some(0x10)];
    let target = 0x10;

    println!("== the table ==");
    show(&chan);

    println!("\n== iter(): each item is a &Option<usize>; the table is untouched ==");
    let mut sleepers = 0;
    for c in chan.iter() {
        if *c == Some(target) {
            sleepers += 1;
        }
    }
    println!("{sleepers} slots sleep on {target:#x}");
    println!("`c` is a &Option<usize>, so the comparison needs `*c` -- or `Some(&t)`");

    println!("\n== enumerate(): the index rides along ==");
    let mut which = Vec::new();
    for (i, c) in chan.iter().enumerate() {
        if *c == Some(target) {
            which.push(i);
        }
    }
    println!("they are slots {which:?}");
    println!("a kernel almost always needs the INDEX: it is the pid, the fd, the frame");

    println!("\n== iter_mut(): each item is a &mut Option<usize>; the write goes in ==");
    for (i, c) in chan.iter_mut().enumerate() {
        if *c == Some(target) {
            *c = None; // <- the star: assign THROUGH the reference, into the array
            println!("  woke slot {i}");
        }
    }
    println!("after wakeup({target:#x}):");
    show(&chan);
    println!("`c` is a &mut Option<usize>. `*c = None` writes into `chan`.");
    println!("`c = None` would not compile: c is a reference, not an Option.");

    println!("\n== what you may NOT do inside that loop ==");
    println!("    for (i, c) in chan.iter().enumerate() {{");
    println!("        if *c == Some(target) {{ chan[i] = None; }}   // error[E0506]");
    println!("    }}");
    println!("`iter()` holds a shared borrow of the whole table for the whole loop;");
    println!("`chan[i] = ..` writes into it while that borrow is alive.");
    println!("RUN  ./show-errors.sh e0506");

    println!("\n== the off-by-three ==");
    let tail = &chan[3..];
    for (i, c) in tail.iter().enumerate() {
        if c.is_some() {
            println!("  tail slot {i} is asleep -- that is chan[{}], not chan[{i}]", i + 3);
        }
    }
    println!("enumerate counts the thing you handed it, from 0. Slice first,");
    println!("and the numbers are the slice's, not the table's.");
}

fn show(chan: &[Option<usize>]) {
    for (i, c) in chan.iter().enumerate() {
        match c {
            Some(ch) => println!("  [{i}] sleeping on {ch:#x}"),
            None => println!("  [{i}] -"),
        }
    }
}
