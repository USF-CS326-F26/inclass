//! 01 — Structs: naming a bundle of values.
//!
//! A struct bundles several values into one named type that travels as a unit.
//! In memory it is a contiguous block of bytes and nothing else: no object
//! header, no type pointer, no vtable, no allocation. That is the only reason
//! a struct can sit at a hardware boundary.
//!
//!     struct Console { base, chars_out, errors }
//!
//!     +----------------+----------------+----------------+
//!     |      base      |   chars_out    |     errors     |
//!     +----------------+----------------+----------------+
//!      0                8               16               24   <- byte offsets
//!
//! A struct is a PRODUCT type: a Console is a base AND a count AND a count.
//! Program 08 is the other half of that pair.
//!
//! Run:  cargo run --bin 01_structs

use std::mem::size_of;

fn main() {
    println!("== building one ==");

    // A struct literal names the type and every field.
    let uart = Console { base: 0x1000_0000, chars_out: 0, errors: 0 };
    println!("uart.base      = {:#x}", uart.base);
    println!("uart.chars_out = {}", uart.chars_out);

    println!("\n== field init shorthand ==");
    // When a local already has the field's name, write it once.
    let base = 0x1001_0000;
    let second = Console { base, chars_out: 7, errors: 0 };
    println!("Console {{ base }} means Console {{ base: base }} -> {:#x}", second.base);

    println!("\n== a struct is its fields, back to back ==");
    println!("size_of::<Console>()   = {}", size_of::<Console>());
    println!("3 x size_of::<usize>() = {}", 3 * size_of::<usize>());
    println!("no header, no vtable, no allocation -- the value IS the bytes");

    println!("\n== which is why an array of them is exactly N times as big ==");
    println!("size_of::<[Console; 4]>() = {}", size_of::<[Console; 4]>());
    let table = [Console { base: 0, chars_out: 0, errors: 0 }; 4];
    println!("first element at {:p}", &table[0]);
    println!("next  element at {:p}   <- {} bytes further on",
             &table[1], size_of::<Console>());

    println!("\n== structs nest by value ==");
    // `stats` is stored INSIDE the driver, not behind a pointer to it.
    let driver = Driver {
        console: Console { base: 0x1000_0000, chars_out: 3, errors: 0 },
        stats: Stats { reads: 12, writes: 4 },
    };
    println!("size_of::<Driver>() = {}   (= Console {} + Stats {})",
             size_of::<Driver>(), size_of::<Console>(), size_of::<Stats>());
    println!("driver         at {:p}", &driver);
    println!("driver.console at {:p}", &driver.console);
    println!("driver.stats   at {:p}", &driver.stats);
    println!("both sit INSIDE the driver's own {} bytes -- no pointer, no",
             size_of::<Driver>());
    println!("second allocation, nothing to dereference");
    println!("which one comes first is the compiler's choice (program 06)");
    println!("reads = {}, writes = {}", driver.stats.reads, driver.stats.writes);

    println!("\n== mutation needs `mut` on the binding ==");
    let mut counter = Stats { reads: 0, writes: 0 };
    counter.reads += 1;
    counter.writes += 2;
    println!("after two updates: {:?}", counter);
    // let fixed = Stats { reads: 0, writes: 0 };
    // fixed.reads += 1;            // error[E0594]: cannot assign to `fixed.reads`

    println!("\n== private fields are how a module protects itself ==");
    let log = ErrorLog::new();
    println!("count = {}", log.count());
    // println!("{}", log.count_field);  // error[E0616]: field is private
    println!("`count_field` is private, so the only way in is a method");
    println!("that is how a kernel module stops the rest of the kernel");
    println!("from corrupting its data behind its back");
}

/// A memory-mapped serial port, plus what we have sent through it.
///
/// `pub` on the struct exports the type; `pub` on a field exports access to
/// that field. Leaving it off makes the field private to this file.
#[derive(Debug, Clone, Copy)]
pub struct Console {
    pub base: usize,
    pub chars_out: usize,
    pub errors: usize,
}

/// Two counters. Program 02 gives this one behavior.
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub reads: usize,
    pub writes: usize,
}

/// A struct whose fields are themselves structs, stored by value.
pub struct Driver {
    pub console: Console,
    pub stats: Stats,
}

/// A struct with nothing public about it at all.
pub struct ErrorLog {
    count_field: usize,
}

impl ErrorLog {
    pub fn new() -> ErrorLog {
        ErrorLog { count_field: 0 }
    }

    pub fn count(&self) -> usize {
        self.count_field
    }
}
