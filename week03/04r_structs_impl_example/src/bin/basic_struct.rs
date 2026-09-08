// ╔══════════════════════════════════════════════════════════════════════╗
// ║  START HERE. One struct, one impl block, and nothing else.           ║
// ║      cargo run --bin basic_struct                                    ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Before the disk extents and the device numbers, the concept on its own:
// a struct with two fields, and the three shapes a function in an impl block
// can take.

fn main() {
    // ── 1. an associated function makes a value ──────────────────────────
    //
    // No `self` parameter, so there is no existing Counter to talk about.
    // Called through the type, with `::`.
    let mut hits = Counter::new("cache hits");
    println!("1. Counter::new(..)      -> {} = {}", hits.label(), hits.value());

    // ── 2. `&self` reads; the caller keeps the value ─────────────────────
    //
    // Borrowing, so you may call these as often as you like.
    println!("2. hits.value()          -> {}", hits.value());
    println!("   hits.value()          -> {}   still ours", hits.value());
    println!("   hits.is_zero()        -> {}", hits.is_zero());

    // ── 3. `&mut self` writes, under an exclusive borrow ─────────────────
    //
    // While this method runs, no other reference to `hits` exists anywhere.
    // That is what `&mut` promises — not "I may write", but "nobody else is
    // looking while I do".
    hits.bump();
    hits.bump();
    hits.bump();
    println!("3. after three bump()s   -> {}", hits.value());
    println!("   hits.is_zero()        -> {}", hits.is_zero());

    // ── 4. `self` by value takes it away ─────────────────────────────────
    //
    // `into_total` consumes the Counter. After this line the name `hits` is
    // struck off: the value was given away, and the compiler knows it.
    let total = hits.into_total();
    println!("4. hits.into_total()     -> {total}");

    // ── 5. the error worth reading out loud ──────────────────────────────
    println!("5. uncomment the last block and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME. rustc says:
    //
    //   error[E0382]: borrow of moved value: `hits`
    //      |     let total = hits.into_total();
    //      |                      ------------ `hits` moved due to this
    //      |                                   method call
    //      |     println!("{}", hits.value());
    //      |                    ^^^^ value borrowed here after move
    //      |
    //   note: `Counter::into_total` takes ownership of the receiver `self`,
    //         which moves `hits`
    //
    // Choosing the receiver IS the ownership decision, applied to methods.
    // `&self` and `&mut self` hand the value back; `self` does not.
    //
    // println!("{}", hits.value());
    // ---------------------------------------------------------------------
}

/// Two fields that travel as a unit.
///
/// `count` is private: the only way to change it is a method on this type,
/// which is how a module stops the rest of the program from corrupting it.
pub struct Counter {
    label: &'static str,
    count: usize,
}

impl Counter {
    /// Associated function: no `self`. `Self` is this type's own name.
    pub fn new(label: &'static str) -> Self {
        Counter { label, count: 0 }
    }

    /// `&self`: reads a field, changes nothing.
    pub fn value(&self) -> usize {
        self.count
    }

    /// `&self`, and a method may call another method on the same value.
    pub fn is_zero(&self) -> bool {
        self.value() == 0
    }

    /// `&self` returning a borrow of a field, rather than moving it out.
    /// Returning `self.label` by value works here only because `&'static str`
    /// is `Copy`; a `String` field would need `&str` or `.clone()`.
    pub fn label(&self) -> &str {
        self.label
    }

    /// `&mut self`: the unique borrow that lets it write.
    pub fn bump(&mut self) {
        self.count += 1;
    }

    /// `self`: consumes the Counter. The name says so — `into_` is the
    /// convention for a method that takes the value apart.
    pub fn into_total(self) -> usize {
        self.count
    }
}
