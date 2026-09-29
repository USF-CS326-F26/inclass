// error[E0308]: mismatched types
//               expected `usize`, found `u64`
//
// On RV64 a usize is 64 bits, as wide as a u64, and Rust still keeps the two
// apart: an integer changes type only where you convert it, with `as` or the
// `try_into` rustc suggests. `csrr` fills in whatever type the variable was
// declared with, and here that was u64.
//
// FIX 1: `let root = ((satp & PPN_BITS) << 12) as usize;`: say the
//        conversion.
// FIX 2: `let satp: usize;` with `PPN_BITS: usize`: the type rv6 gives
//        addresses, from the start.
#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const PPN_BITS: u64 = (1 << 44) - 1;

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
    let satp: u64;
    unsafe { asm!("csrr {}, satp", out(reg) satp) };
    let root: usize = (satp & PPN_BITS) << 12;
    let _ = root;
    halt()
}
