use clap::{App, AppSettings, Arg, ArgMatches};

use crate::iqtree;

fn get_args(version: &str) -> ArgMatches {
    App::new("myte")
        .version(version)
        .about("A tool for automatic genomic tree building")
        .author("Heru Handika <hhandi1@lsu.edu")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("check").about("Check dependencies"))
        .subcommand(
            App::new("gene")
                .about("Batch gene tree estimation using IQ-Tree")
                .arg(
                    Arg::with_name("dir")
                        .short("d")
                        .long("dir")
                        .help("Inputs folder path to locus alignment")
                        .takes_value(true)
                        .value_name("DIR"),
                ),
        )
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    match args.subcommand() {
        ("check", Some(_)) => println!("It's check dependencies"),
        ("gene", Some(gene_matches)) => parse_gene_cli(gene_matches),
        _ => unreachable!(),
    }
}

fn parse_gene_cli(matches: &ArgMatches) {
    let path = matches.value_of("dir").expect("CANNOT GET DIRECTORY PATH");
    let version = 2;
    iqtree::build_gene_trees_all(path, version);
}
