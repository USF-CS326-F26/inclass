//! 03 — derive: four traits the compiler will write for you.
//!
//! `#[derive(...)]` asks rustc for the mechanical implementations:
//!
//!     Debug        {:?} printing -- and what assert_eq! shows on failure
//!     Clone        an explicit .clone()
//!     Copy         assignment DUPLICATES instead of moving
//!     PartialEq    == and != , field by field
//!
//! `Copy` is the load-bearing one, and it is not about speed: copying eight
//! bytes was always cheap. It makes assignment stop MOVING, which is what lets
//! a by-value `self` method be called twice on the same value.
//!
//!     Copy XOR Drop.  A destructor means one release point; a copy means many
//!     values. Allow both and the release runs once per copy -- a double free.
//!
//! Run:  cargo run --bin 03_derive_and_copy

fn main() {
    println!("== Debug: what {{:?}} and assert_eq! print ==");
    let t = Ticks { count: 42, hz: 10_000_000 };
    println!("{{:?}}  -> {t:?}");
    println!("{{:#?}} ->");
    println!("{t:#?}");

    println!("\n== PartialEq: == field by field ==");
    let a = Ticks { count: 42, hz: 10_000_000 };
    let b = Ticks { count: 43, hz: 10_000_000 };
    println!("t == a  -> {}", t == a);
    println!("t == b  -> {}", t == b);
    println!("nothing was written by hand; the derive compares every field");

    println!("\n== Copy: assignment duplicates ==");
    let mut first = Ticks { count: 1, hz: 100 };
    let second = first;             // a copy, not a move
    first.count = 999;              // touching one does not touch the other
    println!("first  = {first:?}  at {:p}", &first);
    println!("second = {second:?}    at {:p}   <- an independent value", &second);

    println!("\n== which is what makes a by-value method callable twice ==");
    let ms = Ticks { count: 30_000_000, hz: 10_000_000 };
    println!("ms.seconds()     = {}", ms.seconds());
    println!("ms.half()        = {:?}", ms.half());
    println!("ms.seconds()     = {}   <- `seconds(self)` copied, it did not move",
             ms.seconds());
    println!("drop the Copy derive and the second call is error[E0382]");

    println!("\n== a type that owns something moves instead ==");
    let n1 = Named { label: String::from("timer") };
    let n2 = n1.clone();            // an explicit, visible allocation
    println!("n1.label at {:p}", n1.label.as_ptr());
    println!("n2.label at {:p}   <- clone() allocated a second buffer",
             n2.label.as_ptr());
    let moved = n1;                 // a move: Named is Clone but not Copy
    println!("moved = {moved:?}");
    // println!("{n1:?}");           // error[E0382]: borrow of moved value

    println!("\n== Copy XOR Drop ==");
    {
        let _guard = Held { id: 7 };
        println!("Held {{ id: 7 }} created");
    } // <- its Drop runs here
    println!("Held has a destructor, so it can never derive Copy:");
    println!("  error[E0184]: the trait `Copy` cannot be implemented for this");
    println!("                type; the type has a destructor");
    println!("try it -- add #[derive(Clone, Copy)] to `Held` and rebuild");
    println!("\n**RUN** ../examples/show-errors.sh e0184");
}

/// Plain data: two integers and no resource, so `Copy` is the right choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticks {
    pub count: u64,
    pub hz: u64,
}

impl Ticks {
    /// Takes `self` by value. Legal to call twice only because Ticks is Copy.
    pub fn seconds(self) -> u64 {
        self.count / self.hz
    }

    pub fn half(self) -> Ticks {
        Ticks { count: self.count / 2, hz: self.hz }
    }
}

/// Owns a heap buffer, so `Clone` yes, `Copy` never.
#[derive(Debug, Clone)]
pub struct Named {
    pub label: String,
}

/// Has a destructor, so `Copy` is rejected outright.
pub struct Held {
    pub id: usize,
}

impl Drop for Held {
    fn drop(&mut self) {
        println!("  drop(Held {{ id: {} }})   <- released at the closing brace", self.id);
    }
}
