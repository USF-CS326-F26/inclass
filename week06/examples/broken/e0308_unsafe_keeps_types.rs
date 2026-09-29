// error[E0308]: mismatched types
//               expected `u32`, found `u64`
//
// `unsafe` lets you dereference the pointer. It does not change what the
// pointer points at: `*const u64` still yields a u64, and a u64 still does not
// fit a u32 without saying how. Type checks, borrow checks and bounds checks
// all run inside the block, exactly as outside it.
//
// FIX 1: `let ticks: u64 = …`: take the type the device register has.
// FIX 2: `… as u32`, if you mean to keep only the low 32 bits.
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::read_volatile;

const MTIME: *const u64 = 0x0200_BFF8 as *const u64;

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
    let ticks: u32 = unsafe { read_volatile(MTIME) };
    let _ = ticks;
    halt()
}
