//! 05 — One bad file, one exit status: a later success does not erase it.
//!
//!     $ cat gone.txt todo.txt
//!     cat: gone.txt: No such file or directory      <- fd 2, the complaint
//!     buy milk                                      <- fd 1, the data
//!     $ echo $?
//!     1                                             <- the WHOLE run
//!
//! The `i32` a command returns is its exit status: 0 for success, anything
//! else for failure. This program replays that transcript against a
//! one-file "filesystem" in memory, three ways. Only one of them prints
//! `buy milk` AND ends with 1.
//!
//! None of the three is really `cat`. Each file arrives in one piece, with
//! no read loop (04 has the loop). The one idea is the status variable, and
//! where it is allowed to change.
//!
//! Run:  cargo run --bin 05_exit_status

/// What the program wrote to fd 1 and fd 2.
#[derive(Default)]
struct Screen {
    out: Vec<String>,
    err: Vec<String>,
}

// fold: ---- the "filesystem": one file, and an open that can refuse ------
/// `open` + `read` in one call, for this program only: the whole file, or
/// the news that there is none.
fn open_and_read(path: &str) -> Result<&'static str, &'static str> {
    const FILES: [(&str, &str); 1] = [("todo.txt", "buy milk")];
    FILES.iter()
        .find(|(name, _)| *name == path)
        .map(|(_, text)| *text)
        .ok_or("No such file or directory")
}

/// Stops at the first failure.
fn gives_up(paths: &[&str], s: &mut Screen) -> i32 {
    for p in paths {
        match open_and_read(p) {
            Ok(text) => s.out.push(text.to_string()),
            Err(why) => {
                s.err.push(format!("cat: {p}: {why}"));
                return 1;
            }
        }
    }
    0
}

/// Every file gets its turn, and the LAST one decides the status.
fn forgets(paths: &[&str], s: &mut Screen) -> i32 {
    let mut status = 0;
    for p in paths {
        match open_and_read(p) {
            Ok(text) => {
                s.out.push(text.to_string());
                status = 0; // WRONG: this erases the failure before it
            }
            Err(why) => {
                s.err.push(format!("cat: {p}: {why}"));
                status = 1;
            }
        }
    }
    status
}

/// Every file gets its turn, and any failure sticks.
fn remembers(paths: &[&str], s: &mut Screen) -> i32 {
    let mut status = 0;
    for p in paths {
        match open_and_read(p) {
            Ok(text) => s.out.push(text.to_string()),
            Err(why) => {
                s.err.push(format!("cat: {p}: {why}"));
                status = 1; // only ever set, never cleared
            }
        }
    }
    status
}

fn show(s: &Screen, status: i32) {
    for line in &s.err {
        println!("  fd 2 | {line}");
    }
    for line in &s.out {
        println!("  fd 1 | {line}");
    }
    println!("  $? = {status}");
}

fn main() {
    let paths = ["gone.txt", "todo.txt"];

    println!("== gives up at the first failure ==");
    let mut s = Screen::default();
    let status = gives_up(&paths, &mut s);
    show(&s, status);
    println!("the status is right, and todo.txt was never printed.");

    println!("\n== lets the last file decide ==");
    let mut s = Screen::default();
    let status = forgets(&paths, &mut s);
    show(&s, status);
    println!("both lines are right, and the 0 is a lie: a file was missing.");

    println!("\n== remembers any failure ==");
    let mut s = Screen::default();
    let status = remembers(&paths, &mut s);
    show(&s, status);
    println!("the transcript, exactly. The 1 describes the run, not the last file.");

    println!("\n== why the complaint goes to fd 2 ==");
    println!("$ cat gone.txt todo.txt > both");
    println!("  the screen gets fd 2: {:?}", s.err);
    println!("  the file `both` gets fd 1: {:?}", s.out);
    println!("the redirect moved fd 1 only, so `both` holds data and nothing else.");

    println!("\n== the shell reads the status ==");
    for (status, meaning) in [(0, "some line matched"), (1, "nothing matched"), (2, "a file would not open")] {
        let runs = if status == 0 { "runs" } else { "is skipped" };
        println!("  grep -q ERROR log -> {status} ({meaning}): `&& echo read the log` {runs}");
    }
    println!("grep's 1 is the answer `no`, not a failure. A grep that always said 0");
    println!("would make every `if grep ..` take the same branch.");

    println!("\n== one byte ==");
    for count in [1, 3, 255, 256, 257] {
        println!("  exit({count:>3}) -> $? = {}", count as u8);
    }
    println!("the status reaches the shell as one byte. Return the number of failed files");
    println!("and the 256th failure reads as success. cat says 1 for any failure.");
}
