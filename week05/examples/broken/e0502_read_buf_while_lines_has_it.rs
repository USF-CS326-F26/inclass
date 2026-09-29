// error[E0502]: cannot borrow `buf[_]` as immutable because it is also
//               borrowed as mutable
//
// `Lines::new(&mut buf)` lends the WHOLE array to `lines` for as long as
// `lines` is in use, and it is used again on the next line. Reading buf[0]
// in between would read bytes that `next_line` may be moving.
//
// FIX 1: read through `lines`: the line it returned is the part of `buf`
//        you may look at, and it is a slice of the same bytes.
// FIX 2: finish with `lines` first. After its last use, `buf` is yours
//        again, and `buf[0]` compiles.
struct Lines<'b> {
    buf: &'b mut [u8],
    start: usize,
}

impl<'b> Lines<'b> {
    fn new(buf: &'b mut [u8]) -> Lines<'b> {
        Lines { buf, start: 0 }
    }

    fn next_line(&mut self) -> Option<&[u8]> {
        let nl = self.buf[self.start..].iter().position(|&b| b == b'\n')?;
        let from = self.start;
        self.start += nl + 1;
        Some(&self.buf[from..from + nl])
    }
}

fn main() {
    let mut buf = *b"boot\nsh\n";
    let mut lines = Lines::new(&mut buf);
    let first = lines.next_line().map(|l| l.len());
    println!("buf[0] is {}", buf[0]);
    let second = lines.next_line().map(|l| l.len());
    println!("{first:?} then {second:?}");
}
