// ╔══════════════════════════════════════════════════════════════════════╗
// ║  10c — echo — IN-CLASS EXAMPLE                                       ║
// ║  Same shape as the exercise, different nouns: `basename` instead of  ║
// ║  `echo`, written against a LOCAL façade with ulib's exact signatures ║
// ║  -- so the ceremony can be shown without the oslings tree.           ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/10c_echo/skeleton/echo.rs` and next to
// `ulib/src/lib.rs`. The top half of this file is a stand-in for ulib: an
// `Args` over byte slices, a `write` that MAY accept fewer bytes than it was
// given, the `write_all` that copes, a `main!` macro, and a `testing::run`
// that calls your `run` directly with fd 1 and fd 2 diverted into buffers.
//
// It is host-only on purpose. There is no `#![cfg_attr(target_os = "none",
// no_std, no_main)]` line because there is no rv6 backend here. The exercise
// has one, and its `run` is byte-identical on both.
//
// Two things here are NOT in the exercise: the output is a TERMINATOR per
// operand rather than a separator between them, and the harness can be told
// to short-write, which ulib's host harness cannot.

use std::cell::RefCell;
use std::io::Write;

// ---------------------------------------------------------------------------
// The façade. Every item mirrors ulib/src/lib.rs.
// ---------------------------------------------------------------------------

/// A file descriptor: a small integer the kernel translates into an open file.
pub type Fd = i32;

pub const STDOUT: Fd = 1;
pub const STDERR: Fd = 2;

/// A failed call. rv6 answers every failure with `-1`, so this is a thin
/// wrapper around that number rather than a rich enum.
///
/// UNDERSTAND: this is `ulib::Error`, and it is the far side of 08r's
///   boundary: the kernel collapsed its error to a number, and ulib hands
///   you the number. The cause is gone. That is the contract today.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Error(pub i32);

/// The command line. Built for you by [`main!`] or [`testing::run`].
///
/// UNDERSTAND: arguments are `&[u8]`, not `&str`, because that is what they
///   are: `exec` pushes NUL-terminated bytes onto the new program's stack.
///   Forcing `&str` would put a UTF-8 validation table in every image. So
///   `get` hands you bytes and lets you decide.
#[derive(Clone, Copy)]
pub struct Args<'a> {
    argv: &'a [&'a [u8]],
}

impl<'a> Args<'a> {
    #[doc(hidden)]
    pub fn from_slice(argv: &'a [&'a [u8]]) -> Args<'a> {
        Args { argv }
    }

    /// Number of arguments, including the program name -- i.e. `argc`.
    pub fn len(&self) -> usize {
        self.argv.len()
    }

    pub fn is_empty(&self) -> bool {
        self.argv.is_empty()
    }

    /// Argument `i`, or `None` if there are fewer than `i + 1`.
    ///
    /// UNDERSTAND: `.get(i)` on the slice is the bounds check as an `Option`;
    ///   `.copied()` turns the `Option<&&[u8]>` into `Option<&[u8]>`. Inside
    ///   `1..args.len()` the answer is always `Some`, which is why the
    ///   exercise's `.unwrap()` there is honest.
    pub fn get(&self, i: usize) -> Option<&'a [u8]> {
        self.argv.get(i).copied()
    }

    /// `argv[0]` -- the name the program was invoked as.
    pub fn prog(&self) -> &'a [u8] {
        self.get(0).unwrap_or(b"?")
    }
}

/// Write up to `buf.len()` bytes. May write fewer; see [`write_all`].
///
/// UNDERSTAND: the contract of every `write` in every operating system. A
///   short write is not an error -- it is a number you have to read. Under
///   [`testing::run_short_writes`] this one really does stop short, so that
///   the bug ulib's host harness cannot show you shows up in a test.
pub fn write(fd: Fd, buf: &[u8]) -> Result<usize, Error> {
    CAPTURE.with(|c| {
        let mut cap = c.borrow_mut();
        match cap.as_mut() {
            Some(cap) => {
                let sink = match fd {
                    STDOUT => &mut cap.stdout,
                    STDERR => &mut cap.stderr,
                    _ => return Err(Error(-1)),
                };
                let n = buf.len().min(cap.chunk);
                sink.extend_from_slice(&buf[..n]);
                cap.writes += 1;
                Ok(n)
            }
            None => {
                let ok = match fd {
                    STDOUT => std::io::stdout().write_all(buf).is_ok(),
                    STDERR => std::io::stderr().write_all(buf).is_ok(),
                    _ => false,
                };
                if ok { Ok(buf.len()) } else { Err(Error(-1)) }
            }
        }
    })
}

/// Write every byte, looping over short writes.
///
/// UNDERSTAND: ulib/src/lib.rs, verbatim. `buf = &buf[n..]` copies nothing:
///   it moves the slice's pointer forward and shrinks its length. `n == 0`
///   is a descriptor that accepts nothing and never will -- looping on it
///   would spin forever, so it is turned into an error instead.
pub fn write_all(fd: Fd, mut buf: &[u8]) -> Result<(), Error> {
    while !buf.is_empty() {
        let n = write(fd, buf)?;
        if n == 0 {
            return Err(Error(-1));
        }
        buf = &buf[n..];
    }
    Ok(())
}

