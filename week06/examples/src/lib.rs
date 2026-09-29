//! The week 6 runtime: just enough for a program to boot on QEMU's `virt`
//! machine, print, and power off. Every program in `src/bin/` uses it, and
//! none of it is on the examples page. The programs are.
//!
//! ```text
//!   QEMU jumps to 0x8000_0000 ── _entry ──► __week06_start ──► main()
//!                               (entry!)      (entry!)          │
//!                                             exit(0) ◄─────────┘
//! ```
//!
//! A program opts in with three lines. `entry!` names its `main`, and the
//! panic handler, which every `no_std` program must have exactly one of,
//! hands the report to `report_panic`:
//!
//! ```ignore
//! week06::entry!(main);
//!
//! #[panic_handler]
//! fn panic(info: &core::panic::PanicInfo) -> ! {
//!     week06::report_panic(info)
//! }
//! ```
//!
//! This is scaffolding for the examples. rv6 has its own version of each
//! piece.
#![no_std]

use core::fmt;

/// The NS16550A UART's transmit register on `virt`. Under `-nographic
/// -serial mon:stdio` a byte stored here comes out on QEMU's stdout.
const UART_TX: *mut u8 = 0x1000_0000 as *mut u8;

/// The SiFive test finisher on `virt`: one store here and QEMU exits.
const FINISHER: *mut u32 = 0x10_0000 as *mut u32;

struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            // SAFETY: `virt` maps the UART here, and it is ready from reset.
            unsafe { core::ptr::write_volatile(UART_TX, b) };
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = fmt::Write::write_fmt(&mut Console, args);
}

/// `print!` over the UART: `core::fmt` does the formatting, with no heap.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::_print(format_args!($($arg)*)) };
}

/// `println!` over the UART.
#[macro_export]
macro_rules! println {
    () => { $crate::_print(format_args!("\n")) };
    ($($arg:tt)*) => { $crate::_print(format_args!("{}\n", format_args!($($arg)*))) };
}

/// Power QEMU off. `0` is success; anything else becomes QEMU's exit status.
pub fn exit(code: u16) -> ! {
    let word = match code {
        0 => 0x5555,
        n => 0x3333 | (n as u32) << 16,
    };
    // SAFETY: `virt` maps the finisher here; the store ends the machine.
    unsafe { core::ptr::write_volatile(FINISHER, word) };
    loop {
        core::hint::spin_loop();
    }
}

/// What every program's `#[panic_handler]` calls: print the report, then
/// power off with status 1, the way rv6's handler prints `OSLINGS:FAIL`.
pub fn report_panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    exit(1)
}

/// `entry!(main)` makes the program bootable. It emits `_entry`, the first
/// instruction QEMU runs, into the `.entry` section that `link.ld` places at
/// 0x8000_0000. `_entry` points `sp` at the stack `link.ld` reserves and
/// jumps to `__week06_start`, which runs `main` and powers off.
#[macro_export]
macro_rules! entry {
    ($main:path) => {
        core::arch::global_asm!(
            ".pushsection .entry, \"ax\"",
            ".globl _entry",
            "_entry:",
            "    la   sp, __stack_top",
            "    j    __week06_start",
            ".popsection",
        );

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn __week06_start() -> ! {
            $main();
            $crate::exit(0)
        }
    };
}
