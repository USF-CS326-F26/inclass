// ╔══════════════════════════════════════════════════════════════════════╗
// ║  START HERE. One trait, two types, one generic function.             ║
// ║      cargo run --bin basic_trait                                     ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Before the sinks and the allocators, the concept on its own: the timer
// interrupt calls something once per tick, and two different things want to
// be called.

fn main() {
    // ── 1. a trait is a promise: one required method, one default ────────
    //
    // `beat` has no body: every implementer must write it. `beat_for` has a
    // body: written once, here, and inherited by every implementer.
    println!("1. trait Heartbeat {{ fn beat(&mut self); fn beat_for(..) {{ default }} }}");

    // ── 2. the first implementer ─────────────────────────────────────────
    let mut p = Printer;
    print!("2. Printer.beat_for(3)   -> ");
    p.beat_for(3);
    println!();

    // ── 3. the second implementer: beat_for already works for it ─────────
    //
    // Counter wrote `beat` and nothing else. `beat_for` was compiled before
    // Counter existed, against a `self` that only promised to have `beat`.
    let mut c = Counter { ticks: 0 };
    c.beat_for(5);
    println!("3. Counter.beat_for(5)   -> ticks = {}   (nobody wrote beat_for for Counter)",
             c.ticks);

    // ── 4. a generic function: the bound is what lets it call beat_for ───
    print!("4. run_timer(&mut p, 2)  -> ");
    run_timer(&mut p, 2);
    println!();
    run_timer(&mut c, 2);
    println!("   run_timer(&mut c, 2)  -> ticks = {}   same function, other type", c.ticks);
    println!("   two copies of run_timer were compiled: one for Printer, one for Counter");

    // ── 5. the error worth reading out loud ──────────────────────────────
    println!("5. uncomment the last block and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME. rustc says:
    //
    //   error[E0046]: not all trait items implemented, missing: `beat`
    //      |     fn beat(&mut self);
    //      |     ------------------- `beat` from trait
    //   ...
    //      |     impl Heartbeat for Silent {}
    //      |     ^^^^^^^^^^^^^^^^^^^^^^^^^ missing `beat` in implementation
    //
    // A default method is optional. A required method is not, and the
    // compiler names the one you skipped. `beat_for` would have called it.
    //
    // struct Silent;
    // impl Heartbeat for Silent {}
    // ---------------------------------------------------------------------
}

/// What the timer interrupt wants from anything it drives.
pub trait Heartbeat {
    /// Required: no body.
    fn beat(&mut self);

    /// Default: a body, written once, in terms of `beat`.
    fn beat_for(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.beat();
        }
    }
}

/// A unit struct: zero bytes, exists to be a type.
pub struct Printer;

impl Heartbeat for Printer {
    fn beat(&mut self) {
        print!("tick ");
    }
}

pub struct Counter {
    pub ticks: usize,
}

impl Heartbeat for Counter {
    fn beat(&mut self) {
        self.ticks += 1;
    }
}

/// Generic over H, and `H: Heartbeat` is the whole of what it may assume.
/// Delete the bound and `h.beat_for` is error[E0599].
pub fn run_timer<H: Heartbeat>(h: &mut H, ticks: usize) {
    h.beat_for(ticks);
}
