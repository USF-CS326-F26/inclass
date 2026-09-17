//! 13 — Who picks the type: the call site, the code, or the input.
//!
//! Programs 06 and 07 chose every type at compile time: `banner(&mut rec)`
//! names Recorder on the spot. A kernel cannot. `sys_write(1, ..)` has to
//! work whether fd 1 was left on the console or redirected into a file by a
//! shell that ran long after the kernel was built, so something has to hold
//! a choice the compiler never saw:
//!
//!     who names the type       how it dispatches        in rv6
//!     the call site            generic: a copy per U    Scheduler/RoundRobin
//!     the code, once           dyn: one body, a vtable  Out in the shell
//!     the input, at run time   dyn, or a tag + match    the fd table
//!
//! Only the third row is a question the compiler cannot answer. This program
//! answers it twice -- once with `&mut dyn Uart` and once with rv6's own
//! answer, a one-byte TAG and a `match` -- from the same argv, so the two
//! print the same bytes in the same places.
//!
//! Run:  cargo run --bin 13_dispatch_at_run_time
//!       cargo run --bin 13_dispatch_at_run_time -- file file null

use std::mem::size_of;

pub trait Uart {
    fn putc(&mut self, b: u8);
    fn puts(&mut self, s: &[u8]) {
        for &b in s {
            self.putc(b);
        }
    }
    fn putln(&mut self, s: &[u8]) {
        self.puts(s);
        self.putc(b'\n');
    }
}

pub struct Console;
impl Uart for Console {
    fn putc(&mut self, b: u8) {
        print!("{}", b as char);
    }
}

pub struct Recorder(pub Vec<u8>);
impl Uart for Recorder {
    fn putc(&mut self, b: u8) {
        self.0.push(b);
    }
}

pub struct Tally(pub usize);
impl Uart for Tally {
    fn putc(&mut self, _: u8) {
        self.0 += 1;
    }
}

/// Static dispatch: the call site names U, so the compiler emits one copy of
/// this body per U, each at its own address.
fn log_line<U: Uart>(u: &mut U, s: &[u8]) {
    u.putln(s);
}

/// Dynamic dispatch: one copy, at one address, whatever it is handed.
fn log_line_dyn(u: &mut dyn Uart, s: &[u8]) {
    u.putln(s);
}

/// rv6's `FileKind`: what `sys_open` worked out, kept in the file table.
/// One byte wide, `Copy`, and no vtable anywhere near it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Tty,
    File,
    Null,
}

impl Kind {
    /// The one line where a string the compiler never saw becomes a choice.
    fn of(name: &str) -> Kind {
        match name {
            "file" => Kind::File,
            "null" => Kind::Null,
            _ => Kind::Tty,
        }
    }
}

/// xv6's `devsw[]`, with names where it has major numbers.
struct Sinks {
    tty: Console,
    file: Recorder,
    null: Tally,
}

impl Sinks {
    /// `dyn` in RETURN position: one slot, holding whichever sink the tag
    /// names. Three arms, three types, and no cast anywhere.
    fn open(&mut self, k: Kind) -> &mut dyn Uart {
        match k {
            Kind::Tty => &mut self.tty,
            Kind::File => &mut self.file, // <- each arm unsizes itself: no `as`
            Kind::Null => &mut self.null,
        }
    }

    /// The same choice with no trait object: rv6's `sys_write`. Every arm is
    /// a direct call into one of `log_line`'s copies.
    fn write(&mut self, k: Kind, s: &[u8]) {
        match k {
            Kind::Tty => log_line(&mut self.tty, s),
            Kind::File => log_line(&mut self.file, s),
            Kind::Null => log_line(&mut self.null, s),
        }
    }
}

