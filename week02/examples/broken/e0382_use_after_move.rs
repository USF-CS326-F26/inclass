// error[E0382]: borrow of moved value
//
// The use-after-free, caught at compile time. `s` gave the buffer away on the
// `let t = s;` line, so the name `s` is struck off from there on.
//
// FIX 1: use `t` instead.
// FIX 2: `let t = s.clone();` — pay for a second buffer, visibly.
// FIX 3: `let t = &s;` — borrow instead of taking.
fn main() {
    let s = String::from("kernel");
    let t = s;
    println!("{s} {t}");
}
