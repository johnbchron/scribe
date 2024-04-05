use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
  path: PathBuf,
}

fn main() {
  let args = Args::parse();

  println!("Hello, world!");
  dbg!(args);
}
