# Week 03 in class — Structs, enums, and the tables that hold them

Companion material for L04 (September 3) and L05 (September 8, 2026). The
lecture pages on the course site make the arguments; this material is the part
you *run* on screen while students run it too. Two exercises come due this
week — `04r_structs_impl` on Thursday and `05r_enums_match` on Friday — and the
first three parts of the session are aimed squarely at them.

```
week03/
├── slides.html                 51 slides, reveal.js, same theme as the course deck
├── README.md                   this file — session plan and talking points
├── examples/
│   ├── Cargo.toml
│   ├── src/bin/*.rs            12 runnable programs, one concept each
│   ├── broken/*.rs             7 programs that must NOT compile, with the fixes
│   └── show-errors.sh          compiles each broken file and shows rustc's message
├── 04r_structs_impl_example/   the exercise's shape, in a different domain
└── 05r_enums_match_example/    the same, for Friday's exercise
```

## Running it

```bash
cd examples
cargo run --bin 01_structs      # ... through 12_tables_and_iterators
```

```bash
cd examples && ./show-errors.sh
```

`show-errors.sh` walks all seven compile failures, pausing between each; pass a
substring to jump to one: `./show-errors.sh e0004`. Nothing under `broken/` is
part of the package, so `cargo build` always succeeds.

The two worked examples are separate projects, each with its own tests:

```bash
cd 04r_structs_impl_example && cargo run --bin basic_struct && cargo test && cargo run
cd 05r_enums_match_example  && cargo run --bin basic_enum   && cargo test && cargo run
```

Open `slides.html` in any browser — no server needed. Press `s` for speaker
notes/timer, `Esc` for the slide grid, `f` for full screen.

## The twelve programs

| Program | The one idea | The line to point at |
|---|---|---|
| `01_structs` | a struct is its fields, back to back | `size_of::<Console>()` with no header |
| `02_impl_and_self` | `.` is a method, `::` is an associated function | `Stats::new()` beside `s.record_read()` |
| `03_derive_and_copy` | `Copy` makes assignment stop *moving* | the same by-value method called twice |
| `04_newtype` | same bits, a genuinely different type | an alias accepting the swap a newtype rejects |
| `05_const_fn` | arithmetic the compiler already did | a `static` table with no start-up loop |
| `06_repr_c` | layout is a promise only `#[repr(C)]` makes | `flag` at offset 10, then at offset 0 |
| `07_drop_guard` | release became a closing brace | "release" printed on a path with no call |
| `08_enums` | exactly one of these, and nothing else | `size_of::<Option<&u8>>() == 8` |
| `09_match` | the compiler lists what you forgot | the careful column vs. the `_` column |
| `10_option` | absence with a type of its own | `Ok(1)` and `Err(-1)` side by side |
| `11_slices` | pointer + length, bounds-checked | `&[u8]` is 16 bytes, `&[u8; 8]` is 8 |
| `12_tables_and_iterators` | a fixed table, searched lazily | the closure running twice, not eight times |

## The seven failures

| File | Error | Fix shown in the header comment |
|---|---|---|
| `e0004_non_exhaustive.rs` | a variant was added; a `match` was not | add the arm, or `_` and lose the check |
| `e0308_newtype_mismatch.rs` | a raw integer where a newtype belongs | wrap it, or unwrap it visibly |
| `e0382_method_moved_it.rs` | a by-value `self` consumed the value | derive `Copy`, or take `&self` |
| `e0184_copy_with_drop.rs` | `Copy` on a type with a destructor | drop one of the two |
| `e0015_non_const_in_const.rs` | a non-`const fn` in a const context | make the callee `const fn` |
| `e0507_move_out_of_self.rs` | moving a field out of `&self` | return a borrow, or `clone()` |
| `e0080_const_index.rs` | a constant index past the end | fix the index, or use `get()` |

Every one of these has been checked against the installed toolchain — the error
codes in the table are the codes rustc actually emits.

## A 75-minute plan

