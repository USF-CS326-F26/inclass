//! 08 — Exclusive borrows: `&mut T`. Read it as *exclusive*, not "mutable".
//!
//! The whole rule, on one line:
//!
//!     ALIASING  XOR  MUTATION
//!
//!     many &T   ..... or ..... exactly one &mut T   ..... never both
//!
//!     &T   &T   &T          &mut T
//!      \    |    /             |
//!       ▼   ▼   ▼              ▼
//!     [ the value ]        [ the value ]
//!      read-only            read/write, nobody else looking
//!
//! Run:  cargo run --bin 08_borrow_mut

#[derive(Debug)]
struct Allocator {
    free: Vec<usize>,
    handed_out: usize,
}

fn main() {
    println!("== mutating through &mut ==");
    let mut a = Allocator { free: vec![0x8000_1000, 0x8000_2000], handed_out: 0 };
    println!("before: free={} handed_out={}", hex(&a.free), a.handed_out);
    let page = take_page(&mut a);            // lend it out, exclusively
    println!("took {:#x}", page.unwrap());
    println!("after:  free={} handed_out={}   <- main still owns the allocator", hex(&a.free), a.handed_out);

    give_back(&mut a, 0x8000_9000);
    println!("gave back:  free={} handed_out={}", hex(&a.free), a.handed_out);

    println!("\n== exactly one &mut at a time ==");
    {
        let m = &mut a;
        // let m2 = &mut a;                  // error[E0499]: second mutable borrow
        m.handed_out += 100;
        println!("mutated through m: handed_out = {}", m.handed_out);
    }
    println!("borrow over, `a` usable again: handed_out = {}", a.handed_out);

    println!("\n== &T and &mut T cannot overlap ==");
    {
        let snapshot = a.free.len();         // read finishes here
        // let r = &a.free;                  // if this borrow were still live...
        a.free.push(0x8000_A000);            // ...this would be error[E0502]
        println!("len was {snapshot}, now {}", a.free.len());
    }
    // Why it matters: `push` may reallocate. A live &T would then point at a
    // freed buffer. That is a use-after-free the compiler just prevented.

    println!("\n== the iterator-invalidation bug, in the language ==");
    let mut v = vec![1, 2, 3];
    // for x in &v { v.push(*x); }           // error[E0502] — and an infinite
    //                                       // loop over a moving buffer in C++
    let extra: Vec<i32> = v.iter().map(|x| x * 10).collect(); // borrow ends here
    v.extend(extra);                                          // now mutate
    println!("v = {v:?}");

    println!("\n== reborrowing: passing a &mut along ==");
    let m = &mut a;
    bump(m);        // implicit reborrow: `m` is usable again after the call
    bump(m);
    println!("after two bumps: handed_out = {}", m.handed_out);

    println!("\n== two &mut into DISJOINT parts is fine ==");
    let mut table = [0usize; 6];
    let (lo, hi) = table.split_at_mut(3);    // std proves they do not overlap
    lo[0] = 0xAAAA;
    hi[0] = 0xBBBB;
    println!("table = {table:#x?}");
}

/// `&mut` in a signature is a promise: while this runs, nobody else can even
/// *look* at the allocator.
fn take_page(a: &mut Allocator) -> Option<usize> {
    let p = a.free.pop()?;
    a.handed_out += 1;
    Some(p)
}

fn give_back(a: &mut Allocator, addr: usize) {
    a.free.push(addr);
    a.handed_out -= 1;
}

fn bump(a: &mut Allocator) {
    a.handed_out += 1;
}

/// Takes a slice — it does not care whether the pages live in a Vec, an array,
/// or a window into either. See 09_slices.
fn hex(pages: &[usize]) -> String {
    let items: Vec<String> = pages.iter().map(|p| format!("{p:#x}")).collect();
    format!("[{}]", items.join(", "))
}