/// Write `n` in decimal, right-aligned in at least `width` columns.
///
/// UNDERSTAND: ulib's, verbatim. It exists so commands never pull in
///   `core::fmt`: `write!` drags in 12-18 KiB of formatting machinery, and a
///   user program on rv6 has a hard image budget.
pub fn write_usize(fd: Fd, n: usize, width: usize) -> Result<(), Error> {
    let mut digits = [0u8; 20];
    let mut i = digits.len();
    let mut v = n;
    loop {
        i -= 1;
        digits[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    let len = digits.len() - i;
    for _ in len..width {
        write_all(fd, b" ")?;
    }
    write_all(fd, &digits[i..])
}

/// The process's real arguments, as bytes. Used by [`main!`].
#[doc(hidden)]
pub fn host_argv() -> Vec<Vec<u8>> {
    std::env::args_os().map(|a| a.into_encoded_bytes()).collect()
}

/// Declare `run` as this program's entry point.
///
/// UNDERSTAND: the host half of `ulib::main!`, and nothing else. On rv6 the
///   real macro also expands to a `_start` symbol placed first in the image,
///   which unpacks `argc`/`argv` from the stack `exec` built and calls the
///   SAME `run`. Your command file has no `cfg` in it because the macro has
///   both halves.
#[macro_export]
macro_rules! main {
    ($run:ident) => {
        fn main() {
            let owned = $crate::host_argv();
            let refs: ::std::vec::Vec<&[u8]> = owned.iter().map(|v| &v[..]).collect();
            let code = $run($crate::Args::from_slice(&refs));
            ::std::process::exit(code);
        }
    };
}

struct Capture {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    chunk: usize,
    writes: usize,
}

thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}

/// Host-only test harness, the shape of `ulib::testing`.
pub mod testing {
    use super::{Args, Capture, CAPTURE};

    /// What a run produced: the exit code, both streams, and how many times
    /// `write` was called -- the last is not in ulib, and it is what makes a
    /// short-writing console visible.
    pub struct Output {
        pub code: i32,
        pub stdout: Vec<u8>,
        pub stderr: Vec<u8>,
        pub writes: usize,
    }

    impl Output {
        pub fn out(&self) -> &str {
            std::str::from_utf8(&self.stdout).expect("stdout was not UTF-8")
        }
        pub fn err(&self) -> &str {
            std::str::from_utf8(&self.stderr).expect("stderr was not UTF-8")
        }
    }

    /// Run `f` with the given argv, capturing fd 1 and fd 2.
    ///
    /// UNDERSTAND: `ulib::testing::run`, and the whole point of the façade:
    ///   it calls `run` DIRECTLY. No process, no `dyn Write` threaded
    ///   through your command, no test-only path. The source under test is
    ///   the source that ships.
    pub fn run(argv: &[&str], f: for<'a> fn(Args<'a>) -> i32) -> Output {
        run_short_writes(argv, usize::MAX, f)
    }

    /// Like [`run`], but every `write` accepts at most `chunk` bytes.
    ///
    /// UNDERSTAND: not in ulib. A UART, a pipe, and a socket all do this to
    ///   you; ulib's host harness never does, which is why a bare `write`
    ///   passes there and truncates on rv6. Here it fails a test.
    pub fn run_short_writes(argv: &[&str], chunk: usize, f: for<'a> fn(Args<'a>) -> i32) -> Output {
        CAPTURE.with(|c| {
            *c.borrow_mut() =
                Some(Capture { stdout: Vec::new(), stderr: Vec::new(), chunk, writes: 0 });
        });
        let owned: Vec<Vec<u8>> = argv.iter().map(|a| a.as_bytes().to_vec()).collect();
        let refs: Vec<&[u8]> = owned.iter().map(|v| &v[..]).collect();
        let code = f(Args::from_slice(&refs));
        CAPTURE.with(|c| {
            let cap = c.borrow_mut().take().expect("capture was active");
            Output { code, stdout: cap.stdout, stderr: cap.stderr, writes: cap.writes }
        })
    }
}

// ---------------------------------------------------------------------------
// The command. This half is the twin of `echo.rs`.
// ---------------------------------------------------------------------------

/// The last component of a path: trailing slashes dropped, everything up to
/// and including the last remaining slash dropped. `/` stays `/`.
///
/// UNDERSTAND: the first byte-level algorithm of the course, and it is Part
///   I's `Option<usize>` over Part IV's bytes: `rposition` finds the last
///   `b'/'` or says `None`, and slicing does the rest. No String, no str,
///   no allocation -- the result borrows from the argument.
pub fn basename(path: &[u8]) -> &[u8] {
    let mut end = path.len();
    while end > 1 && path[end - 1] == b'/' {
        end -= 1;
    }
    let path = &path[..end];
    match path.iter().rposition(|&b| b == b'/') {
        Some(i) if i + 1 < path.len() => &path[i + 1..],
        _ => path,
    }
}

