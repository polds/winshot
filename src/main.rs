extern crate winshot;
extern crate winapi;

use winapi::user32;

fn main() {
    println!("Hello, world!");

    user32::MessageBoxExW(None, Some("lpText"), Some("lpCaption"), Some("uType"), Some("wLanguageId"))
}
