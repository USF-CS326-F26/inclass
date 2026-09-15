// The live demo. Run it, then read it:
//
//     cargo run
//
// Six short sections over one descriptor table, matching `src/lib.rs` and the
// exercise: the table's size, the lowest-free rule and why the shell needs
// it, a write through a slot, a full table, and the one list that allocates.
// The last section is an error you can uncomment on screen -- that is the
// part worth doing slowly.

use collections_example::{
    advance, close_fd, dup_fd, fd_of, new_fd_table, open_fd, open_inums, OpenFile, NOFILE,
};
use std::mem::size_of;

const CONSOLE: u32 = 1;
const README: u32 = 7;

fn main() {
    println!("== 1. the table is one lump of memory, sized at compile time ==");
    println!("size_of::<Option<u32>>()      = {}   the exercise's slot", size_of::<Option<u32>>());
    println!("size_of::<Option<OpenFile>>() = {}  this slot: two fields + a tag, aligned",
             size_of::<Option<OpenFile>>());
    println!("size_of::<[Option<OpenFile>; {NOFILE}]>() = {}   = {NOFILE} x {}",
             size_of::<[Option<OpenFile>; NOFILE]>(), size_of::<Option<OpenFile>>());
    println!("reserved whether one file is open or eight. No allocator was asked.");

    println!("\n== 2. the lowest free slot, and why 3 ==");
    let mut table = new_fd_table();
    for _ in 0..3 {
        let fd = open_fd(&mut table, CONSOLE);
        println!("  open_fd(CONSOLE) -> {fd:?}");
    }
    let fd = open_fd(&mut table, README);
    println!("  open_fd(README)  -> {fd:?}   the first file a program opens is always 3");
    show(&table);

    println!("\n== 3. redirection is two calls, and it only works bottom-up ==");
    println!("  `cmd > out` in the shell: close 1, then dup the file.");
    println!("  close_fd(1)      -> {}", close_fd(&mut table, 1));
    println!("  dup_fd(3)        -> {:?}   it landed in slot 1 because 1 was the LOWEST free",
             dup_fd(&mut table, 3));
    show(&table);
    println!("  a search that started anywhere else would break every shell script.");

    println!("\n== 4. writing through a slot to a field ==");
    println!("  advance(3, 512)  -> {}", advance(&mut table, 3, 512));
    println!("  advance(3, 512)  -> {}", advance(&mut table, 3, 512));
    println!("  table[3]         = {:?}", table[3]);
    println!("  table[1]         = {:?}   the dup was a COPY, taken before the advance",
             table[1]);
    println!("  `if let Some(Some(f)) = table.get_mut(3)` left f as a &mut OpenFile,");
    println!("  so `f.offset += n` changed the table, not a copy of one record.");
    println!("  fd_of(README)    -> {:?}   the lowest descriptor on that inode",
             fd_of(&table, README));

    println!("\n== 5. a full table refuses, and says so ==");
    loop {
        match open_fd(&mut table, 20) {
            Some(fd) => println!("  open_fd(20)      -> Some({fd})"),
            None => {
                println!("  open_fd(20)      -> None   this is `open` returning -EMFILE");
                break;
            }
        }
    }
    println!("  the failure lands at the call that asked for too much, and it is");
    println!("  testable. A table that grew would fail later, somewhere else.");

    println!("\n== 6. the one list that allocates ==");
    let mut inums = open_inums(&table);
    println!("  open_inums(&table) = {inums:?}");
    inums.push(99);
    println!("  after push(99)     = {inums:?}   the Vec grew");
    println!("  table.len()        = {}   the table did not, and cannot", table.len());
    println!("  host code and tests may collect. The kernel walks the table in place.");

    println!("\n== 7. the error worth reading out loud ==");
    println!("uncomment the block at the bottom of src/main.rs and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME in class. The exit-time close-all loop, written the way
    // everyone writes it first. rustc says:
    //
    //   error[E0506]: cannot assign to `table[_]` because it is borrowed
    //      |     for (fd, slot) in table.iter().enumerate() {
    //      |                       ------------------------
    //      |                       |
    //      |                       `table[_]` is borrowed here
    //      |                       borrow later used here
    //      |         if slot.is_some() {
    //      |             table[fd] = None;
    //      |             ^^^^^^^^^^^^^^^^ `table[_]` is assigned to here but
    //      |                              it was already borrowed
    //
    // The iterator holds a shared borrow of the whole table for the whole
    // loop. The fix is not to fight it: `iter_mut()` hands out one `&mut`
    // per slot, in turn, and `*slot = None` is the write. Exactly the loop
    // in `open_fd`.
    //
    // for (fd, slot) in table.iter().enumerate() {
    //     if slot.is_some() {
    //         table[fd] = None;
    //     }
    // }
    // ---------------------------------------------------------------------
}

fn show(table: &[Option<OpenFile>]) {
    let cells: Vec<String> = table
        .iter()
        .map(|slot| match slot {
            Some(f) => format!("{}@{}", f.inum, f.offset),
            None => "-".to_string(),
        })
        .collect();
    println!("  table: [{}]", cells.join(" | "));
}
