//! 10 — Fenceposts: count the starts, and count both ends.
//!
//! A line matches if the needle occurs in it as one contiguous run of bytes:
//! `ear` is in `heart`, but not in `era`. Sliding a needle along a line is a
//! fencepost count, and both ends are posts:
//!
//!     line     h e a r t        5 bytes
//!     starts   0 1 2 3          a 2-byte needle: 4 starts, the last is 5 - 2
//!
//! Rust has two range forms, and only one includes its end:
//!
//!     1..5     1, 2, 3, 4       stops BEFORE 5
//!     1..=5    1, 2, 3, 4, 5    up to and including 5
//!
//! Lengths are `usize`, which has no negative numbers: `2 - 5` panics in a
//! debug build, and `2usize.checked_sub(5)` answers None instead.
//!
//! The search itself is 13c's to write, so this program asks std's
//! `str::contains` and `str::find`, and only counts where a needle COULD start.
//!
//! Run:  cargo run --bin 10_fenceposts

use std::hint::black_box;

fn main() {
    println!("== two ranges ==");
    let a: Vec<u32> = (1..5).collect();
    let b: Vec<u32> = (1..=5).collect();
    println!("(1..5)  = {a:?}      stops before 5");
    println!("(1..=5) = {b:?}   up to and including 5");

    println!("\n== where a 2-byte needle can start in a 5-byte line ==");
    let line = "heart";
    let k = 2;
    let letters: Vec<String> = line.chars().map(String::from).collect();
    println!("  {}", letters.join(" "));
    for start in 0..=line.len() - k {
        let carets = format!("{}{}", "  ".repeat(start), "^ ".repeat(k));
        println!("  {:<11} start {start}", carets.trim_end());
    }
    println!("0..=(5 - 2) is {} starts. 0..(5 - 2) is {}, and never tries start 3.",
             (0..=line.len() - k).count(), (0..line.len() - k).count());

    println!("\n== contiguous: ear is in heart, not in era ==");
    println!("\"heart\".contains(\"ear\") = {}", "heart".contains("ear"));
    println!("\"heart\".find(\"ear\")     = {:?}   it starts at 1", "heart".find("ear"));
    println!("\"era\".contains(\"ear\")   = {}   all three letters, not in one run",
             "era".contains("ear"));

    println!("\n== the last start is the one people skip ==");
    let (hay, needle) = ("syslog", "log");
    let last = hay.len() - needle.len();
    println!("\"syslog\".find(\"log\") = {:?}", hay.find(needle));
    println!("last start = {} - {} = {last}", hay.len(), needle.len());
    println!("(0..{last}).contains(&{last})  = {}   so a scan over 0..{last} misses `log`",
             (0..last).contains(&last));
    println!("(0..={last}).contains(&{last}) = {}", (0..=last).contains(&last));
    println!("`logfile` matches either way. Only lines that END in the needle go missing.");

    println!("\n== two sizes to decide before counting ==");
    println!("\"heart\".contains(\"\")   = {}    an empty needle is found in every line",
             "heart".contains(""));
    println!("\"\".contains(\"\")        = {}    even in an empty one", "".contains(""));
    println!("\"he\".contains(\"heart\") = {}   longer than the line: no start at all",
             "he".contains("heart"));
    println!("its last start would be 2 - 5, and a usize cannot hold -3:");
    println!("  5usize.checked_sub(2) = {:?}", 5usize.checked_sub(2));
    println!("  2usize.checked_sub(5) = {:?}", 2usize.checked_sub(5));
    println!("  2usize.saturating_sub(5) = {}   not a fix: 0..=0 would still try start 0",
             2usize.saturating_sub(5));

    println!("\n== what 2 - 5 does on usize, in a debug build ==");
    std::panic::set_hook(Box::new(|info| {
        println!("  panicked: {}", info.payload_as_str().unwrap_or("?"));
    }));
    let (len, needle_len) = (black_box(2usize), black_box(5usize));
    let r = std::panic::catch_unwind(|| len - needle_len);
    let _ = std::panic::take_hook();
    println!("catch_unwind(|| 2 - 5) = {}", if r.is_err() { "Err: it panicked" } else { "Ok" });
    println!("a release build wraps to a number near 2^64 instead, and the first index");
    println!("that uses it panics anyway. On rv6 either one prints the single word panic.");
}
