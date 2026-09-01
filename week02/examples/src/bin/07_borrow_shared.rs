//! 07 — Shared borrows: `&T`. Lend it, don't give it away.
//!
//!     owner:  s: [ ptr | len | cap ] ────▶ [ k e r n e l ]
//!                    ▲   ▲
//!     r1: [ ptr ]────┘   │      many &T at once: all read-only
//!     r2: [ ptr ]────────┘
//!
//! A reference is one machine word (for a sized type). It does not own; when
//! it goes out of scope nothing is freed.
//!
//! Run:  cargo run --bin 07_borrow_shared

#[derive(Debug)]
struct FreeList {
    pages: Vec<usize>,
}

fn main() {
    println!("== lending instead of giving ==");
    let list = FreeList { pages: vec![0x8000_1000, 0x8000_2000, 0x8000_3000] };
    println!("count = {}", count(&list));      // & = "borrow, please"
    println!("first = {:#x}", first(&list).unwrap());
    println!("the caller still owns `list`: {} pages", list.pages.len());

    println!("\n== many readers at once ==");
    let r1 = &list;
    let r2 = &list;
    let r3 = &list.pages;
    println!("r1 {:p}  r2 {:p}  r3 -> buffer {:p}", r1, r2, r3.as_ptr());
    println!("all three point at the same memory; none of them can change it");

    println!("\n== a borrow cannot outlive the owner ==");
    // The classic dangling pointer, rejected at compile time:
    //     fn dangle() -> &String { let s = String::from("x"); &s }
    //     error[E0106]: missing lifetime specifier
    // The fix is to return the value itself — see `own_it` below.
    let owned = own_it();
    println!("returned an owned String {owned:?} instead of a reference to dead stack");

    println!("\n== borrows end at their LAST USE, not the closing brace ==");
    let mut v = vec![1, 2, 3];
    let sum: i32 = v.iter().sum();     // borrow starts
    println!("sum = {sum}");           // borrow's last use
    v.push(4);                         // legal: the borrow is already over
    println!("pushed; v = {v:?}");
    // If `println!("{sum} {:?}", v.iter().max())` came AFTER the push it would
    // simply create a new borrow — also fine. What is rejected is overlap.

    println!("\n== &T is Copy, so passing it along is free ==");
    let text = String::from("uart0");
    show(&text);
    show(&text);                       // borrow again, as often as you like
    println!("still owned by main: {text:?}");
}

fn count(l: &FreeList) -> usize {
    l.pages.len()
} // `l` goes out of scope: nothing is freed, it never owned anything

fn first(l: &FreeList) -> Option<&usize> {
    l.pages.first()
}

fn own_it() -> String {
    String::from("kernel")
}

fn show(s: &str) {
    println!("  show() sees {s:?} at {:p}", s.as_ptr());
}
