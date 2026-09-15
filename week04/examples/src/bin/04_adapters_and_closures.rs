//! 04 — Adapters are a pipeline; every stage has a type.
//!
//! The global file table keeps a reference count per open file. Questions the
//! kernel asks it -- how many are live, which slots, the total -- are all one
//! walk with a different question at each step. Iterator ADAPTERS let you
//! write the walk as a chain, and the thing to watch is the ITEM TYPE
//! changing at each link:
//!
//!     refs.iter()        &Option<u16>     a reference into the table
//!         .flatten()     &u16             an Option is an iterator of 0 or 1 items
//!         .copied()      u16              dereference each one
//!         .map(f)        whatever f returns
//!         .collect()     Vec<_>           the ONE adapter that allocates
//!
//! The closures in between capture their environment, which is what lets a
//! walk refer to a `page_size` or a counter declared outside it.
//!
//! Run:  cargo run --bin 04_adapters_and_closures

const NFILE: usize = 8;

fn main() {
    // How many descriptors point at each open file; None = slot unused.
    let refs: [Option<u16>; NFILE] = [Some(3), None, Some(1), Some(2), None, None, Some(1), None];
    println!("refs = {refs:?}");

    println!("\n== filter + count ==");
    let live = refs.iter().filter(|r| r.is_some()).count();
    println!("refs.iter().filter(|r| r.is_some()).count()  = {live}");

    println!("\n== flatten: an Option is an iterator of zero or one ==");
    let inner: Vec<&u16> = refs.iter().flatten().collect();
    println!("refs.iter().flatten().collect::<Vec<&u16>>() = {inner:?}");
    println!("None contributed nothing; each Some contributed its &u16");

    println!("\n== copied: &u16 becomes u16 ==");
    let counts: Vec<u16> = refs.iter().flatten().copied().collect();
    println!("refs.iter().flatten().copied().collect()     = {counts:?}");
    println!("the Vec now OWNS numbers instead of borrowing the table");

    println!("\n== map with a captured value ==");
    let page = 4096usize;
    let bytes: Vec<usize> = refs.iter().flatten().map(|&r| r as usize * page).collect();
    println!("map(|&r| r as usize * page)                   = {bytes:?}");
    println!("the closure reached `page` from the enclosing scope -- that is a capture");

    println!("\n== collect needs to know the container ==");
    let a: Vec<u16> = refs.iter().flatten().copied().collect();
    let b = refs.iter().flatten().copied().collect::<Vec<u16>>();
    println!("let a: Vec<u16> = ...collect()      -> {a:?}");
    println!("...collect::<Vec<u16>>()            -> {b:?}");
    println!("with neither annotation: error[E0282]: type annotations needed");
    println!("`collect` can build a Vec, a String, a HashMap -- it has to be told");

    println!("\n== sum ==");
    println!("refs.iter().flatten().sum::<u16>()   = {}", refs.iter().flatten().sum::<u16>());

    println!("\n== enumerate + filter_map: the indices of the live slots ==");
    let slots: Vec<usize> = refs
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.map(|_| i))
        .collect();
    println!("enumerate().filter_map(|(i, r)| r.map(|_| i)) = {slots:?}");
    println!("`r.map(|_| i)` turns Some(count) into Some(index) and leaves None alone");

    println!("\n== lazy: the closure runs only as far as needed ==");
    let mut looked = 0;
    let pair = refs
        .iter()
        .map(|r| {
            looked += 1;
            *r
        })
        .find(|r| *r == Some(2));
    println!("first slot with refcount 2: {:?}", pair.flatten());
    println!("the map closure ran {looked} times, not {NFILE}: find stopped it");

    println!("\n== the same thing as a loop ==");
    let mut by_hand = Vec::new();
    #[allow(clippy::manual_flatten)] // the loop IS the point here
    for r in refs.iter() {
        if let Some(n) = r {
            by_hand.push(*n);
        }
    }
    println!("for / if let / push   -> {by_hand:?}");
    println!("flatten / copied / collect -> {counts:?}");
    println!("every chain is a loop you could have written. The chain has no");
    println!("mutable temporaries and no index to get wrong -- and `collect` is");
    println!("the one link that allocates, so it is the one the kernel leaves out.");
}