| Min | Slides | What happens |
|-----|--------|--------------|
| 0–4 | 1–3 | the kernel written in integers; four `usize` parameters, one swap |
| 4–14 | 4–10 | structs, `impl`, the three receivers **RUN** `01_structs`, `02_impl_and_self` |
| 14–20 | 11–13 | `derive`, `Copy`, and `Copy` XOR `Drop` **RUN** `03_derive_and_copy` |
| 20–30 | 14–19 | the newtype and bit packing **RUN** `04_newtype`, then `show-errors.sh e0308` |
| 30–36 | 20–22 | `const fn` and const contexts **RUN** `05_const_fn` |
| 36–42 | 23–25 | `#[repr(C)]` and the offsets **RUN** `06_repr_c` |
| 42–49 | 26–30 | `Drop`, and the guard **RUN** `07_drop_guard`, then `show-errors.sh e0184` |
| 49–55 | 31–34 | enums, variants that carry data, layout **RUN** `08_enums` |
| 55–64 | 35–39 | `match`, exhaustiveness, the `_` trap, guards **RUN** `09_match`, then `show-errors.sh e0004` |
| 64–68 | 40–41 | `Option` **RUN** `10_option` |
| 68–73 | 42–48 | arrays, slices, bounds checks, fixed tables **RUN** `11_slices`, `12_tables_and_iterators` |
| 73–75 | 49–51 | the error table, and on to `04r` and `05r` |

Cut first if you are short on time, in this order: slides 45–48 and
`12_tables_and_iterators` (Part V is a bridge to `06r`, not to this week's
exercises), then slide 25 (`Worse Than Breaking`), then `03_derive_and_copy` —
folding `Copy` into the `04_newtype` demo instead. Never cut slides 36–38: the
exhaustiveness error and the guard fall-through are the two ideas students take
into Friday.

If the two exercise companions are going on screen instead, budget 10 minutes
each and drop Part V entirely.

## Talking points that land

- **Read the receiver as an ownership decision, not a syntax choice.** `&self`
  hands the value back, `&mut self` promises nobody else is looking, `self`
  takes it away. Every "why won't this compile" in `04r` is one of those three
  chosen wrongly.
- **`Copy` does not make copying cheap.** Copying eight bytes was always cheap.
  It makes assignment stop *moving* — which is the only reason a small
  plain-data type can offer by-value methods at all.
- **A newtype costs nothing and catches everything.** Same size, same register,
  same instructions. Show `size_of` first, so nobody thinks they are paying for
  the safety, then show E0308.
- **`#[repr(C)]` fails silently at compile time.** Delete it and the build is
  clean; the first symptom is a jump to a garbage address much later. Say out
  loud that the bug is not that it breaks — it is that nothing *promises* it
  will not.
- **Exhaustiveness is a refactoring tool, not a syntax rule.** Its value is not
  in the `match` you are writing now, but in the twelve you will not remember
  when you change the enum next month. `_` is what you trade that away for.
- **Guards fall through.** A failing guard resumes matching at the next arm
  rather than leaving the `match`. That is why a wakeup on the wrong channel
  does not wake the wrong process, and it is the one thing in `05r` that is not
  guessable.

## Questions students ask, with short answers

**"Why not just use a type alias?"** Because `type Pte = u64;` creates an
*alias*, not a type: the two are interchangeable in every expression, so the
compiler has nothing to check. Only a `struct` makes a genuinely new type.
`04_newtype` shows both, side by side, in the first twenty lines.

**"Does `const fn` mean it can *only* run at compile time?"** No — `const` only
*adds* an ability. The same function is callable at run time with a value that
does not exist until then. `05_const_fn` calls one both ways.

**"When do I actually need `#[repr(C)]`?"** When something that is not the Rust
compiler reads the bytes: assembly, hardware, a C library, or a file somebody
else wrote. Never for a struct only your Rust code touches — you would be
giving up an optimization for nothing.

**"Why can't I just write `_` everywhere?"** You can, and it compiles. It also
covers variants that do not exist yet, so the compiler stops telling you about
the decisions you have not made. `_` is right for open domains — a syscall
number from a user program, an interrupt cause from hardware — and wrong for a
closed set you defined yourself.

**"Is `Option` slow?"** `Option<&T>` is the same size as `&T`, because a
reference can never be all-zeroes and that unused pattern becomes `None`.
`Option<u32>` costs a word. Either way, the alternative is a sentinel value
somebody forgets to check.

**"If a fixed array is so limiting, why not a `Vec`?"** Three reasons, none of
them taste: there is no allocator when the table is first needed, the trap path
may not allocate at all, and a hard limit fails at the call that asked for too
much rather than somewhere else, later. `12_tables_and_iterators` ends on this.

## What comes next

`oslings` exercises `04r_structs_impl` (Thursday) and `05r_enums_match`
(Friday), then `06r_collections` the following week — which is where Part IV
stops being a preview. After that the same three ideas arrive as kernel code:
`Pte` drives a real Sv39 walk in `33k`, `Proc` and `ProcState` are `34k`, and
the `#[repr(C)]` `Context` is read by hand-written assembly in `35k`.
