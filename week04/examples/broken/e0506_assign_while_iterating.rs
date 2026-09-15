// error[E0506]: cannot assign to `table[_]` because it is borrowed
//
// The for loop's iterator holds a SHARED borrow of the whole table until the
// loop ends. `table[i] = None` writes into it while that borrow is alive.
// (On a Vec the same line is E0502 -- indexing a Vec is a method call.)
// This is exit() closing every descriptor; it is also 06r's alloc_slot.
//
// FIX 1: `for (i, slot) in table.iter_mut().enumerate() { *slot = None; }`
//        -- one &mut per slot, handed out in turn, and the write goes through it.
// FIX 2: `for i in 0..table.len()` -- no iterator, so no borrow is held.
// (A `return` right after the write is accepted: the borrow checker can see
// the iterator is never used again. A loop that CONTINUES is not.)
fn close_all(table: &mut [Option<u32>]) -> usize {
    let mut closed = 0;
    for (i, slot) in table.iter().enumerate() {
        if slot.is_some() {
            table[i] = None;
            closed += 1;
        }
    }
    closed
}

fn main() {
    let mut fds = [Some(1), Some(1), None, Some(7)];
    println!("closed {}", close_all(&mut fds));
}
