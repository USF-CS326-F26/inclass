# 06r Collections — in-class worked example

The `06r_collections` exercise has students build a **fixed-size process
table**: a lowest-free-slot allocator over `[Option<u32>; NPROC]`, a read-only
search, a bounds-checked free, and one `Vec` for the tests. This is the same
code with different nouns: a **per-process file-descriptor table**, where the
slot index handed back *is* the number `open` returns to the user program.
Walk through this one on screen, then send them off to the exercise, where the
shape is already familiar and only the domain is new.

```sh
cargo run --bin basic_table    # START HERE: one array, two loops, nothing else
cargo test                     # eleven tests, the same kind of contract as the exercise
cargo run                      # the descriptor-table demo, with an error to uncomment
```

## Start with `src/bin/basic_table.rs`

Before the descriptors, the concept on its own: an array of four `Option<u32>`
slots and the two loops you can write over it.

| step | what it shows | the line to point at |
|---|---|---|
| 1 | `[None; 4]` is four slots, back to back | `size_of_val(&slots)` → 32, fixed |
| 2 | `iter()` reads; each item is a `&Option<u32>` | `.filter(is_some).count()` |
| 3 | `iter_mut()` writes; each item is a `&mut Option<u32>` | `*slot = Some(7)` — the star |
| 4 | the index comes back as an `Option<usize>` | `position(is_none)` → `Some(2)`, then `None` |
| 5 | the error | `*slot = None` inside `iter()` → E0594 |

The payoff is the last one. `iter()` borrowed for reading, and the star is a
write; rustc says *"`slot` is a `&` reference, so the data it refers to cannot
be written"*. Which iterator you asked for decides which loop you may write —
and one level up, whether the function was handed `&[T]` or `&mut [T]` decides
which iterator you may ask for.

## The mapping

| exercise (`skeleton/lib.rs`) | here (`src/lib.rs`) | kernel role |
|---|---|---|
| `NPROC` (8) | `NOFILE` (8) | the hard limit, chosen once |
| `Option<u32>` — a pid | `Option<OpenFile>` — a record | a slot: free, or occupied |
| `new_table()` | `new_fd_table()` | `[None; N]`, no allocator |
| `alloc_slot(pid) -> Option<usize>` | `open_fd(inum) -> Option<usize>` | lowest free slot; the index becomes the descriptor |
| `find_pid` | `fd_of` | the read-only twin: `iter`, not `iter_mut` |
| `free_slot(index) -> bool` | `close_fd(fd) -> bool` | an untrusted index, checked before use |
| — | `dup_fd` | why "lowest free" is a rule, not a preference |
| — | `advance` | writing through a slot to a *field* |
| `live_pids() -> Vec<u32>` | `open_inums() -> Vec<u32>` | host code and tests may allocate |

Deliberate differences, so it is not find-and-replace — and so that walking
through this does not hand over the exercise:

- **The slot holds a record, not a number.** `Option<OpenFile>` is 24 bytes
  where `Option<u32>` is 8; the demo prints both. Reading the inode number out
  of a slot means reaching *into* the record — `f.inum` — which is why
  `open_inums` needs a `map` and `live_pids` does not.
- **`open_inums` is the adapter chain, not the push loop.** The exercise's hint
  walks you to `for … if let Some(pid) … push(*pid)`. This is
  `iter().flatten().map(..).collect()`. Same walk, two spellings; ask which
  link allocates (only `collect`, and only once).
- **`advance` uses `.get_mut(fd)` instead of `if fd >= len`.** The bounds
  check becomes an `Option`, and the write goes to a field of a live record —
  the shape of `f.off += n` in `sys_read`. `close_fd` keeps the exercise's
  explicit check so both spellings are on screen.
- **`dup_fd` explains the rule.** `close_fd(1); dup_fd(3)` returns `Some(1)`.
  That is `cmd > file` in the shell, and it works only because the search
  starts from the bottom.

## Running the demo

`cargo run` walks six sections in order. The part to slow down for is the
third:

```
== 3. redirection is two calls, and it only works bottom-up ==
  close_fd(1)      -> true
  dup_fd(3)        -> Some(1)   it landed in slot 1 because 1 was the LOWEST free
  table: [1@0 | 7@0 | 1@0 | 7@0 | - | - | - | -]
```

Nothing in `dup_fd` mentions slot 1. It found the lowest free slot, and the
shell had just made sure that was 1.

## The four things to say out loud

1. **The signature decides the loop.** `fd_of` takes `&[..]` and may only
   `iter()`; `open_fd` takes `&mut [..]` and may `iter_mut()`. Change the
   parameter and the body's options change with it. E0596 is the compiler
   pointing at the parameter, not the loop.
2. **`*slot = …` lands in the caller's array.** `iter_mut` hands out one
   `&mut` per slot, in turn. The star is the write, and it goes through the
   reference into the table that was passed in. There is no copy to forget to
   put back — which is the bug `advance_writes_through_the_slot_not_a_copy`
   exists to catch.
3. **The index is the descriptor, so lowest-free is a promise.** Every shell
   redirection depends on it. Show `dup_fd` landing in slot 1.
4. **Bounds-check anything a user program chose.** `close_fd(10_000)` returns
   `false`; `table[10_000]` would panic, and a kernel panic is a dead machine.
   `.get(fd)` and `if fd >= len` are the same check in two spellings.

## Live variations, if there is time

- Rewrite `open_fd` with `iter().enumerate()` and `table[fd] = Some(..)` —
  keep the `return`, and it compiles (the borrow checker sees the iterator is
  never used again). Delete the `return` and read E0506. The uncomment block in
  `src/main.rs` is this, as the exit-time close-all loop.
- Delete the bounds check from `close_fd` and run the tests:
  `closing_a_descriptor_from_outside_the_table_is_rejected_without_panicking`
  panics with `index out of bounds`. That is the machine dying.
- Drop the `.map(|f| f.inum)` from `open_inums` and read E0277: *"a value of
  type `Vec<u32>` cannot be built from an iterator over elements of type
  `&OpenFile`"*. `collect` knows what it is building and checks what it is fed.
- Make `advance` copy the record out (`let mut f = table[fd].unwrap();`), bump
  the copy, and return `true`. Nine tests still pass; two go red —
  `advance_writes_through_the_slot_not_a_copy`, and
  `dup_copies_the_record_into_the_lowest_free_slot`, which advanced fd 3
  before duplicating it. Ask what the other nine were failing to notice.
