//! 03 — An index, or nothing. Never -1.
//!
//! xv6's `iget(inum)` looks in the in-memory inode cache for a slot already
//! holding that inode; failing that it takes the first empty slot and fills
//! it; failing THAT it panics ("iget: no inodes"). Three outcomes, and the
//! honest return type for "which slot?" is
//!
//!     Option<usize>       Some(index), or None
//!
//! In C this function returns a pointer, and the empty case is NULL, and the
//! caller forgets to check. Here the caller cannot use the index without
//! opening the Option, so the check is not optional.
//!
//! The search itself is one adapter:
//!
//!     cache.iter().position(|&s| s == Some(inum))     -> Option<usize>
//!
//! and the fill is the `iter_mut().enumerate()` loop from program 02, ending in
//! `find`. Both are the loop you would write by hand, spelled so that the
//! result is a value rather than a side effect.
//!
//! Run:  cargo run --bin 03_slot_search

/// How many inodes may be cached at once. xv6's NINODE is 50.
const NINODE: usize = 6;

/// Which slot holds `inum`, if any. Read-only, so `&[...]`.
fn slot_of(cache: &[Option<u32>], inum: u32) -> Option<usize> {
    cache.iter().position(|&s| s == Some(inum))
}

/// The slot holding `inum`: an existing one, or a freshly filled one.
/// Returns `None` when the cache is full -- where xv6 panics.
fn iget(cache: &mut [Option<u32>], inum: u32) -> Option<usize> {
    // 1. already cached?
    if let Some(i) = slot_of(cache, inum) {
        return Some(i);
    }
    // 2. an empty slot? `find` hands back the (index, &mut slot) pair, and
    //    writing through the slot fills the cache.
    if let Some((i, slot)) = cache.iter_mut().enumerate().find(|(_, s)| s.is_none()) {
        *slot = Some(inum);
        return Some(i);
    }
    // 3. no room. xv6: panic("iget: no inodes"). Here: let the caller decide.
    None
}

fn main() {
    let mut cache: [Option<u32>; NINODE] = [None; NINODE];

    println!("== an empty cache ==");
    show(&cache);

    println!("\n== iget fills the lowest free slot ==");
    for inum in [3, 7, 3, 11] {
        let before = slot_of(&cache, inum);
        let got = iget(&mut cache, inum);
        let how = if before.is_some() { "hit" } else { "filled" };
        println!("iget({inum:>2}) -> {got:?}   {how}");
    }
    show(&cache);
    println!("the second iget(3) came back Some(0): same slot, nothing written");

    println!("\n== the search, as an adapter ==");
    println!("cache.iter().position(|&s| s == Some(7))  = {:?}",
             cache.iter().position(|&s| s == Some(7)));
    println!("cache.iter().position(|&s| s == Some(99)) = {:?}",
             cache.iter().position(|&s| s == Some(99)));
    println!("`position` is `enumerate` + `find` + `map(|(i, _)| i)`, and it stops");
    println!("at the first match. It is the for-loop with an early return.");

    println!("\n== three spellings of the same search ==");
    let a = cache.iter().position(|s| s.is_none());
    let b = cache.iter().enumerate().find(|(_, s)| s.is_none()).map(|(i, _)| i);
    // clippy would rewrite this into spelling one; the point is to see all three.
    #[allow(clippy::needless_range_loop)]
    let c = {
        let mut found = None;
        for i in 0..cache.len() {
            if cache[i].is_none() {
                found = Some(i);
                break;
            }
        }
        found
    };
    println!("position   -> {a:?}");
    println!("enumerate  -> {b:?}");
    println!("for loop   -> {c:?}");
    println!("same machine code, near enough. Pick the one that reads as the question.");

    println!("\n== the cache fills, and then it is full ==");
    for inum in [20, 21, 22, 99] {
        match iget(&mut cache, inum) {
            Some(i) => println!("iget({inum:>2}) -> Some({i})"),
            None => println!("iget({inum:>2}) -> None      xv6 panics here; we returned a value"),
        }
    }
    show(&cache);
    println!("a full cache is not a bug in iget. It is an answer, and the type");
    println!("makes the caller read it before using the slot number.");

    println!("\n== why the index matters ==");
    println!("in the file-descriptor table the slot index IS the descriptor:");
    println!("the first open() returns 3 because slots 0, 1, 2 are taken.");
    println!("close(1); open(..) returns 1 -- and that is how `cmd > file` works.");
}

fn show(cache: &[Option<u32>]) {
    let cells: Vec<String> = cache
        .iter()
        .map(|s| match s {
            Some(inum) => format!("{inum:>3}"),
            None => "  -".to_string(),
        })
        .collect();
    println!("  [{}]", cells.join(" |"));
}
