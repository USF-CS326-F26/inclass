//! 06 — Generics: one body, compiled once per type.
//!
//! A generic function is written against a type parameter, and the BOUND on
//! that parameter is everything the body may assume:
//!
//!     fn banner<U: Uart>(u: &mut U)       "any U at all, provided U: Uart"
//!     fn banner<U>(u: &mut U) where U: Uart    the same, spelled for long lists
//!     fn banner(u: &mut impl Uart)        the same again, shorthand
//!
//! Without the bound, `U` is SOME type and nothing more; calling `u.putc(..)`
//! is error[E0599] -- and rustc's help text names the fix.
//!
//! What the compiler emits is not one clever function. It is one COPY per
//! type the function is called with, each with the method calls resolved and
//! inlined. That is MONOMORPHIZATION, and the program below proves it by
//! printing the type name from inside the body.
//!
//! Run:  cargo run --bin 06_generics_and_bounds

use std::any::type_name;

pub trait Uart {
    fn putc(&mut self, b: u8);
    fn puts(&mut self, s: &[u8]) {
        for &b in s {
            self.putc(b);
        }
    }
}

pub struct Console;
impl Uart for Console {
    fn putc(&mut self, b: u8) {
        print!("{}", b as char);
    }
}

pub struct Recorder(pub Vec<u8>);
impl Uart for Recorder {
    fn putc(&mut self, b: u8) {
        self.0.push(b);
    }
}

pub struct Tally(pub usize);
impl Uart for Tally {
    fn putc(&mut self, _: u8) {
        self.0 += 1;
    }
}

/// Spelling 1: the bound inline.
fn banner<U: Uart>(u: &mut U) {
    u.puts(b"rv6 ready\n");
    println!("  [banner compiled for {}]", type_name::<U>());
}

/// Spelling 2: a `where` clause. Identical meaning; reads better when long.
fn banner_where<U>(u: &mut U)
where
    U: Uart,
{
    u.puts(b"rv6 ready\n");
    println!("  [banner_where compiled for {}]", type_name::<U>());
}

/// Spelling 3: `impl Trait` in argument position. Still one copy per type;
/// you just cannot name the parameter.
fn banner_impl(u: &mut impl Uart) {
    u.puts(b"rv6 ready\n");
    println!("  [banner_impl compiled for some impl Uart]");
}

/// Two bounds joined with `+`: the body compares AND copies.
fn largest<T: PartialOrd + Copy>(xs: &[T]) -> Option<T> {
    let mut best = *xs.first()?;
    for &x in xs {
        if x > best {
            best = x;
        }
    }
    Some(best)
}

/// `impl Trait` in RETURN position: "an iterator, whose type I am not naming".
/// This is how to return a list without a Vec -- nothing is allocated.
fn live(table: &[Option<u32>]) -> impl Iterator<Item = u32> + '_ {
    table.iter().flatten().copied()
}

fn main() {
    let mut console = Console;
    let mut rec = Recorder(Vec::new());
    let mut tally = Tally(0);

    println!("== one body, three names ==");
    banner(&mut console);
    banner(&mut rec);
    banner(&mut tally);
    println!("three different type names printed by ONE function body.");
    println!("there are three copies of `banner` in this binary, one per U.");

    println!("\n== three spellings, one meaning ==");
    banner_where(&mut rec);
    banner_impl(&mut rec);
    println!("rec now holds {} bytes: every spelling really called Recorder::putc",
             rec.0.len());

    println!("\n== what the bound buys ==");
    println!("    fn shout<T>(v: T) {{ v.putc(1) }}      // no bound");
    println!("    error[E0599]: no method named `putc` found for type parameter `T`");
    println!("    help: perhaps you need to restrict type parameter `T` with it: `T: Uart`");
    println!("inside a generic body you may use exactly what the bound promises.");
    println!("RUN  ./show-errors.sh e0599");

    println!("\n== bounds join with + ==");
    println!("largest(&[3u32, 9, 4]) = {:?}", largest(&[3u32, 9, 4]));
    println!("largest(&[7u8, 2])     = {:?}", largest(&[7u8, 2]));
    println!("largest::<u32> and largest::<u8> are two functions. `PartialOrd + Copy`");
    println!("is the whole of what the body was allowed to do to a T.");

    println!("\n== returning an iterator instead of a Vec ==");
    let table = [Some(7), None, Some(9), Some(2), None];
    print!("live(&table) yields:");
    for pid in live(&table) {
        print!(" {pid}");
    }
    println!();
    println!("no Vec was built. Where a heap is not allowed, return an iterator.");

    println!("\n== the price ==");
    println!("one copy per type: fast calls, inlining, and a bigger binary.");
    println!("program 07 makes the other trade.");
}
