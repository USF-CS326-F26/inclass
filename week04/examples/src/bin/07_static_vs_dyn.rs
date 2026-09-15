//! 07 — `dyn`: one copy of the code, and a fat pointer to find the method.
//!
//! Program 06 compiled one `banner` per type. The alternative is ONE banner
//! that takes a `&mut dyn Uart` -- a reference that carries, alongside the
//! data pointer, a pointer to a table of that type's methods:
//!
//!     &Console          [ptr]              8 bytes: thin
//!     &dyn Uart         [ptr][vtable]     16 bytes: fat
//!     &[u8]             [ptr][len]        16 bytes: fat, the other way
//!
//!     vtable for Console as Uart:
//!       +--------+--------+--------+--------+--------+--------+
//!       | drop   | size   | align  | putc   | puts   | putln  |
//!       +--------+--------+--------+--------+--------+--------+
//!
//! A call through `dyn` is one load from the vtable and one indirect jump.
//! Nothing inlines across it. What it buys: a `&mut dyn Uart` is ONE type, so
//! three different implementers fit in the same array, struct field, or slot.
//!
//! Run:  cargo run --bin 07_static_vs_dyn

use std::any::type_name;
use std::mem::size_of;

pub trait Uart {
    fn putc(&mut self, b: u8);
    fn puts(&mut self, s: &[u8]) {
        for &b in s {
            self.putc(b);
        }
    }
    fn putln(&mut self, s: &[u8]) {
        self.puts(s);
        self.putc(b'\n');
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

/// Static dispatch: one copy per U.
fn banner_generic<U: Uart>(u: &mut U) {
    u.putln(b"rv6 ready");
    println!("  [generic: compiled for {}]", type_name::<U>());
}

/// Dynamic dispatch: one copy, ever. The method is found at run time.
fn banner_dyn(u: &mut dyn Uart) {
    u.putln(b"rv6 ready");
    println!("  [dyn: this body knows only `dyn Uart`]");
}

fn main() {
    println!("== two fat pointers, one thin ==");
    println!("size_of::<&Console>()  = {}", size_of::<&Console>());
    println!("size_of::<&dyn Uart>() = {}   pointer + vtable pointer", size_of::<&dyn Uart>());
    println!("size_of::<&[u8]>()     = {}   pointer + length", size_of::<&[u8]>());
    println!("a slice and a trait object are both `pointer + one more word`.");
    println!("the slice's word is a length; the trait object's is a method table.");

    let mut console = Console;
    let mut rec = Recorder(Vec::new());
    let mut tally = Tally(0);

    println!("\n== the same three, through one body ==");
    banner_dyn(&mut console);
    banner_dyn(&mut rec);
    banner_dyn(&mut tally);
    println!("one function. Three targets. The vtable chose putc each time.");

    println!("\n== versus the generic ==");
    banner_generic(&mut console);
    banner_generic(&mut tally);

    println!("\n== the coercion point ==");
    let u: &mut dyn Uart = &mut rec; // <- here the concrete type is forgotten
    u.puts(b"!");
    println!("`let u: &mut dyn Uart = &mut rec;` -- from this line on, u is a fat");
    println!("pointer and Recorder is gone. rec.0.len() = {} proves it still wrote.",
             rec.0.len());

    println!("\n== what dyn buys: heterogeneous storage ==");
    let mut sinks: [&mut dyn Uart; 3] = [&mut console, &mut rec, &mut tally];
    for s in sinks.iter_mut() {
        s.putln(b"tick");
    }
    println!("[&mut dyn Uart; 3] held a Console, a Recorder, and a Tally.");
    println!("[&mut U; 3] cannot: U is ONE type, chosen once.");
    println!("Recorder now has {} bytes; Tally counted {}", rec.0.len(), tally.0);

    println!("\n== the cost ==");
    println!("each call: load the method pointer from the vtable, jump through it.");
    println!("no inlining across the call. The shell uses dyn at 115200 baud, where");
    println!("that is nothing. The scheduler uses generics, because it runs every tick.");

    println!("\n== what a vtable cannot hold ==");
    println!("a generic method would need one entry per type it is ever called");
    println!("with -- not a number the compiler knows. Such a trait is not");
    println!("`dyn compatible`, and using it as `dyn` is error[E0038].");
    println!("RUN  ./show-errors.sh e0038");
}
