use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str;

use regex::Regex;

pub fn fix_astral_dependency(path: &str) {
    let fname = "astral";
    let jar_path = Path::new(path);
    let jar_full_path = jar_path
        .canonicalize()
        .expect("CANNOT CREATE FULL PATH TO ASTRAL JAR FILE");
    let mut file = File::create(fname).expect("CANNOT CREATE AN ASTRAL EXECUTABLE");
    writeln!(file, "#!/bin/bash").unwrap();
    writeln!(
        file,
        "java -D\"java.library.path={}/lib\" -jar {} \"$@\"",
        jar_full_path.parent().unwrap().to_string_lossy(),
        jar_full_path.to_string_lossy()
    )
    .expect("CANNOT WRITE ASTRAL EXECUTABLE");

    make_astral_executable(fname);
}

pub fn check_dependencies() {
    log::info!("Dependencies:");
    check_iqtree();
    check_astral();
}

fn make_astral_executable(fname: &str) {
    Command::new("chmod")
        .arg("+x")
        .arg(fname)
        .spawn()
        .expect("CANNOT EXECUTE chmod");
}

fn check_iqtree() {
    let out = Command::new("iqtree2").arg("--version").output();

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
        Err(_) => log::info!("{:18}: {}", "\x1b[0;41m[NOT FOUND]\x1b[0m", "IQ-TREE"),
    }
}

fn check_astral() {
    let out = Command::new("astral").arg("--version").output();

    match out {
        Ok(out) => log::info!("[OK]\t{}", str::from_utf8(&out.stdout).unwrap().trim()),
        Err(_) => log::info!("{:18}: {}", "[NOT FOUND]", "ASTRAL"),
    }
}
