// error[E0506]: cannot assign to `buf[_]` because it is borrowed
//
// The same loan as e0502, written to instead of read. `lines` holds `buf`
// until its last use, and `next_line` trusts that the bytes it left in the
// buffer are still there. A write in between would change a line that
// `lines` has not handed out yet.
//
// FIX 1: change the line you were given, in an array of your own: copy the
//        bytes out, then edit the copy.
// FIX 2: finish with `lines` first. After its last use, `buf` is yours to
//        write again.
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
    buf[5] = b'S';
    let second = lines.next_line().map(|l| l.len());
    println!("{first:?} then {second:?}, and buf[5] is {}", buf[5]);
}
