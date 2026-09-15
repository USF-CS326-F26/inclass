// ╔══════════════════════════════════════════════════════════════════════╗
// ║  06r — Arrays, Slices, and Vec — IN-CLASS EXAMPLE                    ║
// ║  Same shape as the exercise, different nouns: a per-process table    ║
// ║  of open files instead of the kernel's table of processes. The slot  ║
// ║  index handed back is the number `open` returns to the user program. ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/06r_collections/skeleton/lib.rs`. Every item
// here has a twin there. If you can explain why the twins are the same shape,
// you have understood fixed tables; the domain is decoration.
//
// The slots here hold a RECORD, not a bare number, and the list at the end is
// built with an adapter chain rather than a push loop. You still have to write
// the exercise's two loops yourself.

/// The most files one process may hold open at once.
///
/// UNDERSTAND: xv6's NOFILE is 16; 8 keeps a full table small enough to print
///   on one line. It plays the role `NPROC` plays in the exercise: a hard limit
///   chosen once, at compile time. When the table is full `open` fails -- and
///   that is the design, because the alternative is a table that grows on the
///   trap path, where there is no allocator to grow it with.
pub const NOFILE: usize = 8;

/// One open file, as the per-process table sees it.
///
/// UNDERSTAND: the exercise's slot holds a `u32` pid and nothing else. This
///   slot holds a record with two fields, which changes two things: an empty
///   slot is `Option<OpenFile>` rather than `Option<u32>`, and reading the
///   inode number out of a slot means reaching INTO the record (`f.inum`).
///   `Copy` so that a slot can be duplicated by plain assignment -- which is
///   exactly what `dup` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenFile {
    /// Which file this descriptor refers to.
    pub inum: u32,
    /// How far into it the next read will start.
    pub offset: usize,
}

/// A brand-new descriptor table with every slot free.
///
/// UNDERSTAND: `[None; NOFILE]` repeats one value NOFILE times, and the whole
///   thing is `NOFILE * size_of::<Option<OpenFile>>()` bytes of plain memory
///   -- no allocator, no header, no pointer to anywhere. Twin of `new_table`.
pub fn new_fd_table() -> [Option<OpenFile>; NOFILE] {
    [None; NOFILE]
}

/// Open `inum`: put a record in the lowest-numbered free slot and return that
/// slot's index. That index IS the file descriptor.
///
/// UNDERSTAND: twin of `alloc_slot`, and the same loop: `iter_mut()` hands
///   out one `&mut Option<OpenFile>` per slot, in turn; `enumerate()` brings
///   the index along; `*slot = Some(..)` writes through the reference into the
///   caller's array. "Lowest free" is not a preference here -- the shell relies
///   on it. See `dup_fd`.
pub fn open_fd(table: &mut [Option<OpenFile>], inum: u32) -> Option<usize> {
    for (fd, slot) in table.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(OpenFile { inum, offset: 0 });
            return Some(fd);
        }
    }
    None
}

/// The lowest descriptor currently open on `inum`, if any.
///
/// UNDERSTAND: twin of `find_pid`, and the read-only twin of `open_fd`. It
///   takes `&[..]`, so `iter()` is the only loop it may write -- `iter_mut`
///   here would be E0596. The signature decides the loop, not the body.
pub fn fd_of(table: &[Option<OpenFile>], inum: u32) -> Option<usize> {
    for (fd, slot) in table.iter().enumerate() {
        if let Some(f) = slot {
            if f.inum == inum {
                return Some(fd);
            }
        }
    }
    None
}

/// Close descriptor `fd`. Returns true only if something was open there.
///
/// UNDERSTAND: twin of `free_slot`. `fd` came from a user program, so it is
///   checked BEFORE it is used as an index: `table[fd]` on a bad index panics,
///   and in the kernel a panic is a dead machine. This is the check
///   `sys_close` makes in exercise 50k, and `close(99)` returning `-1` is it.
pub fn close_fd(table: &mut [Option<OpenFile>], fd: usize) -> bool {
    if fd >= table.len() {
        return false;
    }
    let was_open = table[fd].is_some();
    table[fd] = None;
    was_open
}

