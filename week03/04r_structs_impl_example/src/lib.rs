// ╔══════════════════════════════════════════════════════════════════════╗
// ║  04r — Structs, Methods, and `const fn` — IN-CLASS EXAMPLE           ║
// ║  Same shape as the exercise, different nouns: a run of disk blocks   ║
// ║  instead of a region of physical memory, a device number instead of  ║
// ║  a page table entry, and interrupts instead of a page.               ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/04r_structs_impl/skeleton/lib.rs`. Every item
// here has a twin there. If you can explain why the twins are the same shape,
// you have understood structs; the domain is decoration.
//
// The arithmetic is deliberately NOT the exercise's arithmetic. Rounding up
// here is a division, not a bit mask; the packing here is one shift, not two.
// You still have to work the page-table version out yourself.

/// The size of one disk block, in bytes.
///
/// UNDERSTAND: 512 is the unit the disk hardware reads and writes. It plays
///   the same role `PAGE_SIZE` plays in the exercise: the grain everything is
///   counted in.
pub const BLOCK_SIZE: usize = 512;

/// How many whole blocks it takes to hold `bytes` bytes.
///
/// UNDERSTAND: this is "round up", and there are two ways to write it. The
///   exercise uses the bit-mask form, which works only because a page size is
///   a power of two. This is the arithmetic form, which works for any divisor:
///   add one less than the divisor, then divide. 0 bytes needs 0 blocks; 1
///   byte needs 1; exactly 512 needs 1; 513 needs 2.
pub const fn blocks_for(bytes: usize) -> usize {
    (bytes + BLOCK_SIZE - 1) / BLOCK_SIZE
}

// ---------------------------------------------------------------------------
// Section 1 — a struct, and the methods that belong to it.
// ---------------------------------------------------------------------------

/// A run of consecutive disk blocks: `count` of them, starting at `first`.
///
/// UNDERSTAND: the exercise stores a start and an END. This stores a start and
///   a COUNT. Both describe a run; each makes a different question cheap. With
///   an end, "how big is it" is a subtraction. With a count, it is already
///   there — but "does it contain block N" needs the addition instead. Pick the
///   one that makes your commonest question free, and be consistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent {
    pub first: u32,
    pub count: u32,
}

impl Extent {
    /// Is block `b` part of this extent?
    ///
    /// UNDERSTAND: `&self` means "borrow the Extent this was called on", so
    ///   inside the body `self.first` is that extent's first block. You call it
    ///   with a dot: `data.contains(9)`. Because `&self` borrows rather than
    ///   takes, the caller still owns the extent afterwards.
    ///
    ///   The comparison is half-open on purpose — `first` is in, `first +
    ///   count` is out — so two extents laid end to end have no gap and no
    ///   overlap.
    pub fn contains(&self, b: u32) -> bool {
        b >= self.first && b < self.first + self.count
    }

    /// How many bytes this extent covers.
    pub fn bytes(&self) -> usize {
        self.count as usize * BLOCK_SIZE
    }

    /// An extent starting at `first` and big enough to hold `bytes` bytes.
    ///
    /// UNDERSTAND: no `self` parameter, which makes this an ASSOCIATED
    ///   FUNCTION rather than a method — there is no existing Extent to talk
    ///   about, because its job is to make one. You call it through the type:
    ///   `Extent::of_bytes(8, 5000)`.
    ///
    ///   Rust has no `constructor` keyword. `new` is a convention; name the
    ///   function for what it makes.
    pub fn of_bytes(first: u32, bytes: usize) -> Extent {
        Extent { first, count: blocks_for(bytes) as u32 }
    }

    /// Every block number in this extent, in order.
    ///
    /// UNDERSTAND: the exercise walks a cursor and asks, each time round,
    ///   whether the next whole unit still fits. Here the count is known up
    ///   front, so the loop is a plain range and there is no edge case to get
    ///   wrong. That difference is the point: where you put the bound decides
    ///   how easy the loop is to write correctly.
    pub fn blocks(&self) -> Vec<u32> {
        (0..self.count).map(|i| self.first + i).collect()
    }

