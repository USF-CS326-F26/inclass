// error[E0599]: no method named `chars` found for reference `&[u8]` in the
//               current scope
//
// `chars` decodes UTF-8, so it lives on `str`, whose bytes are PROMISED to
// be valid UTF-8. A `&[u8]` promises nothing: it might be half of a ñ, or a
// disk block. There is no char to hand back until somebody checks.
//
// FIX 1: stay in bytes: `line.first().is_some_and(|b| b.is_ascii_uppercase())`.
//        Every command this week works this way.
// FIX 2: check first: `std::str::from_utf8(line)` returns a Result, and the
//        `&str` inside an `Ok` has `.chars()`.
fn starts_upper(line: &[u8]) -> bool {
    line.chars().next().is_some_and(|c| c.is_uppercase())
}

fn main() {
    println!("{}", starts_upper(b"Boot ok"));
}
