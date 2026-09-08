//! 08 — Enums: exactly one of these.
//!
//! A struct is "this AND that" -- a product type. An enum is "this OR that" --
//! a SUM type. Each name is a VARIANT, and the type has exactly that many
//! values. There is no Trap equal to 47.
//!
//!     C:     #define TIMER 0        an int wearing a costume:
//!            int cause = 47;        compiles, runs, means nothing
//!
//!     Rust:  enum Trap { Timer, ... }
//!            let c: Trap = 47;      error[E0308]
//!
//! A variant may carry data of its own, which is where enums stop being
//! #defines with better manners. In memory that is a DISCRIMINANT (a tag saying
//! which variant) plus room for the largest payload:
//!
//!     Trap::PageFault { addr: 0x8000_1000 }
//!     +-----+--------------------------+
//!     | tag |   addr (largest payload) |
//!     +-----+--------------------------+
//!
//! Run:  cargo run --bin 08_enums

use std::mem::size_of;

fn main() {
    println!("== unit variants: a closed set of names ==");
    let a = Level::Machine;
    let b = Level::User;
    println!("Level::Machine = {a:?}, Level::User = {b:?}");
    println!("a == b -> {}", a == b);
    println!("size_of::<Level>() = {}   <- 3 variants fit in one byte",
             size_of::<Level>());
    println!("there is no Level equal to 47; the type has three values");

    println!("\n== variants that carry data ==");
    // Struct-style fields:
    let fault = Trap::PageFault { addr: 0x8000_1000 };
    // Tuple-style fields:
    let call = Trap::Syscall(63);
    // No fields at all:
    let tick = Trap::Timer;
    println!("{}", show(fault));
    println!("{}", show(call));
    println!("{}", show(tick));
    println!("one value answers two questions at once: what happened, and with what");

    println!("\n== how they are built ==");
    println!("  Trap::Timer                        no fields");
    println!("  Trap::Syscall(63)                  tuple fields, built like a call");
    println!("  Trap::PageFault {{ addr: 0x8000 }}    named fields, built like a struct");

    println!("\n== the layout: a tag plus the biggest payload ==");
    println!("size_of::<Trap>()   = {}", size_of::<Trap>());
    println!("size_of::<usize>()  = {}   <- the largest payload", size_of::<usize>());
    println!("the tag rides along, rounded up for alignment");
    println!("size_of::<[Trap; 4]>() = {}", size_of::<[Trap; 4]>());

    println!("\n== a C API says this with an int and an out-parameter ==");
    println!("  int  cause;");
    println!("  long extra;      /* meaningful for SOME causes. which? */");
    println!("Here the pairing is enforced: a Timer has no address to reach.");

    println!("\n== Option<T> is just an enum the library wrote ==");
    println!("  enum Option<T> {{ Some(T), None }}");
    let found: Option<u32> = Some(7);
    let missing: Option<u32> = None;
    println!("Some(7) = {found:?}, None = {missing:?}");
    println!("size_of::<u32>()         = {}", size_of::<u32>());
    println!("size_of::<Option<u32>>() = {}   <- 4 bytes + a tag, aligned",
             size_of::<Option<u32>>());

    println!("\n== niche optimization: sometimes the tag is free ==");
    println!("size_of::<&u8>()         = {}", size_of::<&u8>());
    println!("size_of::<Option<&u8>>() = {}   <- SAME SIZE", size_of::<Option<&u8>>());
    println!("a reference is never null, so all-zeroes is a bit pattern it can");
    println!("never hold -- and that unused pattern becomes `None`");

    println!("\n== enums are how a kernel writes a closed set ==");
    for t in [Trap::Timer, Trap::Syscall(63), Trap::PageFault { addr: 0x8000_1000 },
              Trap::Illegal] {
        println!("  {:<38} kernel-handled: {}", show(t), t.is_kernel_handled());
    }
    println!("program 09 is the `match` that reads them");
}

/// Three privilege levels, and nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Machine,
    Supervisor,
    User,
}

/// Why the hardware stopped the program. Some causes carry a detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trap {
    /// The timer fired. Nothing else to say.
    Timer,
    /// A program asked for a service, by number. Tuple-style fields.
    Syscall(usize),
    /// A memory access failed, at this address. Struct-style fields.
    PageFault { addr: usize },
    /// The instruction was not one the CPU knows.
    Illegal,
}

impl Trap {
    /// A taste of program 09: one arm per variant, no catch-all.
    pub fn is_kernel_handled(&self) -> bool {
        match self {
            Trap::Timer => true,
            Trap::Syscall(_) => true,
            Trap::PageFault { .. } => true,
            Trap::Illegal => false,
        }
    }
}

/// The `Debug` derive prints integers in decimal, and an address is only
/// readable in hex. This is not a `match` lesson -- it just makes the columns
/// below legible.
fn show(t: Trap) -> String {
    match t {
        Trap::Timer => String::from("Timer"),
        Trap::Syscall(num) => format!("Syscall({num})"),
        Trap::PageFault { addr } => format!("PageFault {{ addr: {addr:#x} }}"),
        Trap::Illegal => String::from("Illegal"),
    }
}
