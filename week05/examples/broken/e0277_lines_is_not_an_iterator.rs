// error[E0277]: `Lines<'_>` is not an iterator
//
// `for` calls `Iterator::next`, and `Lines` has no such method, only
// `next_line`. It cannot have one: `next(&mut self) -> Option<Self::Item>`
// gives `Item` no way to borrow from this one call, and every line is a
// slice of the buffer that the NEXT call may overwrite.
//
// FIX 1: `while let Some(line) = lines.next_line() { .. }` -- call, match,
//        and stop at the first None.
// FIX 2: the same loop longhand: `loop { match lines.next_line() {
//        Some(line) => .., None => break } }`.
struct Lines<'b> {
    buf: &'b mut [u8],
    start: usize,
}

impl<'b> Lines<'b> {
    fn next_line(&mut self) -> Option<&[u8]> {
        let nl = self.buf[self.start..].iter().position(|&b| b == b'\n')?;
        let from = self.start;
        self.start += nl + 1;
        Some(&self.buf[from..from + nl])
    }
}

fn main() {
    let mut buf = *b"boot\nsh\n";
    let lines = Lines { buf: &mut buf, start: 0 };
    for line in lines {
        println!("{} bytes", line.len());
    }
}
