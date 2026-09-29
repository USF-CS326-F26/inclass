// error: `#[panic_handler]` function required, but not found
//
// Indexing, overflow and `unwrap` can all panic, and std is where a panic
// normally goes. With `#![no_std]` there is nowhere, so the program must say:
// exactly one function in the whole program, marked `#[panic_handler]`. This
// error has no code; the whole message is in its one line.
//
// FIX 1: add one: `#[panic_handler] fn panic(info: &PanicInfo) -> ! { … }`.
//        It must not return: `!` is the type of "never comes back".
// FIX 2: in examples/, `week06::report_panic(info)` as its body, as every
//        program there does.
#![no_std]
#![no_main]

fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn _entry() -> ! {
    halt()
}
