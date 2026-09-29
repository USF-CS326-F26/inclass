//! 12 — `satp` names the root table, and here it reads 0: Bare, no tree.
//!
//! One CSR decides whether addresses are translated, and through which tree.
//! It packs three fields into 64 bits:
//!
//!      63  60 59         44 43                        0
//!     +------+-------------+---------------------------+
//!     | MODE |    ASID     |   PPN of the root table   |
//!     +------+-------------+---------------------------+
//!
//!     MODE   8 is Sv39, 0 is Bare: no translation at all
//!     ASID   an address-space tag for the TLB; rv6 leaves it 0
//!     PPN    the root table's address >> 12
//!
//! MODE is the top hex digit, so a `satp` that starts with 8 is Sv39. MODE 0
//! turns translation off for supervisor and user mode. This program runs in
//! machine mode, which translates none of its own accesses while mstatus.MPRV
//! is clear. It only reads the register: writing it is week 10's job.
//!
//! Run:  cargo run --bin 12_satp_reads_zero
#![no_std]
#![no_main]

use core::arch::asm;
use core::hint::black_box;
use week07::{println, Hex};

week07::entry!(main);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    week07::report_panic(info)
}

const PPN_BITS: u64 = (1 << 44) - 1;

fn main() {
    println!("== pack satp by hand ==");
    let root = black_box(0x87FF_F000u64);
    let mode = black_box(8u64) << 60;
    println!("MODE   8 << 60              = {:.16}", Hex(mode));
    println!("PPN    {} >> 12    = {:.16}", Hex(root), Hex(root >> 12));
    println!("satp   MODE | PPN           = {:.16}", Hex(mode | (root >> 12)));
    println!("{} is the top page of RAM, one page below 0x8800_0000", Hex(root));
    println!("satp holds its page number, not its address: the low 12 bits are 0 anyway");

    println!("\n== and back ==");
    let satp = black_box(0x8000_0000_0008_7FFFu64);
    println!("satp                        = {:.16}", Hex(satp));
    println!("satp >> 60                  = {}   MODE: Sv39", satp >> 60);
    println!("(satp >> 44) & 0xFFFF       = {}   the ASID", (satp >> 44) & 0xFFFF);
    println!("satp & ((1 << 44) - 1)      = {}   the root's PPN", Hex(satp & PPN_BITS));
    println!("that << 12                  = {}   the root table", Hex((satp & PPN_BITS) << 12));
    let modes = [
        (0u64, "Bare: no translation"),
        (8, "Sv39: three levels"),
        (9, "Sv48: four levels"),
        (10, "Sv57: five levels"),
    ];
    for (m, what) in modes {
        println!("MODE {m:>2}: satp starts {:.16}   {what}", Hex(m << 60));
    }
    println!("stop at the PPN and you have 0x8_7FFF, a page number and not a table");

    println!("\n== the real satp ==");
    let (real, mstatus): (u64, u64);
    unsafe {
        asm!("csrr {}, satp", out(reg) real);
        asm!("csrr {}, mstatus", out(reg) mstatus);
    }
    println!("csrr satp = {}", Hex(real));
    println!("MODE {}, ASID {}, PPN {}: Bare, with no root table",
             real >> 60, (real >> 44) & 0xFFFF, Hex(real & PPN_BITS));
    println!("program 01 read the same 0 at reset, and nothing here writes it");
    println!("MODE 0 is paging off for supervisor and user mode");
    println!("mstatus.MPRV = {}: machine mode's own loads and stores are not translated",
             (mstatus >> 17) & 1);
    println!("its fetches never are, so this program runs on physical addresses either way");
}
