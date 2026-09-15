// ╔══════════════════════════════════════════════════════════════════════╗
// ║  START HERE. A whole program: argv in, bytes out, a number back.     ║
// ║      cargo run --bin basic_command -- a b c; echo $?                 ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Before basename, the ceremony on its own. This is a real command: it
// prints each argument on its own line, numbered, and exits 0. Every line of
// `run` would be byte-identical on rv6.

use echo_example::{write_all, write_usize, Args, STDOUT};

// ── 1. the whole interface of a Unix program is this one line ────────────
//
// `main!` builds the Args from the real argv, calls `run`, and exits with
// what it returns. There is no `fn main` in this file.
echo_example::main!(run);

fn run(args: Args) -> i32 {
    // ── 2. args.len() is argc; args.get(0) is the name you were called by ─
    let _ = write_all(STDOUT, b"argc = ");
    let _ = write_usize(STDOUT, args.len(), 0);
    let _ = write_all(STDOUT, b", argv[0] = ");
    let _ = write_all(STDOUT, args.prog());
    let _ = write_all(STDOUT, b"\n");

    // ── 3. the loop: 1..len, and get(i) is an Option<&[u8]> ─────────────
    //
    // Inside the range the index is always valid, so `.unwrap()` is honest.
    for i in 1..args.len() {
        let arg = args.get(i).unwrap();

        // ── 4. bytes out, and `let _ =` on each Result ───────────────────
        //
        // There is nothing a command can do if the console is gone. Say so.
        let _ = write_usize(STDOUT, i, 2);
        let _ = write_all(STDOUT, b": ");
        let _ = write_all(STDOUT, arg);
        let _ = write_all(STDOUT, b"\n");
    }

    // ---------------------------------------------------------------------
    // UNCOMMENT ME. rustc says:
    //
    //   error[E0308]: mismatched types
    //      |     let _ = write_all(STDOUT, ": ");
    //      |             ---------         ^^^^ expected `&[u8]`, found `&str`
    //      |             |
    //      |             arguments to this function are incorrect
    //
    // A string literal is a &str: bytes plus a UTF-8 promise. write_all
    // takes &[u8]: bytes, no promise. `b": "` is the two bytes, and it is
    // the only thing that type-checks -- the kernel boundary is bytes.
    //
    // let _ = write_all(STDOUT, ": ");
    // ---------------------------------------------------------------------

    // ── 5. the number: the shell reads it as $? ──────────────────────────
    0
}
