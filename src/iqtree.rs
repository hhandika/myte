use std::fs;
use std::io::{self, Result, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;

use glob::glob;
use spinners::{Spinner, Spinners};

pub fn build_gene_trees_all(path: &str, version: i8) {
    let mut genes = Genes::new(path);
    let paths = genes.get_paths();
    println!("All paths: ");
    paths.iter().for_each(|path| {
        let prefix = path.file_name().unwrap().to_string_lossy();
        let mut iqtree = Iqtree::new(version, path, &prefix);
        iqtree.run_iqtree();
    });
}

impl Files for Genes {}
impl Files for Iqtree<'_> {}

struct Genes {
    path: String,
}

impl Genes {
    fn new(path: &str) -> Self {
        Self {
            path: String::from(path),
        }
    }

    // Search for alignments and get the path for them
    fn get_paths(&mut self) -> Vec<PathBuf> {
        let pattern = format!("{}/*.nexus", self.path);
        self.get_files(&pattern)
    }
}

trait Files {
    fn get_files(&mut self, pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .expect("COULD NOT FIND FILES")
            .filter_map(|ok| ok.ok())
            .collect()
    }
}

struct Iqtree<'a> {
    version: i8,
    path: &'a Path,
    prefix: &'a str,
    command: String,
}

impl<'a> Iqtree<'a> {
    fn new(version: i8, path: &'a Path, prefix: &'a str) -> Self {
        Self {
            version,
            path,
            prefix,
            command: String::from("iqtree"),
        }
    }

    fn run_iqtree(&mut self) {
        self.get_iqtree_version();
        let spin = self.set_spinner();
        let out = self.call_iqtree();
        self.check_iqtree_success(&out);
        let files = self.get_result_files();
        self.move_result_files(&files).unwrap();
        spin.stop();
        self.print_done();
    }

    fn set_spinner(&mut self) -> Spinner {
        let msg = "IQ-TREE is processing...\t".to_string();
        Spinner::new(Spinners::Moon, msg)
    }

    fn print_done(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "\x1b[0;32mDONE!\x1b[0m").unwrap();
    }

    // Build gen tree using IQ-TREE
    fn call_iqtree(&mut self) -> Output {
        let mut out = Command::new(self.command.clone());
        out.arg("-s")
            .arg(self.path)
            .arg("-T")
            .arg("AUTO")
            .arg("--prefix")
            .arg(self.prefix.clone())
            .output()
            .expect("FAILED TO RUN IQ-TREE")
    }

    fn get_iqtree_version(&mut self) {
        if self.version == 2 {
            self.command.push('2');
        };
    }

    fn check_iqtree_success(&self, out: &Output) {
        if !out.status.success() {
            println!();
            io::stdout().write_all(&out.stdout).unwrap();
            io::stdout().write_all(&out.stderr).unwrap();
        }
    }

    fn get_result_files(&mut self) -> Vec<PathBuf> {
        let pattern = format!("{}.*", self.prefix);
        self.get_files(&pattern)
    }

    fn move_result_files(&mut self, files: &[PathBuf]) -> Result<()> {
        let dir = Path::new(self.prefix);
        fs::create_dir_all(dir)?;
        files.iter().for_each(|file| {
            let outdir = dir.join(file);
            fs::rename(file, outdir).expect("CANNOT MOVE IQ-TREE RESULT FILES");
        });

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_gene_paths_test() {
        let path = "test_files";
        let mut genes = Genes::new(path);
        let gene_paths = genes.get_paths();

        assert_eq!(2, gene_paths.len());
    }

    #[test]
    fn get_iqtree_version_test() {
        let version = 2;
        let path = Path::new(".");
        let mut iqtree = Iqtree::new(version, path, "loci");
        iqtree.get_iqtree_version();
        assert_eq!("iqtree2", iqtree.command);
    }
}
