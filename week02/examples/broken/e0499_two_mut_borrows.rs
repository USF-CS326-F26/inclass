// error[E0499]: cannot borrow `v` as mutable more than once at a time
//
// &mut means EXCLUSIVE. Two writers to the same value at the same time is the
// data race, and it is rejected even in single-threaded code.
//
// FIX 1: end the first borrow before starting the second (separate blocks, or
//        just use them one after the other).
// FIX 2: for disjoint pieces of a slice, use `split_at_mut`.
fn main() {
    let mut v = vec![1, 2, 3];
    let a = &mut v;
    let b = &mut v;
    a.push(4);
    b.push(5);
}