/// The whole job: argv in, bytes out, a number back.
///
/// UNDERSTAND: twin of `echo`'s `run`, with two differences on purpose.
///   Each operand is followed by `b"\n"` -- a TERMINATOR, so there is no
///   `i > 1` -- and with no operands at all this command FAILS: a message
///   on fd 2, and `1` back to the shell. `echo` cannot fail; most commands
///   can, and fd 2 is where they say so, so that `cmd > out` never puts an
///   error message in the data.
///
///   `let _ =` on every `write_all`: there is nothing a command can do if
///   the console is gone, and `let _ =` says so in the source.
pub fn run(args: Args) -> i32 {
    if args.len() < 2 {
        let _ = write_all(STDERR, b"basename: missing operand\n");
        return 1;
    }
    for i in 1..args.len() {
        let _ = write_all(STDOUT, basename(args.get(i).unwrap()));
        let _ = write_all(STDOUT, b"\n");
    }
    0
}

// ---------------------------------------------------------------------------
// The tests. Read them first: they are the contract, and they are the same
// kind of contract the exercise's tests are.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::testing;
    use super::{run, write, write_all, Error, STDOUT};

    #[test]
    fn one_operand_is_stripped_to_its_last_component() {
        let out = testing::run(&["basename", "/usr/lib"], run);
        assert_eq!(out.out(), "lib\n");
        assert_eq!(out.code, 0);
    }

    #[test]
    fn a_bare_name_is_returned_unchanged() {
        assert_eq!(testing::run(&["basename", "README"], run).out(), "README\n");
    }

    #[test]
    fn trailing_slashes_are_not_a_component() {
        assert_eq!(testing::run(&["basename", "/usr/lib/"], run).out(), "lib\n");
        assert_eq!(testing::run(&["basename", "/usr/lib///"], run).out(), "lib\n");
    }

    #[test]
    fn the_root_is_its_own_basename() {
        assert_eq!(testing::run(&["basename", "/"], run).out(), "/\n");
        assert_eq!(testing::run(&["basename", "///"], run).out(), "/\n");
    }

    #[test]
    fn every_operand_gets_its_own_line() {
        // A terminator, not a separator: n operands, n newlines.
        let out = testing::run(&["basename", "a/b", "c/d/", "e"], run);
        assert_eq!(out.out(), "b\nd\ne\n");
    }

    #[test]
    fn an_empty_operand_is_an_empty_line() {
        assert_eq!(testing::run(&["basename", "", "x"], run).out(), "\nx\n");
    }

    #[test]
    fn no_operands_is_a_message_on_stderr_and_exit_1() {
        let out = testing::run(&["basename"], run);
        assert_eq!(out.out(), "");
        assert_eq!(out.err(), "basename: missing operand\n");
        assert_eq!(out.code, 1);
    }

    #[test]
    fn stdout_and_stderr_are_different_descriptors() {
        // `cmd > out` captures fd 1 only. An error on fd 2 never lands in
        // the data; an answer on fd 1 never lands in the terminal.
        let ok = testing::run(&["basename", "/x/y"], run);
        assert_eq!((ok.out(), ok.err()), ("y\n", ""));
        let bad = testing::run(&["basename"], run);
        assert_eq!((bad.out(), bad.err()), ("", "basename: missing operand\n"));
    }

    #[test]
    fn a_console_that_takes_three_bytes_at_a_time_still_gets_every_byte() {
        let out = testing::run_short_writes(&["basename", "/usr/local/lib"], 3, run);
        assert_eq!(out.out(), "lib\n");
        // "lib" is one write_all of 3 bytes plus one of 1: 2 calls. Under a
        // 3-byte console that is still 2. Now a longer name:
        let out = testing::run_short_writes(&["basename", "/usr/kernel.elf"], 3, run);
        assert_eq!(out.out(), "kernel.elf\n");
        assert_eq!(out.writes, 4 + 1); // 10 bytes in 3s = 4 calls, then "\n"
    }

    #[test]
    fn a_bare_write_hands_back_a_count_and_keeps_the_rest() {
        // What `write` does on a 3-byte console: three bytes go, nine stay,
        // and nothing complains. The count is the only warning you get.
        let out = testing::run_short_writes(&["probe"], 3, |_| {
            let n = write(STDOUT, b"hello world\n");
            assert_eq!(n, Ok(3));
            0
        });
        assert_eq!(out.out(), "hel");
    }

    #[test]
    fn write_all_reports_a_stalled_descriptor_instead_of_spinning() {
        // A console that accepts zero bytes per call would make a naive loop
        // spin forever. write_all turns n == 0 into an error.
        let out = testing::run_short_writes(&["probe"], 0, |_| {
            assert_eq!(write_all(STDOUT, b"x"), Err(Error(-1)));
            0
        });
        assert_eq!(out.out(), "");
    }
}
