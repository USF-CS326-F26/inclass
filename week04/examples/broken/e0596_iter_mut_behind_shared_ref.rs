// error[E0596]: cannot borrow `*table` as mutable, as it is behind a `&`
//               reference
//
// The loop is fine. The PARAMETER is wrong. `&[T]` bought the right to read;
// `iter_mut` needs the right to write, and no loop can add that. The caller
// may well have passed `&mut table` -- the signature threw the `mut` away.
//
// FIX 1: `table: &mut [Option<u32>]` -- the signature says what the body does.
// FIX 2: if it really only reads, use `iter()` and return what you found.
fn alloc_slot(table: &[Option<u32>], pid: u32) -> Option<usize> {
    for (i, slot) in table.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(pid);
            return Some(i);
        }
    }
    None
}

fn main() {
    let mut procs = [None; 4];
    println!("{:?}", alloc_slot(&mut procs, 42));
}
