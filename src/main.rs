use clap::crate_version;
use std::panic;
use std::time::Instant;

mod cli;
mod deps;
mod tree;
mod utils;

fn main() {
    panic::set_hook(Box::new(move |panic_info| {
        log::error!("{}", panic_info);
    }));
    let version = crate_version!();
    let time = Instant::now();
    cli::parse_cli(version);
    let duration = time.elapsed();
    if duration.as_secs() < 60 {
        log::info!("Execution time: {:?}", duration);
    } else {
        utils::print_formatted_duration(duration.as_secs());
    }
}
