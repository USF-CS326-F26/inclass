//! 08 — A device register is an address: a raw pointer reads it.
//!
//! QEMU's `virt` board maps its devices into the physical address space. Two
//! of them tell time, and neither needs a driver, only a pointer:
//!
//!     0x0010_1000   goldfish RTC   wall-clock nanoseconds since 1970
//!     0x0200_4000   CLINT mtimecmp one u64 alarm per hart: base + 8 * hart
//!     0x0200_BFF8   CLINT mtime    a counter, 10,000,000 ticks per second
//!
//! Building the pointer is safe. Reading through it is `unsafe`, because only
//! you know the address is a live device and not garbage.
//!
//! Run:  cargo run --bin 08_mmio_clock
#![no_std]
#![no_main]

use core::ptr::read_volatile;
use week06::println;

week06::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week06::report_panic(info)
}

const MTIME: *const u64 = 0x0200_BFF8 as *const u64;
const MTIMECMP: *mut u64 = 0x0200_4000 as *mut u64;   // one alarm per hart
const RTC: *const u32 = 0x0010_1000 as *const u32;    // TIME_LOW, then TIME_HIGH

/// SAFETY, for the caller: this is QEMU's `virt`, where MTIME is the CLINT.
unsafe fn wait_until(deadline: u64) {
    while read_volatile(MTIME) < deadline {}
}

// fold: days since 1970-01-01 → (year, month, day), from Howard Hinnant's algorithm
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + (m <= 2) as i64, m, d)
}

fn main() {
    println!("== the counter moves on its own ==");
    let (a, b) = unsafe { (read_volatile(MTIME), read_volatile(MTIME)) };
    println!("mtime = {a}");
    println!("mtime = {b}   (+{} between two loads)", b - a);

    println!("\n== .add counts elements, not bytes ==");
    for hart in 0..4 {
        let p = unsafe { MTIMECMP.add(hart) };
        println!("MTIMECMP.add({hart}) = {p:p}");
    }
    println!("each step is size_of::<u64>() = 8 bytes; nothing was dereferenced");

    println!("\n== wait_until: poll the counter ==");
    let start = unsafe { read_volatile(MTIME) };
    unsafe { wait_until(start + 100_000) };   // 10 ms
    let end = unsafe { read_volatile(MTIME) };
    println!("asked for 100000 ticks (10 ms), waited {}", end - start);

    println!("\n== a second device: the wall clock ==");
    let ns = unsafe { read_volatile(RTC) as u64 | (read_volatile(RTC.add(1)) as u64) << 32 };
    let secs = ns / 1_000_000_000;
    let (y, m, d) = civil((secs / 86_400) as i64);
    let t = secs % 86_400;
    println!("RTC = {ns} ns since 1970");
    println!("    = {y}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC", t / 3600, t / 60 % 60, t % 60);
    println!("low word first: reading TIME_LOW latches TIME_HIGH, so the two halves match");
}
