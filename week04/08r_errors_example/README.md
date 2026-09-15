# 08r Errors — in-class worked example

The `08r_errors` exercise has students build a **directory** you look names up
in, with three ways a read can fail and a system-call edge where the failure
becomes a number. This is the same code with different nouns: a **registry of
devices** — `console`, `disk`, `null`, `zero` — with four ways a read can fail
and the same edge. Walk through this one on screen, then send them off to the
exercise, where the shape is already familiar and only the domain is new.

```sh
cargo run --bin basic_result   # START HERE: one Option, one Result, one `?`
cargo test                     # ten tests, the same kind of contract as the exercise
cargo run                      # the device-registry demo, with an error to uncomment
```

## Start with `src/bin/basic_result.rs`

Before the registry, the concept on its own: a table of free page frames, a
question that may have no answer, a request that may be refused, and the
operator that chains two requests.

| step | what it shows | the line to point at |
|---|---|---|
| 1 | `Option`: a question whose honest answer may be nothing | `first_free(&[false; 2]) -> None` — not an error |
| 2 | `Result`: a request, refused with a reason | `first_free(free).ok_or(AllocError::OutOfFrames)` |
| 3 | `match` opens each one; every arm is listed | `Err(AllocError::OutOfFrames) =>` |
| 4 | `?` chains two requests; the first `Err` returns early | `alloc_two` on a table with one frame left |
| 5 | the error | returning `first_free(free)` from a `Result` function → E0308 |

The payoff is the last one. An `Option` is not a `Result`, and rustc's
suggestion — *"try wrapping the expression in `Ok`"* — would be wrong. Nobody
said what `None` *means* here. `.ok_or(AllocError::OutOfFrames)` is that
decision, in one word, and it is yours to make.

## The mapping

| exercise (`skeleton/lib.rs`) | here (`src/lib.rs`) | kernel role |
|---|---|---|
| `NAME_MAX` (14) | `DEV_NAME_MAX` (8) | a fixed-size record's limit |
| `FsError` (3 variants) | `DevError` (4 variants) | one variant per way to fail |
| `InodeKind::{File, Dir}` | `DevKind::{Char, Block}` | the kind a read cares about |
| `Inode { inum, kind, size }` | `Device { major, kind }` | the record a name resolves to |
| `FileSystem` | `Registry` | private fields; everything is a method |
| `find -> Option<Inode>` | `find -> Option<Device>` | absence |
| `lookup -> Result<Inode, FsError>` | `lookup -> Result<Device, DevError>` | the first `.ok_or`: absence becomes failure |
| `read(inode) -> Result<&[u8], _>` | `getc(dev) -> Result<u8, _>` | the kind check, and a second `.ok_or` |
| `read_file` | `getc_by_name` | the `?` chain |
| `sys_read -> i64` | `sys_getc -> i64` | the boundary: one `match`, real errnos |

Deliberate differences, so it is not find-and-replace — and so that walking
through this does not hand over the exercise:

- **Absence becomes failure at two sites, with two policies.** `lookup` turns
  `None` into `NoSuchDevice`; `getc` turns an empty input queue — `.first()`
  is an `Option<&u8>` — into `WouldBlock`. Same operator, different decision.
  That is why the enum has four variants, not three, and why `push_input` can
  make `WouldBlock` disappear without touching the registry.
- **The kind check refuses the other kind.** Reading a block device byte-wise
  is `NotATty` (ENOTTY, 25 — "not a typewriter"), where the exercise refuses a
  directory with EISDIR. Four different real errno numbers, so nobody copies a
  table across.
- **Success is a value, not a length.** `sys_getc` returns the byte, so `zero`
  returns `0` and that is success. Ask why a syscall can return 0 on success
  but never −0 on failure, and what that says about the sign convention.
- **The registry is built from a fixed list; only the queue varies.** `new()`
  installs four devices. The demo delivers a byte to `null` and watches the
  same `getc` call change its answer.

## Running the demo

`cargo run` walks four sections in order. The part to slow down for is the
second:

```
getc_by_name(console   ) -> Ok(108)                passed both
getc_by_name(eightchr  ) -> Err(NoSuchDevice)      stopped at lookup(name)?
getc_by_name(ninechars ) -> Err(NameTooLong)       stopped at lookup(name)?
getc_by_name(disk      ) -> Err(NotATty)           stopped inside getc(dev)
getc_by_name(null      ) -> Err(WouldBlock)        stopped inside getc(dev)
```

`getc_by_name` is two lines. Five names, four different exits, and the
function never mentions any of them: `?` returned whichever `Err` it met.

## The four things to say out loud

1. **`find` is a fact; `lookup` is a decision; `.ok_or` is the line between.**
   A name that is not there is the honest answer to a question. A call that
   needed the name has failed. Keep the two apart and the policy is visible
   in exactly one place.
2. **`?` returns from the function you are in.** So it can live only in a
   function that returns `Result` (or `Option`). That is not a restriction to
   work around; it is the shape of every kernel function that touches a
   device or a file, and `sys_getc` is where the shape ends.
3. **The boundary is one `match`, and it is the only place errno is spelled.**
   Everything behind it speaks `DevError`; everything in front of it speaks
   integers. List every variant, never `Err(_)`: when 41k adds one, this
   `match` is where the compiler makes you choose its number.
4. **A panic is one red test here and a dead machine on rv6.** `expect` when
   the previous line proved it is `Ok`; `unwrap_or`, `if let`, `match`
   otherwise. `Result` for what the world did to you; panic for what you did
   to yourself.

## Live variations, if there is time

- Add `Busy` to `DevError` and rebuild: `sys_getc` refuses to compile until
  it has a number. Then replace its four `Err` arms with `Err(_) => -1` and add
  `Busy` again — nothing complains. That silence is what `_` costs.
- Replace the `?` in `getc_by_name` with `.unwrap()` and run the tests:
  `getc_by_name_passes_the_first_error_straight_up` panics on the first
  absent name. Read the panic message together; that is the machine dying.
- Make `lookup` skip the length check and run the tests: three go red, each
  with `NoSuchDevice` (or `-19`) where `NameTooLong` (or `-36`) was expected.
  The caller still learned something was wrong, just not *what*. Ask whether
  that is good enough for a syscall. (It is not: ENOENT and ENAMETOOLONG tell
  a shell to do different things.)
- Uncomment the `match` in `src/main.rs` and read E0004 on a `Result` — the
  same exhaustiveness check as last week's enums, because `Result` is one.
