// error[E0038]: the trait `Sink` is not dyn compatible
//
// A vtable has one slot per method. `put_each` is GENERIC over I, so it would
// need one slot per type it is ever called with -- not a number the compiler
// knows. `fn new() -> Self` and methods without `self` fail for the same
// reason. `&mut dyn Sink` therefore cannot exist for this trait.
//
// FIX 1: `where Self: Sized` on put_each -- it exists for concrete types and
//        vanishes from the vtable.
// FIX 2: make it non-generic: `put_each(&mut self, items: &[&[u8]])`.
// FIX 3: `fn banner<S: Sink>(s: &mut S)` -- go generic and never need dyn.
trait Sink {
    fn put(&mut self, bytes: &[u8]);
    fn put_each<I: Iterator<Item = &'static [u8]>>(&mut self, items: I) {
        for item in items {
            self.put(item);
        }
    }
}

struct Console;

impl Sink for Console {
    fn put(&mut self, bytes: &[u8]) {
        print!("{}", String::from_utf8_lossy(bytes));
    }
}

fn banner(out: &mut dyn Sink) {
    out.put(b"rv6\n");
}

fn main() {
    banner(&mut Console);
}