    /// The extent that follows this one immediately.
    pub fn after(&self, count: u32) -> Extent {
        Extent { first: self.first + self.count, count }
    }
}

// ---------------------------------------------------------------------------
// Section 2 — the newtype: one integer wearing a type.
// ---------------------------------------------------------------------------

/// How many bits the minor number occupies.
pub const MINOR_BITS: u32 = 20;

/// Selects the minor field: the low 20 bits.
///
/// UNDERSTAND: `(1 << 20) - 1` is twenty binary ones. ANDing with it keeps
///   those twenty bits and forces every higher bit to zero — that is what a
///   MASK is.
pub const MINOR_MASK: u32 = (1 << MINOR_BITS) - 1;

/// A Unix device number: a major number saying which driver, and a minor
/// number saying which unit that driver should use.
///
/// UNDERSTAND: this is a NEWTYPE — a struct with one unnamed field, which you
///   reach as `self.0`. At run time a `DevNo` IS the `u32`; at compile time it
///   is a distinct type, so a raw number cannot be passed where a device
///   number belongs. `#[repr(transparent)]` promises the layout is identical
///   to the `u32` inside.
///
///   The exercise packs a page table entry, where the address must first be
///   shifted DOWN to drop the bits a page boundary always has as zero. Nothing
///   here is dropped, so the packing is one shift instead of two — the skill is
///   the same, the layout is not.
///
/// ```text
///   bits [31..20]  major (which driver)    bits [19..0]  minor (which unit)
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DevNo(pub u32);

/// The console is always major 1, by convention.
pub const MAJOR_CONSOLE: u32 = 1;
/// The disk is major 2.
pub const MAJOR_DISK: u32 = 2;

impl DevNo {
    /// Pack a major and a minor number into one word.
    ///
    /// UNDERSTAND: `const fn`, so a device number can be built by the compiler
    ///   and baked into the executable — see `CONSOLE` below.
    pub const fn new(major: u32, minor: u32) -> DevNo {
        DevNo((major << MINOR_BITS) | minor)
    }

    /// Which driver handles this device.
    pub const fn major(self) -> u32 {
        self.0 >> MINOR_BITS
    }

    /// Which unit that driver should use.
    pub const fn minor(self) -> u32 {
        self.0 & MINOR_MASK
    }

    /// Is this the console?
    ///
    /// UNDERSTAND: a method calling another method on the same value.
    ///   `self.major()` runs the method above.
    ///
    ///   This also shows why `DevNo` derives `Copy`. `major` takes `self` by
    ///   value, so without `Copy` this call would move `self` away and the
    ///   compiler would refuse. Copying four bytes is free; a `DevNo` should
    ///   behave exactly like the integer it is.
    pub const fn is_console(self) -> bool {
        self.major() == MAJOR_CONSOLE
    }
}

/// Built by the compiler, not at start-up. This only compiles because
/// `DevNo::new` is a `const fn`.
pub const CONSOLE: DevNo = DevNo::new(MAJOR_CONSOLE, 0);

// ---------------------------------------------------------------------------
// Section 3 — a struct that cleans up after itself.
// ---------------------------------------------------------------------------

/// What one CPU is currently doing about interrupts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cpu {
    pub intr_on: bool,
    pub depth: usize,
}

impl Cpu {
    pub const fn new() -> Cpu {
        Cpu { intr_on: true, depth: 0 }
    }
}

/// Interrupts are off for exactly as long as this value lives.
///
/// UNDERSTAND: the exercise's guard holds a page and puts it back. This one
///   holds a *state* and restores it. Same mechanism, and in a real kernel the
///   second is more common than the first: a critical section turns interrupts
///   off, does its work, and must turn them back on down every path out —
///   including the early `return` you forgot about.
///
///   The `<'a>` is the lifetime from 03r. An `IntrGuard<'a>` borrows the `Cpu`
///   EXCLUSIVELY, so it cannot outlive that Cpu, and while the guard exists
///   nothing else can reach the Cpu at all — not even its owner. "Interrupts
///   are off" stops being a comment and becomes something the compiler checks.
pub struct IntrGuard<'a> {
    cpu: &'a mut Cpu,
    was_on: bool,
}

