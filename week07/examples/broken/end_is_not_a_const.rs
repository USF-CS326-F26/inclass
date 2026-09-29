// error: pointers cannot be cast to integers during const eval
//
// rustc works out a `const` while it compiles this file. `end` gets its
// address later, when the linker lays out the whole image, so at compile time
// there is no number to give. That is why `end` is a symbol and not a constant.
// The same is true of every static's address: the linker picks them all.
// This error has no code; the whole message is its one line.
//
// FIX 1: work it out at run time, where it is needed:
//        `let first_free = unsafe { &end as *const u8 as usize };`
#![no_std]
#![no_main]

use core::panic::PanicInfo;

extern "C" {
    static end: u8;
}

const FIRST_FREE: usize = unsafe { &end as *const u8 as usize };

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
    let _ = FIRST_FREE;
    halt()
}
