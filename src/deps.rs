use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

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

fn make_astral_executable(fname: &str) {
    Command::new("chmod")
        .arg("+x")
        .arg(fname)
        .spawn()
        .expect("CANNOT EXECUTE chmod");
}
