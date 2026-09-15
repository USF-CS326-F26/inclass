# 07r Traits — in-class worked example

The `07r_traits` exercise has students build an **`Out` sink** with two
implementers, a **`Scheduler`** with two policies, and one generic `trace`
that drives every combination. This is the same code with different nouns: a
byte **`Sink`**, a page-frame **`Allocator`** with first-fit and next-fit
policies, and one generic `trace`. Walk through this one on screen, then send
them off to the exercise, where the shape is already familiar and only the
domain is new.

```sh
cargo run --bin basic_trait    # START HERE: one trait, two types, one generic function
cargo test                     # twelve tests, the same kind of contract as the exercise
cargo run                      # the sink/allocator demo, with an error to uncomment
```

## Start with `src/bin/basic_trait.rs`

Before the sinks and the allocators, the concept on its own: the timer
interrupt calls something once per tick, and two different things want to be
called.

| step | what it shows | the line to point at |
|---|---|---|
| 1 | a trait is a promise: one required method, one default | `fn beat(&mut self);` — no body |
| 2 | `impl Heartbeat for Printer` writes only `beat` | `beat_for(3)` prints three ticks anyway |
| 3 | a second implementer, and `beat_for` already works for it | `ticks = 5` with no `beat_for` written |
| 4 | a generic function; the bound lets it call `beat_for` | `run_timer<H: Heartbeat>` on both types |
| 5 | the error | `impl Heartbeat for Silent {}` → E0046 |

The payoff is the third one. `beat_for` was compiled before `Counter` existed,
against a `self` that promised only `beat`. That is why a fifth implementer
costs zero lines — and why the only way to break it is to skip the required
method, which the compiler names (*"missing: `beat`"*).

## The mapping

| exercise (`skeleton/lib.rs`) | here (`src/lib.rs`) | kernel role |
|---|---|---|
| `Out::write_str(&str)` | `Sink::put(&[u8])` | the one required method |
| `write_line` (default) | `put_line` (default) | written once, inherited by all |
| `StringOut` / `CountingOut` | `VecSink` / `TallySink` | store it / measure it |
| `write_banner(&mut dyn Out)` | `put_banner(&mut dyn Sink)` | dynamic dispatch: one copy, a vtable |
| `write_listing(&mut impl Out)` | `put_map(&mut impl Sink)` | static dispatch: one copy per type |
| — | `put_usize` | digits by hand; why `ulib::write_usize` exists |
| `Scheduler::pick_next(&[ProcState])` | `Allocator::pick(&[bool])` | the policy sees the least it needs |
| `run_for` (default) | `take`, `take_many` (defaults) | the loop every policy inherits |
| `RoundRobin { next }` | `NextFit { next }` | a cursor that resumes after |
| `Priority` (stateless) | `FirstFit` (a unit struct) | remembers nothing |
| `trace<S, O>` writes names | `trace<A, S>` writes numbers | two bounds, every combination |

Deliberate differences, so it is not find-and-replace — and so that walking
through this does not hand over the exercise:

- **The sink speaks bytes.** `put(&[u8])`, not `write_str(&str)`, so a number
  has to become digits by hand (`put_usize`) — which is exactly why `ulib`
  ships `write_usize` and why `core::fmt` stays out of user programs. Friday's
  `10c` is written against this shape.
- **The default method mutates the table.** `run_for` only reads `states`;
  `take` marks `free[i] = false`. So `pick` takes `&[bool]` and `take` takes
  `&mut [bool]`. Ask why the *policy* is handed the read-only view: the
  smallest input that lets it work is a security property.
- **The stateless policy is zero bytes.** `size_of::<FirstFit>() == 0`; the
  exercise's `Priority` carries a `Vec<u8>`. A unit struct exists only to hang
  an `impl` on — the same trick `10c`'s `Args` façade does not need but the
  kernel's device drivers use constantly.
- **`NextFit`'s cursor is a speed choice; `RoundRobin`'s is correctness.** A
  frame just handed out is now `false`, so rescanning from 0 would also be
  right, only slower. Remove round-robin's cursor and one slot runs forever.
  Ask which is which; the tests only pin down one of them.

## Running the demo

`cargo run` walks four sections in order. The part to slow down for is the
third:

```
  FirstFit   table after    NextFit    table after
  Some(3)    #####...       Some(3)    #####...
             .####...                  .####...   <- frame 0 freed
  Some(0)    #####...       Some(5)    .#####..
```

Same trait, same `take`, same table. Only `pick` differed, and `take` — which
neither policy contains a line of — did the marking both times.

## The four things to say out loud

1. **A default method is compiled against a `self` that does not exist yet.**
   `put_line` calls `put` on types nobody has written. That is why a fifth
   sink costs zero lines, and why the only way to break it is to skip the
   required method (E0046).
2. **The bound is the contract, read in both directions.** `<S: Sink>` is what
   the body may assume and what the caller must prove. *"No method named `put`
   found for type parameter `S`"* is not a missing `use`; it is a missing
   bound, and rustc's help text is the fix.
3. **`dyn` costs a word and a jump, and buys heterogeneous storage.**
   `&dyn Sink` is 16 bytes: a pointer and a vtable pointer. One indirect call
   per method, no inlining across it. In exchange, three different sinks fit
   in one array. Generics cannot do that; `dyn` cannot be inlined. The shell
   chose `dyn`; the scheduler chose generics.
4. **`pick -> Option<usize>` is the same decision as `pick_next`.** "Nothing
   free" and "nothing runnable" are ordinary answers, not errors. The policy
   says `None`; the default method decides what to do about it.

## Live variations, if there is time

- Delete `S: Sink` from `trace`'s signature and read E0599. Then delete
  `A: Allocator` instead and read the same error for `take`. The uncomment
  block in `src/main.rs` is the two-line version.
- Add `fn reset() -> Self;` to `Allocator` and rebuild: `put_banner` is fine,
  but any `&mut dyn Allocator` becomes E0038 (*"not dyn compatible"*). A
  vtable cannot hold a method with no `self`, or one that returns `Self`.
- Write `put_banner(sink: &mut Sink)` without the `dyn` and read E0782
  (*"expected a type, found a trait"*) — edition 2021 requires the word, so
  that a trait object is never mistaken for a type.
- Make `NextFit::pick` resume *at* the frame it just handed out instead of
  after it. Every test still passes — then ask why, and contrast the exercise's
  `RoundRobin`, where the same change makes one slot run forever.
