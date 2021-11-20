use std::io::Result;

use crate::deps;
use crate::tree::{self, InputFmt};
use crate::utils;
use clap::{crate_description, crate_name, App, AppSettings, Arg, ArgMatches};

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

fn get_args(version: &str) -> ArgMatches {
    App::new(crate_name!())
        .version(version)
        .about(crate_description!())
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
                        .value_name("STRING")
                        .default_value("-T 1"),
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
                        .value_name("ALIGNMENT-FORMAT"),
                ),
        )
        .subcommand(
            App::new("auto")
                .about(
                    "Estimate species tree, gene trees, gene and site concordance factors, and MSC tree",
                )
                .arg(
                    Arg::with_name("dir")
                        .short("d")
                        .long("dir")
                        .help("Inputs folder path to locus alignment")
                        .takes_value(true)
                        .value_name("STRING"),
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
                        .value_name("STRING")
                        .default_value("-T 1"),
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
                        .value_name("ALIGNMENT-FORMAT"),
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
                                .value_name("ASTRAL-JAR-PATH"),
                        ),
                ),
        )
        .get_matches()
}

pub fn parse_cli(version: &str) {
    let args = get_args(version);
    setup_logger().expect("Failed setting up a log file.");
    match args.subcommand() {
        ("auto", Some(auto_matches)) => parse_auto_cli(auto_matches, &version),
        ("gene", Some(gene_matches)) => parse_gene_cli(gene_matches, &version),
        ("check", Some(_)) => display_app_info(&version),
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
    display_app_info(version);
    print_species_tree_header(msg_len);
    log_input(&path, &params_s);
    tree::build_species_tree(path, &params_s);
    print_gene_tree_header(msg_len);
    log_input(&path, &params_g);
    tree::build_gene_trees(path, &params_g, &input_fmt);
    print_cf_tree_header(msg_len);
    tree::estimate_concordance_factor(path);
    print_msc_tree_header(msg_len);
    tree::estimate_msc_tree(path);
    print_complete();
}

fn parse_gene_cli(matches: &ArgMatches, version: &str) {
    let path = get_path(matches);
    let msg_len = 80;
    let params = parse_params_gene(matches);
    let input_fmt = parse_input_fmt(matches);
    display_app_info(version);
    print_gene_tree_header(msg_len);
    tree::build_gene_trees(path, &params, &input_fmt);
    print_complete();
}

fn parse_astral_cli(matches: &ArgMatches) {
    let path = matches
        .value_of("jar")
        .expect("CANNOT PARSE ASTRAL JAR PATH");
    deps::fix_astral_dependency(path);
}

fn parse_params_gene(matches: &ArgMatches) -> Option<String> {
    let input = matches
        .value_of("opts-g")
        .expect("CANNOT PARSE PARAMS INPUT");
    Some(String::from(input.trim()))
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

fn display_app_info(version: &str) {
    log::info!("{} v{}", crate_name!(), version);
    log::info!("{}", crate_description!());
    log::info!("Developed by Heru Handika\n");
    utils::get_system_info();
    deps::check_dependencies();
}

fn print_complete() {
    log::info!("COMPLETED!");
    log::info!("Please, check each program log for commands and other details!\n")
}

fn log_input(path: &str, params: &Option<String>) {
    log::info!("{:18}: {}", "Input", path);
    match params {
        Some(param) => log::info!("{:18}: {}", "Opt params", param),
        None => log::info!("{:18}: None", "Params"),
    }
}

fn setup_logger() -> Result<()> {
    let log_dir = std::env::current_dir()?;
    let target = log_dir.join("myte.log");
    let tofile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S %Z)} - {l} - {m}\n",
        )))
        .build(target)?;

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}\n")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(tofile)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    Ok(())
}
