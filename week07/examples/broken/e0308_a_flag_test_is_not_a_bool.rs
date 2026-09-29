// error[E0308]: mismatched types
//               expected `bool`, found `u64`
//
// C counts any nonzero integer as true, so `if (pte & PTE_V)` tests a flag.
// Rust's `if` takes only a bool, and `pte & V` is still a u64: the flag bit,
// or 0. Say the question you mean, which is "is it nonzero?".
//
// FIX 1: `if pte & V != 0`: Rust's `&` binds tighter than `!=` (program 09).
// FIX 2: `if (pte & V) != 0`: the same test, and it reads the same in C.
#![no_std]
#![no_main]

use core::hint::black_box;
use core::panic::PanicInfo;

const V: u64 = 1 << 0;

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
    let pte: u64 = black_box(0x2048_D007);
    let kind = if pte & V { "valid" } else { "invalid" };
    let _ = black_box(kind);
    halt()
}
