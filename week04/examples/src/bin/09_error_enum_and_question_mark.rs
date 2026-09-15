//! 09 — Your own error type, and the `?` that carries it.
//!
//! `exec` opens a file, checks the ELF magic, checks the size fits in memory,
//! and only then loads. Three fallible steps, three different reasons to stop:
//!
//!     enum ExecError { NotFound, BadMagic, TooBig }
//!
//! The `?` operator is what makes the chain readable:
//!
//!     let bytes = open(name)?;         if open failed, return its Err NOW
//!     check_magic(bytes)?;             same
//!     let size = check_size(bytes)?;   same
//!
//! `expr?` is exactly
//!
//!     match expr {
//!         Ok(v)  => v,
//!         Err(e) => return Err(From::from(e)),
//!     }
//!
//! so it can only appear in a function that returns a Result (or Option): it
//! needs somewhere to `return Err` TO. The `From::from` is the quiet part: it
//! converts the error type on the way out, so an inner error can become an
//! outer one with no visible code. The last section shows that.
//!
//! Run:  cargo run --bin 09_error_enum_and_question_mark

/// One variant per way `load` can fail. Nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError {
    NotFound,
    BadMagic,
    TooBig,
}

/// A loaded program: for now, just how big it was.
#[derive(Debug, PartialEq, Eq)]
pub struct Image {
    pub size: usize,
}

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const MAX_IMAGE: usize = 64 * 1024;

// Three "files". A real filesystem arrives in 40k; today they are statics.
static INIT: [u8; 128] = {
    let mut a = [0u8; 128];
    a[0] = 0x7f;
    a[1] = b'E';
    a[2] = b'L';
    a[3] = b'F';
    a
};
static NOTES: &[u8] = b"remember to fix the scheduler\n";
static HUGE: [u8; 70_000] = {
    let mut a = [0u8; 70_000];
    a[0] = 0x7f;
    a[1] = b'E';
    a[2] = b'L';
    a[3] = b'F';
    a
};

/// A directory lookup: absence, so Option.
fn find_file(name: &str) -> Option<&'static [u8]> {
    match name {
        "init" => Some(&INIT),
        "notes.txt" => Some(NOTES),
        "huge" => Some(&HUGE),
        _ => None,
    }
}

/// The first fallible step. `.ok_or` decides what absence means here.
fn open(name: &str) -> Result<&'static [u8], ExecError> {
    find_file(name).ok_or(ExecError::NotFound)
}

/// The second. Nothing to return on success, so `Ok(())`.
fn check_magic(bytes: &[u8]) -> Result<(), ExecError> {
    if bytes.len() < 4 || bytes[..4] != ELF_MAGIC {
        return Err(ExecError::BadMagic);
    }
    Ok(())
}

/// The third.
fn check_size(bytes: &[u8]) -> Result<usize, ExecError> {
    if bytes.len() > MAX_IMAGE {
        return Err(ExecError::TooBig);
    }
    Ok(bytes.len())
}

/// The chain. Four lines; three exits.
fn load(name: &str) -> Result<Image, ExecError> {
    let bytes = open(name)?;
    check_magic(bytes)?;
    let size = check_size(bytes)?;
    Ok(Image { size })
}

/// The same function with every `?` written out.
#[allow(clippy::question_mark)] // clippy knows. That is the demo.
fn load_longhand(name: &str) -> Result<Image, ExecError> {
    let bytes = match open(name) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };
    match check_magic(bytes) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }
    let size = match check_size(bytes) {
        Ok(n) => n,
        Err(e) => return Err(e),
    };
    Ok(Image { size })
}

// ---- the last section: `?` converts on the way out ----------------------

/// The shell's errors: its own, plus anything exec can say.
#[derive(Debug, PartialEq, Eq)]
pub enum ShellError {
    Exec(ExecError),
    BadArgs,
}

/// Written once per pair of types. After this, `?` does the wrapping.
impl From<ExecError> for ShellError {
    fn from(e: ExecError) -> ShellError {
        ShellError::Exec(e)
    }
}

/// The shell runs a command line. Two error types meet here.
fn run_cmd(argv: &[&str]) -> Result<Image, ShellError> {
    let name = argv.first().ok_or(ShellError::BadArgs)?;
    let image = load(name)?; // ExecError -> ShellError, via From, silently
    Ok(image)
}

fn main() {
    println!("== each ? is a different exit ==");
    for name in ["init", "nope", "notes.txt", "huge"] {
        println!("load({name:>10}) -> {:?}", load(name));
    }
    println!("init passed all three checks; the other three each stopped at a");
    println!("different `?`, and the Err says which one.");

    println!("\n== longhand: what ? expands to ==");
    for name in ["init", "nope", "notes.txt", "huge"] {
        assert_eq!(load(name), load_longhand(name));
    }
    println!("load_longhand agrees on all four. It is 15 lines where load is 4,");
    println!("and every one of those `Err(e) => return Err(e)` is the same line.");

    println!("\n== ? needs somewhere to return to ==");
    println!("    fn sys_exec(name: &str) -> i64 {{ let img = load(name)?; ... }}");
    println!("    error[E0277]: the `?` operator can only be used in a function that");
    println!("                  returns `Result` or `Option`");
    println!("a function returning an integer has no Err to return. That is the");
    println!("boundary, and the boundary is where ? stops and match begins.");
    println!("RUN  ./show-errors.sh e0277");

    println!("\n== From: ? converts the error on the way out ==");
    println!("run_cmd(&[])       -> {:?}", run_cmd(&[]));
    println!("run_cmd(&[\"nope\"]) -> {:?}", run_cmd(&["nope"]));
    println!("run_cmd(&[\"init\"]) -> {:?}", run_cmd(&["init"]));
    println!("load returned Err(ExecError::NotFound). run_cmd returned");
    println!("Err(ShellError::Exec(NotFound)). Nobody wrote the wrapping at the");
    println!("call: `?` called From::from, and `impl From<ExecError> for ShellError`");
    println!("was written once.");
}