/// Duplicate descriptor `fd` into the lowest free slot; return the new one.
///
/// UNDERSTAND: no twin in the exercise -- this is why "lowest free" is the
///   rule. The shell redirects `cmd > out` by closing 1 and then opening
///   `out`: the new file lands in slot 1 BECAUSE the search starts at the
///   bottom. `.get(fd)` is the bounds check as an `Option`; `.copied()` turns
///   `Option<&Option<OpenFile>>` into `Option<Option<OpenFile>>`, and `?`
///   on each layer says "if there is nothing here, there is nothing to dup".
pub fn dup_fd(table: &mut [Option<OpenFile>], fd: usize) -> Option<usize> {
    let file = (*table.get(fd)?)?;
    for (new_fd, slot) in table.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(file); // a copy: OpenFile is Copy
            return Some(new_fd);
        }
    }
    None
}

/// Move descriptor `fd`'s offset forward by `n`. True if it was open.
///
/// UNDERSTAND: writing through a slot to a FIELD of the record inside it.
///   `.get_mut(fd)` is the bounds check as an `Option<&mut Option<OpenFile>>`;
///   `if let Some(Some(f))` opens both layers at once and leaves `f` as a
///   `&mut OpenFile` -- so `f.offset += n` lands in the table, not in a copy.
///   This is the shape of `f.off += n` in `sys_read`.
pub fn advance(table: &mut [Option<OpenFile>], fd: usize, n: usize) -> bool {
    if let Some(Some(f)) = table.get_mut(fd) {
        f.offset += n;
        true
    } else {
        false
    }
}

/// Every open inode number, in descriptor order.
///
/// UNDERSTAND: twin of `live_pids`, and it returns a `Vec` for the same
///   reason: this runs on your laptop, in a test, where there is a heap. The
///   exercise walks you to a `for` / `if let` / `push` loop; this is the same
///   walk as an adapter chain. `flatten` drops the `None`s and unwraps the
///   `Some`s, `map` reaches into each record, `collect` is the one link that
///   allocates. The kernel would walk the table in place and never collect.
pub fn open_inums(table: &[Option<OpenFile>]) -> Vec<u32> {
    table.iter().flatten().map(|f| f.inum).collect()
}

