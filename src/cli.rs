use std::io::{self, Result, Write};

use clap::{crate_name, App, AppSettings, Arg, ArgMatches};

use crate::deps;
use crate::tree::{self, InputFmt};
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
                )
                .arg(
                    Arg::with_name("input-fmt")
                        .short("f")
                        .long("input-fmt")
                        .help("Sets input format")
                        .required(true)
                        .takes_value(true)
                        .default_value("nexus")
                        .possible_values(&["fasta", "phylip", "nexus"])
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
                    Arg::with_name("opts-s")
                        .long("opts-s")
                        .help("Inputs params for IQ-TREE gene tree analyses")
                        .require_equals(true)
                        .takes_value(true)
                        .value_name("PARAMS"),
                )
                .arg(
                    Arg::with_name("opts-g")
                        .long("opts-g")
                        .help("Inputs params for IQ-TREE gene tree analyses")
                        .require_equals(true)
                        .takes_value(true)
                        .value_name("PARAMS"),
                )
                .arg(
                    Arg::with_name("input-fmt")
                        .short("f")
                        .long("input-fmt")
                        .help("Sets input format")
                        .required(true)
                        .takes_value(true)
                        .default_value("nexus")
                        .possible_values(&["fasta", "phylip", "nexus"])
                        .value_name("PARAMS"),
                ),
        )
        .subcommand(
            App::new("deps")
                .about("Solves dependency issues")
                .subcommand(
                    App::new("astral")
                        .about("Fixes astral dependency issues")
                        .arg(
                            Arg::with_name("jar")
                                .short("-j")
                                .long("jar")
                                .help("Inputs path to the ASTRAL jar file.")
                                .takes_value(true)
                                .required(true)
                                .value_name("ASTRAL JAR"),
                        ),
                ),
        )
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    match args.subcommand() {
        ("auto", Some(auto_matches)) => parse_auto_cli(auto_matches, &version),
        ("gene", Some(gene_matches)) => parse_gene_cli(gene_matches, &version),
        ("check", Some(_)) => display_app_info(&version).unwrap(),
        ("deps", Some(deps_matches)) => parse_deps_cli(deps_matches),
        _ => unreachable!(),
    }
}

fn parse_deps_cli(args: &ArgMatches) {
    match args.subcommand() {
        ("astral", Some(astral_matches)) => parse_astral_cli(astral_matches),
        _ => unreachable!(),
    }
}

fn parse_auto_cli(matches: &ArgMatches, version: &str) {
    let path = get_path(matches);
    let msg_len = 80;
    let params_s = parse_params_species(matches);
    let params_g = parse_params_gene(matches);
    let input_fmt = parse_input_fmt(matches);
    display_app_info(version).unwrap();
    print_species_tree_header(msg_len);
    tree::build_species_tree(path, &params_s);
    print_gene_tree_header(msg_len);
    tree::build_gene_trees(path, &params_g, &input_fmt);
    print_cf_tree_header(msg_len);
    tree::estimate_concordance_factor(path);
    print_msc_tree_header(msg_len);
    tree::estimate_msc_tree(path);
    println!("\nCOMPLETED!\n");
}

fn parse_gene_cli(matches: &ArgMatches, version: &str) {
    let path = get_path(matches);
    let msg_len = 80;
    let params = parse_params_gene(matches);
    let input_fmt = parse_input_fmt(matches);
    display_app_info(version).unwrap();
    print_gene_tree_header(msg_len);
    tree::build_gene_trees(path, &params, &input_fmt);
    print_cf_tree_header(msg_len);
    tree::estimate_concordance_factor(path);
    print_msc_tree_header(msg_len);
    tree::estimate_msc_tree(path);
    println!("\nCOMPLETED!\n");
}

fn parse_astral_cli(matches: &ArgMatches) {
    let path = matches
        .value_of("jar")
        .expect("CANNOT PARSE ASTRAL JAR PATH");
    deps::fix_astral_dependency(path);
}

fn parse_params_gene(matches: &ArgMatches) -> Option<String> {
    let mut opts = None;
    if matches.is_present("opts-g") {
        let input = matches
            .value_of("opts-g")
            .expect("CANNOT PARSE PARAMS INPUT");
        opts = Some(String::from(input.trim()));
    }

    opts
}

fn parse_params_species(matches: &ArgMatches) -> Option<String> {
    let mut opts = None;
    if matches.is_present("opts-s") {
        let input = matches
            .value_of("opts-s")
            .expect("CANNOT PARSE PARAMS INPUT");
        opts = Some(String::from(input.trim()));
    }

    opts
}

fn parse_input_fmt(matches: &ArgMatches) -> InputFmt {
    let input_fmt = matches
        .value_of("input-fmt")
        .expect("CANNOT READ FORMAT INPUT");
    match input_fmt {
        "fasta" => InputFmt::Fasta,
        "nexus" => InputFmt::Nexus,
        "phylip" => InputFmt::Phylip,
        _ => unreachable!("Specify input format"),
    }
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
