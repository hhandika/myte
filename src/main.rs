use clap::crate_version;
use std::time::Instant;

mod cli;
mod iqtree;

fn main() {
    let version = crate_version!();
    let time = Instant::now();
    cli::parse_cli(version);
    let duration = time.elapsed();

    println!("\nExecution time: {:?}", duration);
}
