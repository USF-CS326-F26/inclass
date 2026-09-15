// error[E0599]: no method named `put` found for mutable reference `&mut S`
//               in the current scope
//
// `S` is SOME type -- the body may assume only what the bound says, and there
// is no bound. This is not a missing `use`; rustc's help text says the fix:
//   "perhaps you need to restrict type parameter `S` with it: `S: Sink`"
//
// FIX 1: `fn log_all<S: Sink>(sink: &mut S, ..)` -- the bound is the contract
//        in both directions: what the body may call, what the caller must prove.
// FIX 2: `sink: &mut impl Sink` (same thing), or `&mut dyn Sink` (one copy).
trait Sink {
    fn put(&mut self, bytes: &[u8]);
}

struct VecSink(Vec<u8>);

impl Sink for VecSink {
    fn put(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

fn log_all<S>(sink: &mut S, lines: &[&[u8]]) {
    for line in lines {
        sink.put(line);
    }
}

fn main() {
    let mut v = VecSink(Vec::new());
    log_all(&mut v, &[b"boot", b"ok"]);
    println!("{}", v.0.len());
}