impl<'a> IntrGuard<'a> {
    /// Turn interrupts off and hold them off.
    ///
    /// UNDERSTAND: an associated function again — its job is to make an
    ///   `IntrGuard`, so there is no `self` to take.
    ///
    ///   The ORDER of these three lines is the whole trick, and it is exactly
    ///   the trap in the exercise's constructor. Read and write through the
    ///   `&mut` FIRST; only then move it into the struct literal. After that
    ///   line the guard holds the only `&mut` to the Cpu, so reaching for
    ///   `cpu` again — or inside the literal itself — is E0499 or E0502.
    pub fn acquire(cpu: &'a mut Cpu) -> IntrGuard<'a> {
        let was_on = cpu.intr_on;
        cpu.intr_on = false;
        cpu.depth += 1;
        IntrGuard { cpu, was_on }
    }

    /// Are interrupts on right now?
    ///
    /// UNDERSTAND: while the guard lives it holds the only `&mut` to the Cpu,
    ///   so this method is the only way anybody can ask the question at all.
    pub fn intr_on(&self) -> bool {
        self.cpu.intr_on
    }

    /// How deeply nested the critical sections are.
    pub fn depth(&self) -> usize {
        self.cpu.depth
    }

    /// What interrupts were doing before this guard took over.
    pub fn was_on(&self) -> bool {
        self.was_on
    }

    /// Restore now, rather than at the closing brace.
    ///
    /// UNDERSTAND: nothing to implement, and the empty body is not an
    ///   oversight. `self` is taken BY VALUE, so calling `guard.release()`
    ///   moves the guard into this function and it dies right here, at this
    ///   closing brace — which runs `Drop` and restores interrupts. The
    ///   standard library's `drop` is this function and nothing else:
    ///   `pub fn drop<T>(_x: T) { }`. Release travels with ownership.
    pub fn release(self) {}
}

impl Drop for IntrGuard<'_> {
    /// Put interrupts back the way they were.
    ///
    /// UNDERSTAND: you never call this. The compiler emits the call wherever
    ///   the guard's owner stops existing — a closing brace, an early
    ///   `return`, the end of `release` above — and there is no path through
    ///   the program that skips it.
    ///
    ///   It is also why `IntrGuard` cannot derive `Copy` the way `DevNo` does.
    ///   `Copy` says "duplicating the bits duplicates the value"; `Drop` says
    ///   "release runs exactly once". Allow both and interrupts come back on
    ///   once per copy, in the middle of the critical section that turned them
    ///   off. `rustc` says so directly: `error[E0184]`.
    fn drop(&mut self) {
        self.cpu.depth -= 1;
        self.cpu.intr_on = self.was_on;
    }
}

// ---------------------------------------------------------------------------
// Section 4 — a preview: a struct that something else reads.
// ---------------------------------------------------------------------------

/// The first block of a filesystem image, as it sits on disk.
///
/// UNDERSTAND: nothing to work out here — read it and move on.
///
///   Rust normally reserves the right to reorder a struct's fields, or pad
///   between them, if that makes the layout better. `#[repr(C)]` gives that
///   right up: the fields sit in memory in the order written, using C's rules.
///   That matters exactly when something other than Rust reads the bytes.
///
///   In the exercise that something is hand-written assembly reading registers
///   by byte offset. Here it is `mkfs`, a separate program that WROTE this
///   block, and the disk image it produced. Reorder the fields and the same
///   bytes are read as different numbers — with no compiler error anywhere.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Superblock {
    pub magic: u32,    // offset 0
    pub size: u32,     // offset 4
    pub nblocks: u32,  // offset 8
    pub ninodes: u32,  // offset 12
}

