// error[E0507]: cannot move out of `self.name` which is behind a shared
// reference
//
// `&self` is a borrow: the caller still owns the Device and expects it back
// intact. Moving the String out would leave a hole in a value somebody else
// owns, so the compiler refuses.
//
// Only a by-value `self` may take a field away -- nobody is left to notice.
//
// FIX 1: return a borrow: `fn name(&self) -> &str { &self.name }`.
// FIX 2: `self.name.clone()` -- a second buffer, allocated visibly.
// FIX 3: take `self` by value if the Device really is finished with.
struct Device {
    name: String,
    major: u32,
}

impl Device {
    fn name(&self) -> String {
        self.name
    }
}

fn main() {
    let d = Device { name: String::from("uart0"), major: 1 };
    println!("{} {}", d.name(), d.major);
}
