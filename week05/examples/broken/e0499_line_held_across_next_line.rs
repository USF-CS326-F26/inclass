// error[E0499]: cannot borrow `lines` as mutable more than once at a time
//
// A line is a slice of the buffer `lines` holds on loan, and the next call
// to `next_line` may slide or refill that buffer. So `first` keeps `lines`
// borrowed for as long as `first` is used, and the second call needs it
// again. C compiles this and prints whatever the refill left there.
//
// FIX 1: finish with each line before asking for the next: test it, print
//        it, then call `next_line` again. `while let` does that for you.
// FIX 2: copy out what you need first -- a length, a flag, a few bytes into
//        an array of your own -- and keep the copy, not the slice.
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
    let mut lines = Lines { buf: &mut buf, start: 0 };
    let first = lines.next_line();
    let second = lines.next_line();
    println!("{first:?} then {second:?}");
}
