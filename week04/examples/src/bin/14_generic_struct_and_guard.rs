//! 14 — A struct with a type parameter, and the guard that hands out the T.
//!
//! Programs 06 and 07 put the parameter on a FUNCTION, where it makes copies
//! of one body. Put it on a STRUCT and it makes types: one definition, and a
//! separate type -- its own layout, its own methods -- for every T anybody
//! writes down.
//!
//!     struct Lock<T> { held: bool, data: T }      one definition
//!
//!     Lock<Recorder>    32 bytes   a Vec inside, and the flag
//!     Lock<Console>      1 byte    a zero-sized payload costs nothing
//!     Lock<u64>         16 bytes   Lock<Tally> is 16 too, and a different type
//!
//! Week 03's program 07 guarded a bool. This guards a T, which makes it
//! `rv6/src/spinlock.rs`: `SpinLock<T>` with a `SpinLockGuard<'a, T>`, the
//! type exercise 37k asks you to write. The kernel compiles two of them,
//! `SpinLock<i64>` for a semaphore's count and `SpinLock<FileSystem>` for the
//! one global filesystem, and they share no code at all.
//!
//! Run:  cargo run --bin 14_generic_struct_and_guard

use std::mem::size_of;
use std::ops::{Deref, DerefMut};

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

/// A lock with a payload. NO bound on the struct: taking a lock does not
/// care what it is guarding, so `Lock<u64>` is as good as `Lock<Recorder>`.
struct Lock<T> {
    held: bool,
    data: T,
}

/// Holding this value IS holding the lock, and it is the only way to the T.
/// Two parameters: the lifetime of the borrow, and the type it guards.
struct Guard<'a, T> {
    lock: &'a mut Lock<T>,
}

impl<T> Lock<T> {
    /// A `const fn`, so a `static` can hold one before anything has run.
    /// rv6 needs exactly this for `static FS: SpinLock<FileSystem>`.
    const fn new(data: T) -> Self {
        Lock { held: false, data }
    }

    /// Take the lock. On the host the borrow checker IS the lock: the guard
    /// holds the only `&mut`, so nothing else can reach the data at all.
    fn lock(&mut self) -> Guard<'_, T> {
        self.held = true;
        Guard { lock: self }
    }

    fn is_held(&self) -> bool {
        self.held
    }
}

/// The bound sits HERE, on one impl block, and not on the struct: `banner`
/// exists for a `Lock<Recorder>` and does not exist for a `Lock<u64>`.
impl<T: Uart> Lock<T> {
    fn banner(&mut self) {
        self.lock().putln(b"rv6 ready");
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.lock.data
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.lock.data
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.held = false;
        println!("  [Drop for Guard: released]");
    }
}

/// `const fn new` again, and this is as far as the host version goes: see
/// the last section for what the kernel has to do instead.
static TICKS: Lock<u64> = Lock::new(0);

