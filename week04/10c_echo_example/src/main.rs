// The live demo. Run it, then read it:
//
//     cargo run                  # the narrated walkthrough
//     cargo run -- /usr/lib/ a/b # ...or run basename for real, then `echo $?`
//
// Four sections: the harness calling `run` directly, the same run on a
// console that takes three bytes at a time, what a bare `write` does there,
// and the real thing. The last section is an error you can uncomment on
// screen -- that is the part worth doing slowly.

use echo_example::{run, testing, write, Args, STDOUT};

fn main() {
    // If the user gave real arguments, be the command.
    let owned = echo_example::host_argv();
    if owned.len() > 1 {
        let refs: Vec<&[u8]> = owned.iter().map(|v| &v[..]).collect();
        let code = run(Args::from_slice(&refs));
        std::process::exit(code);
    }

    println!("== 1. the harness calls run() directly ==");
    for argv in [
        &["basename", "/usr/lib"][..],
        &["basename", "a/b", "c/d/", "e"],
        &["basename"],
    ] {
        let out = testing::run(argv, run);
        println!("testing::run({argv:?}, run)");
        println!("  stdout = {:?}", out.out());
        println!("  stderr = {:?}", out.err());
        println!("  code   = {}   writes = {}", out.code, out.writes);
    }
    println!("no process was spawned. `run` was called as a function, with fd 1");
    println!("and fd 2 pointed at two Vecs. The source under test is the source");
    println!("that ships -- there is no test-only path through it.");

    println!("\n== 2. the same run, on a console that takes three bytes at a time ==");
    let easy = testing::run(&["basename", "/usr/kernel.elf"], run);
    let hard = testing::run_short_writes(&["basename", "/usr/kernel.elf"], 3, run);
    println!("run(..)                     -> {:?}   writes = {}", easy.out(), easy.writes);
    println!("run_short_writes(.., 3, ..) -> {:?}   writes = {}", hard.out(), hard.writes);
    println!("same bytes out. The program did not change; the console did, and");
    println!("`write_all` made {} calls instead of {} to cope. That loop is",
             hard.writes, easy.writes);
    println!("    while !buf.is_empty() {{ let n = write(fd, buf)?; buf = &buf[n..]; }}");

    println!("\n== 3. what a bare write does there ==");
    let out = testing::run_short_writes(&["probe"], 3, |_| {
        let n = write(STDOUT, b"kernel.elf\n");
        println!("write(STDOUT, b\"kernel.elf\\n\") -> {n:?}");
        0
    });
    println!("stdout = {:?}   three bytes went, eight stayed, nothing complained",
             out.out());
    println!("a short write is not an error. It is a number, and the only warning");
    println!("you get is that number. ulib's host harness never short-writes, so a");
    println!("bare `write` in your command passes every test -- and truncates on rv6.");

    println!("\n== 4. the real thing ==");
    println!("    cargo run -- /usr/lib/ a/b; echo $?");
    println!("main() sees real arguments, calls the SAME run(), and exits with its");
    println!("return value. That is what ulib::main!(run) expands to on the host.");

    println!("\n== 5. the error worth reading out loud ==");
    println!("uncomment the block at the bottom of src/main.rs and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME in class. rustc says:
    //
    //   error[E0277]: the `?` operator can only be used in a function that
    //                 returns `Result` or `Option` (or another type that
    //                 implements `FromResidual`)
    //      |     fn run_wrong(args: Args) -> i32 {
    //      |     ------------------------------- this function should return
    //      |                                     `Result` or `Option` to accept `?`
    //      |         write_all(STDOUT, b"\n")?;
    //      |                                 ^ cannot use the `?` operator in a
    //      |                                   function that returns `i32`
    //
    // A command returns a number to the shell; there is no Err to return.
    // The Result from write_all stops HERE, and `let _ =` is how you say so.
    //
    // fn run_wrong(args: Args) -> i32 {
    //     let _ = args;
    //     echo_example::write_all(STDOUT, b"\n")?;
    //     0
    // }
    // ---------------------------------------------------------------------
}
