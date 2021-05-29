use std::io::{self, Result, Write};

use clap::{crate_name, App, AppSettings, Arg, ArgMatches};

use crate::tree;
use crate::utils;

fn get_args(version: &str) -> ArgMatches {
    App::new("myte")
        .version(version)
        .about("A tool for phylogenomic tree building")
        .author("Heru Handika <hhandi1@lsu.edu>")
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
                )
                .arg(
                    Arg::with_name("opts-g")
                        .long("opts-g")
                        .help("Inputs params for IQ-TREE gene tree analyses")
                        .takes_value(true)
                        .value_name("PARAMS"),
                ),
        )
        .subcommand(
            App::new("auto")
                .about(
                    "Auto estimate species tree, gene trees, and gene and site concordance factor",
                )
                .arg(
                    Arg::with_name("dir")
                        .short("d")
                        .long("dir")
                        .help("Inputs folder path to locus alignment")
                        .takes_value(true)
                        .value_name("DIR"),
                )
                .arg(
                    Arg::with_name("opts-g")
                        .long("opts-g")
                        .help("Inputs params for IQ-TREE gene tree analyses")
                        .takes_value(true)
                        .value_name("PARAMS"),
                ),
        )
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    match args.subcommand() {
        ("auto", Some(auto_matches)) => parse_auto_cli(auto_matches, &version),
        ("check", Some(_)) => display_app_info(&version).unwrap(),
        ("gene", Some(gene_matches)) => parse_gene_cli(gene_matches, &version),
        _ => unreachable!(),
    }
}

fn parse_auto_cli(matches: &ArgMatches, version: &str) {
    let path = get_path(matches);
    let iqtree_version = 2;
    let msg_len = 80;
    let params = parse_params_gene(matches);
    display_app_info(version).unwrap();
    print_species_tree_header(msg_len);
    tree::build_species_tree(path);
    print_gene_tree_header(msg_len);
    tree::build_gene_trees(path, iqtree_version, params);
    print_cf_tree_header(msg_len);
    tree::estimate_concordance_factor(path);
    print_msc_tree_header(msg_len);
    tree::estimate_msc_tree(path);
    println!("\nCOMPLETED!\n");
}

fn parse_gene_cli(matches: &ArgMatches, version: &str) {
    let path = get_path(matches);
    let iqtree_version = 2;
    let params = parse_params_gene(matches);
    display_app_info(version).unwrap();
    tree::build_gene_trees(path, iqtree_version, params);
    println!("\nCOMPLETED!\n");
}

fn parse_params_gene(matches: &ArgMatches) -> Option<String> {
    let mut opts = None;
    if matches.is_present("opts-g") {
        let input = matches
            .value_of("opts-g")
            .expect("CANNOT PARSE PARAMS INPUT");
        let params = input.replace("params=", "");
        opts = Some(String::from(params.trim()));
    }

    opts
}

fn get_path<'a>(matches: &'a ArgMatches) -> &'a str {
    matches.value_of("dir").expect("CANNOT GET DIRECTORY PATH")
}

fn print_species_tree_header(len: usize) {
    let text = "IQ-TREE: SPECIES TREE ANALYSES";
    utils::print_divider(text, len);
}

fn print_gene_tree_header(len: usize) {
    let text = "IQ-TREE: GENE TREE ANALYSES";
    utils::print_divider(text, len);
}

fn print_cf_tree_header(len: usize) {
    let text = "IQ-TREE: CONCORDANCE FACTOR ANALYSES";
    utils::print_divider(text, len);
}

fn print_msc_tree_header(len: usize) {
    let text = "ASTRAL-MP: MULTI-SPECIES COALESCENCE MODEL ANALYSES";
    utils::print_divider(text, len);
}

fn display_app_info(version: &str) -> Result<()> {
    let io = io::stdout();
    let mut handle = io::BufWriter::new(io);
    writeln!(handle, "{} v{}", crate_name!(), version)?;
    writeln!(handle, "Genomics tools for phylogenetic tree estimation")?;
    writeln!(handle, "Developed by Heru Handika")?;
    writeln!(handle)?;
    utils::get_system_info(&mut handle)?;

    Ok(())
}
