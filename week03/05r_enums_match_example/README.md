# 05r Enums and `match` — in-class worked example

The `05r_enums_match` exercise has students build the **process lifecycle** as
an enum, a transition table, and an `Option`. This is the same code with a
different lifecycle: one slot of the **buffer cache**, which is filled from
disk, locked by a process, written through, flushed, and evicted. Walk through
this one on screen, then send them off to the exercise, where the technique is
already familiar and only the states are new.

```sh
cargo run --bin basic_enum   # START HERE: one enum, one match, one Option
cargo test                   # ten tests, the same kind of contract as the exercise
cargo run                    # the buffer-cache demo, with an E0004 to uncomment
```

## Start with `src/bin/basic_enum.rs`

Before the buffer cache, the concept on its own: a `Signal` with four cases,
two of which carry data.

| step | what it shows |
|---|---|
| 1 | an enum is a **closed set** — there is no `Signal` equal to 47 |
| 2 | `match` is an **expression**, one arm per variant |
| 3 | the four pattern features: `\|`, `{ .. }`, a binding, and `_` |
| 4 | `Option`, because not every signal can be caught |
| 5 | E0004, uncommented live |

The payoff is step 5. Delete the `User` arm and rustc says
`non-exhaustive patterns: `Signal::User(_)` not covered`. Read it out loud, then
make the point that matters: the compiler is not being fussy about syntax, it is
naming a case nobody has decided about. Replace it with `_ => false` and the
error goes away — and so does every future warning about every variant not
thought of yet.

## The mapping

| exercise (`skeleton/lib.rs`) | here (`src/lib.rs`) |
|---|---|
| `ProcState` — 5 variants, 2 carrying data | `BufState` — 4 variants, 3 carrying data |
| `Event` — 7 variants | `BufEvent` — 6 variants |
| `can_run` — one arm per variant, no `_` | `holds_data` |
| `next_state` — 7 rows, `match (state, event)` | `next_state` — 6 rows, same shape |
| the guard `if c == w` on a channel | the guard `if b == by` on the holder |
| `exit_status` via `if let` | `holder` via `if let` |
| `apply` (given) | `step` (given) |

Deliberate differences, so it is not find-and-replace:

- **The table is shorter and the graph is a different shape.** The process
  lifecycle is a loop through five states; the cache lifecycle branches — a
  buffer can be locked, written, flushed, and evicted in more than one order.
  Students who try to rename their way from one to the other will get a table
  that does not match the exercise's tests.
- **Three variants carry the same field.** That makes `cached_block` a `match`
  where the exercise's `exit_status` is an `if let`, and it is a good moment to
  ask *which shape fits which question*: `if let` when one variant matters,
  `match` when several do.
- **The rule the table enforces is a safety property you can state.** A dirty
  buffer must never be evicted, because that loses a write; a locked one must
  never be evicted, because somebody is using it. Every missing arrow has a
  sentence like that behind it — which is exactly what to ask students for
  about the process table.

## Running the demo

`cargo run` walks the lifecycle one event at a time, then shows the guard:

```
  Unlock { by: 3 } -> Some(Clean { block: 42 })    the holder
  Unlock { by: 4 } -> None                         somebody else
```

The arm's *pattern* matches both times. On the second the *guard* fails, and
matching **continues with the next arm** rather than leaving the `match` — so
it falls through to `_ => None`. That fall-through is the whole point, and it
is what a process waiting on the console's channel relies on when the disk
wakes its own sleepers.

## The four things to say out loud

1. **An enum is a closed set, and `match` is how you prove you covered it.**
   `holds_data` has one arm per variant and no `_`. Add a fifth `BufState` and
   the compiler hands you every place that now needs a decision — by file and
   line, before anything runs.
2. **`_` switches that off forever.** A catch-all covers variants that do not
   exist yet. Contrast `holds_data` (no `_`, closed domain we own) with
   `next_state`'s `_ => None` (honest: the pairs it covers are enumerable and
   we mean all of them). The rule: `_` for open domains, never for closed ones
   you defined yourself.
3. **A failing guard falls through; it does not exit the match.** Rewrite the
   unlock arm with the test inside the body and the `else` has to reproduce
   what the fall-through would have done. In a real transition table every
   guard failure restates the default by hand.
4. **`None` is a value the caller cannot ignore.** `next_state` returns
   `Option<BufState>`, so "that move is illegal" has a type. `step` is the only
   place it is opened up, and it is a two-arm `match` — `.unwrap_or(state)` is
   the same thing spelled shorter.

## Live variations, if there is time

- Drop the `if b == by` guard from the unlock arm and re-run. Nine tests still
  pass; only `only_the_process_holding_a_buffer_may_unlock_it` goes red — any
  process can now release a buffer somebody else is using.
- Move `_ => None` to the **top** of the `match`. It compiles, every test goes
  red, and rustc emits four `unreachable pattern` warnings. `match` takes the
  first arm that fits, so a catch-all placed early swallows everything below it.
- Delete one arm from `holds_data` and read E0004 together. Then "fix" it with
  `_ => false` and ask what was just given up.
- Add a fifth `BufState` — say `Reading { block: u32 }`, for a buffer whose
  disk read is still in flight — and let the compiler produce the to-do list.
  Exactly **two** functions stop compiling: `holds_data` and `cached_block`,
  the two with one arm per variant. `must_be_flushed_first` keeps building
  because `matches!` has the same blind spot as `_`, and — the one worth
  dwelling on — **`next_state` keeps building too**, silently deciding via
  `_ => None` that nothing legal can ever happen to a `Reading` buffer. That is
  the honest `_` from point 2 turning into the dangerous one the moment the
  domain stops being closed. Ask them where the same trap sits in the process
  table.
