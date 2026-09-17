// error[E0107]: missing generics for struct `FdTable`
//
// 07 asserts that `[&mut U; 3]` cannot hold three sinks, because U is ONE
// type. Here is the proof, twice: `FdTable<U>` is not a type until you name
// U, so every layer that touches one must name it too -- and once you have
// written `FdTable<Console>`, the assignment in main is error[E0308].
//
// FIX 1: `fn banner<U: Uart>(t: &mut FdTable<U>)` -- go generic too, and hand
//        the choice one layer up. Every layer up pays the same tax.
// FIX 2: `struct FdTable<'a> { out: &'a mut dyn Uart }` -- one field type,
//        any sink, re-pointed at run time. Program 13 does this.
// FIX 3: `enum Kind { Tty, File, Null }` and a match -- rv6's `FileKind`.
trait Uart {
    fn putc(&mut self, b: u8);
    fn puts(&mut self, s: &[u8]) {
        for &b in s {
            self.putc(b);
        }
    }
}

struct Console;

impl Uart for Console {
    fn putc(&mut self, b: u8) {
        print!("{}", b as char);
    }
}

struct Recorder(Vec<u8>);

impl Uart for Recorder {
    fn putc(&mut self, b: u8) {
        self.0.push(b);
    }
}

struct FdTable<U: Uart> {
    out: U,
}

fn banner(t: &mut FdTable) {
    t.out.puts(b"rv6 ready\n");
}

fn main() {
    let mut fds = FdTable { out: Console };
    banner(&mut fds);
    fds.out = Recorder(Vec::new());
}
