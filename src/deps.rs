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

pub fn check_dependencies() -> Result<()> {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);
    utils::get_system_info().unwrap();
    writeln!(handle, "Dependencies:")?;
    check_fastp(&mut handle)?;
    check_spades(&mut handle)?;
    writeln!(handle)?;
    Ok(())
}

fn check_fastp<W: Write>(handle: &mut W) -> Result<()> {
    let out = Command::new("fastp").arg("--version").output();

    match out {
        Ok(out) => writeln!(
            handle,
            "[OK]\t{}",
            str::from_utf8(&out.stderr).unwrap().trim()
        )?,
        Err(_) => writeln!(handle, "\x1b[0;41m[NOT FOUND]\x1b[0m\tfastp")?,
    }

    Ok(())
}

// fn check_spades<W: Write>(handle: &mut W) -> Result<()> {
//     let out = Command::new("spades.py").arg("--version").output();
//     match out {
//         Ok(out) => writeln!(
//             handle,
//             "[OK]\t{}",
//             str::from_utf8(&out.stdout).unwrap().trim()
//         )?,
//         Err(_) => writeln!(handle, "\x1b[0;41m[NOT FOUND]\x1b[0m\tSPAdes")?,
//     }

//     Ok(())
// }
