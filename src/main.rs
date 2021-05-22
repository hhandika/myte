use clap::crate_version;
use std::time::Instant;

mod cli;
mod iqtree;
mod utils;

fn main() {
    let version = crate_version!();
    let time = Instant::now();
    cli::parse_cli(version);
    let duration = time.elapsed();
    if duration.as_secs() < 60 {
        println!("Execution time: {:?}", duration);
    } else {
        utils::print_formatted_duration(duration.as_secs());
    }
}
