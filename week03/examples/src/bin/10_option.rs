//! 10 — Option<T>: absence with a type of its own.
//!
//! Rust has no null. A possibly-absent value has a different TYPE, defined in
//! the library rather than the language:
//!
//!     enum Option<T> { Some(T), None }
//!
//! Tony Hoare, who put null references into ALGOL W in 1965, called it his
//! "billion-dollar mistake" in 2009. The problem is not that absence exists --
//! it is that in C and Java an absent value has the SAME TYPE as a present one,
//! so the compiler cannot say which dereferences need checking.
//!
//!     File*        may be NULL. Nothing says so. Nothing checks.
//!     Option<File> a different type from File. The compiler makes you look.
//!
//! Run:  cargo run --bin 10_option

use std::mem::size_of;

fn main() {
    let table = device_table();

    println!("== a function that might have no answer says so in its type ==");
    println!("find_device(&table, 1) = {:?}", find_device(&table, 1));
    println!("find_device(&table, 9) = {:?}", find_device(&table, 9));
    println!("the second one is not an error and not a crash -- it is `None`");

    println!("\n== opening one up: match ==");
    for major in [1u32, 9] {
        let msg = match find_device(&table, major) {
            Some(d) => format!("major {} is {}", d.major, d.name),
            None => String::from("no such device"),
        };
        println!("  {msg}");
    }

    println!("\n== ... or if let, when only one case matters ==");
    if let Some(d) = find_device(&table, 2) {
        println!("  found {} at major {}", d.name, d.major);
    }

    println!("\n== ... or let ... else, to bind or bail ==");
    println!("  device_name(&table, 1) = {:?}", device_name(&table, 1));
    println!("  device_name(&table, 9) = {:?}", device_name(&table, 9));

    println!("\n== the combinators worth knowing ==");
    let found = find_device(&table, 1);
    let missing = find_device(&table, 9);
    println!("                         {:<18}  {}", "major 1 (present)", "major 9 (absent)");
    println!("  .is_some()             {:<18}  {}",
             found.is_some(), missing.is_some());
    println!("  .map(|d| d.name)       {:<18}  {}",
             fmt(found.map(|d| d.name)), fmt(missing.map(|d| d.name)));
    println!("  .and_then(|d| d.drv)   {:<18}  {}",
             fmt(found.and_then(|d| d.driver)), fmt(missing.and_then(|d| d.driver)));
    println!("  ... .unwrap_or(\"-\")    {:<18}  {}",
             found.map(|d| d.name).unwrap_or("-"),
             missing.map(|d| d.name).unwrap_or("-"));
    println!("  ... .ok_or(-1)         {:<18}  {}",
             format!("{:?}", found.map(|d| d.major).ok_or(-1)),
             format!("{:?}", missing.map(|d| d.major).ok_or(-1)));
    println!("`map` transforms what is there and leaves `None` alone;");
    println!("`and_then` is for when the step itself may also come up empty;");
    println!("`unwrap_or` supplies a fallback; `ok_or` turns absence into an error.");

    println!("\n== `?` on an Option: return None early ==");
    println!("  driver_of(&table, 1) = {:?}", driver_of(&table, 1));
    println!("  driver_of(&table, 3) = {:?}   <- device exists, no driver",
             driver_of(&table, 3));
    println!("  driver_of(&table, 9) = {:?}   <- no device at all",
             driver_of(&table, 9));

    println!("\n== Option is how you say `nothing is available` honestly ==");
    let mut pool = vec![10u32, 11, 12];
    for _ in 0..4 {
        match take_one(&mut pool) {
            Some(n) => println!("  took {n}, {} left", pool.len()),
            None => println!("  nothing left -- and the caller cannot ignore that"),
        }
    }
    println!("compare: C returns 0 or -1 or NULL and hopes you check");

    println!("\n== the cost ==");
    println!("size_of::<u32>()          = {}", size_of::<u32>());
    println!("size_of::<Option<u32>>()  = {}   <- 4 bytes plus a tag, aligned",
             size_of::<Option<u32>>());
    println!("size_of::<&u32>()         = {}", size_of::<&u32>());
    println!("size_of::<Option<&u32>>() = {}   <- FREE: null is the niche",
             size_of::<Option<&u32>>());
    println!("a reference can never be all-zeroes, so that pattern IS `None`");

    println!("\n== what you should almost never write ==");
    println!("  .unwrap()   panics on None -- in a kernel that halts the machine");
    println!("  .expect(\"\") the same, with a message");
    println!("Use them in tests and in code where None is genuinely impossible.");
    println!("Everywhere else, `match`, `if let`, `unwrap_or`, or `?`.");
}

/// One row of a device table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
    pub major: u32,
    pub name: &'static str,
    /// Not every device has a driver bound to it yet.
    pub driver: Option<&'static str>,
}

fn device_table() -> [Device; 3] {
    [
        Device { major: 1, name: "console", driver: Some("uart") },
        Device { major: 2, name: "disk", driver: Some("virtio") },
        Device { major: 3, name: "null", driver: None },
    ]
}

/// The answer may not exist, and the return type says so.
pub fn find_device(table: &[Device], major: u32) -> Option<Device> {
    for d in table {
        if d.major == major {
            return Some(*d);
        }
    }
    None
}

/// `let ... else`: bind the value, or leave.
pub fn device_name(table: &[Device], major: u32) -> Option<&'static str> {
    let Some(d) = find_device(table, major) else {
        return None;
    };
    Some(d.name)
}

/// `?` on an `Option` returns `None` from the whole function on the first miss.
pub fn driver_of(table: &[Device], major: u32) -> Option<&'static str> {
    let d = find_device(table, major)?;
    let name = d.driver?;
    Some(name)
}

/// "Nothing is available" is a value, not a sentinel and not a panic.
pub fn take_one(pool: &mut Vec<u32>) -> Option<u32> {
    if pool.is_empty() {
        return None;
    }
    let last = pool.len() - 1;
    Some(pool.remove(last))
}

/// Debug-format an Option in a fixed width, so the two columns line up.
fn fmt<T: std::fmt::Debug>(v: Option<T>) -> String {
    format!("{v:?}")
}
