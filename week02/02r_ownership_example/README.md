# 02r Ownership — in-class worked example

The `02r_ownership` exercise has students hand-build a **page allocator** out of
a `Vec<usize>` of free page numbers. This is the same code with a different
`Vec`: a **pool of free process IDs**, the way `fork` and `wait` hand PIDs out
and take them back. Walk through this one on screen, then send them off to the
exercise, where the shape is already familiar and only the domain is new.

```sh
cargo run --bin basic_move   # START HERE: one Vec, three functions, nothing else
cargo test                   # the same seven-test contract, rewritten for PIDs
cargo run                    # the PID demo, with a move error to uncomment
```

## Start with `src/bin/basic_move.rs`

Before the PID pool, the concept on its own: a `Vec<i32>` of test scores and
three functions that do the only three things a function can do with a value it
is handed by value.

| function | what it does with ownership |
|---|---|
| `make_scores() -> Vec<i32>` | makes it, moves it **out** to the caller |
| `add_score(mut scores, s) -> Vec<i32>` | moves **in**, changes it, moves back **out** |
| `total(scores) -> i32` | moves **in** and keeps it — the Vec is dropped at that function's closing brace |

The payoff is the third one. After `let sum = total(scores);` the binding
`scores` is unusable, because the buffer really was freed inside `total`. The
last line of the file is commented out; uncomment it on screen and read
E0382 together — including rustc's own note suggesting `total` should have
borrowed instead, which is next week in one sentence.

One line worth pausing on: the file prints `scores.as_ptr()` before and after a
round trip through `add_score` and the address is identical. A move copies the
three-word handle, never the elements. Moves are cheap.

## The mapping

| exercise (`solution/lib.rs`) | here (`src/lib.rs`)   | kernel role       |
|---|---|---|
| `NO_PAGE`                    | `NO_PID`              | the sentinel      |
| `new_free_list(count)`       | `new_pid_pool(count)` | `init` at boot    |
| `take_page(free)`            | `claim_pid(pool)`     | `kalloc` / `fork` |
| `give_back(free, page)`      | `release_pid(pool, pid)` | `kfree` / `wait` |
| `measure_label(label)`       | `measure_command(name)`  | the moved `String` |

Deliberate differences, so it is not pure find-and-replace:

- **Pages start at 0, PIDs start at 1** — PID 0 is the kernel's own scheduler
  process. Ownership does not care; the test does. Good place to ask what
  `new_pid_pool(0)` returns and why the loop condition is `<=` here and `<`
  there.
- The failure story is different but the shape is identical: an exhausted
  process table is `fork` returning -1, the same way an empty free list is
  `kalloc` returning null.

## Running the demo

`cargo run` prints the pool after every operation, so the moves are visible as
they happen:

```
spawned sh   as pid 4   free pids: [1, 2, 3]
spawned ls   as pid 3   free pids: [1, 2]
spawned grep as pid 2   free pids: [1]
```

Three live processes, three different PIDs, and none of them still in the pool.
Nobody had to *remember* to remove them — `remove` moved the number out of the
only pool that exists, because `claim_pid` owned it outright while it ran.

## The four things to say out loud

1. **Every `let (pool, pid) = claim_pid(pool);` is a move out and a move back
   in.** The `pool` on the left is a new binding shadowing a dead one. Ask what
   happens if you use the old one — then show them (bottom of `src/main.rs`).
2. **`claim_pid` returns the pool even when it fails.** It owns the pool at that
   moment; dropping it would free the caller's process table. This is the line
   students most often get wrong in the exercise.
3. **The PID is copied, the pool is moved.** `usize` is `Copy`, `Vec<usize>` and
   `String` are not. That is the whole reason `release_pid(pool, pid)` leaves
   `pid` usable and `measure_command(name)` does not leave `name` usable.
4. **This is clumsy on purpose.** Handing a value back just so the caller can
   keep using it is exactly the pain that borrowing (`&`) removes in 03r. Let
   them feel it for one exercise first.

## Live variations, if there is time

- Change `pool.remove(last)` to `pool.remove(0)` and re-run the tests: FIFO PIDs
  instead of LIFO, and three tests go red — `claiming_a_pid_removes_it_from_the_pool`,
  `releasing_a_pid_makes_it_available_again`, and `a_pid_is_copied_not_moved`.
  Note which four still pass: ownership is untouched by this change, only the
  order is. Ask why the end of a `Vec` is the cheap end.
- Delete the `mut` from a parameter and read E0596 together.
- Write `(name, name.len())` in `measure_command` and read E0382 together — the
  tuple takes ownership before `.len()` is reached.
