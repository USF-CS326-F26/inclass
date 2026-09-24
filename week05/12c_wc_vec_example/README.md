# 12c wc — in-class example: the version rv6 cannot run

The `12c_wc` README gives one sentence to the obvious approach and then rules
it out: *"read the whole input, split it on whitespace, count the pieces —
... There is no `Vec` to hold the pieces and no heap to grow one on."* This
crate is that approach, written out in plain `std`, so there is something
concrete to point at when explaining why the exercise streams instead.

```sh
cargo test                        # the exercise's five tests, plus two more
cargo run -- src/main.rs Cargo.toml
wc src/main.rs Cargo.toml         # same bytes as the system wc
printf '  a   b  \n' | cargo run    # stdin: 1 line, 2 words, 10 bytes
```

It uses the same definitions of line, word, and byte, the same `is_space`, the
same output format, and the same argument handling as the exercise's `run`.
Only the counting is different.

## The whole difference is three lines

```rust
let mut data = Vec::new();
r.read_to_end(&mut data)?;                         // Vec #1: the entire input
data.split(|&b| is_space(b))
    .filter(|w| !w.is_empty())
    .collect::<Vec<&[u8]>>()                       // Vec #2: every word
    .len()
```

| | exercise (`solution/wc.rs`) | here (`src/main.rs`) |
|---|---|---|
| input buffer | `[0u8; 512]` on the stack, reused | `Vec<u8>`, as large as the input |
| memory of the past | one `bool`, `in_word` | a `Vec<&[u8]>`, one entry per word |
| words split across chunks | handled by `in_word` | cannot happen, because every byte is already in memory |
| runs of whitespace | clearing `in_word` again does nothing | `split` makes empty pieces, and `filter` drops them |
| numbers out | `ulib::write_usize(STDOUT, n, 8)` | `write!(out, "{:8}", n)` |
| memory for a 1 GiB file | 512 bytes | 1 GiB, plus 16 bytes for each word |

## Things to point out

1. **`split` cuts at every whitespace byte.** `b"a  b"` becomes
   `["a", "", "b"]`, so the `filter` is what gives the "a run of whitespace is
   one separator" rule. Delete the `filter`, run `cargo test`, and
   `runs_of_whitespace_count_as_one_separator` fails. The exercise gets the
   same rule from `in_word` without building any pieces.
2. **The words are borrowed, not copied.** `words(data) -> Vec<&[u8]>` holds
   pointers into `data`, which is why its signature ties the result's lifetime
   to the input. `the_words_are_slices_of_the_input_not_copies` checks this
   with `ptr::eq`. It is cheap, but the whole input still has to stay in
   memory for as long as the Vec does.
3. **`read_to_end` is the exercise's loop, minus the counting.** It calls
   `read` again and again just as `count_fd` does. The difference is that it
   keeps every chunk, growing the Vec when it fills up, where `count_fd`
   looks at each byte once and then reuses the buffer.
4. **The second Vec only exists to be counted.** We build a list of every word
   just to call `.len()` on it. Replacing `.collect::<Vec<_>>().len()` with
   `.count()` removes Vec #2 but keeps Vec #1. Removing Vec #1 as well means
   processing the input a chunk at a time, and then a word can be split
   across two chunks. Handling that is the `in_word` state machine, which is
   the exercise.

## Live variations, if there is time

- Do the `.count()` change from point 4 on screen. The tests still pass. Then
  ask what else has to change to remove `read_to_end`.
- `cargo build --release` and compare the binary's size with a `no_std` build
  of the exercise's `wc` (`oslings ship`). Part of the difference is
  `core::fmt`, which the exercise avoids by using `write_usize`.
- Run it on a large file and watch the memory:
  `/usr/bin/time -l target/release/wc_vec bigfile` on macOS, or
  `/usr/bin/time -v` on Linux. Peak RSS grows with the file. The exercise's
  `wc` would use the same amount for any file.
