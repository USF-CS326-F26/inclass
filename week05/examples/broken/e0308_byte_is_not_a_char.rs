// error[E0308]: mismatched types
//               expected `u8`, found `char`
//
// Walking a `&[u8]` gives bytes. `'\t'` is a char: four bytes, one Unicode
// scalar value. A u8 and a char never compare, even when the byte is ASCII,
// because Rust will not guess an encoding for you.
//
// FIX 1: `b == b'\t'` -- a byte literal is a u8.
// FIX 2: `b as char == '\t'` also compiles, and reads each byte as a whole
//        character: fine for ASCII, wrong for both bytes of ñ.
fn count_tabs(line: &[u8]) -> usize {
    let mut tabs = 0;
    for &b in line {
        if b == '\t' {
            tabs += 1;
        }
    }
    tabs
}

fn main() {
    println!("{}", count_tabs(b"pid\tstate\tname"));
}
