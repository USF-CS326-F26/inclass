// ╔══════════════════════════════════════════════════════════════════════╗
// ║  START HERE. One array, two loops, and nothing else.                 ║
// ║      cargo run --bin basic_table                                     ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Before the descriptor table, the concept on its own: an array of four
// slots, a loop that reads them, a loop that writes them, and the index that
// comes back as an Option.

fn main() {
    // ── 1. `[None; 4]` is four Nones in a row ────────────────────────────
    //
    // The type says how many. The memory is exactly that many slots, back
    // to back, wherever this variable lives -- no allocator involved.
    let mut slots: [Option<u32>; 4] = [None; 4];
    println!("1. [None; 4]             -> {slots:?}");
    println!("   size_of               -> {} bytes, fixed", std::mem::size_of_val(&slots));

    // ── 2. `iter()` reads: each item is a &Option<u32> ───────────────────
    slots[1] = Some(3);
    let live = slots.iter().filter(|slot| slot.is_some()).count();
    println!("2. after slots[1] = Some(3): {slots:?}");
    println!("   iter().filter(is_some).count() -> {live}");

    // ── 3. `iter_mut()` writes: each item is a &mut Option<u32> ──────────
    //
    // `slot` is a reference INTO the array. The star assigns through it.
    for slot in slots.iter_mut() {
        if slot.is_none() {
            *slot = Some(7);
            break; // only the first one
        }
    }
    println!("3. iter_mut(), *slot = Some(7) on the first None: {slots:?}");

    // ── 4. `enumerate()`: the index of the first free slot, or None ──────
    //
    // The answer is `Option<usize>`: an index, or nothing. Never -1.
    let free = slots.iter().position(|slot| slot.is_none());
    println!("4. first free slot       -> {free:?}");
    slots[2] = Some(1);
    slots[3] = Some(1);
    let free = slots.iter().position(|slot| slot.is_none());
    println!("   with every slot taken -> {free:?}   the honest answer");

    // ── 5. the error worth reading out loud ──────────────────────────────
    println!("5. uncomment the last block and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME. rustc says:
    //
    //   error[E0594]: cannot assign to `*slot`, which is behind a `&` reference
    //      |     for slot in slots.iter() {
    //      |                 ------------ this iterator yields `&` references
    //      |         *slot = None;
    //      |         ^^^^^^^^^^^^ `slot` is a `&` reference, so the data it
    //      |                      refers to cannot be written
    //
    // `iter()` borrowed for reading; the star is a write. The loop you may
    // write is decided by which iterator you asked for -- and, one level up,
    // by whether the function was handed `&[T]` or `&mut [T]`.
    //
    // for slot in slots.iter() {
    //     *slot = None;
    // }
    // ---------------------------------------------------------------------
}
