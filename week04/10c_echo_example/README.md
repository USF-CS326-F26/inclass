# 10c echo — in-class worked example

The `10c_echo` exercise has students write **`echo`** against `ulib`: argv in,
bytes out, a number back, with the two-line ceremony that makes the same file
run on a laptop today and on rv6 in December. This is the same shape with
different nouns: **`basename`** — every operand stripped to its last path
component, one per line — written against a **local façade with `ulib`'s
exact signatures**, in plain `std`, so the whole ceremony can be shown without
the oslings tree. Walk through this one on screen, then send them off to the
exercise, where the shape is already familiar and only the command is new.

```sh
cargo run --bin basic_command -- a b c   # START HERE: a real command, numbered arguments
cargo test                               # eleven tests, the same kind of contract as the exercise
cargo run                                # the harness demo, with an error to uncomment
cargo run -- /usr/lib/ a/b; echo $?      # ...or run basename for real
```

It is host-only on purpose. There is no `#![cfg_attr(target_os = "none",
no_std, no_main)]` line because there is no rv6 backend here; the exercise has
one, and its `run` is byte-identical on both targets. Say that out loud before
anyone asks why this crate cannot be copied into `commands/`.

## Start with `src/bin/basic_command.rs`

Before `basename`, the ceremony on its own: a real command that prints each
argument on its own line, numbered, and exits 0.

| step | what it shows | the line to point at |
|---|---|---|
| 1 | the whole interface of a Unix program | `echo_example::main!(run);` — there is no `fn main` |
| 2 | `args.len()` is `argc`; `args.get(0)` is the name | `argv[0] = target/debug/basic_command` |
| 3 | the loop, and `get(i)` is an `Option<&[u8]>` | `.unwrap()` inside `1..args.len()` is honest |
| 4 | bytes out, `let _ =` on every `Result` | `write_all(STDOUT, b": ")` |
| 5 | the number the shell reads as `$?` | `cargo run --bin basic_command -- a b; echo $?` |

The payoff is the uncomment block: `write_all(STDOUT, ": ")` is E0308,
*"expected `&[u8]`, found `&str`"*. A string literal is bytes plus a UTF-8
promise; the kernel boundary is bytes with no promise. `b": "` is the only
thing that type-checks, and that is the type system telling you what a UART
is.

## The mapping

| exercise (`echo.rs`) | here | role |
|---|---|---|
| `#![cfg_attr(target_os = "none", no_std, no_main)]` | absent — host only | the bare-metal switch |
| `ulib::main!(run)` | `echo_example::main!(run)` — the host half only | argv → `Args` → `run` → `exit` |
| `fn run(args: Args) -> i32` | `fn run(args: Args) -> i32` | the whole job description |
| `args.get(i).unwrap()` in `1..len` | the same | `argv[i]` as bytes; an honest `unwrap` |
| `if i > 1 { b" " }` — a separator | `b"\n"` after each — a terminator | the two joining rules |
| `let _ = write_all(STDOUT, ..)` | the same, plus `write_all(STDERR, ..)` | fd 1 and fd 2 |
| return `0` | `0`, or `1` with no operands | `$?` |
| `ulib::testing::run` | `testing::run` + `testing::run_short_writes` | the harness calls `run` directly |
| — | `basename(&[u8]) -> &[u8]` | the first byte-level algorithm |

Deliberate differences, so it is not find-and-replace — and so that walking
through this does not hand over the exercise:

- **Terminator, not separator.** Every operand is followed by `\n`; there is
  no `i > 1`. The mirror image of `echo`'s rule, decided *before* the loop
  was written. Ask what `echo a "" b` prints and why the answer needs the
  other rule.
- **A second descriptor and a second exit code.** No operands → a message on
  fd 2 and `1` back. `echo` cannot fail; most commands can, and fd 2 is where
  they say so — which is why `cmd > out` never puts *"missing operand"* in the
  data. The harness captures both streams separately.
- **The bytes are inspected, not just copied.** `rposition(|&b| b == b'/')` is
  Part I's `Option<usize>` over Part IV's bytes: the shape `wc` and `grep`
  take next week.
- **The façade can short-write.** `run_short_writes(argv, 3, run)` makes every
  `write` accept at most three bytes. `ulib`'s host harness cannot do that,
  which is why a bare `write` in your command passes every test there and
  truncates on rv6. Here it fails a test.

## Running the demo

`cargo run` walks four sections in order. The part to slow down for is the
second and third together:

```
run(..)                     -> "kernel.elf\n"   writes = 2
run_short_writes(.., 3, ..) -> "kernel.elf\n"   writes = 5

write(STDOUT, b"kernel.elf\n") -> Ok(3)
stdout = "ker"   three bytes went, eight stayed, nothing complained
```

The program did not change between the first two lines; the console did, and
`write_all` made five calls instead of two. The last two lines are what a
bare `write` does on that console: `Ok(3)`, and silence.

## The four things to say out loud

1. **A command's whole interface is three things:** argv in, fd 1 and fd 2
   out, a number back. `fn run(args: Args) -> i32` is that interface, and
   `main!` is the adapter between it and whatever is underneath.
2. **Separator versus terminator is decided before the loop.** `echo` writes
   the space *before* every argument but the first; `basename` writes the
   newline *after* every operand. Both are right; mixing them is the bug
   students actually write.
3. **`write` returns a count, and a harness that never short-writes is lying
   to you kindly.** `buf = &buf[n..]` is the whole of `write_all`, and it is
   the loop L07 spent a section on. Show the count first, then the loop.
4. **The `Result` from `write_all` is the one you may discard, and `let _ =`
   is how you say so.** A command returning `i32` has no `Err` to return, so
   `?` is E0277 there — the boundary from 08r, seen from the user side.

## Live variations, if there is time

- Replace `write_all` with `write` in `run` and run `cargo test`:
  `a_console_that_takes_three_bytes_at_a_time_still_gets_every_byte` goes
  red with `"ker\n"`. Every other test still passes — which is exactly the
  situation in `ulib`, where there is no such test to fail.
- Return `0` on no operands and see which test notices
  (`no_operands_is_a_message_on_stderr_and_exit_1`, and
  `stdout_and_stderr_are_different_descriptors` if the message goes too).
- Pass a non-UTF-8 argument — `cargo run -- $'\xff'` — and note that nothing
  in `run` cares. Then try `args.str(i)` in the exercise's `ulib` and see
  where the `None` would have to be handled.
- Change `run_short_writes(.., 3, ..)` to `0` in section 2 of the demo and
  watch `write_all` return `Err(Error(-1))` instead of spinning forever. That
  `if n == 0` is one line in `ulib`, and this is the line it saves you from.
