//! 02 — impl: giving a type behavior, and choosing the receiver.
//!
//! A function whose first parameter is some form of `self` is a METHOD, called
//! with a dot. One with no `self` is an ASSOCIATED FUNCTION, called through the
//! type with `::`. Rust has no `constructor` keyword; `new` is a convention.
//!
//!     s.record_read()          method             ->  fn record_read(&mut self)
//!     Stats::new()             associated fn      ->  fn new() -> Self
//!
//! Choosing between &self, &mut self and self is the ownership decision from
//! last week, applied to methods:
//!
//!     &self       shared borrow    caller keeps it   reading
//!     &mut self   unique borrow    caller keeps it   mutating
//!     self        by value         caller loses it   consuming (or Copy)
//!
//! Run:  cargo run --bin 02_impl_and_self

fn main() {
    println!("== associated function: `::`, because there is no value yet ==");
    let mut s = Stats::new();
    println!("Stats::new()      -> {s:?}");
    println!("Stats::started(9) -> {:?}", Stats::started(9));

    println!("\n== &self reads; the caller keeps the value ==");
    println!("s.total()   = {}", s.total());
    println!("s.total()   = {}   <- called twice, still ours", s.total());
    println!("s.is_idle() = {}", s.is_idle());

    println!("\n== &mut self mutates, under an exclusive borrow ==");
    s.record_read();
    s.record_read();
    s.record_write();
    println!("after 2 reads and 1 write: {s:?}, total {}", s.total());
    println!("while `record_read` runs, no other reference to `s` exists");

    println!("\n== self by value consumes -- unless the type is Copy ==");
    // Stats derives Copy, so this call copies 16 bytes and `s` survives.
    let doubled = s.scaled(2);
    println!("s.scaled(2) = {doubled:?}");
    println!("s is still usable: {s:?}   <- only because Stats is Copy");

    println!("\n== ... and consuming is sometimes exactly what you want ==");
    let report = s.into_report();
    println!("s.into_report() -> {report:?}");
    println!("`into_report` takes self to say: this Stats is finished with");

    println!("\n== a type that is NOT Copy: self really does take it away ==");
    let dev = Device::new("uart0");
    println!("dev.name_len() = {}   (&self: borrows)", dev.name_len());
    println!("dev.name_len() = {}   (still ours)", dev.name_len());
    let name = dev.into_name();
    println!("dev.into_name() -> {name:?}");
    // println!("{}", dev.name_len());   // error[E0382]: borrow of moved value

    println!("\n== Self is the type's own name, inside its impl block ==");
    println!("`-> Self` and `-> Stats` mean the same thing in there");
    println!("chained: {:?}", Stats::new().plus_read().plus_read().plus_write());
}

/// Two counters, and every shape of receiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub reads: usize,
    pub writes: usize,
}

impl Stats {
    /// Associated function: no `self`, called as `Stats::new()`.
    ///
    /// There is no value to call a method on yet, because making one is the
    /// job. Every constructor in a kernel has this shape.
    pub fn new() -> Self {
        Stats { reads: 0, writes: 0 }
    }

    /// Another one. `new` is a convention, not a keyword -- name it for what
    /// it makes.
    pub fn started(reads: usize) -> Self {
        Stats { reads, writes: 0 }
    }

    /// `&self`: reads two fields, changes nothing, caller keeps the value.
    pub fn total(&self) -> usize {
        self.reads + self.writes
    }

    /// `&self` again, and a method may call another method on the same value.
    pub fn is_idle(&self) -> bool {
        self.total() == 0
    }

    /// `&mut self`: while this runs, no other reference to the value exists.
    pub fn record_read(&mut self) {
        self.reads += 1;
    }

    pub fn record_write(&mut self) {
        self.writes += 1;
    }

    /// `self` by value on a small `Copy` type: the call copies 16 bytes and
    /// the caller's value is untouched.
    pub fn scaled(self, factor: usize) -> Self {
        Stats { reads: self.reads * factor, writes: self.writes * factor }
    }

    /// `self` by value to CONSUME: after this the Stats is finished with, and
    /// the name says so. (`into_` is the convention for that.)
    pub fn into_report(self) -> (usize, usize, usize) {
        (self.reads, self.writes, self.total())
    }

    /// Taking and returning `Self` is what makes calls chain.
    pub fn plus_read(mut self) -> Self {
        self.reads += 1;
        self
    }

    pub fn plus_write(mut self) -> Self {
        self.writes += 1;
        self
    }
}

/// A type that owns a heap buffer, so it can never be `Copy`.
pub struct Device {
    name: String,
}

impl Device {
    pub fn new(name: &str) -> Self {
        Device { name: name.to_string() }
    }

    /// Borrows: call it as often as you like.
    pub fn name_len(&self) -> usize {
        self.name.len()
    }

    /// Consumes: the Device is gone, and the String comes out of it.
    pub fn into_name(self) -> String {
        self.name
    }
}