fn main() {
    let mut script: Vec<String> = std::env::args().skip(1).collect();
    if script.is_empty() {
        script = ["tty", "file", "null"].iter().map(|s| s.to_string()).collect();
    }
    let mut sinks = Sinks { tty: Console, file: Recorder(Vec::new()), null: Tally(0) };

    println!("== one body, three addresses ==");
    println!("log_line::<Console>  at {:p}", log_line::<Console> as *const ());
    println!("log_line::<Recorder> at {:p}   a second function", log_line::<Recorder> as *const ());
    println!("log_line::<Tally>    at {:p}   and a third", log_line::<Tally> as *const ());
    println!("log_line_dyn         at {:p}   one copy, for all three",
             log_line_dyn as *const ());
    println!("is Console's copy Recorder's? {}",
             log_line::<Console> as *const () == log_line::<Recorder> as *const ());
    println!("program 06 printed three type names. These are the three functions");
    println!("behind them: naming `log_line::<Console>` is what made the copy.");
    println!("(the addresses move every run; that they DIFFER is the point.)");

    println!("\n== a tag the source does not know ==");
    println!("script = {:?}   <- argv, or this default", script);
    for name in &script {
        let k = Kind::of(name); // <- the whole run-time choice, in one byte
        let out = sinks.open(k);
        out.putln(b"  rv6 ready");
        println!("  file {:2} bytes, null {:2}   <- Kind::of({:?}) = {:?}",
                 sinks.file.0.len(), sinks.null.0, name, k);
    }
    println!("ONE slot, re-pointed once per step, and `out` never learned which");
    println!("sink it held. A step that opened the tty printed a line; the rest");
    println!("went into a Vec and a counter, which is what the counts are.");

    println!("\n== the same tag, with no vtable ==");
    sinks.file.0.clear();
    sinks.null.0 = 0; // <- the same script again, so the two blocks compare
    for name in &script {
        let k = Kind::of(name); // <- the same one byte, and no vtable after it
        sinks.write(k, b"  rv6 ready");
        println!("  file {:2} bytes, null {:2}   <- Kind::of({:?}) = {:?}",
                 sinks.file.0.len(), sinks.null.0, name, k);
    }
    println!("the same bytes in the same places, through a compare and a jump:");
    println!("size_of::<Kind>() = {}, against size_of::<&mut dyn Uart>() = {}.",
             size_of::<Kind>(), size_of::<&mut dyn Uart>());
    println!("add `Pipe` to Kind and every match missing that arm stops");
    println!("compiling. Add a fourth `impl Uart` and nothing stops -- or notices.");

    println!("\n== why open's return type cannot be a generic ==");
    println!("    fn open(&mut self, k: Kind) -> &mut impl Uart   // ONE type, not three");
    println!("    error[E0308]: `match` arms have incompatible types");
    println!("        Kind::File => &mut self.file,   found to be `&mut Recorder`");
    println!("        Kind::Null => &mut self.null,   expected that, found `&mut Tally`");
    println!("`impl Uart` is one type the compiler works out and you may not name.");
    println!("`dyn Uart` is one type that stands in for all of them, which is why");
    println!("it is what a slot filled at run time has to hold.");
    println!("RUN  ./show-errors.sh e0107");

    println!("\n== C fills the same table in by hand ==");
    println!("    struct devsw {{ int (*read)(..); int (*write)(..); }};  // file.h");
    println!("    devsw[CONSOLE].write = consolewrite;    // console.c, at boot");
    println!("    if (!devsw[major].write) return -1;     // the slot may be empty");
    println!("that is a vtable, written by hand, one per device instead of one per");
    println!("type -- and that `if` guards a slot C cannot prove anybody filled.");

    println!("\n== which one to reach for ==");
    println!("generic  the call site names the type and the call is hot:");
    println!("         Scheduler/RoundRobin, because pick_next runs every tick.");
    println!("dyn      one slot must hold a type chosen later, or one copy of a");
    println!("         big body is worth an indirect call: Out in the shell.");
    println!("tag      the set is closed and small, and there is no allocator:");
    println!("         FileKind in the fd table, read and write branching on it.");
    println!("reach for dyn when the type must outlive the decision, not because");
    println!("the word sounds dynamic. 06 pays code size for speed, 07 shows the");
    println!("pointer, and this one shows who is doing the choosing.");
}
