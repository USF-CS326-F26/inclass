// ╔══════════════════════════════════════════════════════════════════════╗
// ║  12c — wc — IN-CLASS EXAMPLE: the version rv6 cannot run             ║
// ║  Read ALL the input into a Vec, split it on whitespace into another  ║
// ║  Vec, and count the pieces. Plain std, host only.                    ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/12c_wc/solution/wc.rs`. Same definitions of
// line, word, and byte, same output format, and the same five tests at the
// bottom -- but none of the constraints:
//
//   exercise (rv6)                        here (std)
//   ------------------------------------  ------------------------------------
//   `[0u8; 512]` on the stack, reused     `Vec<u8>` on the heap, as big as the
//                                         input
//   one `bool` (`in_word`) of history     a `Vec<&[u8]>` holding every word
//   `ulib::write_usize(STDOUT, n, 8)`     `write!(out, "{:8}", n)` (core::fmt)
//   `&[u8]` arguments                     `String` arguments
//
// Memory used by the exercise: 512 bytes plus a few counters, whatever the
// size of the input. Memory used here: the whole input, plus 16 bytes (a
// pointer and a length) for every word in it. A 1 GiB file of short words
// needs well over 3 GiB of heap. On rv6 there is no heap at all.

use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Default, Clone, Copy, Debug, PartialEq)]
struct Counts {
    lines: usize,
    words: usize,
    bytes: usize,
}

/// The same definition of whitespace as the exercise.
fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Every word in `data`, in order.
///
/// UNDERSTAND: `split` cuts at EVERY whitespace byte, so two spaces in a
///   row produce an empty piece between them: `b"a  b"` splits into
///   `["a", "", "b"]`. The `filter` throws the empty pieces away, and that
///   is how a run of whitespace ends up counting as a single separator. The
///   exercise gets the same result from `in_word` instead.
///
///   The words are not copied. Each `&[u8]` borrows from `data`, which is
///   why the return type has to carry `data`'s lifetime: this Vec cannot
///   outlive the buffer it points into.
fn words(data: &[u8]) -> Vec<&[u8]> {
    data.split(|&b| is_space(b)).filter(|w| !w.is_empty()).collect()
}

/// Count a buffer that is already entirely in memory.
fn count(data: &[u8]) -> Counts {
    Counts {
        lines: data.iter().filter(|&&b| b == b'\n').count(),
        words: words(data).len(),
        bytes: data.len(),
    }
}

/// Read ALL of `r` into memory, then count it.
///
/// UNDERSTAND: `read_to_end` calls `read` in a loop, just as the exercise
///   does, but instead of looking at each chunk and discarding it, it
///   appends the chunk to the Vec and grows the Vec when it fills up. By
///   the time `count` runs, the whole input is in memory. That is why the
///   chunk boundaries never come up here: when you have all of the input,
///   no word is ever split between two reads.
fn count_reader(r: &mut dyn Read) -> io::Result<Counts> {
    let mut data = Vec::new();
    r.read_to_end(&mut data)?;
    Ok(count(&data))
}

fn report(out: &mut dyn Write, c: Counts, label: Option<&str>) {
    let _ = write!(out, "{:8}{:8}{:8}", c.lines, c.words, c.bytes);
    if let Some(name) = label {
        let _ = write!(out, " {name}");
    }
    let _ = writeln!(out);
}

/// The same argument handling as the exercise's `run`: standard input when
/// there are no file arguments, a labeled row per file, a `total` row only
/// when there is more than one, and exit status 1 if any file failed.
///
/// The streams are passed in so the tests can capture them. ulib does this
/// with its own harness; in std we pass them as arguments.
fn run(args: &[String], stdin: &mut dyn Read, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    if args.len() < 2 {
        return match count_reader(stdin) {
            Ok(c) => {
                report(out, c, None);
                0
            }
            Err(_) => 1,
        };
    }

    let mut status = 0;
    let mut total = Counts::default();
    for path in &args[1..] {
        match File::open(path).and_then(|mut f| count_reader(&mut f)) {
            Ok(c) => {
                report(out, c, Some(path));
                total.lines += c.lines;
                total.words += c.words;
                total.bytes += c.bytes;
            }
            Err(_) => {
                let _ = writeln!(err, "wc: cannot open {path}");
                status = 1;
            }
        }
    }
    if args.len() > 2 {
        report(out, total, Some("total"));
    }
    status
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = run(&args, &mut io::stdin().lock(), &mut io::stdout().lock(), &mut io::stderr());
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_stdin(input: &[u8]) -> Vec<String> {
        let mut out = Vec::new();
        run(&["wc".to_string()], &mut &input[..], &mut out, &mut io::sink());
        String::from_utf8(out).unwrap().split_whitespace().map(String::from).collect()
    }

    // ── The exercise's five tests, same inputs, same expectations ─────────

    #[test]
    fn counts_lines_words_and_bytes() {
        assert_eq!(run_stdin(b"one two\nthree\n"), ["2", "3", "14"]);
    }

    #[test]
    fn runs_of_whitespace_count_as_one_separator() {
        assert_eq!(run_stdin(b"  a   b  \n")[1], "2", "two words despite the extra spaces");
    }

    #[test]
    fn a_file_without_a_trailing_newline_still_counts_its_words() {
        let f = run_stdin(b"no newline here");
        assert_eq!(f[0], "0", "no newline means no completed line");
        assert_eq!(f[1], "3");
    }

    #[test]
    fn empty_input_is_all_zeros() {
        assert_eq!(run_stdin(b""), ["0", "0", "0"]);
    }

    #[test]
    fn names_each_file_and_prints_a_total() {
        let dir = std::env::temp_dir().join(format!("wc_vec_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a");
        let b = dir.join("b");
        std::fs::write(&a, b"x\n").unwrap();
        std::fs::write(&b, b"y z\n").unwrap();
        let (a, b) = (a.to_string_lossy().into_owned(), b.to_string_lossy().into_owned());

        let mut out = Vec::new();
        let code = run(&["wc".into(), a.clone(), b.clone()], &mut io::empty(), &mut out, &mut io::sink());
        let text = String::from_utf8(out).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(code, 0);
        assert!(text.contains(&format!(" {a}\n")), "first file labeled: {text:?}");
        assert!(text.contains(&format!(" {b}\n")), "second file labeled: {text:?}");
        assert!(text.contains("       2       3       6 total\n"), "total row: {text:?}");
    }

    // ── Two more that only make sense for this version ────────────────────

    #[test]
    fn the_words_are_slices_of_the_input_not_copies() {
        let data = b"  a   bc\td\n";
        let w = words(data);
        assert_eq!(w, [&b"a"[..], b"bc", b"d"]);
        // Each word points INTO `data`. Nothing was copied.
        assert!(std::ptr::eq(w[1].as_ptr(), &data[6]));
    }

    #[test]
    fn a_missing_file_is_reported_on_stderr_and_exits_1() {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&["wc".into(), "/no/such/file".into()], &mut io::empty(), &mut out, &mut err);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert_eq!(err, b"wc: cannot open /no/such/file\n");
    }
}