fn main() {
    println!("== one definition, a type for every T ==");
    println!("size_of::<Recorder>()        = {:2}   the payload on its own",
             size_of::<Recorder>());
    println!("size_of::<Lock<Recorder>>()  = {:2}   the payload, the flag, the padding",
             size_of::<Lock<Recorder>>());
    println!("size_of::<Lock<Console>>()   = {:2}   a zero-sized payload costs nothing",
             size_of::<Lock<Console>>());
    println!("size_of::<Lock<u64>>()       = {:2}   the flag, seven bytes of padding, the u64",
             size_of::<Lock<u64>>());
    println!("size_of::<Lock<Tally>>()     = {:2}   the same size, not the same type",
             size_of::<Lock<Tally>>());
    println!("06 emitted one copy of a function body per type it was called");
    println!("with. A struct with a parameter emits a TYPE per type: one");
    println!("definition above, and the four Lock layouts here.");

    println!("\n== the guard is the permission ==");
    let mut rec = Lock::new(Recorder(Vec::new()));
    println!("is_held() before = {}", rec.is_held());
    {
        let mut g = rec.lock(); // <- to the closing brace, only g reaches the T
        g.putln(b"rv6 ready");
        println!("wrote {} bytes through the guard, which derefs to Recorder",
                 g.0.len());
    }
    println!("is_held() after  = {}", rec.is_held());
    println!("while the guard lived, `rec.is_held()` was not false, and not");
    println!("true: it was error[E0502], cannot borrow `rec` as immutable");
    println!("because it is also borrowed as mutable. Nobody can even ask.");

    println!("\n== where the bound goes ==");
    let mut tal = Lock::new(Tally(0));
    let mut ticks = Lock::new(0u64);
    println!("banner takes the lock itself, so each call releases at its end:");
    rec.banner();
    tal.banner();
    println!("Lock<Recorder> now holds {} bytes, and Lock<Tally> counted {}.",
             rec.data.0.len(), tal.data.0);
    println!("a Lock<u64> has no banner, and locks perfectly well without one:");
    {
        let mut g = ticks.lock();
        *g += 1; // <- DerefMut on a u64: 37k's counter, 38k's `*count -= 1`
        *g += 1;
    }
    println!("it counted to {}. Asking it for a banner does not compile:", ticks.data);
    println!("    ticks.banner();");
    println!("    error[E0599]: the method `banner` exists for struct");
    println!("                  `Lock<u64>`, but its trait bounds were not satisfied");
    println!("    note: trait bound `u64: Uart` was not satisfied");
    println!("`new`, `lock` and `is_held` live in `impl<T>` and exist for every");
    println!("T. `banner` lives in `impl<T: Uart>`, so it exists for the sinks.");
    println!("that is rv6: `SpinLock<T>` carries no bound, and `T: Send` sits");
    println!("by itself on the `unsafe impl Sync`.");

    println!("\n== a lifetime and a type ==");
    println!("size_of::<Guard<'_, Recorder>>() = {:2}   one thin pointer",
             size_of::<Guard<'_, Recorder>>());
    println!("size_of::<&mut dyn Uart>()       = {:2}   07's fat pointer, for contrast",
             size_of::<&mut dyn Uart>());
    println!("the guard needs both parameters: `'a` for how long it borrows,");
    println!("`T` for what it hands out. Leave the lifetime off and rustc says");
    println!("error[E0106]: missing lifetime specifier, then writes the fix:");
    println!("    struct Guard<'a, T> {{ lock: &'a Lock<T> }}");
    println!("program 12's `Args<'a>` is this shape with the lifetime only.");

    println!("\n== what rv6 writes instead ==");
    println!("    pub struct SpinLock<T> {{ locked: AtomicBool, data: UnsafeCell<T> }}");
    println!("    pub struct SpinLockGuard<'a, T> {{ lock: &'a SpinLock<T> }}");
    println!("    unsafe impl<T: Send> Sync for SpinLock<T> {{}}");
    println!("TICKS.is_held() = {}, because `new` is a const fn and a static",
             TICKS.is_held());
    println!("can hold one. `TICKS.lock()` is error[E0596]: cannot borrow");
    println!("immutable static item `TICKS` as mutable -- and that is the whole");
    println!("reason for the kernel's version. One CPU's borrow checker cannot");
    println!("serialize two CPUs, so `lock` takes `&self`, spins on the atomic,");
    println!("and buys the shared-to-mutable step with one UnsafeCell.");

    println!("\n== when a struct needs a type parameter ==");
    println!("it needs one when it HOLDS a T. The parameter has to appear in a");
    println!("field, or the struct is error[E0392] and rustc offers PhantomData.");
    println!("it does not when the set is closed -- 13's Sinks, rv6's FileKind");
    println!("-- or when one slot must hold several types at once, which is");
    println!("07's `&mut dyn Uart`. 06 pays code size for speed and 13 shows");
    println!("who chooses; a generic struct pays in layout, one type per T.");
    println!("RUN  ./show-errors.sh e0392");
}
