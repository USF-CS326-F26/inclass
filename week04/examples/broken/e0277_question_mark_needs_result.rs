// error[E0277]: the `?` operator can only be used in a function that returns
//               `Result` or `Option` (or another type that implements
//               `FromResidual`)
//
// `?` is a `return Err(e)` in disguise, and this function returns i64 -- there
// is no Err to return. This IS the boundary: the place where `?` stops and
// `match` starts, because past here the caller only understands a number.
// 10c's `fn run(args: Args) -> i32` is the same wall.
//
// FIX 1: `match reg.lookup(name) { Ok(d) => .., Err(e) => -errno_for(e) }`
//        -- collapse to a number HERE, exactly once.
// FIX 2: move the `?` down into a helper that returns Result; match its result here.
#[derive(Debug, Clone, Copy)]
struct Device {
    major: u32,
}

#[derive(Debug)]
enum DevError {
    NoSuchDevice,
}

struct Registry;

impl Registry {
    fn lookup(&self, name: &str) -> Result<Device, DevError> {
        if name == "console" { Ok(Device { major: 1 }) } else { Err(DevError::NoSuchDevice) }
    }
    fn getc(&self, dev: Device) -> Result<u8, DevError> {
        if dev.major == 1 { Ok(b'l') } else { Err(DevError::NoSuchDevice) }
    }
}

fn sys_getc(reg: &Registry, name: &str) -> i64 {
    let dev = reg.lookup(name)?;
    reg.getc(dev).map(|b| b as i64).unwrap_or(-1)
}

fn main() {
    println!("{}", sys_getc(&Registry, "console"));
}
