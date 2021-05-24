use clap::{crate_name, crate_version};
use std::io::{self, Result, Write};
use std::time::Instant;

mod cli;
mod tree;
mod utils;

fn main() {
    let version = crate_version!();
    let time = Instant::now();
    display_app_info(version).unwrap();
    cli::parse_cli(version);
    let duration = time.elapsed();
    if duration.as_secs() < 60 {
        println!("Execution time: {:?}", duration);
    } else {
        utils::print_formatted_duration(duration.as_secs());
    }
}

fn display_app_info(version: &str) -> Result<()> {
    let io = io::stdout();
    let mut handle = io::BufWriter::new(io);
    writeln!(handle, "{} v{}", crate_name!(), version)?;
    writeln!(handle, "Genomics tools for phylogenetic tree estimation")?;
    writeln!(handle, "Developer by Heru Handika")?;
    writeln!(handle)?;
    utils::get_system_info(&mut handle)?;
    writeln!(handle)?;

    Ok(())
}
