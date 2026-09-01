//! 10 — Choosing a signature: the whole decision, in one file.
//!
//!     Does the callee need to KEEP it?          -> take  T        (move)
//!     Does it only need to READ it?             -> take &T        (shared)
//!     Does it need to CHANGE the caller's copy? -> take &mut T    (exclusive)
//!     Does the caller need it back afterwards?  -> take T, return T
//!
//! Default to `&T`. Reach for `T` when you mean "this is mine now" — which,
//! in a kernel, is exactly what a page allocator or a queue insert means.
//!
//! Run:  cargo run --bin 10_how_to_pass

#[derive(Debug)]
struct Buffer {
    bytes: Vec<u8>,
}

fn main() {
    let mut b = Buffer { bytes: vec![1, 2, 3] };

    // READ: borrow.
    println!("len via &Buffer      = {}", len(&b));

    // CHANGE IN PLACE: exclusive borrow.
    append(&mut b, 4);
    println!("after append(&mut)   = {:?}", b.bytes);

    // KEEP IT: move. The caller is done with the value.
    let handle = enqueue(b);
    println!("enqueued, queue owns it now: {handle}");

    // GIVE IT BACK: move in, move out.
    let mut b2 = Buffer { bytes: vec![7, 7] };
    b2 = zero_and_return(b2);
    println!("after move-in/move-out = {:?}", b2.bytes);

    println!("\n== the same four, with the errors they prevent ==");
    let owned = String::from("kernel");

    // &T: the callee cannot change it and cannot free it.
    let n = count_bytes(&owned);
    println!("count_bytes(&owned) = {n}, owned still here: {owned:?}");

    // T: after this line `owned` is not a name you can use.
    let shouty = consume_and_upper(owned);
    println!("consume_and_upper -> {shouty:?}");
    // println!("{owned}");  // error[E0382]: borrow of moved value: `owned`

    println!("\n== a full mini-allocator, borrow-checked ==");
    let mut alloc = PageAlloc { free: vec![0x8000_3000, 0x8000_2000, 0x8000_1000] };
    let a = alloc.take();                 // &mut self
    let c = alloc.take();
    println!("took {:#x} and {:#x}, {} left", a.unwrap(), c.unwrap(), alloc.len());
    if let Some(p) = a {
        alloc.give(p);                    // &mut self again
    }
    println!("returned one, {} left, top of list = {:#x}", alloc.len(), alloc.peek().unwrap());
}

struct PageAlloc {
    free: Vec<usize>,
}

impl PageAlloc {
    /// `&self` — reads only. Many of these may coexist.
    fn len(&self) -> usize {
        self.free.len()
    }

    /// `&self` returning a borrow tied to self: the result cannot outlive the
    /// allocator, and the allocator cannot be mutated while it is alive.
    fn peek(&self) -> Option<&usize> {
        self.free.last()
    }

    /// `&mut self` — exclusive. Nothing else may touch the free list here,
    /// which is precisely the invariant a real kalloc() needs.
    fn take(&mut self) -> Option<usize> {
        self.free.pop()
    }

    fn give(&mut self, page: usize) {
        self.free.push(page);
    }
}

fn len(b: &Buffer) -> usize {
    b.bytes.len()
}

fn append(b: &mut Buffer, byte: u8) {
    b.bytes.push(byte);
}

fn enqueue(b: Buffer) -> String {
    format!("Buffer({} bytes) parked in the queue", b.bytes.len())
} // dropped here — the caller could not have used it anyway

fn zero_and_return(mut b: Buffer) -> Buffer {
    for x in b.bytes.iter_mut() {
        *x = 0;
    }
    b
}

fn count_bytes(s: &str) -> usize {
    s.len()
}

fn consume_and_upper(s: String) -> String {
    s.to_uppercase()
}
