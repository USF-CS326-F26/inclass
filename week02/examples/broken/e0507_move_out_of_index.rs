// error[E0507]: cannot move out of index of `Vec<String>`
//
// If this were allowed, the Vec would still believe it owns element 0 and
// would free it at the end of the scope — while `first` also freed it. A
// double free.
//
// FIX 1: borrow it:  let first = &v[0];
// FIX 2: clone it:   let first = v[0].clone();
// FIX 3: take it out for real: let first = v.remove(0);  (Vec gives up the slot)
fn main() {
    let v = vec![String::from("uart0"), String::from("virtio0")];
    let first = v[0];
    println!("{first}");
}
