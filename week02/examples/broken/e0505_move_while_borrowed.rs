// error[E0505]: cannot move out of `s` because it is borrowed
//
// A borrow is a claim on the value staying put. You cannot hand the value to
// someone else while a reference to it is still alive.
//
// FIX: use `r` before the move, or borrow again after it.
fn main() {
    let s = String::from("kernel");
    let r = &s;
    let t = s;
    println!("{r} {t}");
}
