### Thursday, Sep 17

**06r_collections**
1. [02 · iter_mut(): the write goes in](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#02_iter_mut_and_enumerate/s4): the `iter_mut().enumerate()` loop and the `*slot = …` write that `alloc_slot` needs
2. [E0506 · assign while iterating](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#e0506): why `table[i] = …` inside `table.iter()` won't compile
3. [03 · iget fills the lowest free slot](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#03_slot_search/s2): the lowest free slot, returned as `Option<usize>` ([full table → `None`](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#03_slot_search/s5))
4. [04 · the same thing as a loop](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#04_adapters_and_closures/s9): `for` / `if let` / `push`, the shape of `live_pids`
5. [E0596 · iter_mut behind a shared ref](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#e0596): the signature decides whether you may write
6. [01 · one function, three containers](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#01_array_slice_vec): why every function takes a slice when the tests pass arrays, and array vs. `Vec`

**07r_traits**
1. [05 · three implementers, each wrote ONLY putc](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#05_trait_required_default/s1): markers 1–2 (`Recorder` ≈ `StringOut`, `Tally` ≈ `CountingOut`)
2. [05 · a default is written once](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#05_trait_required_default/s2): why `write_line` works as soon as `write_str` does
3. [06 · one body, three names](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#06_generics_and_bounds/s1): a bounded generic function like `trace`
4. [06 · what the bound buys](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#06_generics_and_bounds/s3), then [E0599](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#e0599): "no method named … for type parameter" means a missing bound
5. [06 · three spellings, one meaning](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#06_generics_and_bounds/s2): `<O: Out>`, `where`, and `impl Out`
6. [07 · the same three, through one body](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#07_static_vs_dyn/s2), then [versus the generic](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#07_static_vs_dyn/s3): `&mut dyn Out` (`write_banner`) vs. `impl Out` (`write_listing`)

### Friday, Sep 18

**08r_errors**
1. [08 · the line between them](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#08_option_vs_result/s3): `.ok_or(…)` turns absence into failure, which is `lookup`'s last line
2. [E0308 · Option is not Result](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#e0308): returning `find`'s `Option` from `lookup`, and the fix
3. [09 · each ? is a different exit](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#09_error_enum_and_question_mark/s1): `load` is `read_file` with three steps
4. [09 · longhand: what ? expands to](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#09_error_enum_and_question_mark/s2): if you wrote a `match` in `read_file`, collapse it
5. [09 · ? needs somewhere to return to](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#09_error_enum_and_question_mark/s3), then [E0277](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#e0277): why `sys_read` uses `match`, not `?`
6. [10 · at the wall: sys_read speaks i64](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#10_errno_boundary/s2): the one `match` that turns a `Result` into a negative errno
7. [10 · the lossy wall](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#10_errno_boundary/s3): what `Err(_) => -1` throws away (spell out each variant)

**10c_echo**
1. [11 · a byte-string literal](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#11_bytes_vs_strings/s2): `b" "` is bytes and `" "` is not
2. [12 · a console that takes three bytes at a time](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#12_argv_and_write_all/s4): why `write_all` and not `write`
3. [12 · argv, as the program sees it](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#12_argv_and_write_all/s1): `argv[0]` is the name, and `get` past the end is `None`
4. [12 · the Result from write_all](http://cs326-f26.cs.usfca.edu/inclass/week04-examples.html#12_argv_and_write_all/s5): why it's `let _ = write_all(…)` inside `run`
