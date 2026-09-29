// error[E0133]: dereference of raw pointer is unsafe and requires unsafe
//               function or block
//
// Making the pointer was safe: `0x0200_BFF8 as *const u64` is only a number
// with a type. Reading through it is the step the compiler cannot check,
// because only you know that address is QEMU's clock and not garbage.
//
// FIX 1: `unsafe { read_volatile(MTIME) }`: unsafe, because you vouch for the
//        address; volatile, because it is a device (program 09).
// FIX 2: `unsafe fn now()`, which moves the promise to every caller instead.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

const MTIME: *const u64 = 0x0200_BFF8 as *const u64;

fn now() -> u64 {
    *MTIME
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
    let _ = now();
    halt()
}
