// ╔══════════════════════════════════════════════════════════════════════╗
// ║  08r — Errors as Values — IN-CLASS EXAMPLE                           ║
// ║  Same shape as the exercise, different nouns: a registry of devices  ║
// ║  instead of a directory of files, four ways a read can fail instead  ║
// ║  of three, and the same system-call edge where a Result becomes a    ║
// ║  number.                                                             ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/08r_errors/skeleton/lib.rs`. Every item here
// has a twin there. If you can explain why the twins are the same shape, you
// have understood errors as values; the domain is decoration.
//
// Absence becomes failure at TWO sites here, with two different policies,
// and success is a byte rather than a length. Both are worth asking about.

/// The longest name the device table can hold.
///
/// UNDERSTAND: twin of `NAME_MAX`. A device name lives in a fixed-size record,
///   so a name that does not fit is not unusual input -- it is impossible
///   input, and the registry says so rather than truncating it silently.
pub const DEV_NAME_MAX: usize = 8;

/// Everything that can go wrong reading a byte from a device by name.
///
/// UNDERSTAND: twin of `FsError`. An ordinary enum, one variant per way a
///   call can fail, so that the caller can `match` on it and the compiler
///   checks the arms are exhaustive. Four variants rather than three because
///   absence turns into failure at two different places below, and each place
///   gets to say what absence MEANS there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevError {
    /// No device is registered under that name.
    NoSuchDevice,
    /// The name is longer than the table can hold.
    NameTooLong,
    /// The device is a block device; it has no byte stream to read from.
    NotATty,
    /// The device is a character device with nothing waiting right now.
    WouldBlock,
}

/// Bytes on demand (a console) or blocks on demand (a disk).
///
/// UNDERSTAND: twin of `InodeKind`. `getc` makes sense for one and not the
///   other, and the kind is how it knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevKind {
    Char,
    Block,
}

/// The registry's record of one device.
///
/// UNDERSTAND: twin of `Inode`. A device does not hold its own name -- the
///   name is the registry's, and the record is what the name resolves to. So
///   resolving a name and reading from the device are two steps, and each can
///   fail for its own reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
    /// The driver number. Also the index into the input queues below.
    pub major: u32,
    pub kind: DevKind,
}

/// The whole (very small) device layer: the name table, and the bytes waiting
/// to be read from each character device.
///
/// UNDERSTAND: twin of `FileSystem`. No `pub` on the fields: everything you
///   do to this type is a method on it. `input[major]` is what the hardware
///   has delivered and nobody has read yet -- a keyboard buffer, in effect.
pub struct Registry {
    entries: Vec<(String, Device)>,
    input: Vec<Vec<u8>>,
}

// The real Unix numbers, and the ones `sys_getc` speaks.
pub const ENODEV: i64 = 19;
pub const ENAMETOOLONG: i64 = 36;
pub const ENOTTY: i64 = 25;
pub const EAGAIN: i64 = 11;

impl Registry {
    /// Four devices: a console with a line waiting, a disk, `null`, `zero`.
    pub fn new() -> Registry {
        let mut reg = Registry { entries: Vec::new(), input: vec![Vec::new()] };
        reg.add("console", DevKind::Char, b"ls\n");
        reg.add("disk", DevKind::Block, b"");
        reg.add("null", DevKind::Char, b"");
        reg.add("zero", DevKind::Char, &[0]);
        reg
    }

    /// Install one device. The major number is just the next free slot.
    fn add(&mut self, name: &str, kind: DevKind, pending: &[u8]) {
        let major = self.input.len() as u32;
        self.input.push(pending.to_vec());
        self.entries.push((name.to_string(), Device { major, kind }));
    }

    /// Deliver bytes to a device's input queue, as an interrupt would.
    /// Host-only; the demo uses it to make `WouldBlock` come and go.
    pub fn push_input(&mut self, name: &str, bytes: &[u8]) {
        if let Some(dev) = self.find(name) {
            self.input[dev.major as usize].extend_from_slice(bytes);
        }
    }

