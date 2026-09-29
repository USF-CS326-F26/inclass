// error[E0080]: attempt to shift left by `60_i32`, which would overflow
//
// MODE belongs in bits 63..60, but SV39 is a u32, so `SV39 << 60` shifts a
// 32-bit value by 60. A const is computed while compiling, and there a shift
// by the type's width or more is an error. The `as u64` comes too late: it
// widens the result, after the u32 shift has already failed.
//
// FIX 1: `(SV39 as u64) << 60`: widen first, then shift.
// FIX 2: `const SV39: u64 = 8;`: give the field the register's type.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

const SV39: u32 = 8;
const ROOT: u64 = 0x87FF_F000;
const SATP: u64 = (SV39 << 60) as u64 | (ROOT >> 12);

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
    let _ = core::hint::black_box(SATP);
    halt()
}
