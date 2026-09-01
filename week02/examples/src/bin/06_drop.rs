//! 06 — Drop: where free() went.
//!
//! The owner's scope ends, so the resource is released — right there, at a
//! point you can see in the source. No collector decides when. In a kernel
//! this is how you get "the lock is released at the end of this block" as a
//! fact instead of a comment.
//!
//! Watch the output: drops run in REVERSE declaration order, because later
//! values may refer to earlier ones.
//!
//! Run:  cargo run --bin 06_drop

struct Page {
    addr: usize,
}

impl Page {
    fn alloc(addr: usize) -> Page {
        println!("  kalloc  {addr:#x}");
        Page { addr }
    }
}

impl Drop for Page {
    /// This is the destructor. You never call it; the compiler emits the call
    /// at the exact point the owner's scope ends.
    fn drop(&mut self) {
        println!("  kfree   {:#x}", self.addr);
    }
}

fn main() {
    println!("== reverse declaration order ==");
    {
        let _first = Page::alloc(0x8000_1000);
        let _second = Page::alloc(0x8000_2000);
        let _third = Page::alloc(0x8000_3000);
        println!("  -- end of block --");
    } // third, second, first

    println!("\n== a move changes WHO drops it, not whether ==");
    {
        let p = Page::alloc(0x8000_4000);
        stash(p);                  // ownership moves in; the drop happens there
        println!("  back in main; nothing left to free here");
    }

    println!("\n== returning ownership defers the drop ==");
    {
        let p = make();            // the drop follows the value out
        println!("  main owns {:#x} now", p.addr);
    } // freed here

    println!("\n== drop() is just a move into a function that ends ==");
    {
        let p = Page::alloc(0x8000_6000);
        println!("  releasing early");
        drop(p);                   // std::mem::drop: takes ownership, returns ()
        println!("  ... rest of the block runs with the page already freed");
    }

    println!("\n== early return still frees everything ==");
    maybe_fail(true);
    maybe_fail(false);

    println!("\n== even a panic unwinds and frees (this one is caught) ==");
    let _ = std::panic::catch_unwind(|| {
        let _p = Page::alloc(0x8000_9000);
        panic!("simulated fault");
    });
    println!("  page above was freed during unwinding");
}

fn stash(p: Page) {
    println!("  stash() owns {:#x}", p.addr);
} // dropped here

#[allow(clippy::let_and_return)] // the named binding is the teaching point
fn make() -> Page {
    let p = Page::alloc(0x8000_5000);
    p // moved out: NOT dropped here
}

fn maybe_fail(fail: bool) {
    let p = Page::alloc(0x8000_7000);
    if fail {
        println!("  error path, returning early (owner: {:#x})", p.addr);
        return; // dropped on this path
    }
    println!("  success path");
} // and dropped on this one
