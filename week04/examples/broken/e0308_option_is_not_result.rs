// error[E0308]: mismatched types
//               expected `Result<Device, DevError>`, found `Option<Device>`
//
// `find` reports ABSENCE: a name that is not there, nothing wrong. `lookup`
// must report FAILURE, and nobody said what absence MEANS here. The two enums
// do not convert themselves -- that decision is yours, and it is one line.
//
// FIX 1: `self.find(name).ok_or(DevError::NoSuchDevice)` -- Some -> Ok,
//        None -> Err(the one you chose). That choice is the policy.
// FIX 2: `match self.find(name) { Some(d) => Ok(d), None => Err(..) }` --
//        the same thing, longhand.
#[derive(Debug, Clone, Copy)]
struct Device {
    major: u32,
}

#[derive(Debug)]
enum DevError {
    NoSuchDevice,
    NameTooLong,
}

struct Registry {
    names: Vec<(&'static str, Device)>,
}

impl Registry {
    fn find(&self, name: &str) -> Option<Device> {
        self.names.iter().find(|(n, _)| *n == name).map(|(_, d)| *d)
    }

    fn lookup(&self, name: &str) -> Result<Device, DevError> {
        if name.len() > 8 {
            return Err(DevError::NameTooLong);
        }
        self.find(name)
    }
}

fn main() {
    let reg = Registry { names: vec![("console", Device { major: 1 })] };
    println!("{:?}", reg.lookup("console").map(|d| d.major));
}
