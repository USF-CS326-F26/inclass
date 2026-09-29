//! 06 — `len` counts bytes. A character may be several of them.
//!
//! Four types can hold text, and they promise different things:
//!
//!     u8       1 byte                        nothing: any of 256 values
//!     char     4 bytes                       one Unicode scalar value
//!     &str     16 bytes (pointer, length)    the bytes are valid UTF-8
//!     &[u8]    16 bytes (pointer, length)    nothing
//!
//! In UTF-8 a character takes 1 to 4 bytes, and ASCII takes exactly one. The
//! first byte says how long the character is. Every byte after it has the
//! form 10xxxxxx:
//!
//!     a    0110_0001              61         top bit clear: ASCII
//!     ñ    1100_0011 1011_0001    C3 B1      top bit set, both bytes
//!
//! So a byte-wise search for an ASCII byte, like b'\n', can never land
//! inside ñ. That is why cat, wc and grep can treat every file as bytes.
//!
//! Run:  cargo run --bin 06_bytes_chars_str

use std::mem::size_of;
use std::str;

/// One byte against several patterns at once.
fn is_vowel(b: u8) -> bool {
    matches!(b, b'a' | b'e' | b'i' | b'o' | b'u')
}

fn main() {
    println!("== one char, two bytes ==");
    println!("'ñ'.len_utf8()         = {}   one char, two bytes", 'ñ'.len_utf8());
    println!("\"año\".len()            = {}   len counts bytes", "año".len());
    println!("\"año\".chars().count()  = {}   chars decodes them first", "año".chars().count());
    println!("size_of::<char>()      = {}   a char is not a byte", size_of::<char>());
    println!("size_of::<&str>()      = {}  and a &str is a pointer and a length,", size_of::<&str>());
    println!("size_of::<&[u8]>()     = {}  exactly like a &[u8]. Only the promise differs.",
             size_of::<&[u8]>());

    println!("\n== the bytes of a file holding año and a newline ==");
    let file = "año\n".as_bytes();
    for &b in file {
        println!("  {b:02X}  {b:08b}  {}", if b < 0x80 { "ASCII" } else { "part of a longer char" });
    }
    println!("{} bytes for 3 letters and a newline. `wc` reports {}.", file.len(), file.len());

    println!("\n== one to four bytes ==");
    for s in ["a", "ñ", "€", "🦀"] {
        let bytes: Vec<String> = s.bytes().map(|b| format!("{b:08b}")).collect();
        println!("  {s}  {} byte(s)  {}", s.len(), bytes.join(" "));
    }
    println!("a first byte of 0xxxxxxx is ASCII; 110, 1110 and 11110 say 2, 3 and 4 bytes.");
    println!("every byte after the first starts 10.");

    println!("\n== a search for b'\\n' cannot land inside a character ==");
    let text = "año\nniño\n".as_bytes();
    let newlines: Vec<usize> = (0..text.len()).filter(|&i| text[i] == b'\n').collect();
    let high: Vec<usize> = (0..text.len()).filter(|&i| text[i] >= 0x80).collect();
    println!("b'\\n' at {newlines:?}");
    println!("bytes >= 0x80 at {high:?}   the two ñ, two bytes each");
    println!("0x0A has its top bit clear, and no byte of ñ does. They never collide.");

    println!("\n== b\"-n\" is bytes; \"-n\" is a str ==");
    let flag: &[u8; 2] = b"-n";
    let arg: &[u8] = b"-n"; // what argv hands you
    println!("b\"-n\"                  = {flag:?}   a &[u8; 2]");
    println!("arg == b\"-n\"           -> {}", arg == b"-n");
    println!("arg == \"-n\".as_bytes() -> {}   the same two bytes, reached from a str",
             arg == "-n".as_bytes());
    println!("b'a' = {}, 'a' as u8 = {}, 97u8 as char = {:?}", b'a', 'a' as u8, 97u8 as char);
    println!("a u8 and a char never compare directly. RUN  ./show-errors.sh e0308");

    println!("\n== matches! tests a byte against several patterns ==");
    let word = b"rv6 boots";
    let marks: String = word.iter()
        .map(|&b| if is_vowel(b) { 'v' } else if matches!(b, b'0'..=b'9') { 'd' } else { '.' })
        .collect();
    println!("  {}", String::from_utf8_lossy(word));
    println!("  {marks}   v = a vowel, d = a digit");
    println!("matches!(0xC3, b'a'..=b'z') = {}   ñ's bytes are not letters to a byte test",
             matches!(0xC3u8, b'a'..=b'z'));

    println!("\n== when you really want characters, you pay for a check ==");
    let cut = &"año".as_bytes()[..2]; // a, then the first byte of ñ
    println!("from_utf8(&\"año\".as_bytes()[..2]) = {:?}", str::from_utf8(cut));
    println!("from_utf8_lossy of the same bytes   = {:?}", String::from_utf8_lossy(cut));
    println!("a read that ends between C3 and B1 hands you exactly that.");
    println!("wc -m, which counts characters, has to decode every byte to answer.");
}
