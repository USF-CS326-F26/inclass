// error[E0606]: casting `&u8` as `usize` is invalid
//
// A reference is not a number, so `as` will not turn one into a usize; it
// must become a raw pointer first. rustc's help, "remove the unneeded
// borrow", is wrong here. `end as usize` compiles, but it reads the byte
// stored at `end` and gives you its value. That byte is meaningless: the
// address is the answer.
//
// FIX 1: `&end as *const u8 as usize`: reference, raw pointer, then integer
//        (the line program 02 prints).
// FIX 2: `addr_of!(end) as usize` makes the raw pointer directly, with no
//        unsafe (program 08).
#![no_std]
#![no_main]

use core::panic::PanicInfo;

extern "C" {
    static end: u8;
}

fn first_free() -> usize {
    unsafe { &end as usize }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    halt()
}

fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn _entry() -> ! {
    let _ = first_free();
    halt()
}
