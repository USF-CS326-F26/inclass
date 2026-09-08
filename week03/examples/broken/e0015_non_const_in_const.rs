// error[E0015]: cannot call non-const function in constants
//
// A const context -- a `const` or `static` item, an array length, an array
// repeat, an enum discriminant -- is a place the value must be known before the
// program runs. Only a `const fn` may be called there.
//
// This matters in a kernel because a static must be valid when the first Rust
// function is entered, and at that point there is no heap, no allocator, and
// nothing that could have run an initialization loop.
//
// FIX 1: mark `blocks_for` as `const fn`. Its body is already legal in one.
// FIX 2: if it truly cannot be const, compute the value once at start-up.
fn blocks_for(bytes: u32) -> u32 {
    (bytes + 511) / 512
}

const ROOT_BLOCKS: u32 = blocks_for(5000);

fn main() {
    println!("{ROOT_BLOCKS}");
}
