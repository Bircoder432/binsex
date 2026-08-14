use binsex::{Binary, execute};

static BINARY_BYTES: &'static [u8] = include_bytes!("../../bin.exc");

fn main() {
    let binary: Binary = postcard::from_bytes(BINARY_BYTES).unwrap();
    execute(binary, false);
}
