# Week 02 in class — Rust types, ownership, and borrowing

Companion material for L03 (September 1, 2026). The lecture deck on the course
site makes the argument; this material is the part you *run* on screen while
students run it too.

```
week02/
├── slides.html              37 slides, reveal.js, same theme as the course deck
├── README.md                this file — session plan and talking points
└── examples/
    ├── Cargo.toml
    ├── src/bin/*.rs         10 runnable programs, one concept each
    ├── broken/*.rs          7 programs that must NOT compile, with the fixes
    └── show-errors.sh       compiles each broken file and shows rustc's message
```

## Running it

```bash
cd examples
cargo run --bin 01_scalars      # ... through 10_how_to_pass
```

```bash
cd examples && ./show-errors.sh
```

`show-errors.sh` walks all seven compile failures, pausing between each; pass a
substring to jump to one: `./show-errors.sh e0502`. Nothing under `broken/` is
part of the package, so `cargo build` always succeeds.

Open `slides.html` in any browser — no server needed. Press `s` for speaker
notes/timer, `Esc` for the slide grid, `f` for full screen.

## The ten programs

| Program | The one idea | The line to point at |
|---|---|---|
| `01_scalars` | a value is its bits; `usize` is an address | the PTE decoded from a `u64` |
| `02_compound` | structs/arrays/enums are plain layouts | `size_of::<Option<&u8>>() == 8` |
| `03_stack_and_heap` | the handle is not the buffer | `push` printing a *new* buffer address |
| `04_move` | a move copies 3 words, not the data | 8 MiB "moved" with the same pointer |
| `05_copy_types` | `Copy` XOR `Drop` | two independent `Pte` copies at different addresses |
| `06_drop` | `free()` became a scope | `kfree` lines in reverse declaration order |
| `07_borrow_shared` | many readers, no ownership | three references printing one buffer address |
| `08_borrow_mut` | `&mut` means *exclusive* | `split_at_mut` — two writers, provably disjoint |
| `09_slices` | ptr + len, checked | one `checksum` over an array, a `Vec`, and a slice |
| `10_how_to_pass` | choosing `T` / `&T` / `&mut T` | the mini `PageAlloc` with `&self` and `&mut self` |

## The seven failures

| File | Error | Fix shown in the header comment |
|---|---|---|
| `e0382_use_after_move.rs` | use of moved value | use `t`, `clone()`, or borrow |
| `e0499_two_mut_borrows.rs` | two `&mut` at once | sequence them, or `split_at_mut` |
| `e0502_shared_and_mut.rs` | `&` and `&mut` overlap | finish the read first; or copy the value out |
| `e0505_move_while_borrowed.rs` | move while borrowed | use the reference before the move |
| `e0507_move_out_of_index.rs` | move out of a `Vec` index | borrow, `clone()`, or `remove()` |
| `e0596_immutable_borrow.rs` | not declared mutable | add `mut` |
| `e0106_dangling_reference.rs` | missing lifetime | return the value, or borrow from a parameter |

Every one of these has been checked against the installed toolchain — the error
codes in the table are the codes rustc actually emits.

## A 75-minute plan

| Min | Slides | What happens |
|-----|--------|--------------|
| 0–5 | 1–3 | the double-free in C; the 70% statistic; why this is an OS topic |
| 5–20 | 4–11 | scalar and compound types **RUN** `01_scalars`, `02_compound` |
| 20–30 | 12–15 | stack handle vs. heap buffer **RUN** `03_stack_and_heap` |
| 30–45 | 16–24 | the ownership rule, moves, `Copy`, `Drop` **RUN** `04`, `05`, `06` |
| 45–48 | 18 | **RUN** `./show-errors.sh e0382` — read the error out loud, together |
| 48–65 | 25–32 | borrowing, aliasing XOR mutation, slices **RUN** `07`, `08`, `09` |
| 65–70 | 33 | the error table **RUN** `./show-errors.sh`, predict each one first |
| 70–75 | 34–37 | where `unsafe` comes in; on to `r02_ownership`, `r03_borrowing` |

Cut first if you are short on time: `02_compound`, slide 14 (`String` vs `&str`),
slide 31 (`split_at_mut`). Never cut the `03` demo — the printed addresses are
what makes ownership concrete.

## Talking points that land

- **Read `&mut` as "exclusive," never as "mutable."** The guarantee is not
  permission to write; it is that nobody else is looking while you do.
- **"Moved" means "given away."** E0382 is a use-after-free reported before the
  program ran. Say that sentence every time the error appears.
- **`push` may reallocate.** This one fact justifies E0502 completely. Show the
  changing buffer address in `03_stack_and_heap`, then the error in
  `e0502_shared_and_mut.rs`, back to back.
- **`Copy` and `Drop` are mutually exclusive.** If a value with a destructor
  could be copied, the destructor would run once per copy. The double free is
  not avoided by care; it is a combination the type system rejects.
- **Borrows end at their last use, not at the closing brace.** Students who
  learned Rust from pre-2018 material will fight the compiler over this.

## Questions students ask, with short answers

**"Is a move expensive?"** No — three words of stack, usually optimized away.
`04_move` moves an 8 MiB `Vec` and prints the same buffer address afterwards.

**"Why can't I just `clone()` everything?"** You can, and it compiles. It also
allocates, and in a kernel there may be no allocator yet. `clone()` is the right
answer when you genuinely need a second value; it is the wrong answer when you
only needed to read.

**"Why does `&mut` block even reading?"** Because a reader could observe a
half-finished update, and because the writer may move the buffer out from under
the reader. Exclusivity is what makes both impossible.

**"What if I really need two writers?"** Then you need `unsafe`, a lock, or a
type built from them (`SpinLock<T>`, later this term). The checker is not
refusing a plan — it is refusing an unproven one.

**"Does `unsafe` turn the borrow checker off?"** No. It unlocks five extra
operations (raw-pointer deref, `static mut` access, calling `unsafe` fns,
`unsafe` traits, unions). The borrow rules still apply to everything else; what
changes is who supplies the proof.

## What comes next

`oslings` exercises `r02_ownership` and `r03_borrowing`, then the physical page
allocator, where `kalloc`'s free list is exactly the aliasing-plus-mutation case
that safe Rust refuses — and the reason its `unsafe` block carries a written
argument instead of a comment that says "don't free twice."
