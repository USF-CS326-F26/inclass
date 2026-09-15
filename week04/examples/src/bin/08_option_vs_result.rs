//! 08 — Absence is a fact. Failure is a decision.
//!
//! The page allocator keeps one bit per frame. Two questions look alike and
//! are not:
//!
//!     first_free(&free)  -> Option<usize>              is there a free frame?
//!     alloc_frame(&mut free) -> Result<usize, AllocError>   give me one.
//!
//! A search that finds nothing has not FAILED; it has answered. `Option` is the
//! type for that. `alloc_frame` promised a frame and could not deliver, and
//! must say why. `Result` is the type for that. The line where one becomes the
//! other is
//!
//!     first_free(free).ok_or(AllocError::OutOfFrames)
//!
//! and that line is where the POLICY lives: "no free frame" means "allocation
//! failed" HERE, at this call, and nowhere else.
//!
//! This program builds with ONE WARNING, on purpose. Read it.
//!
//! Run:  cargo run --bin 08_option_vs_result

/// Every way frame allocation can go wrong. One variant per way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    /// Every frame is in use.
    OutOfFrames,
    /// The caller tried to free a frame that was already free: a double free.
    AlreadyFree,
}

/// The lowest free frame, if there is one. A question, not a request.
fn first_free(free: &[bool]) -> Option<usize> {
    free.iter().position(|&f| f)
}

/// Take a frame. This is a request, and it can be refused.
fn alloc_frame(free: &mut [bool]) -> Result<usize, AllocError> {
    let i = first_free(free).ok_or(AllocError::OutOfFrames)?;
    free[i] = false;
    Ok(i)
}

/// Give a frame back. `Ok(())`: success with nothing to say.
fn free_frame(free: &mut [bool], i: usize) -> Result<(), AllocError> {
    if free[i] {
        return Err(AllocError::AlreadyFree);
    }
    free[i] = true;
    Ok(())
}

fn main() {
    let mut free = [false, false, true, true];
    println!("free = {free:?}   (true = available)");

    println!("\n== Option: the honest answer may be `nothing` ==");
    println!("first_free(&free) = {:?}", first_free(&free));
    println!("first_free(&[false, false]) = {:?}   not an error. There are none.",
             first_free(&[false, false]));

    println!("\n== Result: a request, and a reason when it is refused ==");
    for _ in 0..3 {
        let r = alloc_frame(&mut free);
        println!("alloc_frame(..) = {r:?}   free = {free:?}");
    }
    println!("the third call did not get a frame, and the value says why.");

    println!("\n== the line between them ==");
    println!("    first_free(free).ok_or(AllocError::OutOfFrames)");
    println!("is exactly:");
    println!("    match first_free(free) {{");
    println!("        Some(i) => Ok(i),");
    println!("        None    => Err(AllocError::OutOfFrames),");
    println!("    }}");
    println!("Some -> Ok, None -> Err(the one you chose). That choice IS the policy.");

    println!("\n== Ok(()) : success with nothing to report ==");
    println!("free_frame(&mut free, 2) = {:?}", free_frame(&mut free, 2));
    println!("free_frame(&mut free, 2) = {:?}   the double free, caught",
             free_frame(&mut free, 2));
    println!("free = {free:?}");

    println!("\n== opening a Result ==");
    match alloc_frame(&mut free) {
        Ok(i) => println!("match: got frame {i}"),
        Err(AllocError::OutOfFrames) => println!("match: out of frames"),
        Err(AllocError::AlreadyFree) => println!("match: cannot happen here, but the arm must exist"),
    }
    if let Ok(i) = alloc_frame(&mut free) {
        println!("if let: got frame {i}");
    } else {
        println!("if let: refused");
    }
    println!("`match` lists every variant. Add one to AllocError and this match");
    println!("stops compiling until you decide what it means here.");

    println!("\n== the warning, on purpose ==");
    println!("the next line calls alloc_frame and drops the Result on the floor.");
    println!("rustc noticed when this program was built. Scroll up: it said");
    println!("    warning: unused `Result` that must be used");
    println!("that is #[must_use] on Result, and it is the -1 nobody checked in C.");
    // THIS LINE WARNS ON PURPOSE. Leave it. The warning is the demo.
    alloc_frame(&mut free);
    println!("the honest way to discard one, when there is truly nothing to do:");
    println!("    let _ = alloc_frame(&mut free);");
    let _ = alloc_frame(&mut free);
    println!("`let _ =` says `I looked, and I chose not to care` -- in the source.");
}