// ---------------------------------------------------------------------------
// The tests. Read them first: they are the contract, and they are the same
// kind of contract the exercise's tests are.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const CONSOLE: u32 = 1;
    const README: u32 = 7;

    #[test]
    fn a_fresh_table_has_no_open_files() {
        let table = new_fd_table();
        assert_eq!(table.len(), NOFILE);
        assert!(table.iter().all(|slot| slot.is_none()));
        assert!(open_inums(&table).is_empty());
        assert_eq!(fd_of(&table, CONSOLE), None);
    }

    #[test]
    fn the_table_is_one_fixed_lump_of_memory() {
        // Green from the start. N slots, back to back, no header, no pointer
        // elsewhere: what lets the kernel keep this table before any
        // allocator exists. The slot is bigger than the exercise's, because
        // it holds a record rather than a u32 -- see the demo for the number.
        assert_eq!(
            std::mem::size_of::<[Option<OpenFile>; NOFILE]>(),
            NOFILE * std::mem::size_of::<Option<OpenFile>>()
        );
    }

    #[test]
    fn three_console_opens_take_0_1_2_and_the_first_file_is_fd_3() {
        let mut table = new_fd_table();
        assert_eq!(open_fd(&mut table, CONSOLE), Some(0));
        assert_eq!(open_fd(&mut table, CONSOLE), Some(1));
        assert_eq!(open_fd(&mut table, CONSOLE), Some(2));
        // Which is why every program's first open() returns 3.
        assert_eq!(open_fd(&mut table, README), Some(3));
        assert_eq!(table[3], Some(OpenFile { inum: README, offset: 0 }));
        assert_eq!(table[4], None);
    }

    #[test]
    fn open_fails_when_every_descriptor_is_taken() {
        let mut table = new_fd_table();
        for fd in 0..NOFILE {
            assert_eq!(open_fd(&mut table, fd as u32 + 10), Some(fd));
        }
        assert_eq!(open_fd(&mut table, 99), None);
        // A refused open must not disturb what is already there.
        assert_eq!(table[NOFILE - 1].map(|f| f.inum), Some(NOFILE as u32 + 9));
    }

    #[test]
    fn a_closed_descriptor_is_the_next_one_handed_out() {
        let mut table = new_fd_table();
        for fd in 0..NOFILE {
            open_fd(&mut table, fd as u32 + 10);
        }
        assert!(close_fd(&mut table, 3));
        assert_eq!(table[3], None);
        assert_eq!(open_fd(&mut table, README), Some(3));
        assert_eq!(table[3].map(|f| f.inum), Some(README));
    }

    #[test]
    fn closing_twice_reports_nothing_was_open() {
        let mut table = new_fd_table();
        open_fd(&mut table, CONSOLE);
        assert!(close_fd(&mut table, 0));
        assert!(!close_fd(&mut table, 0));
    }

    #[test]
    fn closing_a_descriptor_from_outside_the_table_is_rejected_without_panicking() {
        let mut table = new_fd_table();
        open_fd(&mut table, CONSOLE);
        assert!(!close_fd(&mut table, NOFILE));
        assert!(!close_fd(&mut table, 10_000));
        assert_eq!(table[0].map(|f| f.inum), Some(CONSOLE));
    }

    #[test]
    fn dup_copies_the_record_into_the_lowest_free_slot() {
        let mut table = new_fd_table();
        open_fd(&mut table, CONSOLE); // 0
        open_fd(&mut table, CONSOLE); // 1
        open_fd(&mut table, CONSOLE); // 2
        open_fd(&mut table, README); // 3
        advance(&mut table, 3, 100);

        // `cmd > out`: close 1, then dup the file into the lowest free slot.
        assert!(close_fd(&mut table, 1));
        assert_eq!(dup_fd(&mut table, 3), Some(1));
        assert_eq!(table[1], table[3]);
        assert_eq!(table[1], Some(OpenFile { inum: README, offset: 100 }));

        // Nothing to dup: a closed slot, and a slot past the end.
        close_fd(&mut table, 2);
        assert_eq!(dup_fd(&mut table, 2), None);
        assert_eq!(dup_fd(&mut table, NOFILE), None);
    }

    #[test]
    fn an_inode_can_be_found_by_its_lowest_descriptor() {
        let mut table = new_fd_table();
        open_fd(&mut table, CONSOLE);
        open_fd(&mut table, README);
        open_fd(&mut table, README);
        assert_eq!(fd_of(&table, README), Some(1));
        assert_eq!(fd_of(&table, 99), None);
        close_fd(&mut table, 1);
        assert_eq!(fd_of(&table, README), Some(2));
    }

    #[test]
    fn advance_writes_through_the_slot_not_a_copy() {
        let mut table = new_fd_table();
        open_fd(&mut table, README);
        assert!(advance(&mut table, 0, 512));
        assert!(advance(&mut table, 0, 512));
        // The table changed. A version that copied the record out, bumped the
        // copy, and forgot to put it back would leave this at 0.
        assert_eq!(table[0].map(|f| f.offset), Some(1024));
        // A closed slot, and an index past the end: false, and no panic.
        assert!(!advance(&mut table, 1, 1));
        assert!(!advance(&mut table, NOFILE, 1));
    }

    #[test]
    fn open_inums_lists_the_table_in_order_and_then_grows() {
        let mut table = new_fd_table();
        open_fd(&mut table, CONSOLE);
        open_fd(&mut table, README);
        open_fd(&mut table, 9);
        close_fd(&mut table, 1);

        let mut inums = open_inums(&table);
        assert_eq!(inums, vec![CONSOLE, 9]);

        // The Vec grows on demand; the table it came from cannot.
        inums.push(README);
        assert_eq!(inums, vec![CONSOLE, 9, README]);
        assert_eq!(table.len(), NOFILE);
    }
}
