//! 07 — Drop and the guard: release became a closing brace.
//!
//! When a value goes out of scope the compiler emits a call to its type's
//! `Drop`. You do not write the call, you cannot forget it, and its site is a
//! closing brace -- including paths you did not think about: an early `return`,
//! a `?`, the end of a function the value was passed into.
//!
//! A GUARD is a value whose existence IS the held resource:
//!
//!     let g = lock.acquire();   <- held from here
//!     ...                          nothing else can reach the lock
//!     }                         <- released here, by the compiler
//!
//! Two properties earn this a place in a kernel. It is DETERMINISTIC, so a drop
//! can carry real work; and it runs WHEREVER THE OWNER ENDS. C++ has had this
//! since the mid-1980s as RAII. What Rust adds is that ownership is checked, so
//! RAII cannot be defeated by an accidental copy or a second owner.
//!
//! Run:  cargo run --bin 07_drop_guard

fn main() {
    println!("== drop runs at the closing brace ==");
    {
        let _a = Tracked::new("first");
        let _b = Tracked::new("second");
        println!("  both alive");
    }
    println!("dropped in REVERSE declaration order -- second, then first");

    println!("\n== a guard: the resource is the value's lifetime ==");
    let mut lock = Lock::new("proc table");
    println!("held before  = {}", lock.is_held());
    {
        let guard = lock.acquire();
        println!("held inside  = {}", guard.is_held());
        println!("  ... critical section ...");
        // Nothing else can reach `lock` here: the guard holds the only &mut.
        // println!("{}", lock.is_held());   // error[E0502]
    }
    println!("held after   = {}   <- nobody called release()", lock.is_held());

    println!("\n== the borrow order inside `acquire` is the whole trick ==");
    println!("  read/write through the &mut FIRST,");
    println!("  THEN move it into the struct -- after that the guard owns it");
    println!("do it the other way round and you get E0499 or E0502");

    println!("\n== release early, by consuming the guard ==");
    {
        let guard = lock.acquire();
        println!("held inside  = {}", guard.is_held());
        guard.release();            // takes `self` by value: the guard dies here
        println!("held after release() = {}   <- and the borrow ended too",
                 lock.is_held());
    }
    println!("`pub fn release(self) {{}}` -- an empty body that frees something.");
    println!("std's `drop` is exactly that: pub fn drop<T>(_x: T) {{ }}");

    println!("\n== it runs on the paths you forgot ==");
    println!("early_return(true):");
    early_return(true);
    println!("early_return(false):");
    early_return(false);
    println!("one release site in the source, every path covered");

    println!("\n== moving the value moves the duty to release it ==");
    let t = Tracked::new("moved");
    println!("  handing it to a function that keeps it:");
    consume(t);
    println!("  ... it was dropped inside `consume`, not here");

    println!("\n== Copy XOR Drop, one more time ==");
    println!("A guard can never be Copy. A copy would release once per copy,");
    println!("which is the double free wearing a hat:");
    println!("  error[E0184]: the trait `Copy` cannot be implemented for this");
    println!("                type; the type has a destructor");
    println!("This is the shape of a spinlock guard -- swap the bool for a");
    println!("lock word and Drop for the unlock, and a kernel cannot leave a");
    println!("lock held.");
    println!("\n**RUN** ../examples/show-errors.sh e0184");
}

/// A value that announces its own destruction.
pub struct Tracked {
    name: &'static str,
}

impl Tracked {
    pub fn new(name: &'static str) -> Tracked {
        println!("  new  {name}");
        Tracked { name }
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        println!("  drop {}", self.name);
    }
}

/// The resource: a flag that must be put back.
pub struct Lock {
    name: &'static str,
    held: bool,
}

impl Lock {
    pub fn new(name: &'static str) -> Lock {
        Lock { name, held: false }
    }

    pub fn is_held(&self) -> bool {
        self.held
    }

    /// Hand out a guard. The `&mut self` borrow travels into the guard, so
    /// while the guard lives this `Lock` cannot be reached any other way.
    pub fn acquire(&mut self) -> LockGuard<'_> {
        // Set the flag BEFORE the borrow is moved into the struct literal.
        self.held = true;
        println!("  acquire {}", self.name);
        LockGuard { lock: self }
    }
}

/// Holding this value IS holding the lock.
pub struct LockGuard<'a> {
    lock: &'a mut Lock,
}

impl LockGuard<'_> {
    /// Reaching the lock at all goes through the guard while it is alive.
    pub fn is_held(&self) -> bool {
        self.lock.held
    }

    /// Release now, rather than at the closing brace.
    ///
    /// `self` is taken by value, so calling this MOVES the guard in and it
    /// dies at this closing brace -- running `Drop` and freeing the lock.
    pub fn release(self) {}
}

impl Drop for LockGuard<'_> {
    /// You never call this. The compiler emits the call wherever the guard's
    /// owner stops existing, and there is no path through the program that
    /// skips it.
    fn drop(&mut self) {
        self.lock.held = false;
        println!("  release {}", self.lock.name);
    }
}

/// Two exits, one release site.
fn early_return(bail: bool) {
    let _t = Tracked::new("scoped");
    if bail {
        println!("    bailing out early");
        return;
    }
    println!("    ran to the end");
}

/// Takes the value by value, so this function is where it dies.
fn consume(t: Tracked) {
    println!("  consume got {}", t.name);
}