    /// Is there a device called `name`? Its record if so.
    ///
    /// UNDERSTAND: twin of `find`. Absence is not failure: a name that is not
    ///   registered is the honest answer to a question, and `Option` is the
    ///   type for it. The policy that "not there" means "this call failed"
    ///   belongs one level up.
    pub fn find(&self, name: &str) -> Option<Device> {
        self.entries.iter().find(|(n, _)| n == name).map(|(_, d)| *d)
    }

    /// Resolve `name` to a device, or say why we could not.
    ///
    /// UNDERSTAND: twin of `lookup`, and the first of two `.ok_or`s in this
    ///   file. `Some(d)` becomes `Ok(d)`; `None` becomes `Err(NoSuchDevice)`.
    ///   That one word is the whole policy. The length check comes first,
    ///   because a name that cannot fit cannot be registered, and "too long"
    ///   tells the caller more than "not found".
    pub fn lookup(&self, name: &str) -> Result<Device, DevError> {
        if name.len() > DEV_NAME_MAX {
            return Err(DevError::NameTooLong);
        }
        self.find(name).ok_or(DevError::NoSuchDevice)
    }

    /// The next byte waiting on `dev`, without consuming it.
    ///
    /// UNDERSTAND: twin of `read`, with a second `.ok_or`. A block device has
    ///   no byte stream: `NotATty`, the same shape as the exercise's
    ///   `IsADirectory`. On a character device `.first()` is an
    ///   `Option<&u8>` -- absence again -- and THIS site decides that absence
    ///   means `WouldBlock`. Same operator as `lookup`, different decision;
    ///   that is why the enum has four variants, not three.
    pub fn getc(&self, dev: Device) -> Result<u8, DevError> {
        if dev.kind == DevKind::Block {
            return Err(DevError::NotATty);
        }
        self.input[dev.major as usize].first().copied().ok_or(DevError::WouldBlock)
    }

    /// Look a name up and read its next byte, in one call.
    ///
    /// UNDERSTAND: twin of `read_file`, and the shape of nearly every kernel
    ///   function that touches a device or a file: a chain of fallible steps,
    ///   each ending in `?`. `?` means "if that was an `Err`, return it from
    ///   THIS function right now; otherwise unwrap the `Ok` and carry on".
    ///   It is legal here only because this function returns a `Result` --
    ///   there is somewhere for the `Err` to go.
    pub fn getc_by_name(&self, name: &str) -> Result<u8, DevError> {
        let dev = self.lookup(name)?;
        self.getc(dev)
    }

