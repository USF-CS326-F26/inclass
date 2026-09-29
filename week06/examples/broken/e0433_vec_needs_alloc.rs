// error[E0433]: cannot find type `Vec` in this scope
//
// `Vec` is not in core. It lives in `alloc`, because growing needs a heap,
// and a heap needs an allocator that somebody wrote. `#![no_std]` took away
// std's prelude, and with it `Vec`, `String` and `Box`.
//
// FIX 1: a fixed-size array, `[u64; 16]`, and a count of how many are used:
//        what a kernel does until it has an allocator (38k gives rv6 one).
// FIX 2: `extern crate alloc;` plus a `#[global_allocator]`, and then
//        `alloc::vec::Vec`. Only once there is memory to hand out.
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
    let mut log = Vec::new();
    log.push(1u64);
    halt()
}
