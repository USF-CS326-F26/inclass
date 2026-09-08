// error[E0308]: mismatched types
//
// The payoff of the newtype pattern, and the whole reason to pay for it. A
// block number and a byte count are both "just a u32", and swapping them is a
// bug that compiles, links, and ships. Give the block number its own type and
// the compiler catches every one.
//
// With `type BlockNo = u32;` -- an ALIAS -- both lines below compile happily.
//
// FIX 1: pass a BlockNo: `read_block(BlockNo(7))`.
// FIX 2: if the conversion is genuinely meant, spell it out -- `BlockNo(n.0)`
//        -- so it is one greppable line rather than an invisible accident.
#[repr(transparent)]
#[derive(Clone, Copy)]
struct BlockNo(u32);

#[repr(transparent)]
#[derive(Clone, Copy)]
struct ByteCount(u32);

fn read_block(b: BlockNo) -> u32 {
    b.0
}

fn main() {
    let n = ByteCount(512);
    read_block(n);
    read_block(7);
}
