// error[E0463]: can't find crate for `std`
//
// Every crate links std unless it says otherwise, and this target has no std
// to link: `riscv64gc-unknown-none-elf` means no operating system, so no
// files, threads or heap for std to be built on. `#![no_main]` alone is not
// enough; the program still asked for std by not refusing it.
//
// FIX 1: `#![no_std]` as the first line, with the `!`: an inner attribute
//        speaks for the whole crate. `#[no_std]` without it decorates only the
//        item below, and rustc warns you to add the exclamation mark.
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
    halt()
}
