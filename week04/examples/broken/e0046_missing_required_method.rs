// error[E0046]: not all trait items implemented, missing: `put`
//
// `put_line` has a body in the trait, so it is a DEFAULT: optional. `put` has
// none, so it is REQUIRED. This impl wrote the optional one and skipped the
// required one -- and the default is BUILT ON `put`, so nothing works without
// it. The compiler names exactly what is missing.
//
// FIX 1: write `fn put(&mut self, b: &[u8])` and delete the override; the
//        default `put_line` then works for free.
// FIX 2: keep the override only if it is genuinely different -- `put` is
//        still required either way.
trait Sink {
    fn put(&mut self, bytes: &[u8]);
    fn put_line(&mut self, bytes: &[u8]) {
        self.put(bytes);
        self.put(b"\n");
    }
}

struct Tally {
    bytes: usize,
}

impl Sink for Tally {
    fn put_line(&mut self, bytes: &[u8]) {
        self.bytes += bytes.len() + 1;
    }
}

fn main() {
    let mut t = Tally { bytes: 0 };
    t.put_line(b"rv6");
    println!("{}", t.bytes);
}
