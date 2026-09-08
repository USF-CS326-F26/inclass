//! 05 — const fn: arithmetic the compiler does for you.
//!
//! A `const fn` may ALSO be evaluated by the compiler, before the program runs.
//! `const` only ADDS an ability -- the same function is callable at run time.
//!
//!     const CAP: usize = kib(4);      const-evaluated, baked into the image
//!     let n = kib(runtime_value);     ordinary call, ordinary instructions
//!
//! A CONST CONTEXT is any place the value must be known before the program
//! runs: a `const` or `static` item, an array length, an array repeat, an enum
//! discriminant.
//!
//!     +---------------------+  yes  +------------------+  +-------------------+
//!     |  const / static /   |------>|  const-eval in   |->| a literal in the  |
//!     |  array length?      |       |  the compiler    |  | executable image  |
//!     +---------------------+       +------------------+  +-------------------+
//!               | no
//!               v
//!        code generation -> shifts and adds at run time
//!
//! Why a kernel needs this: a static table must be valid when the first Rust
//! function is entered, and at that point there is no heap, no allocator, and
//! nothing that could have run an initialization loop.
//!
//! Run:  cargo run --bin 05_const_fn

use std::mem::size_of;

fn main() {
    println!("== the same function, both ways ==");
    println!("KERNEL_STACK = kib(8) = {KERNEL_STACK}   <- computed by rustc");
    let pages = read_a_number();
    println!("kib({pages}) = {}   <- computed at run time", kib(pages));
    println!("`const fn` does not mean `only at compile time`");

    println!("\n== a const context forces the evaluation ==");
    const BLOCKS: u32 = blocks_for(5000);
    println!("const BLOCKS: u32 = blocks_for(5000) = {BLOCKS}");
    println!("no divide instruction exists anywhere in this binary for it");
    println!("blocks_for(5000) at run time = {}", blocks_for(read_bytes()));

    println!("\n== an array length is a const context ==");
    let buf = [0u8; BUF_LEN];
    println!("const BUF_LEN: usize = kib(1) / 8 = {BUF_LEN}");
    println!("[0u8; BUF_LEN] is {} bytes on the stack", buf.len());
    println!("size_of::<[u8; BUF_LEN]>() = {}", size_of::<[u8; BUF_LEN]>());

    println!("\n== a whole table, initialized with no code running ==");
    println!("static SLOTS: [Slot; {NSLOT}] = [const {{ Slot::new() }}; {NSLOT}];");
    println!("SLOTS[0]  = {:?}", SLOTS[0]);
    println!("SLOTS[7]  = {:?}", SLOTS[7]);
    println!("all {} of them were written into the executable by the compiler",
             SLOTS.len());
    println!("size_of::<[Slot; {NSLOT}]>() = {} bytes reserved at link time",
             size_of::<[Slot; NSLOT]>());

    println!("\n== ... and a const one, built the same way ==");
    println!("const READY: Slot = Slot::started(3);");
    println!("READY = {READY:?}   <- a literal in the image, not a call");
    println!("READY.is_busy() = {}", READY.is_busy());

    println!("\n== what C had to do instead ==");
    println!("C static initializers allow constants only: `= 0` yes, a call no.");
    println!("Hence the pile of xxx_init() routines whose whole job is to fill");
    println!("in what the language could not. C++ answered with constexpr in");
    println!("2011; Rust's const fn stabilized in 2018.");

    println!("\n== what you may write inside a const fn ==");
    println!("  arithmetic, comparisons, if, match, loop, other const fns");
    println!("  NOT: allocate, call a non-const fn, deref a raw pointer");
    println!("try it: put `read_a_number()` in a const and read E0015");
    println!("\n**RUN** ../examples/show-errors.sh e0015");
}

/// Bytes in `n` kibibytes.
pub const fn kib(n: usize) -> usize {
    n * 1024
}

/// How many 512-byte blocks `bytes` occupies, rounded up.
///
/// `if` and `match` are both allowed inside a `const fn`.
pub const fn blocks_for(bytes: u32) -> u32 {
    if bytes == 0 {
        0
    } else {
        (bytes + 511) / 512
    }
}

/// A const context: this is evaluated by rustc, not at start-up.
pub const KERNEL_STACK: usize = kib(8);

/// So is this, and it is then used as an array length below.
pub const BUF_LEN: usize = kib(1) / 8;

/// How many slots the table has.
pub const NSLOT: usize = 8;

/// One entry of a fixed table. Not `Copy`, on purpose -- see `SLOTS`.
#[derive(Debug)]
pub struct Slot {
    pub owner: u32,
    pub busy: bool,
}

impl Slot {
    /// The compiler can run this, so a `static` array of them needs no loop.
    pub const fn new() -> Slot {
        Slot { owner: 0, busy: false }
    }

    pub const fn started(owner: u32) -> Slot {
        Slot { owner, busy: true }
    }

    pub const fn is_busy(&self) -> bool {
        self.busy
    }
}

/// Eight fully initialized slots, in the executable, before anything runs.
///
/// The `[const { ... }; N]` form is needed because the plain repeat form
/// requires the element type to be `Copy`, and `Slot` is not.
pub static SLOTS: [Slot; NSLOT] = [const { Slot::new() }; NSLOT];

/// A const item built by a `const fn` call.
pub const READY: Slot = Slot::started(3);

/// An ordinary function -- deliberately NOT const, to make the point.
fn read_a_number() -> usize {
    4
}

fn read_bytes() -> u32 {
    5000
}
