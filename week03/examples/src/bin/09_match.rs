//! 09 — match: the compiler as code reviewer.
//!
//! Each ARM is a pattern, `=>`, and the value it produces. `match` is an
//! EXPRESSION, which is why it sits on the right of a `let`.
//!
//!     let n = match t {
//!         Trap::Timer            => 0,       a variant
//!         Trap::Syscall(num)     => num,     ... binding its field
//!         Trap::PageFault { .. } => 1,       ... ignoring its fields
//!         Trap::Illegal          => 2,
//!     };
//!
//! EXHAUSTIVENESS is the safety property. Read forwards it is a nuisance; read
//! backwards it is why the type exists. Add a variant next month and the
//! compiler lists every place that now needs a decision -- by file and line,
//! before the kernel boots. Unless you wrote `_`.
//!
//!     The rule: `_` for open domains (a number from hardware or a user
//!     program), never for closed ones you defined yourself.
//!
//! Run:  cargo run --bin 09_match

fn main() {
    println!("== match is an expression ==");
    for t in traps() {
        let cost = match t {
            Trap::Timer => 1,
            Trap::Syscall(num) => num,
            Trap::PageFault { .. } => 100,
            Trap::Illegal => 0,
        };
        println!("  {:<38} cost {cost}", show(t));
    }

    println!("\n== the four pattern features you need ==");
    for t in traps() {
        let label = match t {
            // `|` -- one arm, two variants.
            Trap::Timer | Trap::Illegal => "no payload to read",
            // `{ .. }` -- has fields, do not care.
            Trap::PageFault { .. } => "has an address, ignoring it",
            // binding -- pull the field out under a name.
            Trap::Syscall(_) => "has a number, see below",
        };
        println!("  {:<38} {label}", show(t));
    }

    println!("\n== binding the payload ==");
    for t in traps() {
        let detail = match t {
            Trap::Syscall(num) => format!("syscall number {num}"),
            Trap::PageFault { addr } => format!("faulting address {addr:#x}"),
            Trap::Timer => String::from("no detail"),
            Trap::Illegal => String::from("no detail"),
        };
        println!("  {:<38} {detail}", show(t));
    }

    println!("\n== exhaustiveness ==");
    println!("Delete one arm and the program does not compile:");
    println!("  error[E0004]: non-exhaustive patterns:");
    println!("                `Trap::Illegal` not covered");
    println!("Its value is not in the match you are writing now, but in the");
    println!("twelve you will not remember when you change the enum next month.");

    println!("\n== the `_` trap ==");
    println!("`_` switches the check off forever: a catch-all covers variants");
    println!("that do not exist yet, so adding one produces no error and no");
    println!("warning. `is_illegal_wrong` below is right today, by luck.");
    for t in traps() {
        println!("  {:<38} careful {:<5} lazy {}",
                 show(t), t.is_illegal(), t.is_illegal_wrong());
    }
    println!("Add a fifth variant and only `is_illegal` breaks the build.");

    println!("\n== ... and when `_` is right ==");
    // The number came from a user program: the domain is genuinely open.
    for num in [1usize, 63, 64, 9999] {
        println!("  syscall {num:<5} -> {}", dispatch(num));
    }
    println!("`_ => -1` here is correct: any integer may arrive.");

    println!("\n== guards: an `if` on an arm ==");
    for t in traps() {
        let kind = match t {
            Trap::Syscall(num) if num < 64 => "known syscall",
            Trap::Syscall(_) => "syscall out of range",
            Trap::PageFault { addr } if addr >= KERNBASE => "kernel page fault",
            Trap::PageFault { .. } => "user page fault",
            Trap::Timer => "timer",
            Trap::Illegal => "illegal instruction",
        };
        println!("  {:<38} {kind}", show(t));
    }
    println!("A failing guard does NOT leave the match -- matching CONTINUES");
    println!("with the next arm. That fall-through is the whole point; testing");
    println!("inside the arm body loses it and forces you to restate the");
    println!("failure path by hand.");
    println!("Guarded arms do not count toward exhaustiveness, so a match");
    println!("resting on one still needs a fallback.");

    println!("\n== matching a pair, to test two values at once ==");
    for (level, t) in [(Level::User, Trap::Syscall(63)),
                       (Level::Supervisor, Trap::Syscall(63)),
                       (Level::User, Trap::Illegal)] {
        let action = match (level, t) {
            (Level::User, Trap::Syscall(_)) => "service it",
            (Level::Supervisor, Trap::Syscall(_)) => "the kernel called itself?!",
            (Level::User, _) => "kill the process",
            (_, _) => "panic",
        };
        println!("  {:<12} {:<26} -> {action}",
                 format!("{level:?}"), show(t));
    }
    println!("that is how a transition table becomes one readable block");

    println!("\n== if let: a one-arm match ==");
    let t = Trap::PageFault { addr: 0x8000_1000 };
    if let Trap::PageFault { addr } = t {
        println!("  if let  -> faulted at {addr:#x}");
    }

    println!("\n== let ... else: bind, or bail out ==");
    println!("  describe_syscall(Trap::Syscall(63)) = {:?}",
             describe_syscall(Trap::Syscall(63)));
    println!("  describe_syscall(Trap::Timer)       = {:?}",
             describe_syscall(Trap::Timer));

    println!("\n== matches!: a match that answers yes or no ==");
    for t in traps() {
        println!("  {:<38} is a fault: {}",
                 show(t), matches!(t, Trap::PageFault { .. }));
    }
    println!("\n**RUN** ../examples/show-errors.sh e0004");
}

/// Where the kernel's own memory starts.
pub const KERNBASE: usize = 0x8000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Supervisor,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trap {
    Timer,
    Syscall(usize),
    PageFault { addr: usize },
    Illegal,
}

impl Trap {
    /// One arm per variant, no catch-all. Adding a variant breaks this, which
    /// is the point.
    pub fn is_illegal(&self) -> bool {
        match self {
            Trap::Illegal => true,
            Trap::Timer => false,
            Trap::Syscall(_) => false,
            Trap::PageFault { .. } => false,
        }
    }

    /// Same answer today. Adding a variant silently classifies it as `false`,
    /// and nothing tells you.
    pub fn is_illegal_wrong(&self) -> bool {
        match self {
            Trap::Illegal => true,
            _ => false,
        }
    }
}

fn traps() -> [Trap; 4] {
    [Trap::Timer, Trap::Syscall(63), Trap::PageFault { addr: 0x8000_1000 }, Trap::Illegal]
}

/// `_` is right here: the number arrives from a user program, so the domain
/// is open and "anything else" is genuinely the rule.
fn dispatch(num: usize) -> i64 {
    match num {
        1 => 0,
        63 => 5,
        64 => 5,
        _ => -1,
    }
}

/// `let ... else` handles the "bind, or leave" shape kernel code is full of.
fn describe_syscall(t: Trap) -> Option<String> {
    let Trap::Syscall(num) = t else {
        return None;
    };
    Some(format!("syscall {num}"))
}

/// The `Debug` derive prints integers in decimal, and an address is only
/// readable in hex. This is not a `match` lesson -- it just makes the columns
/// below legible.
fn show(t: Trap) -> String {
    match t {
        Trap::Timer => String::from("Timer"),
        Trap::Syscall(num) => format!("Syscall({num})"),
        Trap::PageFault { addr } => format!("PageFault {{ addr: {addr:#x} }}"),
        Trap::Illegal => String::from("Illegal"),
    }
}
