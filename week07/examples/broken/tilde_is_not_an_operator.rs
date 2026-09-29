// error: `~` cannot be used as a unary operator
//
// C clears a page offset with `va & ~0xFFF`. Rust has no `~`: its bitwise
// NOT is `!`, the same operator that negates a bool. This error has no code,
// and rustc's help line under it names the fix.
//
// FIX 1: `va & !0xFFF`: all ones except the low 12 bits.
// FIX 2: `va & !(4096 - 1)`: the same mask, built from the page size.
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
    let va: u64 = core::hint::black_box(0x20_00A0_35F0);
    let page = va & ~0xFFF;
    let _ = page;
    halt()
}
