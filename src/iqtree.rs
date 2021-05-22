use std::fs;
use std::fs::File;
use std::io::{self, LineWriter, Read, Result, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;

use glob::glob;
use rayon::prelude::*;
use spinners::{Spinner, Spinners};

pub fn build_gene_trees_all(path: &str, version: i8) {
    let treedir = Path::new("treefiles");
    let mut genes = Genes::new(path, treedir);
    let paths = genes.get_paths();
    genes.create_tree_files_dir(&treedir);
    print_genes_paths(&paths).unwrap();
    let spin = genes.set_spinner();
    paths.par_iter().for_each(|path| {
        let prefix = path.file_stem().unwrap().to_string_lossy();
        let mut iqtree = Iqtree::new(version, path, &prefix, &treedir);
        iqtree.run_iqtree();
    });
    spin.stop();
    genes.create_gene_treeset();
    genes.print_done();
}

fn print_genes_paths(paths: &[PathBuf]) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "\x1b[0;45mAll alignment: \x1b[0m")?;

    paths
        .iter()
        .for_each(|path| writeln!(handle, "{}", path.to_string_lossy()).unwrap());
    writeln!(handle)?;

    Ok(())
}

impl Files for Genes<'_> {}
impl Files for Iqtree<'_> {}

struct Genes<'a> {
    path: String,
    treedir: &'a Path,
}

impl<'a> Genes<'a> {
    fn new(path: &str, treedir: &'a Path) -> Self {
        Self {
            path: String::from(path),
            treedir,
        }
    }

    // Search for alignments and get the path for them
    fn get_paths(&mut self) -> Vec<PathBuf> {
        let pattern = format!("{}/*.nexus", self.path);
        self.get_files(&pattern)
    }

    fn create_tree_files_dir(&mut self, treedir: &Path) {
        fs::create_dir_all(treedir).expect("CANNOT CREATE DIRECTORY FOR TREE FILES");
    }

    fn set_spinner(&mut self) -> Spinner {
        let msg = "IQ-TREE is processing {}...\t".to_string();
        Spinner::new(Spinners::Moon, msg)
    }

    fn create_gene_treeset(&mut self) {
        let pattern = format!("{}/*.treefile", self.treedir.to_string_lossy());
        let trees = self.get_files(&pattern);
        let fname = "all_genes.treefiles";
        let file = File::create(&fname).expect("CANNOT CREATE AN ALL GENE TREE FILE");
        let mut treefiles = LineWriter::new(file);
        trees.iter().for_each(|t| {
            let mut content = String::new();
            let mut tree = File::open(t).expect("CANNOT ACCESS TREE FILE");
            tree.read_to_string(&mut content)
                .expect("CANNOT READ TREE FILES");
            writeln!(treefiles, "{}", content).unwrap();
        });
    }
}

trait Files {
    fn get_files(&mut self, pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .expect("COULD NOT FIND FILES")
            .filter_map(|ok| ok.ok())
            .collect()
    }

    fn print_done(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "\x1b[0;32mDONE!\x1b[0m").unwrap();
    }
}

struct Iqtree<'a> {
    version: i8,
    path: &'a Path,
    prefix: &'a str,
    command: String,
    treedir: &'a Path,
}

impl<'a> Iqtree<'a> {
    fn new(version: i8, path: &'a Path, prefix: &'a str, treedir: &'a Path) -> Self {
        Self {
            version,
            path,
            prefix,
            treedir,
            command: String::from("iqtree"),
        }
    }

    fn run_iqtree(&mut self) {
        self.get_iqtree_version();
        let out = self.call_iqtree();
        self.check_iqtree_success(&out);
        let files = self.get_result_files();
        self.move_result_files(&files).unwrap();
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
            let ext = file.extension().unwrap().to_string_lossy();
            if ext == "treefile" {
                let outdir = self.treedir.join(file);
                fs::rename(file, outdir).expect("CANNOT MOVE IQ-TREE'S TREE FILE");
            } else {
                let outdir = dir.join(file);
                fs::rename(file, outdir).expect("CANNOT MOVE IQ-TREE'S RESULT FILES");
            }
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
        let treefiles = Path::new(".");
        let mut genes = Genes::new(path, treefiles);
        let gene_paths = genes.get_paths();

        assert_eq!(2, gene_paths.len());
    }

    #[test]
    fn get_iqtree_version_test() {
        let version = 2;
        let path = Path::new(".");
        let trees = Path::new(".");
        let mut iqtree = Iqtree::new(version, path, "loci", trees);
        iqtree.get_iqtree_version();
        assert_eq!("iqtree2", iqtree.command);
    }
}
