//! 12 — Fixed tables, and the iterators that walk them.
//!
//! Every operating system keeps lists of things: processes, open files, free
//! pages, buffered keystrokes. Where those lists live, and whether they may
//! grow, is one of the first real design decisions in a kernel.
//!
//!     static mut PROCS: [Proc; NPROC]      a plain array of 64 slots
//!
//! Three reasons it is an array and not a Vec, and none of them is taste:
//!
//!     1. THERE IS NO ALLOCATOR YET. The table is used on the fourth line of
//!        boot. The allocator is itself a data structure and cannot allocate.
//!     2. THE FAULT PATH MUST NOT ALLOCATE. Allocation can fail, can take
//!        unbounded time, and takes locks -- none of which a trap handler
//!        with interrupts off can survive.
//!     3. A HARD LIMIT FAILS HONESTLY. The 65th fork returns -1, at the call
//!        that asked for too much. A Vec fails later, somewhere else.
//!
//! Rust's iterators are LAZY and allocate nothing, which is why this style is
//! legal in a kernel with no heap at all. `collect` is the exception.
//!
//! Run:  cargo run --bin 12_tables_and_iterators

use std::mem::size_of;

/// The compile-time bound. Raising it is a code change with a known cost.
pub const NDEV: usize = 8;

fn main() {
    let table = device_table();

    println!("== a fixed table costs what it costs, always ==");
    println!("size_of::<DevEntry>()       = {}", size_of::<DevEntry>());
    println!("size_of::<[DevEntry; {NDEV}]>() = {}   = {NDEV} x {}",
             size_of::<[DevEntry; NDEV]>(), size_of::<DevEntry>());
    println!("reserved at link time, whether one device is live or {NDEV}");
    println!("you always pay for the worst case -- and in exchange the worst");
    println!("case can never surprise you");

    println!("\n== iter(): each item is &T, the table is untouched ==");
    for e in table.iter() {
        println!("  major {:<3} {:<10} reads {}", e.major, e.name, e.reads);
    }

    println!("\n== enumerate(): kernels care WHERE a thing is ==");
    for (index, e) in table.iter().enumerate() {
        if e.major != 0 {
            println!("  slot {index} holds major {}", e.major);
        }
    }
    println!("enumerate counts from 0 regardless of what it counts:");
    println!("on `table[3..].iter().enumerate()` the numbers are NOT the");
    println!("table's indices -- a classic off-by-three");

    println!("\n== position(): the index of the first match ==");
    println!("position(major == 2) = {:?}",
             table.iter().position(|e| e.major == 2));
    println!("position(major == 9) = {:?}",
             table.iter().position(|e| e.major == 9));

    println!("\n== find(): the first matching item ==");
    match table.iter().find(|e| e.name == "disk") {
        Some(e) => println!("found disk at major {}", e.major),
        None => println!("no disk"),
    }

    println!("\n== any() / all() / count() / filter() / map() ==");
    println!("any(reads > 100)  = {}", table.iter().any(|e| e.reads > 100));
    println!("all(reads < 1000) = {}", table.iter().all(|e| e.reads < 1000));
    println!("count(live)       = {}", table.iter().filter(|e| e.major != 0).count());
    println!("total reads       = {}", table.iter().map(|e| e.reads).sum::<usize>());
    println!("busiest           = {:?}",
             table.iter().filter(|e| e.major != 0).max_by_key(|e| e.reads).map(|e| e.name));

    println!("\n== iter_mut(): each item is &mut T, modified in place ==");
    let mut counters = table;
    for e in counters.iter_mut() {
        if e.major != 0 {
            e.reads += 1;
        }
    }
    println!("after one pass: {:?}",
             counters.iter().filter(|e| e.major != 0).map(|e| e.reads).collect::<Vec<_>>());
    println!("which one you may use is decided by your SIGNATURE, not by taste:");
    println!("a function that took &[T] borrowed for reading, so iter_mut");
    println!("will not compile there");

    println!("\n== lazy: nothing is built, and it stops early ==");
    let mut examined = 0;
    let hit = table
        .iter()
        .map(|e| {
            examined += 1;
            e
        })
        .find(|e| e.major == 2);
    println!("find(major == 2) -> {:?}", hit.map(|e| e.name));
    println!("the closure ran {examined} times, not {NDEV}");
    println!("`map` does not produce a list -- it computes one item each time");
    println!("`find` asks, and `find` stops asking the moment it succeeds");
    println!("the compiled code is the loop you would have written by hand");

    println!("\n== collect() is the one adapter that allocates ==");
    let names: Vec<&str> = table.iter().filter(|e| e.major != 0).map(|e| e.name).collect();
    println!("collect() -> {names:?}");
    println!("it has to put the results SOMEWHERE. Host code and tests only,");
    println!("until the kernel has a heap.");

    println!("\n== a chain that reads as English ==");
    let mut next = 3;
    for _ in 0..3 {
        let from = next;
        let pick = round_robin(&table, &mut next);
        println!("  start at {from} -> picked {:?}, resume at {next}", pick);
    }
    println!("consider N offsets, wrap each into a table index starting where");
    println!("we left off, take the first live one, remember to resume after it");

    println!("\n== the hard limit, and what happens at it ==");
    let mut live = device_table();
    println!("{NDEV} slots, 3 already taken, so 5 installs should fit:");
    for major in 4u32..=11 {
        match install(&mut live, major) {
            Some(slot) => println!("  major {major:<2} -> slot {slot}"),
            None => println!("  major {major:<2} -> None: table full. This is `open` returning -1."),
        }
    }
    println!("The failure lands at the exact call that asked for too much, and");
    println!("it is testable: fill the table and check that the next one fails.");
    println!("With a Vec the limit is `until memory runs out`, which sounds");
    println!("more generous and is much worse: the failure arrives late, at an");
    println!("unrelated allocation elsewhere, and you cannot test it.");

    println!("\n== where Vec DOES belong ==");
    println!("Host commands, tests, and the kernel after it has a heap --");
    println!("allocated at initialization, never on the trap path.");
    println!("`Vec` is not bad. It is a tool with a prerequisite.");
}

/// One row of a fixed device table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DevEntry {
    pub major: u32,
    pub name: &'static str,
    pub reads: usize,
}

impl DevEntry {
    /// `const fn`, so the table below needs no start-up loop.
    pub const fn empty() -> DevEntry {
        DevEntry { major: 0, name: "-", reads: 0 }
    }
}

fn device_table() -> [DevEntry; NDEV] {
    let mut t = [DevEntry::empty(); NDEV];
    t[0] = DevEntry { major: 1, name: "console", reads: 120 };
    t[1] = DevEntry { major: 2, name: "disk", reads: 4096 };
    t[2] = DevEntry { major: 3, name: "null", reads: 0 };
    t
}

/// A policy that never learns how big the table is: it reads `table.len()`.
fn round_robin(table: &[DevEntry], next: &mut usize) -> Option<&'static str> {
    let n = table.len();
    (0..n)
        .map(|off| (*next + off) % n)
        .find(|&i| table[i].major != 0)
        .map(|i| {
            *next = (i + 1) % n;
            table[i].name
        })
}

/// Claim a slot, or report honestly that there is none.
fn install(table: &mut [DevEntry], major: u32) -> Option<usize> {
    for i in 0..table.len() {
        if table[i].major == 0 {
            table[i] = DevEntry { major, name: "new", reads: 0 };
            return Some(i);
        }
    }
    None
}
