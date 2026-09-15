// ╔══════════════════════════════════════════════════════════════════════╗
// ║  START HERE. One Option, one Result, and one `?`.                    ║
// ║      cargo run --bin basic_result                                    ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Before the device registry, the concept on its own: a table of free page
// frames, a question that may have no answer, a request that may be refused,
// and the operator that chains two requests.

fn main() {
    let mut free = [false, true, false, true];

    // ── 1. Option: a question whose honest answer may be "nothing" ───────
    println!("1. first_free(&free)        -> {:?}", first_free(&free));
    println!("   first_free(&[false; 2])  -> {:?}   not an error", first_free(&[false; 2]));

    // ── 2. Result: a request, and a reason when it is refused ────────────
    //
    // `.ok_or` is the bridge: Some -> Ok, None -> Err(the one you chose).
    println!("2. alloc(&mut free)         -> {:?}", alloc(&mut free));
    println!("   alloc(&mut free)         -> {:?}", alloc(&mut free));
    println!("   alloc(&mut free)         -> {:?}", alloc(&mut free));

    // ── 3. opening each one with match ───────────────────────────────────
    match first_free(&free) {
        Some(i) => println!("3. match Option: Some({i})"),
        None => println!("3. match Option: None -- every frame is used"),
    }
    free[2] = true;
    match alloc(&mut free) {
        Ok(i) => println!("   match Result: Ok({i})"),
        Err(AllocError::OutOfFrames) => println!("   match Result: Err(OutOfFrames)"),
    }

    // ── 4. `?` chains two requests; the first Err returns immediately ────
    let mut fresh = [true, true, true];
    println!("4. alloc_two(&mut fresh)    -> {:?}", alloc_two(&mut fresh));
    println!("   alloc_two(&mut fresh)    -> {:?}   the second ? returned early",
             alloc_two(&mut fresh));

    // ── 5. the error worth reading out loud ──────────────────────────────
    println!("5. uncomment the last block and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME. rustc says:
    //
    //   error[E0308]: mismatched types
    //      |     fn alloc_wrong(free: &[bool]) -> Result<usize, AllocError> {
    //      |                                      ------------------------
    //      |                                      expected `Result<usize, AllocError>`
    //      |                                      because of return type
    //      |         first_free(free)
    //      |         ^^^^^^^^^^^^^^^^ expected `Result<usize, AllocError>`,
    //      |                          found `Option<usize>`
    //      |
    //   help: try wrapping the expression in `Ok`  <- and that would be WRONG
    //
    // An Option is not a Result. Nobody said what None MEANS here, and the
    // compiler will not guess. `.ok_or(AllocError::OutOfFrames)` is the
    // decision -- one word, and it is yours to make.
    //
    // fn alloc_wrong(free: &[bool]) -> Result<usize, AllocError> {
    //     first_free(free)
    // }
    // ---------------------------------------------------------------------
}

/// The lowest free frame, if any. A question.
fn first_free(free: &[bool]) -> Option<usize> {
    free.iter().position(|&f| f)
}

/// Every way `alloc` can fail. One variant, today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllocError {
    OutOfFrames,
}

/// Take a frame. A request; it can be refused, and it says why.
fn alloc(free: &mut [bool]) -> Result<usize, AllocError> {
    let i = first_free(free).ok_or(AllocError::OutOfFrames)?;
    free[i] = false;
    Ok(i)
}

/// Two frames, or the first failure. Each `?` is a possible early return.
fn alloc_two(free: &mut [bool]) -> Result<(usize, usize), AllocError> {
    let a = alloc(free)?;
    let b = alloc(free)?;
    Ok((a, b))
}
