use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str;

use ansi_term::Colour::Yellow;
use regex::Regex;

use crate::tree::{ASTRAL_EXE, IQTREE_EXE};

pub fn fix_astral_dependency(path: &str) {
    let fname = "astral.sh";
    let jar_path = Path::new(path);
    let jar_full_path = jar_path
        .canonicalize()
        .expect("Failed getting path to the Astral jar file");
    let mut file = File::create(&fname).expect("Failed creating a file to solve astral dependency");
    writeln!(file, "#!/bin/bash").unwrap();
    writeln!(
        file,
        "java -D\"java.library.path={}/lib\" -jar {} \"$@\"",
        jar_full_path.parent().unwrap().to_string_lossy(),
        jar_full_path.to_string_lossy()
    )
    .expect("Failed in writing an executable file for Astral");

    make_astral_executable(&fname);
}

pub fn check_dependencies() {
    log::info!("{}", Yellow.paint("Dependencies"));
    check_iqtree();
    check_astral();
    println!();
}

fn make_astral_executable(fname: &str) {
    Command::new("chmod")
        .arg("+x")
        .arg(fname)
        .spawn()
        .expect("CANNOT EXECUTE chmod");
}

fn check_iqtree() {
    let out = Command::new(IQTREE_EXE).arg("--version").output();

    match out {
        Ok(out) => {
            let output = str::from_utf8(&out.stdout).unwrap().trim();
            let re = Regex::new(r"(\d+\.)?(\d+\.)?(\*|\d+)")
                .expect("Failed to setup regular expression for version numbers.");
            let version = re
                .find(output)
                .expect("Cannot capture version in the stdout iqtree")
                .as_str();
            log::info!("{:18}: IQ-TREE v{}", "[OK]", version)
        }
        Err(_) => log::info!("{:18}: IQ-TREE", "[NOT FOUND]"),
    }
}

fn check_astral() {
    let out = Command::new(ASTRAL_EXE).arg("--version").output();

    match out {
        Ok(_) => log::info!("{:18}: ASTRAL", "[OK]"),
        Err(_) => log::info!("{:18}: ASTRAL", "[NOT FOUND]"),
    }
}
