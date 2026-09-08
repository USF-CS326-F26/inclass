// The live demo. Run it, then read it:
//
//     cargo run
//
// The buffer cache lifecycle, walked one event at a time. The last section is
// an exhaustiveness error you can uncomment on screen — that is the part worth
// doing slowly, because it is the reason the type is an enum at all.

use enums_example::{cached_block, holder, next_state, step, BufEvent, BufState};

fn main() {
    println!("== a slot walks its whole life ==");

    // Each `step` applies one event. An illegal event leaves the state alone,
    // which is what the `None` from `next_state` buys us.
    let mut b = BufState::Free;
    show("start", b);

    for e in [
        BufEvent::Fill { block: 42 },
        BufEvent::Lock { by: 3 },
        BufEvent::Write,
        BufEvent::Flush,
        BufEvent::Evict,
    ] {
        b = step(b, e);
        show(&format!("{e:?}"), b);
    }

    println!("\n== every arrow the table leaves out is a move refused ==");

    // `next_state` answers `None`, and `step` turns that into "stay put".
    let dirty = BufState::Dirty { block: 42 };
    for e in [BufEvent::Evict, BufEvent::Lock { by: 1 }, BufEvent::Fill { block: 9 }] {
        println!("  Dirty + {:<20} -> {:?}", format!("{e:?}"), next_state(dirty, e));
    }
    println!("evicting a dirty buffer would lose the write, so the cache");
    println!("refuses — and `None` is a value the caller cannot ignore");

    println!("\n== the guard: only the holder may unlock ==");

    let held = BufState::Locked { block: 42, by: 3 };
    println!("  {:?}", held);
    println!("  Unlock {{ by: 3 }} -> {:<28} the holder",
             format!("{:?}", next_state(held, BufEvent::Unlock { by: 3 })));
    println!("  Unlock {{ by: 4 }} -> {:<28} somebody else",
             format!("{:?}", next_state(held, BufEvent::Unlock { by: 4 })));
    println!("The arm's PATTERN matches both times. On the second the GUARD");
    println!("fails, and matching CONTINUES with the next arm rather than");
    println!("leaving the match — so it falls through to `_ => None`.");
    println!("Test the condition inside the arm body instead and you lose");
    println!("that fall-through, and have to restate the failure by hand.");

    println!("\n== pulling the payload back out ==");

    for s in [
        BufState::Free,
        BufState::Clean { block: 7 },
        BufState::Dirty { block: 9 },
        BufState::Locked { block: 2, by: 5 },
    ] {
        println!("  {:<30} block {:<8} holder {}",
                 format!("{s:?}"),
                 format!("{:?}", cached_block(s)),
                 format!("{:?}", holder(s)));
    }
    println!("`cached_block` is a match: three variants carry a block.");
    println!("`holder` is an `if let`: only one variant carries a holder.");

    println!("\n== Option is how `there is no answer` is said out loud ==");
    println!("  next_state(..) -> Option<BufState>   None means illegal");
    println!("  cached_block(..) -> Option<u32>      None means nothing cached");
    println!("A C API returns -1, or 0, or leaves an out-parameter untouched,");
    println!("and hopes you check. Here `Option<u32>` and `u32` are different");
    println!("types, so the compiler makes you look.");

    println!("\n== the error worth reading out loud ==");
    println!("uncomment the block at the bottom of src/main.rs and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME in class. rustc says:
    //
    //   error[E0004]: non-exhaustive patterns: `BufState::Locked { .. }` not
    //                 covered
    //      |     match s {
    //      |           ^ pattern `BufState::Locked { .. }` not covered
    //      |
    //   note: `BufState` defined here
    //   help: ensure the match is exhaustive by adding a match arm with the
    //         missing pattern
    //
    // Read forwards, that is a nuisance. Read backwards, it is why the type
    // exists: add a fifth state in week 9 and the compiler lists every place
    // that now needs a decision, by file and line, before the kernel boots.
    // Write `_ => ...` instead and you get that for exactly zero of them.
    //
    // fn describe(s: BufState) -> &'static str {
    //     match s {
    //         BufState::Free => "empty",
    //         BufState::Clean { .. } => "cached",
    //         BufState::Dirty { .. } => "needs flushing",
    //     }
    // }
    // println!("{}", describe(BufState::Free));
    // ---------------------------------------------------------------------
}

/// One line per state, with what can be read out of it.
fn show(label: &str, s: BufState) {
    println!("  {:<20} -> {:<32} holds data: {}",
             label, format!("{s:?}"), s.holds_data());
}
