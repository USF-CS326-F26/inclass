// error[E0369]: cannot add `usize` to `*mut u8`
//
// A raw pointer has no `+`. The methods rustc suggests, `wrapping_add` and
// `add`, count elements, not bytes: 4096 steps along a `*mut u8` is one page,
// but along a `*mut u64` it is eight. Rust makes you name the step.
//
// FIX 1: `page.wrapping_add(PGSIZE)`: safe to call, and a u8 is one byte.
// FIX 2: do page arithmetic on usize, where + means bytes, and cast back
//        once: `(page as usize + PGSIZE) as *mut u8`.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

const PGSIZE: usize = 4096;

fn page_after(page: *mut u8) -> *mut u8 {
    page + PGSIZE
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
    let _ = page_after(0x8400_6000 as *mut u8);
    halt()
}
