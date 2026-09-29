// error[E0133]: use of extern static is unsafe and requires unsafe function or
//               block
//
// `end` is declared in Rust but defined by the linker script, so rustc cannot
// check that a byte really lives there, or that nothing else changes it.
// Reading it, or making a reference to it, is unsafe, even `&end`, which only
// takes its address.
//
// FIX 1: `unsafe { &end as *const u8 as usize }`, as the lecture writes it.
// FIX 2: `core::ptr::addr_of!(end) as usize`, which is safe: it makes the
//        pointer without making a reference, and never reads the byte.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

extern "C" {
    static end: u8;
}

fn first_free() -> usize {
    &end as *const u8 as usize
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
