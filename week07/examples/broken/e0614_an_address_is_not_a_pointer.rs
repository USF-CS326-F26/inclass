// error[E0614]: type `usize` cannot be dereferenced
//
// A usize that holds an address is still just a number, and `*` works only
// on a pointer or a reference. The cast makes the pointer, and its type,
// `*mut usize`, says the store is 8 bytes wide. Storing through a raw
// pointer is `unsafe`: only you know the page is free.
//
// FIX 1: `unsafe { *(p as *mut usize) = q }`: the store program 07 makes.
// FIX 2: `unsafe { (p as *mut usize).write(q) }`: the same store, as a method.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

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
    let p: usize = 0x8400_6000;
    let q: usize = 0x8400_A000;
    *p = q;
    halt()
}
