use binsex::{Binary, execute};
use clap::Parser;
#[derive(Parser)]
struct Args {
    file: String,
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let file = std::fs::read(args.file).unwrap();
    let binary: Binary = postcard::from_bytes(&file).unwrap();
    execute(binary, args.debug);
}
