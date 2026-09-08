// error[E0080]: evaluation of constant value failed
//
// An index is checked, and in a const context it is checked by the compiler.
// The array has four elements, so index 8 is out of range and the const cannot
// be evaluated -- the program never gets built, let alone run.
//
// At run time the same index is an unsigned compare and a never-taken branch,
// about three instructions, calling `panic_bounds_check` if it fails. In C the
// read simply happens: whatever lives 32 bytes past the array is what you get.
//
// FIX 1: use an index that is in range.
// FIX 2: `TABLE.get(8)` returns `Option`, and `None` is a normal outcome.
static TABLE: [u32; 4] = [10, 20, 30, 40];

const LAST: u32 = TABLE[8];

fn main() {
    println!("{LAST}");
}
