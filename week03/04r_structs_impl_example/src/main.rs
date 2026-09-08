// The live demo. Run it, then read it:
//
//     cargo run
//
// Three sections, matching the three sections of `src/lib.rs` and of the
// exercise: a struct with methods, a newtype over packed bits, and a guard
// that cleans up after itself. The last section is an error you can uncomment
// on screen — that is the part worth doing slowly.

use structs_example::{
    blocks_for, Cpu, DevNo, Extent, IntrGuard, Superblock, BLOCK_SIZE, CONSOLE, FSMAGIC,
    MAJOR_DISK,
};

fn main() {
    println!("== 1. a struct, and the methods that belong to it ==");

    // An associated function, called through the type with `::`. Its job is to
    // MAKE an Extent, so there is no `self` for it to take.
    let file = Extent::of_bytes(8, 5000);
    println!("a 5000-byte file starting at block 8:");
    println!("  Extent::of_bytes(8, 5000) -> {file:?}");
    println!("  blocks_for(5000)          -> {} blocks of {BLOCK_SIZE}",
             blocks_for(5000));
    println!("  file.bytes()              -> {} bytes reserved", file.bytes());
    println!("  the last block is only {} bytes used, and the file owns it all",
             5000 - 9 * BLOCK_SIZE);

    // Methods, called with a dot. `&self` borrows, so `file` is still ours.
    println!("\n  file.blocks()             -> {:?}", file.blocks());
    println!("  file.contains(8)          -> {}", file.contains(8));
    println!("  file.contains(17)         -> {}   the last block", file.contains(17));
    println!("  file.contains(18)         -> {}   one past the end",
             file.contains(18));

    // Half-open, so two extents fit together with no gap and no overlap.
    let next = file.after(2);
    println!("\n  file.after(2)             -> {next:?}");
    println!("  block 17 is in the first, block 18 is in the second, and");
    println!("  neither is in both");

    println!("\n== 2. a newtype: one integer wearing a type ==");

    let disk = DevNo::new(MAJOR_DISK, 3);
    println!("DevNo::new(2, 3)   -> {disk:?}");
    println!("  as a raw word    -> {:#010x}   ({:#034b})", disk.0, disk.0);
    println!("  .major()         -> {}   which driver", disk.major());
    println!("  .minor()         -> {}   which unit", disk.minor());
    println!("  .is_console()    -> {}", disk.is_console());
    println!("  CONSOLE          -> {CONSOLE:?}, is_console = {}",
             CONSOLE.is_console());

    // The point is what does NOT compile. `open_device` takes a DevNo, so a
    // bare integer is a type error rather than a silent bug.
    open_device(disk);
    println!("  open_device(3) would be error[E0308]: expected `DevNo`");

    // And it costs nothing: #[repr(transparent)] means a DevNo IS its u32.
    println!("  size_of::<DevNo>() = {}   == size_of::<u32>() = {}",
             std::mem::size_of::<DevNo>(), std::mem::size_of::<u32>());

    println!("\n  CONSOLE was built by the COMPILER, not at start-up:");
    println!("    pub const CONSOLE: DevNo = DevNo::new(MAJOR_CONSOLE, 0);");
    println!("  no shift instruction for it exists anywhere in this binary");

    println!("\n== 3. a guard that cleans up after itself ==");

    let mut cpu = Cpu::new();
    println!("interrupts on before      -> {}", cpu.intr_on);

    {
        // `acquire` reads and writes through the &mut FIRST, and only then
        // moves it into the guard. After that line the guard holds the only
        // path to the Cpu — nothing else can reach it, not even `cpu` itself.
        let g = IntrGuard::acquire(&mut cpu);
        println!("  inside the critical section:");
        println!("    g.intr_on()           -> {}", g.intr_on());
        println!("    g.depth()             -> {}", g.depth());
        println!("    g.was_on()            -> {}   what it will restore", g.was_on());
        // println!("{}", cpu.intr_on);   // error[E0502]: cpu is borrowed
    } // <- no call here. Drop runs at this brace.

    println!("interrupts on after       -> {}   nobody called release()",
             cpu.intr_on);
    println!("depth back to             -> {}", cpu.depth);

    println!("\n  and on the paths you did not think about:");
    critical_section(&mut cpu, true);
    println!("    after an early return  -> interrupts on = {}", cpu.intr_on);
    critical_section(&mut cpu, false);
    println!("    after running through  -> interrupts on = {}", cpu.intr_on);

    println!("\n  restoring what it FOUND, not what it assumes:");
    // Pretend we are already inside somebody else's critical section.
    let mut busy = Cpu { intr_on: false, depth: 1 };
    println!("    before: interrupts on = {}, depth = {}", busy.intr_on, busy.depth);
    {
        let inner = IntrGuard::acquire(&mut busy);
        println!("    inner:  was_on = {}, depth = {}", inner.was_on(), inner.depth());
    }
    println!("    after:  interrupts on = {}, depth = {}", busy.intr_on, busy.depth);
    println!("    a Drop that simply switched interrupts ON would have turned");
    println!("    them on here, inside the outer section that turned them off");

    println!("\n== 4. a struct something else reads ==");

    let sb = Superblock { magic: FSMAGIC, size: 1000, nblocks: 941, ninodes: 200 };
    println!("Superblock {{ magic: {:#x}, .. }}", sb.magic);
    println!("  is_valid()               -> {}", sb.is_valid());
    println!("  size_of::<Superblock>()  -> {}", std::mem::size_of::<Superblock>());
    println!("  magic   at offset {}", std::mem::offset_of!(Superblock, magic));
    println!("  size    at offset {}", std::mem::offset_of!(Superblock, size));
    println!("  nblocks at offset {}", std::mem::offset_of!(Superblock, nblocks));
    println!("  ninodes at offset {}", std::mem::offset_of!(Superblock, ninodes));
    println!("`mkfs` wrote those bytes; this kernel reads them. Remove");
    println!("#[repr(C)] and rustc may reorder the fields — the same bytes");
    println!("then mean different numbers, with no error anywhere.");

    println!("\n== 5. the error worth reading out loud ==");
    println!("uncomment the block at the bottom of src/main.rs and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME in class. rustc says:
    //
    //   error[E0502]: cannot borrow `cpu.intr_on` as immutable because it is
    //                 also borrowed as mutable
    //      |     let g = IntrGuard::acquire(&mut cpu);
    //      |                                -------- mutable borrow occurs
    //      |     println!("{}", cpu.intr_on);
    //      |                    ^^^^^^^^^^^ immutable borrow occurs here
    //      |     println!("{}", g.intr_on());
    //      |                    - mutable borrow later used here
    //
    // While the guard is alive it holds the ONLY path to the Cpu. That is not
    // a restriction the guard asks for politely — it is the property that
    // makes "interrupts are off right now" true rather than hoped for.
    //
    // let g = IntrGuard::acquire(&mut cpu);
    // println!("{}", cpu.intr_on);
    // println!("{}", g.intr_on());
    // ---------------------------------------------------------------------
}

/// Takes a device number and nothing else.
fn open_device(dev: DevNo) {
    println!("  open_device({dev:?}) -> driver {}, unit {}", dev.major(), dev.minor());
}

/// Two ways out, one restore site — and you cannot forget either.
fn critical_section(cpu: &mut Cpu, bail: bool) {
    let _g = IntrGuard::acquire(cpu);
    if bail {
        return;
    }
}
