// error[E0152]: found duplicate lang item `panic_impl`
//
// "Exactly one" is enforced across the whole program: this crate and every
// crate it links. `#[panic_handler]` fills a slot the compiler calls
// `panic_impl`, and a second one has nowhere to go. It would fail the same
// way if a library you depend on already brought its own handler.
//
// FIX 1: delete one of them.
// FIX 2: if the library's handler is the one you want, write none yourself.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    halt()
}

#[panic_handler]
fn also_panic(_: &PanicInfo) -> ! {
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
