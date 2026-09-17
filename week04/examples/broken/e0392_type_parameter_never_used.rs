// error[E0392]: type parameter `T` is never used
//
// A generic FUNCTION names its parameter in the signature; a generic STRUCT
// has to name it in a FIELD. This lock declares a T and then guards only a
// bool, so `Lock<VecSink>` and `Lock<u64>` would be the same bytes.
//
// FIX 1: hold one -- `struct Lock<T> { held: bool, data: T }`, which is
//        program 14, and rv6's `SpinLock<T>` with `UnsafeCell<T>`.
// FIX 2: `PhantomData<T>` for a T you want in the type and not in the
//        bytes: units, a state machine, a device tag.
// FIX 3: drop it -- `struct Lock { held: bool }` is week03's program 07.
trait Sink {
    fn put(&mut self, bytes: &[u8]);
}

struct VecSink(Vec<u8>);

impl Sink for VecSink {
    fn put(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

struct Lock<T> {
    held: bool,
}

fn main() {
    let mut v = VecSink(Vec::new());
    v.put(b"rv6 ready\n");
    println!("{} bytes written, and nothing locked", v.0.len());
}
