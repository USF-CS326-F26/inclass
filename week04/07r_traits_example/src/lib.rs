// ╔══════════════════════════════════════════════════════════════════════╗
// ║  07r — Traits and Generics — IN-CLASS EXAMPLE                        ║
// ║  Same shape as the exercise, different nouns: a byte sink instead of ║
// ║  a text sink, a page-frame allocator instead of a scheduler, and one ║
// ║  generic `trace` that drives every combination of the two.           ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/07r_traits/skeleton/lib.rs`. Every item here
// has a twin there. If you can explain why the twins are the same shape, you
// have understood traits; the domain is decoration.
//
// The sink here speaks BYTES, so numbers have to become digits by hand. The
// default method here MUTATES the table it is handed. Neither is true of the
// exercise, and both are worth asking about.

// ---------------------------------------------------------------------------
// Section 1 — a trait, and two different types that provide it.
// ---------------------------------------------------------------------------

/// Somewhere bytes can be written.
///
/// UNDERSTAND: a trait is a list of methods a type promises to provide. It
///   holds no data and you never build one; types *implement* it. Twin of
///   `Out`, with one change: `put` takes `&[u8]`, not `&str`. A UART, a disk
///   block, and the argv `exec` builds are all bytes, so the kernel's real
///   sink is this one. Friday's `10c` is written against exactly this shape.
pub trait Sink {
    /// Write `bytes`. Every implementer must supply this one.
    ///
    /// UNDERSTAND: no body, just a signature and a semicolon -- a **required
    ///   method**. A type does not implement `Sink` until it has one, and
    ///   leaving it out is E0046.
    fn put(&mut self, bytes: &[u8]);

    /// Write `bytes`, then a newline.
    ///
    /// UNDERSTAND: a body, so a **default method** -- written once, here, in
    ///   terms of `put`, and inherited by every implementer for free. When
    ///   this line was compiled, no type implementing `Sink` existed yet. It
    ///   assumed only what the trait promises.
    fn put_line(&mut self, bytes: &[u8]) {
        self.put(bytes);
        self.put(b"\n");
    }
}

/// A `Sink` that keeps every byte written to it. The test's sink.
///
/// UNDERSTAND: twin of `StringOut`. A `Vec<u8>` instead of a `String`,
///   because nothing here promises the bytes are text.
pub struct VecSink {
    pub bytes: Vec<u8>,
}

impl VecSink {
    pub fn new() -> VecSink {
        VecSink { bytes: Vec::new() }
    }
}

impl Default for VecSink {
    fn default() -> Self {
        Self::new()
    }
}

impl Sink for VecSink {
    fn put(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }
}

/// A `Sink` that measures what goes past and keeps none of it.
///
/// UNDERSTAND: twin of `CountingOut`. `bytes` is how much went by; `puts` is
///   how many times `put` was called -- which is what makes the default method
///   visible from outside: one `put_line` is always two `put`s.
pub struct TallySink {
    pub bytes: usize,
    pub puts: usize,
}

impl TallySink {
    pub fn new() -> TallySink {
        TallySink { bytes: 0, puts: 0 }
    }
}

impl Default for TallySink {
    fn default() -> Self {
        Self::new()
    }
}

impl Sink for TallySink {
    fn put(&mut self, bytes: &[u8]) {
        self.bytes += bytes.len();
        self.puts += 1;
    }
}

/// Write the boot banner.
///
/// UNDERSTAND: twin of `write_banner`. `&mut dyn Sink` is **dynamic
///   dispatch**: one copy of this function exists, and the reference is a fat
///   pointer -- data pointer plus a pointer to a table of the concrete type's
///   methods -- so `put` is looked up at run time. One indirect call per
///   `put`; in exchange a `&mut dyn Sink` fits in a struct field or an array
///   where a generic parameter could not.
pub fn put_banner(sink: &mut dyn Sink) {
    sink.put_line(b"rv6 kernel ready");
}

/// Write a picture of the frame table: `.` free, `#` used.
///
/// UNDERSTAND: twin of `write_listing`. `&mut impl Sink` is **static
///   dispatch**, shorthand for the `<S: Sink>` form used in `trace` below:
///   one copy per concrete type, `put` resolved at compile time, no lookup.
pub fn put_map(sink: &mut impl Sink, free: &[bool]) {
    sink.put(b"frames: ");
    for &f in free {
        sink.put(if f { b"." } else { b"#" });
    }
    sink.put(b"\n");
}