    /// The system-call boundary: read a byte, but report the outcome as one
    /// integer, the way a system call must.
    ///
    /// UNDERSTAND: twin of `sys_read`. A user program gets one number in
    ///   register `a0`: `>= 0` is the byte, negative is minus an error code.
    ///   This `match` is the ONLY place in the kernel that turns a `DevError`
    ///   into a number, and it lists every variant on purpose: add one to the
    ///   enum and this stops compiling until it has a number. Notice `zero`
    ///   returns `0`, and that is success -- the sign carries the meaning.
    pub fn sys_getc(&self, name: &str) -> i64 {
        match self.getc_by_name(name) {
            Ok(byte) => byte as i64,
            Err(DevError::NoSuchDevice) => -ENODEV,
            Err(DevError::NameTooLong) => -ENAMETOOLONG,
            Err(DevError::NotATty) => -ENOTTY,
            Err(DevError::WouldBlock) => -EAGAIN,
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// The tests. Read them first: they are the contract, and they are the same
// kind of contract the exercise's tests are.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // Two names that are not registered. The first is exactly as long as the
    // table allows; the second is one byte longer.
    const ABSENT: &str = "eightchr";
    const TOO_LONG: &str = "ninechars";

    #[test]
    fn a_registered_name_is_found() {
        let reg = Registry::new();
        let dev = reg.find("console").expect("console is registered");
        assert_eq!(dev.major, 1);
        assert_eq!(dev.kind, DevKind::Char);
        assert_eq!(reg.find("disk").map(|d| d.kind), Some(DevKind::Block));
    }

    #[test]
    fn an_absent_name_is_none_and_not_a_panic() {
        let reg = Registry::new();
        assert_eq!(ABSENT.len(), DEV_NAME_MAX);
        assert_eq!(reg.find(ABSENT), None);
        // find reports absence, not policy: an over-long name is simply not
        // there, and it is lookup's job to call that an error.
        assert_eq!(reg.find(TOO_LONG), None);
    }

    #[test]
    fn lookup_turns_absence_into_a_named_error() {
        let reg = Registry::new();
        assert_eq!(reg.lookup("disk").map(|d| d.major), Ok(2));
        assert_eq!(reg.lookup(ABSENT), Err(DevError::NoSuchDevice));
    }

    #[test]
    fn a_name_too_long_for_the_table_is_rejected_as_too_long() {
        let reg = Registry::new();
        assert_eq!(TOO_LONG.len(), DEV_NAME_MAX + 1);
        // Not NoSuchDevice: the caller learns WHY.
        assert_eq!(reg.lookup(TOO_LONG), Err(DevError::NameTooLong));
    }

    #[test]
    fn a_char_device_with_input_hands_over_a_byte() {
        let reg = Registry::new();
        let console = reg.lookup("console").unwrap();
        assert_eq!(reg.getc(console), Ok(b'l'));
    }

    #[test]
    fn a_block_device_is_not_a_tty() {
        let reg = Registry::new();
        let disk = reg.lookup("disk").unwrap();
        assert_eq!(disk.kind, DevKind::Block);
        assert_eq!(reg.getc(disk), Err(DevError::NotATty));
    }

    #[test]
    fn a_char_device_with_nothing_pending_would_block() {
        let mut reg = Registry::new();
        let null = reg.lookup("null").unwrap();
        assert_eq!(reg.getc(null), Err(DevError::WouldBlock));
        // The same device, once the hardware has delivered something.
        reg.push_input("null", b"\n");
        assert_eq!(reg.getc(null), Ok(b'\n'));
    }

    #[test]
    fn getc_by_name_passes_the_first_error_straight_up() {
        let reg = Registry::new();
        assert_eq!(reg.getc_by_name("console"), Ok(b'l'));
        // Each of these stops at a different `?`, and getc_by_name reports
        // the failure unchanged rather than inventing one of its own.
        assert_eq!(reg.getc_by_name(ABSENT), Err(DevError::NoSuchDevice));
        assert_eq!(reg.getc_by_name(TOO_LONG), Err(DevError::NameTooLong));
        assert_eq!(reg.getc_by_name("disk"), Err(DevError::NotATty));
        assert_eq!(reg.getc_by_name("null"), Err(DevError::WouldBlock));
    }

    #[test]
    fn the_system_call_boundary_turns_errors_into_negative_errnos() {
        let reg = Registry::new();
        assert_eq!(reg.sys_getc("console"), b'l' as i64);
        assert_eq!(reg.sys_getc(ABSENT), -19); // ENODEV
        assert_eq!(reg.sys_getc(TOO_LONG), -36); // ENAMETOOLONG
        assert_eq!(reg.sys_getc("disk"), -25); // ENOTTY
        assert_eq!(reg.sys_getc("null"), -11); // EAGAIN
    }

    #[test]
    fn a_zero_byte_is_a_success_not_an_error() {
        // /dev/zero hands out 0x00 forever. At the boundary that is the
        // number 0 -- and 0 is not negative, so it is success. The sign is
        // the whole convention.
        let reg = Registry::new();
        assert_eq!(reg.getc_by_name("zero"), Ok(0));
        assert_eq!(reg.sys_getc("zero"), 0);
    }
}
