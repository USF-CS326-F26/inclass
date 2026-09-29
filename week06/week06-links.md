### Thursday, Oct 1

**20a_asm_bridge**
1. [01 · ra: where a ret will go](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#01_registers/s2): `call` writes `ra`, `ret` jumps to it, and [`ret` is `jalr zero, 0(ra)`](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#01_registers/s4)
2. [01 · sp: the stack](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#01_registers/s1): it grows down, and it is a multiple of 16
3. [02 · Rust calls assembly](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#02_calling_assembly/s1): `global_asm!`, `.globl`, and the `extern "C"` line that says which register holds what
4. [02 · it stops at the NUL](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#02_calling_assembly/s2): a byte loop with `1b`/`2f` local labels, and how it knows where to stop
5. [03 · the same byte, loaded two ways](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#03_lb_vs_lbu/s1): `lb` against `lbu`, and [a length byte read with the wrong one](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#03_lb_vs_lbu/s2) (Midterm 1)
6. [04 · span_end reads offsets 0 and 8](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#04_repr_c/s1): `#[repr(C)]` fields at the offsets `ld` names, and [what an inserted field does](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#04_repr_c/s3)
7. [05 · the hardware add](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#05_add_wraps/s1): `add` wraps where Rust's `+` panics
8. [06 · the frame framed built](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#06_prologue_and_s0/s1): a 16-byte frame holding `ra` and `s0` (Midterm 1)
9. [06 · a callee that skips the save](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#06_prologue_and_s0/s3): callee-saved, and who notices when it is not
10. [07 · resume: load sp, load ra, ret](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#07_a_ret_that_lands_elsewhere/s2): a `ret` that lands on another stack, in a function nobody called

### Friday, Oct 2

**21r_unsafe_bridge**
1. [08 · the counter moves on its own](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#08_mmio_clock/s1): a device register read through a raw pointer
2. [08 · .add counts elements, not bytes](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#08_mmio_clock/s2): pointer arithmetic, with nothing dereferenced
3. [09 · two plain stores to the UART](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#09_volatile_matters/s1), then [two volatile stores](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#09_volatile_matters/s2): the byte the optimizer dropped
4. [09 · 100,000 plain loads](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#09_volatile_matters/s3), then [volatile](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#09_volatile_matters/s4): the load hoisted out of the loop
5. [10 · a safe wrapper](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#10_unsafe_does_not_turn_off/s1): one `unsafe` block inside a function whose signature makes the promise
6. [10 · bounds checks still run](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#10_unsafe_does_not_turn_off/s2): what `unsafe` does not turn off, then [E0133](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#e0133) and [E0308](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#e0308)

**30k_kernel_basics**
1. [E0463 · forgot no_std](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#e0463): the target has no `std`, and the `!` matters
2. [E0601 · forgot no_main](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#e0601): rustc wants a `main` nobody would call
3. [panic handler required](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#panic_handler_required), then [E0152 · two of them](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#e0152): exactly one, program-wide
4. [13 · one that overflows](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#13_the_panic_handler/s2): a handler written out, and the `!` that says it never returns
5. [12 · where this program lives](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#12_who_calls_main/s1): `_entry` at `0x8000_0000`, and [who called main](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#12_who_calls_main/s2)
6. [11 · formatting, with no heap](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#11_what_core_still_has/s2): what `core` still has, and [E0433](http://cs326-f26.cs.usfca.edu/inclass/week06-examples.html#e0433) for what it does not
