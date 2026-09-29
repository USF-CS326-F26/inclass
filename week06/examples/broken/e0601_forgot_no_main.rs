// error[E0601]: `main` function not found in crate `e0601_forgot_no_main`
//
// Without `#![no_main]`, rustc treats this as an ordinary program: it wants a
// `fn main` and would generate the code that calls it. Nothing would. QEMU
// jumps to `_entry`, the linker script's ENTRY, and nothing runs before it.
// Adding an empty `fn main() {}` silences rustc and fixes nothing.
//
// FIX 1: `#![no_main]` beside `#![no_std]`: "I name my own entry point."
#![no_std]

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
    halt()
}
