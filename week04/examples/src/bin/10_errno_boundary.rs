//! 10 — Where a `Result` becomes a number.
//!
//! A user program does not receive a Result. It receives one register, `a0`.
//! So somewhere at the very edge of the kernel a rich error has to collapse
//! into an integer, and the Unix convention is:
//!
//!     >= 0     success, and the value
//!     <  0     minus an error number:  -9 EBADF, -21 EISDIR, ...
//!
//! The rule this program is about: that collapse happens in EXACTLY ONE
//! PLACE. Everything behind it speaks `FdError`; everything in front of it
//! speaks integers; the `match` in `sys_read` is the wall between them.
//!
//!     read_fd  -> Result<usize, FdError>      the kernel talking to itself
//!     sys_read -> i64                         the kernel talking to a user
//!
//! Run:  cargo run --bin 10_errno_boundary

/// One open file, as the per-process table sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct File {
    pub inum: u32,
    pub readable: bool,
    pub is_dir: bool,
}

/// Every way a read through a descriptor can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdError {
    BadFd,
    NotReadable,
    IsADirectory,
}

// The real numbers. Linux's, xv6-adjacent, and ours from 50k on.
pub const EBADF: i64 = 9;
pub const EISDIR: i64 = 21;

/// Read from descriptor `fd`. Returns how many bytes: the file's size, here.
fn read_fd(fds: &[Option<File>], fd: usize) -> Result<usize, FdError> {
    // .get(fd) is the bounds check as an Option; .copied().flatten() turns
    // Option<&Option<File>> into Option<File>; .ok_or names what None means.
    let file = fds.get(fd).copied().flatten().ok_or(FdError::BadFd)?;
    if file.is_dir {
        return Err(FdError::IsADirectory);
    }
    if !file.readable {
        return Err(FdError::NotReadable);
    }
    Ok(512 * file.inum as usize) // pretend: file size is 512 x inum
}

/// THE BOUNDARY. One match. The only place these numbers are ever spelled.
fn sys_read(fds: &[Option<File>], fd: i64) -> i64 {
    // The raw register first: a negative fd was never an index at all.
    if fd < 0 {
        return -EBADF;
    }
    match read_fd(fds, fd as usize) {
        Ok(n) => n as i64,
        Err(FdError::BadFd) | Err(FdError::NotReadable) => -EBADF,
        Err(FdError::IsADirectory) => -EISDIR,
    }
}

/// What rv6 does today. The cause is lost at the wall.
fn sys_read_lossy(fds: &[Option<File>], fd: i64) -> i64 {
    if fd < 0 {
        return -1;
    }
    match read_fd(fds, fd as usize) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

/// ulib's side of the wall: turn the number back into a Result. Eight lines
/// in ulib/src/lib.rs, and this is all of them.
#[derive(Debug, PartialEq, Eq)]
pub struct UlibError(pub i32);
fn ulib_read(fds: &[Option<File>], fd: i64) -> Result<usize, UlibError> {
    let rc = sys_read(fds, fd);
    if rc < 0 {
        Err(UlibError(rc as i32))
    } else {
        Ok(rc as usize)
    }
}

fn main() {
    let fds: [Option<File>; 5] = [
        Some(File { inum: 1, readable: true, is_dir: false }),  // 0 stdin
        Some(File { inum: 1, readable: false, is_dir: false }), // 1 stdout: write only
        Some(File { inum: 1, readable: false, is_dir: false }), // 2 stderr
        Some(File { inum: 3, readable: true, is_dir: false }),  // 3 a file
        Some(File { inum: 5, readable: true, is_dir: true }),   // 4 a directory
    ];

    println!("== behind the wall: read_fd speaks FdError ==");
    for fd in [0usize, 1, 3, 4, 9] {
        println!("read_fd(fd={fd})  -> {:?}", read_fd(&fds, fd));
    }

    println!("\n== at the wall: sys_read speaks i64 ==");
    println!("{:>6} {:>10} {:>12}", "fd", "sys_read", "meaning");
    for fd in [0i64, 1, 3, 4, 9, -1] {
        let rc = sys_read(&fds, fd);
        let meaning = match rc {
            n if n >= 0 => "bytes".to_string(),
            n if n == -EBADF => "-EBADF".to_string(),
            n if n == -EISDIR => "-EISDIR".to_string(),
            _ => "?".to_string(),
        };
        println!("{fd:>6} {rc:>10} {meaning:>12}");
    }
    println!("one match. Add a variant to FdError and that match stops compiling");
    println!("until it has a number. Nothing else in the kernel ever spells 21.");

    println!("\n== the lossy wall: rv6 today ==");
    for fd in [3i64, 4, 9] {
        println!("sys_read_lossy(fd={fd}) -> {}", sys_read_lossy(&fds, fd));
    }
    println!("`Err(_) => -1` throws the cause away. That is why ulib::Error is");
    println!("a thin i32 and not an enum: the kernel genuinely does not say more.");

    println!("\n== in front of the wall: ulib rebuilds a Result from the number ==");
    for fd in [3i64, 4] {
        println!("ulib_read(fd={fd}) -> {:?}", ulib_read(&fds, fd));
    }
    println!("Result -> i64 -> Result. Types do not cross the trap; numbers do.");

    println!("\n== unwrap, expect, and the dead machine ==");
    let n = read_fd(&fds, 3).expect("fd 3 was just opened");
    println!("read_fd(3).expect(..)  = {n}   honest: the table above shows it is open");
    println!("read_fd(9).unwrap()    would panic:");
    println!("    called `Result::unwrap()` on an `Err` value: BadFd");
    println!("on your laptop that is one red test. On rv6 it prints `panic` and");
    println!("the machine stops. Rule: Result for what the world did to you,");
    println!("panic for what you did to yourself.");

    println!("\n== the siblings that do not panic ==");
    println!("read_fd(9).unwrap_or(0) = {}", read_fd(&fds, 9).unwrap_or(0));
    if let Ok(n) = read_fd(&fds, 0) {
        println!("if let Ok(n) = read_fd(0)  -> {n}");
    }
    println!("read_fd(4).is_err()     = {}", read_fd(&fds, 4).is_err());

    // UNCOMMENT to watch it die:
    // let _ = read_fd(&fds, 99).unwrap();
}
