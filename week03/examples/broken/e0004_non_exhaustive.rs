// error[E0004]: non-exhaustive patterns
//
// THE FEATURE, NOT A BUG. A fifth trap cause was added last week. The compiler
// now lists every match that has not been told what the new one means -- by
// file and line, before the kernel boots. In C every switch keeps compiling and
// silently takes its default arm.
//
// FIX 1: add the missing arm and decide what Breakpoint means here.
// FIX 2: `_ => false` compiles, and switches the check off forever: a catch-all
//        covers variants that do not exist yet. Right for open domains such as
//        a syscall number from a user program; wrong for a closed set you own.
enum Trap {
    Timer,
    Syscall(usize),
    PageFault { addr: usize },
    Illegal,
    Breakpoint,
}

fn is_fatal(t: Trap) -> bool {
    match t {
        Trap::Timer => false,
        Trap::Syscall(_) => false,
        Trap::PageFault { .. } => false,
        Trap::Illegal => true,
    }
}

fn main() {
    println!("{}", is_fatal(Trap::Breakpoint));
}
