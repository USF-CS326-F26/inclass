// error[E0184]: the trait `Copy` cannot be implemented for this type; the type
// has a destructor
//
// COPY XOR DROP. `Copy` says "duplicating the bits duplicates the value"; `Drop`
// says "the release runs exactly once". A type that promised both would run its
// release once per copy -- which is the double free, in a new hat.
//
// This is not a rule you have to remember. It is a combination the type system
// refuses, so the bug cannot be written.
//
// FIX 1: drop the `Copy` derive. A guard is meant to have exactly one owner.
// FIX 2: drop the `Drop` impl, if the type really owns nothing.
#[derive(Clone, Copy)]
struct Guard {
    id: usize,
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("release {}", self.id);
    }
}

fn main() {
    let a = Guard { id: 1 };
    let b = a;
    println!("{} {}", a.id, b.id);
}