/// Write `n` in decimal, one digit at a time.
///
/// UNDERSTAND: no twin -- the exercise's sink takes `&str` and the tests hand
///   it names, so no number ever needs printing. A byte sink has no `{}`:
///   this is what `ulib::write_usize` does, and why it exists (`core::fmt`
///   costs 12-18 KiB of image a user program on rv6 cannot afford).
pub fn put_usize(sink: &mut impl Sink, n: usize) {
    let mut digits = [0u8; 20];
    let mut i = digits.len();
    let mut v = n;
    loop {
        i -= 1;
        digits[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    sink.put(&digits[i..]);
}

// ---------------------------------------------------------------------------
// Section 2 — a second trait, and two policies that disagree.
// ---------------------------------------------------------------------------

/// A frame-allocation **policy**: given which frames are free, which one next?
///
/// UNDERSTAND: twin of `Scheduler`. Same separation: the policy decides WHICH
///   frame, the mechanism (marking it used, mapping it) is written once in
///   the default methods below and never touched when the policy changes.
pub trait Allocator {
    /// The index of the frame to hand out next, or `None` if none is free.
    ///
    /// UNDERSTAND: twin of `pick_next`, and the same three decisions in the
    ///   signature. `&mut self`: a policy may remember something between
    ///   calls. `&[bool]`: it sees the least it needs -- read-only, so no
    ///   policy can corrupt the table. `Option<usize>`: "nothing free" is an
    ///   ordinary answer, not an error.
    fn pick(&mut self, free: &[bool]) -> Option<usize>;

    /// Pick a frame and mark it used.
    ///
    /// UNDERSTAND: twin of `run_for`, in spirit: a default written once
    ///   against `pick` alone. Unlike `run_for` it WRITES to the table, so it
    ///   takes `&mut [bool]` where `pick` took `&[bool]`. The `?` is Friday's
    ///   operator a day early: on `None`, return `None` right here.
    fn take(&mut self, free: &mut [bool]) -> Option<usize> {
        let i = self.pick(free)?;
        free[i] = false;
        Some(i)
    }

    /// Take up to `n` frames and report which. Stops early if memory runs out.
    ///
    /// UNDERSTAND: twin of `run_for`, line for line: a loop, a `match` on the
    ///   `Option`, a `break` on `None`. Every policy that exists now or later
    ///   gets it without another line of code.
    fn take_many(&mut self, free: &mut [bool], n: usize) -> Vec<usize> {
        let mut taken = Vec::new();
        for _ in 0..n {
            match self.take(free) {
                Some(i) => taken.push(i),
                None => break,
            }
        }
        taken
    }
}

/// Always the lowest free frame.
///
/// UNDERSTAND: twin of `Priority`, in that it remembers nothing -- and it says
///   so in its type: a UNIT struct, zero bytes, exists only so there is a type
///   to hang the `impl` on. `size_of::<FirstFit>() == 0`.
pub struct FirstFit;

impl Allocator for FirstFit {
    fn pick(&mut self, free: &[bool]) -> Option<usize> {
        free.iter().position(|&f| f)
    }
}

/// The next free frame after wherever the last search stopped.
///
/// UNDERSTAND: twin of `RoundRobin`, cursor and all. Scan from `next`, wrap
///   with `%`, resume just past the frame handed out. The difference is WHY:
///   round-robin's cursor is what makes it fair -- remove it and one slot runs
///   forever. Next-fit's cursor is a speed choice -- a frame just handed out
///   is now `false`, so rescanning from 0 would also be correct, only slower.
pub struct NextFit {
    pub next: usize,
}

impl NextFit {
    pub fn new() -> NextFit {
        NextFit { next: 0 }
    }
}

impl Default for NextFit {
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator for NextFit {
    fn pick(&mut self, free: &[bool]) -> Option<usize> {
        let n = free.len();
        for off in 0..n {
            let i = (self.next + off) % n;
            if free[i] {
                self.next = (i + 1) % n;
                return Some(i);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Section 3 — one function, every combination.
// ---------------------------------------------------------------------------

/// Serve `requests` allocations from `alloc` and write each frame number
/// through `sink`, one per line; `-` on a line of its own when memory runs out.
///
/// UNDERSTAND: twin of `trace`. Two type parameters, each with a bound, and
///   the bounds are the whole of what the body may assume: `A: Allocator`
///   lets it call `take`, `S: Sink` lets it call `put`. Delete either bound
///   and the matching call is E0599.
///
///   Monomorphization: the compiler emits one copy per combination actually
///   called -- the tests below produce `trace::<FirstFit, VecSink>` and
///   `trace::<NextFit, TallySink>` among others -- with the calls resolved and
///   inlined. Nothing is looked up at run time. The price is code size.
pub fn trace<A: Allocator, S: Sink>(alloc: &mut A, free: &mut [bool], requests: usize, sink: &mut S) {
    for _ in 0..requests {
        match alloc.take(free) {
            Some(i) => {
                put_usize(sink, i);
                sink.put(b"\n");
            }
            None => sink.put_line(b"-"),
        }
    }
}

// ---------------------------------------------------------------------------
// The tests. Read them first: they are the contract, and they are the same
// kind of contract the exercise's tests are.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> [bool; 8] {
        // Frames 1 and 4 already in use.
        [true, false, true, true, false, true, true, true]
    }

    #[test]
    fn a_vec_sink_keeps_every_byte_put_into_it() {
        let mut sink = VecSink::new();
        sink.put(b"boot");
        sink.put(b"ing");
        assert_eq!(sink.bytes, b"booting");
        // The default method -- nobody wrote it for VecSink, and it works.
        sink.put_line(b" ok");
        assert_eq!(sink.bytes, b"booting ok\n");
    }

    #[test]
    fn a_tally_sink_counts_without_storing() {
        let mut tally = TallySink::new();
        tally.put(b"boot");
        assert_eq!((tally.bytes, tally.puts), (4, 1));
        // One put_line is two puts: the bytes, then "\n".
        tally.put_line(b"ing");
        assert_eq!((tally.bytes, tally.puts), (8, 3));
    }

    #[test]
    fn put_line_is_two_puts_and_nobody_wrote_it_twice() {
        // The same default serves both types, and the counting sink proves
        // it is really two calls underneath.
        let mut v = VecSink::new();
        let mut t = TallySink::new();
        v.put_line(b"rv6");
        t.put_line(b"rv6");
        assert_eq!(v.bytes.len(), t.bytes);
        assert_eq!(t.puts, 2);
    }

    #[test]
    fn one_map_routine_serves_two_sinks() {
        let free = table();
        let mut v = VecSink::new();
        put_banner(&mut v); // through &mut dyn Sink
        put_map(&mut v, &free); // through &mut impl Sink
        assert_eq!(v.bytes, b"rv6 kernel ready\nframes: .#..#...\n");

        let mut t = TallySink::new();
        put_banner(&mut t);
        put_map(&mut t, &free);
        // Not one line of put_banner or put_map changed; the bytes went
        // somewhere else entirely.
        assert_eq!(t.bytes, v.bytes.len());
        assert_eq!(t.puts, 2 + 1 + 8 + 1);
    }

    #[test]
    fn put_usize_writes_decimal_digits_without_core_fmt() {
        let mut v = VecSink::new();
        for n in [0, 7, 10, 4096, usize::MAX] {
            put_usize(&mut v, n);
            v.put(b" ");
        }
        assert_eq!(v.bytes, b"0 7 10 4096 18446744073709551615 ");
    }

    #[test]
    fn first_fit_always_takes_the_lowest_free_frame() {
        let free = table();
        let mut ff = FirstFit;
        assert_eq!(ff.pick(&free), Some(0));
        assert_eq!(ff.pick(&free), Some(0)); // pick does not mark; still 0
        let mut used = free;
        used[0] = false;
        assert_eq!(ff.pick(&used), Some(2)); // 1 was already taken
    }

    #[test]
    fn next_fit_resumes_where_it_left_off_and_wraps() {
        let free = table();
        let mut nf = NextFit::new();
        assert_eq!(nf.pick(&free), Some(0));
        assert_eq!(nf.pick(&free), Some(2)); // skipped 1: it was in use
        assert_eq!(nf.pick(&free), Some(3));
        assert_eq!(nf.pick(&free), Some(5)); // skipped 4
        assert_eq!(nf.pick(&free), Some(6));
        assert_eq!(nf.pick(&free), Some(7));
        assert_eq!(nf.pick(&free), Some(0)); // wrapped
        assert_eq!(nf.next, 1);
    }

    #[test]
    fn a_policy_with_nothing_free_picks_nothing() {
        let none = [false; 4];
        let empty: [bool; 0] = [];
        assert_eq!(FirstFit.pick(&none), None);
        assert_eq!(FirstFit.pick(&empty), None);
        let mut nf = NextFit::new();
        assert_eq!(nf.pick(&none), None);
        // An empty table must answer None too, not divide by zero on the way.
        assert_eq!(nf.pick(&empty), None);
    }

    #[test]
    fn take_marks_the_frame_used_so_the_next_take_moves_on() {
        let mut free = table();
        let mut ff = FirstFit;
        assert_eq!(ff.take(&mut free), Some(0));
        assert!(!free[0]);
        assert_eq!(ff.take(&mut free), Some(2));
        assert_eq!(ff.take(&mut free), Some(3));
        assert_eq!(free, [false, false, false, false, false, true, true, true]);
    }

    #[test]
    fn take_many_stops_when_memory_runs_out() {
        let mut free = table();
        let mut nf = NextFit::new();
        // Six free frames; ask for nine and get six.
        assert_eq!(nf.take_many(&mut free, 9), vec![0, 2, 3, 5, 6, 7]);
        assert!(free.iter().all(|&f| !f));
        assert!(nf.take_many(&mut free, 3).is_empty());
    }

    #[test]
    fn a_stateless_policy_is_zero_bytes() {
        assert_eq!(std::mem::size_of::<FirstFit>(), 0);
        assert_eq!(std::mem::size_of::<NextFit>(), std::mem::size_of::<usize>());
    }

    #[test]
    fn one_generic_function_drives_both_policies_and_both_sinks() {
        let mut free = table();
        let mut log = VecSink::new();
        trace(&mut FirstFit, &mut free, 3, &mut log);
        assert_eq!(log.bytes, b"0\n2\n3\n");

        // A different A and a different S, and trace did not change at all.
        // Three frames are left; the fifth request finds nothing.
        let mut tally = TallySink::new();
        let mut nf = NextFit { next: 6 };
        trace(&mut nf, &mut free, 5, &mut tally);
        // "6\n" "7\n" "5\n" "-\n" "-\n": 10 bytes, two puts per line.
        assert_eq!(tally.bytes, 10);
        assert_eq!(tally.puts, 10);
    }
}
