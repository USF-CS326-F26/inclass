// ╔══════════════════════════════════════════════════════════════════════╗
// ║  START HERE. One enum, one match, and nothing else.                  ║
// ║      cargo run --bin basic_enum                                      ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Before the buffer cache, the concept on its own: a set of named cases, a
// match that covers all of them, and an Option for the question that has no
// answer.

fn main() {
    // ── 1. an enum is a closed set of values ─────────────────────────────
    //
    // A `Signal` is one of exactly four things. There is no Signal equal to
    // 47, because the type has four values and 47 is not one of them.
    let signals = [
        Signal::Interrupt,
        Signal::Kill,
        Signal::Stop { seconds: 5 },
        Signal::User(2),
    ];
    println!("1. the four values of Signal:");
    for s in signals {
        println!("     {s:?}");
    }

    // ── 2. match reads which variant you have ────────────────────────────
    //
    // Each arm is a pattern, `=>`, and the value it produces. `match` is an
    // EXPRESSION, which is why it can sit on the right of a `let`.
    println!("2. what each one does to a process:");
    for s in signals {
        let effect = match s {
            Signal::Interrupt => String::from("ask it to stop"),
            Signal::Kill => String::from("stop it now, no cleanup"),
            Signal::Stop { seconds } => format!("suspend it for {seconds}s"),
            Signal::User(n) => format!("deliver user signal {n}"),
        };
        println!("     {:<24} {effect}", format!("{s:?}"));
    }

    // ── 3. the pattern features ──────────────────────────────────────────
    println!("3. the four things a pattern can do:");
    for s in signals {
        let note = match s {
            Signal::Interrupt | Signal::Kill => "`|`  -- one arm, two variants",
            Signal::Stop { .. } => "`{ .. }` -- has fields, do not care",
            Signal::User(_) => "`_` inside -- has a field, ignoring it",
        };
        println!("     {:<24} {note}", format!("{s:?}"));
    }

    // ── 4. Option, for a question with no answer ─────────────────────────
    //
    // Not every signal can be caught by the program it is sent to. The return
    // type says so, and the caller cannot forget to look.
    println!("4. which handler runs, if any:");
    for s in signals {
        match handler_for(s) {
            Some(name) => println!("     {:<24} -> {name}", format!("{s:?}")),
            None => println!("     {:<24} -> None: cannot be caught", format!("{s:?}")),
        }
    }

    // ── 5. the error worth reading out loud ──────────────────────────────
    println!("5. uncomment the last block and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME. rustc says:
    //
    //   error[E0004]: non-exhaustive patterns: `Signal::User(_)` not covered
    //      |     match s {
    //      |           ^ pattern `Signal::User(_)` not covered
    //      |
    //   note: `Signal` defined here
    //   help: ensure the match is exhaustive by adding a match arm with the
    //         missing pattern
    //
    // That is the whole argument for enums in one message. The compiler is
    // not being fussy about syntax — it is telling you about a case you have
    // not decided. Replace the missing arm with `_ => false` and the error
    // goes away, and so does every future warning about every variant you
    // have not thought of yet.
    //
    // fn is_fatal(s: Signal) -> bool {
    //     match s {
    //         Signal::Kill => true,
    //         Signal::Interrupt => false,
    //         Signal::Stop { .. } => false,
    //     }
    // }
    // println!("{}", is_fatal(Signal::Kill));
    // ---------------------------------------------------------------------
}

/// Four cases, and nothing else. Two of them carry data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Named fields, built like a struct: `Signal::Stop { seconds: 5 }`.
    Stop { seconds: u32 },
    /// A tuple field, built like a call: `Signal::User(2)`.
    User(u32),
    Interrupt,
    Kill,
}

/// The name of the handler a program may install, if this signal can be
/// caught at all.
///
/// One arm per variant and no `_`: add a fifth signal and this stops
/// compiling until somebody says whether it can be caught.
fn handler_for(s: Signal) -> Option<&'static str> {
    match s {
        Signal::Interrupt => Some("sigint_handler"),
        Signal::User(_) => Some("sigusr_handler"),
        Signal::Stop { .. } => None,
        Signal::Kill => None,
    }
}
