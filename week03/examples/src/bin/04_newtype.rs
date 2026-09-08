//! 04 — The newtype pattern: same bits, a different type.
//!
//! A kernel written only in integers cannot be checked. A block number, a byte
//! count, a device number and a permission word are all "just a number", and
//! the compiler cannot tell them apart while they share a type.
//!
//!     type BlockNo = u32;      an ALIAS. Changes nothing rustc checks.
//!     struct BlockNo(u32);     a NEWTYPE. A different type, the same bits.
//!
//! A struct with unnamed fields is a TUPLE STRUCT; one with a single field is a
//! newtype. You reach the value by position: `self.0`.
//!
//!     #[repr(transparent)]  promises the wrapper has the EXACT layout and ABI
//!     of the field inside it. Everything you buy is at compile time.
//!
//! Run:  cargo run --bin 04_newtype

use std::mem::size_of;

fn main() {
    println!("== an alias buys nothing ==");
    let n: RawBlock = 7;            // RawBlock IS u32, interchangeably
    let bytes: RawCount = n;        // so this is allowed, and it is a bug
    println!("type RawBlock = u32; type RawCount = u32;");
    println!("a block number assigned straight into a byte count: {bytes}");
    println!("rustc has no opinion, because they are the same type");

    println!("\n== a newtype does ==");
    let block = BlockNo(7);
    let count = ByteCount(512);
    println!("BlockNo(7)     -> {block:?}");
    println!("ByteCount(512) -> {count:?}");
    println!("read_block(block) is fine; read_block(count) is E0308");
    // read_block(count);           // error[E0308]: mismatched types
    // read_block(7);               // error[E0308]: expected `BlockNo`, found integer
    read_block(block);

    println!("\n== and costs nothing at run time ==");
    println!("size_of::<u32>()       = {}", size_of::<u32>());
    println!("size_of::<BlockNo>()   = {}   <- repr(transparent)", size_of::<BlockNo>());
    println!("size_of::<[BlockNo; 8]>() = {}", size_of::<[BlockNo; 8]>());
    println!("same size, same alignment, same register, same instructions");

    println!("\n== reach the value by position: self.0 ==");
    println!("block.0        = {}", block.0);
    println!("block.next()   = {:?}", block.next());
    println!("count.blocks() = {}   (512-byte blocks)", count.blocks());

    println!("\n== a newtype over packed bits ==");
    // A Unix mode word: three 3-bit permission fields side by side.
    //
    //   bits [8..6] owner   bits [5..3] group   bits [2..0] other
    //   within each field: 4 = read, 2 = write, 1 = execute
    let mode = Mode::new(0b111, 0b101, 0b101);   // rwxr-xr-x
    println!("Mode::new(0o7, 0o5, 0o5) = {:#o}   ({:#011b})", mode.0, mode.0);
    println!("mode.owner() = {:#o}", mode.owner());
    println!("mode.group() = {:#o}", mode.group());
    println!("mode.other() = {:#o}", mode.other());
    println!("mode.owner_can_write() = {}", mode.owner_can_write());

    println!("\n== packing is a shift and an OR; unpacking is a shift and a mask ==");
    println!("build : (owner << 6) | (group << 3) | other");
    println!("read  : (self.0 >> 6) & 0o7");
    let ro = Mode::new(0b100, 0b100, 0b100);     // r--r--r--
    println!("read-only {:#o}: owner_can_write() = {}", ro.0, ro.owner_can_write());

    println!("\n== the deliberate reinterpretation is spelled out ==");
    let raw: u32 = block.0;         // one explicit unwrap, greppable
    let back = BlockNo(raw);        // one explicit wrap, greppable
    println!("BlockNo -> u32 -> BlockNo: {back:?}");
    println!("the conversions still exist; now they are visible");
    println!("\n**RUN** ../examples/show-errors.sh e0308");
}

/// A type ALIAS. `RawBlock` and `u32` are interchangeable everywhere.
pub type RawBlock = u32;
/// Another one, and nothing stops you assigning one to the other.
pub type RawCount = u32;

/// A NEWTYPE: a distinct type, the same 32 bits.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockNo(pub u32);

/// A second newtype over the same representation, and not the same type.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteCount(pub u32);

/// A Unix permission word, three 3-bit fields packed into one integer.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode(pub u16);

/// One 3-bit field: rwx.
pub const PERM_MASK: u16 = 0o7;

impl BlockNo {
    pub const fn next(self) -> BlockNo {
        BlockNo(self.0 + 1)
    }
}

impl ByteCount {
    /// How many 512-byte blocks these bytes occupy, rounded up.
    pub const fn blocks(self) -> u32 {
        (self.0 + 511) / 512
    }
}

impl Mode {
    /// Pack three permission fields into one word.
    pub const fn new(owner: u16, group: u16, other: u16) -> Mode {
        Mode((owner << 6) | (group << 3) | other)
    }

    pub const fn owner(self) -> u16 {
        (self.0 >> 6) & PERM_MASK
    }

    pub const fn group(self) -> u16 {
        (self.0 >> 3) & PERM_MASK
    }

    pub const fn other(self) -> u16 {
        self.0 & PERM_MASK
    }

    /// Testing one bit is always this shape: AND with it, compare to zero.
    pub const fn owner_can_write(self) -> bool {
        self.owner() & 0b010 != 0
    }
}

/// Takes a block number and nothing else. A byte count will not compile.
fn read_block(b: BlockNo) {
    println!("read_block({b:?}) -> reading disk block {}", b.0);
}
