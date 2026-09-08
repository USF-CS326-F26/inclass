# 04r Structs and `impl` — in-class worked example

The `04r_structs_impl` exercise has students build a **region of physical
memory**, a **page table entry**, and a **guard that puts its page back**. This
is the same code with different nouns: a **run of disk blocks**, a **device
number**, and a **guard that turns interrupts back on**. Walk through this one
on screen, then send them off to the exercise, where the shape is already
familiar and only the domain is new.

```sh
cargo run --bin basic_struct   # START HERE: one struct, one impl block, nothing else
cargo test                     # eleven tests, the same kind of contract as the exercise
cargo run                      # the disk/device/interrupt demo, with an error to uncomment
```

## Start with `src/bin/basic_struct.rs`

Before the extents and the device numbers, the concept on its own: a `Counter`
with two fields, and the three shapes a function in an `impl` block can take.

| function | receiver | what it does with ownership |
|---|---|---|
| `Counter::new(label)` | none | an **associated function**: makes a value, called with `::` |
| `value(&self)` | `&self` | borrows to read — call it as often as you like |
| `bump(&mut self)` | `&mut self` | the **exclusive** borrow; nobody else is looking while it writes |
| `into_total(self)` | `self` | **consumes** the Counter; the binding is struck off afterwards |

The payoff is the fourth one. After `let total = hits.into_total();` the name
`hits` is unusable, because the value really was given away. The last block of
the file is commented out; uncomment it on screen and read E0382 together —
including rustc's note, *"`Counter::into_total` takes ownership of the receiver
`self`, which moves `hits`"*, which is last week's lesson arriving in a method.

One line worth pausing on: `is_zero(&self)` calls `self.value()`. A method
calling another method on the same value is exactly the shape of `is_valid`
in the exercise, and it starts working the moment the method under it does.

## The mapping

| exercise (`skeleton/lib.rs`) | here (`src/lib.rs`) | kernel role |
|---|---|---|
| `PAGE_SIZE` | `BLOCK_SIZE` | the grain everything is counted in |
| `page_align_up(addr)` | `blocks_for(bytes)` | rounding up to a whole unit |
| `MemRegion { start, end }` | `Extent { first, count }` | a run of storage |
| `contains` / `size` | `contains` / `bytes` | the two questions you ask a run |
| `of_pages(start, pages)` | `of_bytes(first, bytes)` | the associated function that makes one |
| `page_list()` | `blocks()` | enumerate the whole units inside it |
| `Pte(u64)` | `DevNo(u32)` | the newtype over packed bits |
| `new` / `pa` / `flags` | `new` / `major` / `minor` | pack, then unpack each field |
| `is_valid` | `is_console` | a method calling another method |
| `PageGuard<'a>` | `IntrGuard<'a>` | the guard: acquire, `release(self)`, `Drop` |
| `Context` (`#[repr(C)]`) | `Superblock` (`#[repr(C)]`) | bytes something else reads |

Deliberate differences, so it is not find-and-replace — and so that walking
through this does not hand over the exercise:

- **Rounding up is a division here, and a bit mask there.** `blocks_for` is
  `(bytes + BLOCK_SIZE - 1) / BLOCK_SIZE`, which works for any divisor at all.
  The exercise wants the mask form instead, which is available to it only
  because a page size is a power of two — and which the exercise's own skeleton
  walks you up to. Good place to ask *why* one of them has a shortcut the other
  does not, without writing either on the board.
- **`Extent` stores a count; `MemRegion` stores an end.** Both describe a run,
  and each makes a different question free. With a count the loop in `blocks()`
  is a plain range and has no edge case; with an end, the loop condition *is*
  the exercise. Ask which question each shape makes cheap.
- **The packing is one shift, not two.** A device number puts the major
  straight up at bit 20. A page table entry has to shift the address *down*
  first, to drop the twelve bits a page boundary always has as zero, and only
  then up. Same four operations — shift, shift, OR, mask — different layout.
- **The guard restores a state rather than returning a page.** Which is the
  more common kernel shape: a critical section turns interrupts off, does its
  work, and must turn them back on down every path out.

## Running the demo

`cargo run` walks the three sections in order. The part to slow down for is the
third:

```
interrupts on before      -> true
  inside the critical section:
    g.intr_on()           -> false
    g.was_on()            -> true   what it will restore
interrupts on after       -> true   nobody called release()
```

Nothing was called at that closing brace. The compiler emitted the call,
because that is where the guard's owner stopped existing.

## The four things to say out loud

1. **Choosing the receiver IS the ownership decision.** `&self` reads and hands
   the value back, `&mut self` writes under an exclusive borrow, `self`
   consumes. `Copy` is what lets a small plain-data type get away with the
   third — drop the derive from `DevNo` and `is_console` stops compiling,
   because `major(self)` moved the value away.
2. **The order inside `acquire` is the trap.** Read and write through the
   `&mut` **first**; only then move it into the struct literal. After that line
   the guard holds the only path to the `Cpu`. This is the exact mistake
   students make in the exercise's constructor, in different clothes — E0499 or
   E0502 depending on how they wrote it.
3. **The guard restores what it FOUND, not what it assumes.** A `Drop` that
   just switched interrupts on would switch them on inside an outer critical
   section that had turned them off. `was_on` is three extra characters and the
   difference between a guard and a bug. Show the nesting demo.
4. **`Copy` and `Drop` are mutually exclusive, and that is the double free
   again.** `DevNo` derives `Copy` and has no `Drop`; `IntrGuard` has a `Drop`
   and can never be `Copy`. Try adding the derive on screen and read E0184.

## Live variations, if there is time

- Delete `#[repr(transparent)]` from `DevNo` and re-run the tests. Nothing
  breaks — then ask what *would* break, and point at `[Pte; 512]` needing to be
  exactly one 4096-byte page.
- Change `Extent::blocks` to `(0..=self.count)` and watch
  `an_extent_lists_its_blocks_in_order` go red with one block too many. That is
  the exercise's loop-condition bug, in the one place this version can still
  have it.
- Make `drop` set `intr_on = true` unconditionally instead of restoring
  `was_on`. Ten tests still pass; only
  `a_guard_restores_the_state_it_found_not_a_guess` goes red — because it is
  the only one that starts with interrupts already off. Ask what the other ten
  were failing to notice.
- Add `#[derive(Clone, Copy)]` to `IntrGuard` and read E0184 together.
