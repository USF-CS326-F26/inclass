// The live demo. Run it, then read it:
//
//     cargo run
//
// Four sections, matching `src/lib.rs` and the exercise: absence beside
// failure, the `?` chain and which step each name fails at, the boundary
// where a Result becomes a number, and the ways to open a Result that do and
// do not kill the machine. The last section is an error you can uncomment on
// screen -- that is the part worth doing slowly.

use errors_example::{DevError, Registry, EAGAIN, ENAMETOOLONG, ENODEV, ENOTTY};

fn main() {
    let mut reg = Registry::new();

    println!("== 1. absence is a fact; failure is a decision ==");
    println!("{:<12} {:<40} lookup()", "name", "find()");
    for name in ["console", "disk", "eightchr", "ninechars"] {
        println!("{:<12} {:<40} {:?}", name, format!("{:?}", reg.find(name)), reg.lookup(name));
    }
    println!("`find` says None twice, for two different reasons, and does not care.");
    println!("`lookup` says which -- because `.ok_or` chose NoSuchDevice for one,");
    println!("and an `if` before it chose NameTooLong for the other.");

    println!("\n== 2. the ? chain: each name stops at a different step ==");
    for name in ["console", "eightchr", "ninechars", "disk", "null", "zero"] {
        let r = reg.getc_by_name(name);
        let stopped = match r {
            Ok(_) => "passed both",
            Err(DevError::NoSuchDevice) | Err(DevError::NameTooLong) => "stopped at lookup(name)?",
            Err(DevError::NotATty) | Err(DevError::WouldBlock) => "stopped inside getc(dev)",
        };
        println!("getc_by_name({name:<10}) -> {r:<20}   {stopped}", r = format!("{r:?}"));
    }
    println!("getc_by_name is two lines. `?` is a `return Err(e)` in disguise, and");
    println!("it is allowed there only because the function returns a Result.");

    println!("\n  the same device, after the hardware delivers a byte:");
    reg.push_input("null", b"\n");
    println!("  push_input(\"null\", b\"\\n\"); getc_by_name(\"null\") -> {:?}",
             reg.getc_by_name("null"));
    println!("  WouldBlock was never a property of the device. It was a moment.");

    println!("\n== 3. the boundary: one match, and the only place errno is spelled ==");
    println!("{:<12} {:>10}   which is", "name", "sys_getc");
    for name in ["console", "zero", "eightchr", "ninechars", "disk", "null"] {
        reg = Registry::new(); // fresh, so null blocks again
        let rc = reg.sys_getc(name);
        let which = match rc {
            n if n >= 0 => format!("the byte {n:#04x}"),
            n if n == -ENODEV => "-ENODEV".to_string(),
            n if n == -ENAMETOOLONG => "-ENAMETOOLONG".to_string(),
            n if n == -ENOTTY => "-ENOTTY".to_string(),
            n if n == -EAGAIN => "-EAGAIN".to_string(),
            _ => "?".to_string(),
        };
        println!("{name:<12} {rc:>10}   {which}");
    }
    println!("zero returned 0 and that is success: the SIGN carries the meaning.");
    println!("Add a fifth variant to DevError and sys_getc stops compiling until");
    println!("it has a number. Nothing else in this file ever mentions 19.");

    println!("\n== 4. opening a Result: what panics, and what does not ==");
    let byte = reg.getc_by_name("console").expect("console has a line waiting");
    println!("  .expect(\"..\")   -> {byte}   honest: section 2 showed it is Ok");
    println!("  .unwrap_or(0)   -> {}", reg.getc_by_name("null").unwrap_or(0));
    if let Err(e) = reg.getc_by_name("disk") {
        println!("  if let Err(e)   -> {e:?}");
    }
    println!("  .is_err()       -> {}", reg.getc_by_name("eightchr").is_err());
    println!("  .unwrap() on Err(NoSuchDevice) would print");
    println!("      called `Result::unwrap()` on an `Err` value: NoSuchDevice");
    println!("  and stop. On your laptop: one red test. On rv6: the machine.");

    println!("\n== 5. the error worth reading out loud ==");
    println!("uncomment the block at the bottom of src/main.rs and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME in class. A match that forgot two variants. rustc says:
    //
    //   error[E0004]: non-exhaustive patterns: `Err(DevError::NotATty)` and
    //                 `Err(DevError::WouldBlock)` not covered
    //      |     match reg.getc_by_name("console") {
    //      |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^ patterns `Err(DevError::NotATty)`
    //      |                                       and `Err(DevError::WouldBlock)`
    //      |                                       not covered
    //      = note: `Result<u8, DevError>` defined here
    //
    // Last week's exhaustiveness check, on a Result. The compiler lists what
    // you forgot -- and `Err(_)` would switch that off forever.
    //
    // match reg.getc_by_name("console") {
    //     Ok(b) => println!("byte {b}"),
    //     Err(DevError::NoSuchDevice) => println!("no such device"),
    //     Err(DevError::NameTooLong) => println!("name too long"),
    // }
    // ---------------------------------------------------------------------
}
