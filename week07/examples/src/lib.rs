//! The week 7 runtime: just enough for a program to boot on QEMU's `virt`
//! machine, print, and power off. Every program in `src/bin/` uses it, and
//! none of it is on the examples page. The programs are.
//!
//! ```text
//!   the ROM jumps to 0x8000_0000 ── _entry ──► __week07_start ──► main()
//!                                    (entry!)      (entry!)          │
//!                                                  exit(0) ◄─────────┘
//! ```
//!
//! A program opts in with three lines. `entry!` names its `main`, and the
//! panic handler, which every `no_std` program must have exactly one of,
//! hands the report to `report_panic`:
//!
//! ```ignore
//! week07::entry!(main);
//!
//! #[panic_handler]
//! fn panic(info: &core::panic::PanicInfo) -> ! {
//!     week07::report_panic(info)
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

/// A number the way the lecture writes addresses and entries: `0x8123_4000`,
/// uppercase, with `_` between groups of four digits. `{:.8}` asks for at
/// least eight digits, and a width such as `{:>13}` pads it like a string.
pub struct Hex<T>(pub T);

/// The integer types `Hex` takes. `i32` is here because an integer literal
/// with no suffix is one: `Hex((1 << 12) - 1)`.
pub trait Word: Copy {
    fn word(self) -> u64;
}

macro_rules! word {
    ($($t:ty)*) => { $(impl Word for $t { fn word(self) -> u64 { self as u64 } })* };
}
word!(u8 u16 u32 u64 usize i32 i64);

impl<T: Word> fmt::Display for Hex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0u8; 24];
        let (mut at, mut v, mut i) = (buf.len(), self.0.word(), 0);
        while v != 0 || i < f.precision().unwrap_or(1) {
            if i > 0 && i % 4 == 0 {
                at -= 1;
                buf[at] = b'_';
            }
            at -= 1;
            buf[at] = b"0123456789ABCDEF"[(v & 0xF) as usize];
            (v, i) = (v >> 4, i + 1);
        }
        at -= 2;
        buf[at..at + 2].copy_from_slice(b"0x");
        let s = core::str::from_utf8(&buf[at..]).unwrap();
        let pad = f.width().unwrap_or(0).saturating_sub(s.len());
        let right = matches!(f.align(), Some(fmt::Alignment::Right));
        for _ in 0..if right { pad } else { 0 } {
            f.write_str(" ")?;
        }
        f.write_str(s)?;
        for _ in 0..if right { 0 } else { pad } {
            f.write_str(" ")?;
        }
        Ok(())
    }
}

/// `entry!(main)` makes the program bootable. It emits `_entry`, where the
/// boot ROM's jump lands, into the `.entry` section that `link.ld` places at
/// 0x8000_0000. `_entry` points `sp` at the stack `link.ld` reserves and
/// jumps to `__week07_start`, which turns the FPU on, runs `main`, and
/// powers off.
#[macro_export]
macro_rules! entry {
    ($main:path) => {
        core::arch::global_asm!(
            ".pushsection .entry, \"ax\"",
            ".globl _entry",
            "_entry:",
            "    la   sp, __stack_top",
            "    j    __week07_start",
            ".popsection",
        );

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn __week07_start() -> ! {
            // Reset leaves the FPU off (mstatus.FS = 0), and the first float
            // instruction, such as formatting an f64, would trap to mtvec = 0
            // and hang the machine without a word. Turn it on first, in t0
            // alone, so a0-a2 still hold what the boot ROM left there.
            unsafe { core::arch::asm!("li t0, 1 << 13", "csrs mstatus, t0", out("t0") _) };
            $main();
            $crate::exit(0)
        }
    };
}
