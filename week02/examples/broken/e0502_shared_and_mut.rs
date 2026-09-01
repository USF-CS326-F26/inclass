// error[E0502]: cannot borrow `v` as mutable because it is also borrowed as
// immutable
//
// ALIASING XOR MUTATION. `first` points into the Vec's buffer; `push` may
// reallocate that buffer and free the old one. In C++ this compiles and
// `first` becomes a dangling pointer.
//
// FIX 1: finish with `first` before pushing (borrows end at their last use).
// FIX 2: copy what you need out: `let first = v[0];` (i32 is Copy).
fn main() {
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    v.push(4);
    println!("{first}");
}
