// error[E0106]: missing lifetime specifier
// (and, once you add one, error[E0515]: cannot return reference to local)
//
// This is C's returning-a-pointer-to-a-stack-local. Rust asks the question
// that exposes it: borrowed from WHAT? There is no input to borrow from, and
// `s` dies at the closing brace.
//
// FIX 1: return the String itself — move the ownership out.
// FIX 2: take a parameter and return a slice of it:
//            fn head(s: &str) -> &str { &s[..4] }
fn dangle() -> &String {
    let s = String::from("kernel");
    &s
}

fn main() {
    println!("{}", dangle());
}
