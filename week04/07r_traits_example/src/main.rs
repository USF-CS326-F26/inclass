// The live demo. Run it, then read it:
//
//     cargo run
//
// Four sections, matching the three sections of `src/lib.rs` and of the
// exercise: a sink trait with two implementers, an allocator trait with two
// policies, and one generic function that drives every combination. The last
// section is an error you can uncomment on screen -- that is the part worth
// doing slowly.

use std::mem::size_of;
use traits_example::{
    put_banner, put_map, trace, Allocator, FirstFit, NextFit, Sink, TallySink, VecSink,
};

fn main() {
    println!("== 1. one trait, two sinks, and a default nobody wrote twice ==");
    let mut v = VecSink::new();
    let mut t = TallySink::new();
    v.put_line(b"rv6 boot");
    t.put_line(b"rv6 boot");
    println!("VecSink.put_line(b\"rv6 boot\")   -> bytes = {:?}",
             String::from_utf8_lossy(&v.bytes));
    println!("TallySink.put_line(b\"rv6 boot\") -> bytes = {}, puts = {}", t.bytes, t.puts);
    println!("  puts = 2: put_line is two puts. It has a body in the trait and no");
    println!("  impl block mentions it. Each type wrote `put` and nothing else.");

    println!("\n== 2. dyn and impl: two spellings, two costs ==");
    let free = [true, false, true, true, false, true, true, true];
    put_banner(&mut v); // &mut dyn Sink
    put_map(&mut v, &free); // &mut impl Sink
    put_banner(&mut t);
    put_map(&mut t, &free);
    print!("{}", String::from_utf8_lossy(&v.bytes));
    println!("  TallySink saw {} bytes in {} puts -- same routines, other destination",
             t.bytes, t.puts);
    println!("  size_of::<&VecSink>()  = {}   thin: a pointer", size_of::<&VecSink>());
    println!("  size_of::<&dyn Sink>() = {}  fat: a pointer + a vtable pointer",
             size_of::<&dyn Sink>());
    println!("  put_banner is one function that looks `put` up at run time.");
    println!("  put_map is one function PER SINK TYPE, resolved when compiled.");

    println!("\n== 3. two policies, same table, different answers ==");
    println!("  frames 1 and 4 start out used. Five takes each; after the third,");
    println!("  a process exits and frame 0 is free again:");
    let mut ff_free = free;
    let mut nf_free = free;
    let mut ff = FirstFit;
    let mut nf = NextFit::new();
    println!("  {:<10} {:<14} {:<10} table after", "FirstFit", "table after", "NextFit");
    for turn in 0..5 {
        if turn == 3 {
            ff_free[0] = true;
            nf_free[0] = true;
            println!("  {:<10} {:<14} {:<10} {}   <- frame 0 freed", "", map(&ff_free), "", map(&nf_free));
        }
        let a = ff.take(&mut ff_free);
        let b = nf.take(&mut nf_free);
        println!("  {:<10} {:<14} {:<10} {}", fmt(a), map(&ff_free), fmt(b), map(&nf_free));
    }
    println!("  first-fit went back for the hole; next-fit carried on from its cursor.");
    println!("  same trait, same `take`, same table -- only `pick` differed.");
    println!("  `take` marks the frame; it was written once, in the trait, against");
    println!("  `pick` alone. Neither policy has a line about marking.");
    println!("  size_of::<FirstFit>() = {}   it remembers nothing, and its type says so",
             size_of::<FirstFit>());
    println!("  size_of::<NextFit>()  = {}   the cursor", size_of::<NextFit>());

    println!("\n== 4. one generic function, every combination ==");
    let combos = ["FirstFit + VecSink", "FirstFit + TallySink", "NextFit + VecSink", "NextFit + TallySink"];
    let mut log = VecSink::new();
    let mut tally = TallySink::new();
    trace(&mut FirstFit, &mut free.clone(), 3, &mut log);
    trace(&mut FirstFit, &mut free.clone(), 3, &mut tally);
    trace(&mut NextFit { next: 6 }, &mut free.clone(), 4, &mut log);
    trace(&mut NextFit { next: 6 }, &mut free.clone(), 4, &mut tally);
    println!("  trace::<A, S> was called as: {}", combos.join(", "));
    println!("  VecSink got:   {:?}", String::from_utf8_lossy(&log.bytes));
    println!("  TallySink got: {} bytes in {} puts   (equal: {})",
             tally.bytes, tally.puts, tally.bytes == log.bytes.len());
    println!("  four copies of trace exist in this binary, one per combination,");
    println!("  each with its `take` and `put` calls resolved and inlined.");

    println!("\n== 5. the error worth reading out loud ==");
    println!("uncomment the block at the bottom of src/main.rs and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME in class. rustc says:
    //
    //   error[E0599]: no method named `put` found for mutable reference
    //                 `&mut S` in the current scope
    //      |     fn put_twice<S>(sink: &mut S) {
    //      |                  - method `put` not found for this type parameter
    //      |         sink.put(b"x");
    //      |              ^^^ method not found in `&mut S`
    //      |
    //      = help: items from traits can only be used if the type parameter
    //              is bounded by the trait
    //   help: the following trait defines an item `put`, perhaps you need to
    //         restrict type parameter `S` with it:
    //      |     fn put_twice<S: Sink>(sink: &mut S) {
    //      |                  ++++++
    //
    // Inside a generic function the body may assume exactly what the bound
    // says -- and there is no bound. This is not a missing `use`. rustc's
    // own help text is the fix, character for character.
    //
    // fn put_twice<S>(sink: &mut S) {
    //     sink.put(b"x");
    //     sink.put(b"x");
    // }
    // put_twice(&mut log);
    // ---------------------------------------------------------------------
}

fn fmt(frame: Option<usize>) -> String {
    match frame {
        Some(i) => format!("Some({i})"),
        None => "None".to_string(),
    }
}

fn map(free: &[bool]) -> String {
    free.iter().map(|&f| if f { '.' } else { '#' }).collect()
}
