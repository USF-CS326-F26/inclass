### Thursday, Oct 8

**31k_boot**
1. [01 · the six instructions at 0x1000](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#01_the_boot_rom/s2): what ran before `_entry`, and [the words the ROM loads](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#01_the_boot_rom/s3), including the jump target `0x8000_0000`
2. [02 · one item of each kind, in address order](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#02_where_the_linker_put_it/s1): where `link.ld` put `.entry`, `.text`, `.rodata`, `.data` and `.bss`
3. [02 · end is an address, not a value](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#02_where_the_linker_put_it/s3), then [E0133](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#e0133) and [E0606](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#e0606): reading a symbol the linker defines
4. [03 · where sp starts](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#03_the_boot_stack/s1) and [each call moves it down](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#03_the_boot_stack/s2): the top of the stack, and which way it grows
5. [03 · nothing guards the low end](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#03_the_boot_stack/s4): what a stack pointer at the wrong end does, with no message
6. [04 · reset to Rust](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#04_reset_to_rust): the boot order, step by step, with the evidence each step left

**32k_physical_memory**
1. [05 · page number and offset](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#05_page_number_and_offset/s1): `>> 12` and `& 0xFFF` on real addresses
2. [06 · a request, in whole pages](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#06_why_one_page_size/s1): why one block size, and what it wastes
3. [07 · one store, through a cast](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#07_a_page_holds_an_address/s2): a free page holding another page's address, then [E0614](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#e0614) for an address that is not yet a pointer
4. [08 · round __stack_top up to a page](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#08_where_free_memory_starts/s2): add `0xFFF`, clear the low 12 bits, and [two ways to get it wrong](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#08_where_free_memory_starts/s3)
5. [08 · whole pages up to 0x8800_0000](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#08_where_free_memory_starts/s4): counting the pages, from both ends, then [E0369](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#e0369) for pointer arithmetic

### Friday, Oct 9

**33k_paging**
1. [09 · four moves](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#09_bits_by_hand/s1): shift, mask, extract, pack, and [a FAT date](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#09_bits_by_hand/s2)
2. [09 · Rust's operators are not C's](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#09_bits_by_hand/s4): `!` for NOT, and how `&` binds, then [`~`](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#tilde_is_not_an_operator) and [E0308](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#e0308)
3. [10 · the lecture's address, by hand](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#10_split_the_address/s1): VPN[2], VPN[1], VPN[0] and the offset, and [what one entry covers](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#10_split_the_address/s2)
4. [10 · the top: MAXVA](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#10_split_the_address/s4): TRAMPOLINE, and why rv6 stops at bit 38
5. [11 · encode 0x8123_4000 as V R W](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#11_pte_by_hand/s1), then [decode it](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#11_pte_by_hand/s2): a PTE by hand, both ways
6. [11 · two slips](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#11_pte_by_hand/s3): the decodes that look right and are not
7. [12 · pack satp by hand](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#12_satp_reads_zero/s1), and [the real satp](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#12_satp_reads_zero/s3): paging is still off
8. [13 · X W R: the eight combinations](http://cs326-f26.cs.usfca.edu/inclass/week07-examples.html#13_w_xor_x/s2): W xor X, and the two reserved encodings
