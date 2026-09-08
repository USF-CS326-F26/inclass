// ╔══════════════════════════════════════════════════════════════════════╗
// ║  One Vec, three functions.                                           ║
// ║  The whole of "ownership transfers on a function call", and nothing  ║
// ║  else. Start here, before the PID pool in src/lib.rs.                 ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
//     cargo run --bin basic_move

/// MAKES a Vec and moves it OUT to whoever called.
///
/// The Vec is born here, but it does not die here: returning it hands
/// ownership to the caller, so nothing is freed at the closing brace.
fn make_scores() -> Vec<i32> {
    let scores = vec![90, 85, 100];
    scores // no semicolon: this moves `scores` out to the caller
}

/// TAKES a Vec and gives it back.
///
/// `scores: Vec<i32>` with no `&` means the caller's Vec is MOVED in here.
/// We own it for the length of this function, and the `-> Vec<i32>` is a
/// promise to hand ownership back when we are done.
fn add_score(mut scores: Vec<i32>, score: i32) -> Vec<i32> {
    scores.push(score); // `mut` on the parameter is what allows this
    scores
}

/// TAKES a Vec and keeps it.
///
/// This one moves the Vec in and never returns it. At the closing brace the
/// parameter goes out of scope, so the Vec is dropped and its heap buffer is
/// freed right there. The caller does not get it back — and the compiler will
/// not let the caller use it, either.
fn total(scores: Vec<i32>) -> i32 {
    let mut sum = 0;
    for score in &scores {
        sum += score;
    }
    sum
} // <-- `scores` is dropped HERE. This is where free() happens.

fn main() {
    // ── 1. a Vec moves OUT of a function ────────────────────────────────
    let scores = make_scores();
    println!("1. scores = {scores:?}");
    println!("   this binding is now the one owner of that heap buffer");

    // ── 2. a Vec moves IN and back OUT ──────────────────────────────────
    // The `scores` on the right is given away. The `scores` on the left is a
    // NEW binding holding what the function handed back. Reusing the name is
    // normal Rust; it is called shadowing.
    let scores = add_score(scores, 75);
    println!("\n2. after add_score: {scores:?}");
    println!("   moved in, changed, moved back -- one owner the whole time");

    // Bonus: the data never moved, only the ownership of it. The heap buffer
    // is at the same address it was before the round trip through add_score.
    println!("   buffer address: {:p}", scores.as_ptr());
    let scores = add_score(scores, 60);
    println!("   after another trip: {:p} (same buffer, new owner)", scores.as_ptr());

    // ── 3. a Vec moves IN and stays there ───────────────────────────────
    println!("\n3. scores has {} entries, about to call total()", scores.len());
    let sum = total(scores);
    println!("   total = {sum}");
    println!("   `scores` is GONE: total() owned it and dropped it at its `}}`");

    // ── 4. the error ────────────────────────────────────────────────────
    println!("\n4. uncomment the last line of src/bin/basic_move.rs");

    // UNCOMMENT ME. rustc says:
    //
    //   error[E0382]: borrow of moved value: `scores`
    //      |     let scores = add_score(scores, 60);
    //      |         ------ move occurs because `scores` has type `Vec<i32>`,
    //      |                which does not implement the `Copy` trait
    //      |     let sum = total(scores);
    //      |                     ------ value moved here
    //      |     println!("{scores:?}");
    //      |                ^^^^^^ value borrowed here after move
    //      |
    //      = note: consider changing this parameter type in function `total`
    //              to borrow instead if owning the value isn't necessary
    //
    // "moved" means "given away". The buffer really was freed inside total(),
    // so this line would be a use-after-free -- caught before the program ran.
    // And read that last note: rustc is describing 03r. `total` only needed to
    // READ the scores, so it should have asked for `&Vec<i32>` and never taken
    // ownership at all.
    //
    // println!("{scores:?}");
}