/// What a valid filesystem image starts with.
pub const FSMAGIC: u32 = 0x1023_5636;

impl Superblock {
    /// A blank superblock, known at compile time.
    ///
    /// UNDERSTAND: `const fn` again. Because this can be evaluated at compile
    ///   time, a kernel can write `static SB: Superblock = Superblock::zero();`
    ///   and the zeroes are baked into the binary — no start-up loop, which
    ///   matters when there is no start-up code yet.
    pub const fn zero() -> Superblock {
        Superblock { magic: 0, size: 0, nblocks: 0, ninodes: 0 }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == FSMAGIC
    }
}

// ---------------------------------------------------------------------------
// The same kind of contract the exercise has, rewritten for disks. Read these
// first: they say what everything above is for.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_extent_knows_its_own_size() {
        let data = Extent { first: 8, count: 4 };
        assert_eq!(data.bytes(), 4 * BLOCK_SIZE);
        assert_eq!(data.bytes(), 2048);

        let empty = Extent { first: 8, count: 0 };
        assert_eq!(empty.bytes(), 0);
    }

    #[test]
    fn rounding_up_to_whole_blocks() {
        // Nothing needs no blocks.
        assert_eq!(blocks_for(0), 0);
        // Anything at all needs one, however little.
        assert_eq!(blocks_for(1), 1);
        assert_eq!(blocks_for(511), 1);
        // An exact fit does not spill into a second block.
        assert_eq!(blocks_for(512), 1);
        assert_eq!(blocks_for(513), 2);
        assert_eq!(blocks_for(5000), 10);

        // `const fn`, so the compiler can do it before the program runs.
        const N: usize = blocks_for(5000);
        assert_eq!(N, 10);
    }

    #[test]
    fn an_extent_of_bytes_covers_exactly_those_bytes() {
        let file = Extent::of_bytes(8, 5000);
        assert_eq!(file.first, 8);
        assert_eq!(file.count, 10);

        // The last block is only partly used, and the extent still owns it.
        assert_eq!(file.bytes(), 5120);
        assert!(file.bytes() >= 5000);

        assert!(file.contains(8));  // the first block
        assert!(file.contains(17)); // the last block
        assert!(!file.contains(7)); // one before
        assert!(!file.contains(18)); // half-open: one past the end is outside

        let nothing = Extent::of_bytes(8, 0);
        assert_eq!(nothing.count, 0);
        assert!(!nothing.contains(8));
    }

    #[test]
    fn an_extent_lists_its_blocks_in_order() {
        let e = Extent { first: 8, count: 4 };
        assert_eq!(e.blocks(), vec![8, 9, 10, 11]);

        assert!(Extent { first: 8, count: 0 }.blocks().is_empty());
        assert_eq!(Extent { first: 0, count: 1 }.blocks(), vec![0]);

        // Every listed block is one the extent claims, and there are as many
        // of them as the count says.
        let e = Extent::of_bytes(64, 3000);
        assert_eq!(e.blocks().len(), e.count as usize);
        assert!(e.blocks().iter().all(|&b| e.contains(b)));
    }

    #[test]
    fn extents_laid_end_to_end_leave_no_gap_and_no_overlap() {
        let a = Extent { first: 8, count: 4 };
        let b = a.after(2);
        assert_eq!(b, Extent { first: 12, count: 2 });

        // The boundary block belongs to exactly one of them.
        assert!(a.contains(11));
        assert!(!a.contains(12));
        assert!(b.contains(12));
    }

    #[test]
    fn a_device_number_packs_the_major_above_the_minor() {
        let disk = DevNo::new(MAJOR_DISK, 0);
        // 2 << 20 = 0x20_0000, and the minor adds nothing.
        assert_eq!(disk.0, 0x0020_0000);
        assert_eq!(disk.major(), MAJOR_DISK);
        assert_eq!(disk.minor(), 0);

        let part = DevNo::new(MAJOR_DISK, 3);
        assert_eq!(part.0, 0x0020_0003);
        assert_eq!(part.major(), MAJOR_DISK);
        assert_eq!(part.minor(), 3);

        // Derived PartialEq compares two DevNos directly.
        assert_eq!(part, DevNo(0x0020_0003));

        // Neither field may leak into the other.
        let big = DevNo::new(0xfff, MINOR_MASK);
        assert_eq!(big.major(), 0xfff);
        assert_eq!(big.minor(), MINOR_MASK);
    }

    #[test]
    fn a_device_number_can_be_built_at_compile_time() {
        // `const` forces the compiler to evaluate this before the program
        // runs. It only compiles because `DevNo::new` is a `const fn`.
        const CONSOLE1: DevNo = DevNo::new(MAJOR_CONSOLE, 1);
        assert_eq!(CONSOLE1.0, 0x0010_0001);
        assert!(CONSOLE1.is_console());

        assert!(CONSOLE.is_console());
        assert!(!DevNo::new(MAJOR_DISK, 0).is_console());

        // Whole tables get built this way in a kernel.
        const DEVS: [DevNo; 4] = [DevNo(0); 4];
        assert!(!DEVS[0].is_console());
    }

    #[test]
    fn the_guard_turns_interrupts_back_on_at_the_closing_brace() {
        let mut cpu = Cpu::new();
        assert!(cpu.intr_on);

        {
            let g = IntrGuard::acquire(&mut cpu);
            assert!(!g.intr_on());
            assert_eq!(g.depth(), 1);
            assert!(g.was_on());
        } // Nothing was called here. Interrupts are back on anyway.

        assert!(cpu.intr_on);
        assert_eq!(cpu.depth, 0);
    }

    #[test]
    fn releasing_early_restores_interrupts_early() {
        let mut cpu = Cpu::new();
        let g = IntrGuard::acquire(&mut cpu);
        assert!(!g.intr_on());

        // `release` takes the guard by value, so the guard dies inside it and
        // interrupts are back on this line rather than at the end of the test.
        // The exclusive borrow of `cpu` ends with it, which is the only reason
        // the next line is allowed to look at `cpu` at all.
        g.release();

        assert!(cpu.intr_on);
        assert_eq!(cpu.depth, 0);
    }

    #[test]
    fn a_guard_restores_the_state_it_found_not_a_guess() {
        // The CPU is already inside a critical section: interrupts are off
        // and somebody outside us is relying on that.
        let mut cpu = Cpu { intr_on: false, depth: 1 };
        {
            let inner = IntrGuard::acquire(&mut cpu);
            assert!(!inner.was_on()); // it found them OFF
            assert!(!inner.intr_on());
            assert_eq!(inner.depth(), 2);
        }

        // A `Drop` that simply switched interrupts ON would have broken the
        // outer critical section here. Restoring what was FOUND is the whole
        // difference between a guard and a bug.
        assert!(!cpu.intr_on);
        assert_eq!(cpu.depth, 1);
    }

    #[test]
    fn the_layout_is_what_the_disk_expects() {
        // These facts are the whole reason for `#[repr(transparent)]` and
        // `#[repr(C)]`, and they are worth seeing proved.

        // A newtype is its inner value, nothing more. No tag, no padding.
        assert_eq!(core::mem::size_of::<DevNo>(), core::mem::size_of::<u32>());

        // A #[repr(C)] Superblock of four u32 fields is exactly 16 bytes, laid
        // out in the order written — which is how `mkfs` and the kernel can
        // agree about a disk image neither of them wrote alone.
        assert_eq!(core::mem::size_of::<Superblock>(), 4 * 4);
        assert_eq!(core::mem::offset_of!(Superblock, magic), 0);
        assert_eq!(core::mem::offset_of!(Superblock, size), 4);
        assert_eq!(core::mem::offset_of!(Superblock, nblocks), 8);
        assert_eq!(core::mem::offset_of!(Superblock, ninodes), 12);

        assert!(!Superblock::zero().is_valid());
    }
}
