// error[E0382]: use of moved value
//
// `flags` takes `self` BY VALUE, so calling it moves the entry away and the
// second call has nothing left. A by-value receiver is a move unless the type
// is Copy -- and this one is Clone but not Copy.
//
// Copying four bytes was always cheap. What `Copy` buys is that assignment
// stops MOVING, which is what lets a small type offer by-value methods.
//
// FIX 1: `#[derive(Clone, Copy)]` -- the call then copies four bytes.
// FIX 2: take `&self` in both methods -- borrowing never moves. That is the
//        answer for anything too big or too resource-owning to copy.
#[derive(Clone)]
struct Entry(u32);

impl Entry {
    fn flags(self) -> u32 {
        self.0 & 0x3f
    }

    fn is_valid(self) -> bool {
        self.flags() & 1 != 0
    }
}

fn main() {
    let e = Entry(0x2000_0007);
    println!("{} {}", e.flags(), e.is_valid());
}
