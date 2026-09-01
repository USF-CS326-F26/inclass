// error[E0596]: cannot borrow `v` as mutable, as it is not declared as mutable
//
// Immutability is the default, and it is a real guarantee, not a style rule.
//
// FIX: `let mut v = ...`
fn main() {
    let v = vec![1, 2, 3];
    v.push(4);
    println!("{v:?}");
}
