//! 01 — One function, three containers.
//!
//! Last week ended on this picture. This week starts by using it: the same
//! `count_free` walks an array, a slice of that array, and a Vec, and the
//! reason is the `&` in front of each argument.
//!
//!     [Option<u32>; 8]     the table itself: 64 bytes, no header, no pointer
//!     &[Option<u32>]       a borrowed VIEW: pointer + length, 16 bytes
//!     Vec<Option<u32>>     a heap buffer: pointer + capacity + length, 24 bytes
//!
//!     array   |slot|slot|slot|slot|slot|slot|slot|slot|      lives where it was declared
//!     slice   [ptr -----> into somebody's table ][len]     never owns anything
//!     vec     [ptr -----> the heap ][cap][len]             grows, and asks an allocator
//!
//! `&arr`, `&arr[..4]`, and `&v` all become a `&[Option<u32>]` at the call.
//! That coercion is why one signature serves every kind of table.
//!
//! Run:  cargo run --bin 01_array_slice_vec

use std::mem::size_of;

/// The most open files one process may hold. xv6 says 16; 8 fits on a slide.
pub const NOFILE: usize = 8;

/// How many slots are free. Takes a SLICE, so it takes anything.
fn count_free(table: &[Option<u32>]) -> usize {
    table.iter().filter(|slot| slot.is_none()).count()
}

fn main() {
    println!("== three sizes ==");
    println!("size_of::<[Option<u32>; {NOFILE}]>() = {}   the table: 8 x 8 bytes",
             size_of::<[Option<u32>; NOFILE]>());
    println!("size_of::<&[Option<u32>]>()    = {}   a slice: pointer + length",
             size_of::<&[Option<u32>]>());
    println!("size_of::<Vec<Option<u32>>>()  = {}   a Vec: pointer + capacity + length",
             size_of::<Vec<Option<u32>>>());
    println!("the array IS the storage; the other two point at storage");

    println!("\n== the array: fixed, and exactly where it was declared ==");
    let mut arr: [Option<u32>; NOFILE] = [None; NOFILE];
    arr[0] = Some(1); // stdin
    arr[1] = Some(1); // stdout
    arr[2] = Some(1); // stderr -- all three the console
    println!("arr           = {arr:?}");
    println!("count_free(&arr)        = {}", count_free(&arr));
    println!("arr.len()               = {}   fixed by the type, known at compile time",
             arr.len());

    println!("\n== a slice: a view into the same bytes ==");
    let first_half = &arr[..4];
    println!("&arr[..4]     = {first_half:?}");
    println!("count_free(first_half)  = {}", count_free(first_half));
    println!("the slice starts at {:p}; the array starts at {:p} -- same address",
             first_half.as_ptr(), arr.as_ptr());
    println!("nothing was copied. A slice is where to look and how far.");

    println!("\n== a Vec: a copy that lives on the heap and can grow ==");
    let mut v = arr.to_vec();
    println!("arr.to_vec()  = {v:?}");
    println!("count_free(&v)          = {}", count_free(&v));
    println!("v.as_ptr()    = {:p}   NOT the array's address: this is the heap",
             v.as_ptr());
    println!("v.capacity()  = {}   room for {}; the next push must reallocate",
             v.capacity(), v.capacity() - v.len());
    for _ in 0..3 {
        v.push(None);
    }
    println!("after 3 pushes: len {} capacity {}   it asked the allocator",
             v.len(), v.capacity());
    println!("arr.len() is still {}. An array cannot be pushed into.", arr.len());

    println!("\n== indexing is checked, and there is a checked spelling ==");
    println!("arr.get(2)    = {:?}", arr.get(2));
    println!("arr.get(8)    = {:?}   one past the end, and no panic", arr.get(8));
    println!("arr[8] would panic: `index out of bounds: the len is 8 but the index is 8`");
    println!("the kernel never indexes with a number a user program chose --");
    println!("it checks first, or it uses get()");

    println!("\n== the one idea ==");
    println!("count_free(&arr), count_free(&arr[..4]), count_free(&v):");
    println!("the SAME function, because each argument became a &[Option<u32>]");
    println!("write functions over slices, and every kind of table can call them");
}
