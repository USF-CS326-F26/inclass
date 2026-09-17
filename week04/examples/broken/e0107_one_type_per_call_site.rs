// error[E0107]: missing generics for struct `FdTable`
//
// 07 asserts that `[&mut U; 3]` cannot hold three sinks, because U is ONE
// type. Here is the proof, twice: `FdTable<U>` is not a type until you name
// U, so every layer that touches one must name it too -- and once you have
// written `FdTable<Console>`, the assignment in main is error[E0308].
//
// FIX 1: `fn banner<S: Sink>(t: &mut FdTable<S>)` -- go generic too, and hand
//        the choice one layer up. Every layer up pays the same tax.
// FIX 2: `struct FdTable<'a> { out: &'a mut dyn Sink }` -- one field type,
//        any sink, re-pointed at run time. Program 13 does this.
// FIX 3: `enum Kind { Tty, File, Null }` and a match -- rv6's `FileKind`.
trait Sink {
    fn put(&mut self, bytes: &[u8]);
}

struct Console;

impl Sink for Console {
    fn put(&mut self, bytes: &[u8]) {
        print!("{}", String::from_utf8_lossy(bytes));
    }
}

struct VecSink(Vec<u8>);

impl Sink for VecSink {
    fn put(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

struct FdTable<S: Sink> {
    out: S,
}

fn banner(t: &mut FdTable) {
    t.out.put(b"rv6 ready\n");
}

fn main() {
    let mut fds = FdTable { out: Console };
    banner(&mut fds);
    fds.out = VecSink(Vec::new());
}
