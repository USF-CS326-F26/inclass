//! 05 — Copy: values that do not move.
//!
//! If a type owns no resource — no heap buffer, no file, no lock — then a
//! bitwise duplicate is a perfectly good second value. Those types are `Copy`,
//! and assignment duplicates instead of moving.
//!
//!     let x = 5;  let y = x;      x: [ 5 ]   y: [ 5 ]   both usable
//!     let a = s;  let b = a;      a: dead    b: [ptr|len|cap]
//!
//! The rule that makes it sound: **Copy and Drop are mutually exclusive.**
//! If a copy existed of something with a destructor, the release would run
//! once per copy — a double free.
//!
//! Run:  cargo run --bin 05_copy_types

/// Plain data: two integers. Duplicating the bits duplicates the value.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pte {
    ppn: usize,
    flags: u8,
}

/// Same idea, but this one owns a heap buffer, so it cannot be Copy.
#[derive(Debug, Clone)]
struct Device {
    name: String,
}

fn main() {
    println!("== scalars copy ==");
    let x: i32 = 5;
    let y = x;                      // copy, not move
    println!("x = {x}, y = {y}   both still usable; separate slots {:p} {:p}", &x, &y);

    println!("\n== a Copy struct copies ==");
    let mut a = Pte { ppn: 0x8_0005, flags: 0b111 };
    let b = a;                      // copy
    a.flags = 0;                    // mutating `a` does not touch `b`
    println!("a = {a:?}  at {:p}", &a);
    println!("b = {b:?}  at {:p}   <- independent 16-byte copy", &b);

    println!("\n== arrays of Copy types copy ==");
    let t1 = [Pte { ppn: 1, flags: 1 }; 3];
    let mut t2 = t1;                // whole array duplicated
    t2[0].ppn = 99;
    println!("t1[0] = {:?}", t1[0]);
    println!("t2[0] = {:?}   <- t1 untouched", t2[0]);

    println!("\n== a type that owns something moves instead ==");
    let d1 = Device { name: String::from("uart0") };
    let d2 = d1;                    // move: Device is not Copy
    // println!("{d1:?}");          // error[E0382]: borrow of moved value
    println!("d2 = {d2:?}   (d1 is moved-from; Device owns a String)");
    let d3 = d2.clone();            // an explicit, visible allocation
    println!("clone -> different buffers: {:p} vs {:p}",
             d2.name.as_ptr(), d3.name.as_ptr());

    println!("\n== Copy XOR Drop ==");
    // Uncomment either half of this pair and it will not compile:
    //   #[derive(Clone, Copy)] struct Guard;
    //   impl Drop for Guard { fn drop(&mut self) {} }
    // error[E0184]: the trait `Copy` cannot be implemented for this type;
    //               the type has a destructor
    println!("A destructor means one release point. A copy means many values.");
    println!("Rust refuses the combination — that is the double free, rejected.");

    println!("\n== references are Copy; what they point at is not ==");
    let owner = String::from("free list");
    let r1: &String = &owner;
    let r2 = r1;                    // copying a reference is fine: many readers
    println!("r1 = {r1:?} r2 = {r2:?}  both point at {:p}", r1.as_ptr());
}
