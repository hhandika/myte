use std::fs;
use std::fs::File;
use std::io::{self, LineWriter, Read, Result, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;

use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub fn build_species_tree(path: &str) {
    let dir_path = Path::new(path);
    let mut iqtree = SpeciesTree::new(&dir_path);
    let msg = format!(
        "\x1b[0mIQ-TREE is processing species tree for alignments in {}...",
        path
    );
    let spin = iqtree.set_spinner();
    spin.set_message(msg);
    iqtree.estimate_species_tree();
    spin.abandon_with_message("Finished estimating species tree!");
}

pub fn build_gene_trees(path: &str, version: i8) {
    let mut genes = GeneTrees::new(path, version);
    let paths = genes.get_alignment_paths();
    assert!(
        paths.len() > 1,
        "LESS THAN ONE ALIGNMENT FOUND FOR GENE TREE ANALYSES"
    );

    genes.create_tree_files_dir();
    genes.print_genes_paths(&paths).unwrap();

    let num_alignments = paths.len();
    let msg = format!(
        "\x1b[0mIQ-TREE is processing gene trees for {} alignments...",
        num_alignments
    );

    let spin = genes.set_spinner();
    spin.set_message(msg);
    genes.par_process_gene_trees(&paths);

    let finish_msg = format!(
        "\x1b[0mFinished estimating gene trees for {} alignments",
        num_alignments
    );
    spin.abandon_with_message(finish_msg);
    genes.combine_gene_trees();
}

pub fn estimate_concordance_factor(path: &str) {
    let dir_path = Path::new(path);
    let mut iqtree = ConcordFactor::new(&dir_path);
    let msg = "\x1b[0mIQ-TREE is processing concordance factor...";
    let spin = iqtree.set_spinner();
    spin.set_message(msg);
    iqtree.estimate_concordance();
    spin.finish_with_message("\x1b[0mFinished estimating concordance factor!");
}

trait Commons {
    fn get_files(&self, pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .expect("COULD NOT FIND FILES")
            .filter_map(|ok| ok.ok())
            .collect()
    }

    fn get_iqtree_files(&self, prefix: &str) -> Vec<PathBuf> {
        let pattern = format!("{}.*", prefix);
        self.get_files(&pattern)
    }

    fn set_spinner(&mut self) -> ProgressBar {
        let spin = ProgressBar::new_spinner();
        spin.enable_steady_tick(150);
        spin.set_style(ProgressStyle::default_spinner().template("{spinner:.simpleDots} {msg}"));
        spin
    }

    fn check_process_success(&self, out: &Output, path: &Path) {
        if !out.status.success() {
            println!();
            let msg = format!(
                "\x1b[0;41mIQ-TREE FAILED TO PROCESS {}\x1b[0m\n",
                path.to_string_lossy()
            );
            io::stdout().write_all(msg.as_bytes()).unwrap();
            io::stdout().write_all(&out.stdout).unwrap();
            io::stdout().write_all(&out.stderr).unwrap();
            eprintln!(
                "\x1b[0;41mERROR:\x1b[0m IQ-TREE failed to process {}. See the log output above.",
                path.to_string_lossy()
            );
        }
    }
}

impl Commons for GeneTrees<'_> {}
impl Commons for SpeciesTree<'_> {}
impl Commons for ConcordFactor<'_> {}

struct GeneTrees<'a> {
    path: &'a str,
    version: i8,
    command: String,
    treedir: PathBuf,
    parent_dir: PathBuf,
}

impl<'a> GeneTrees<'a> {
    fn new(path: &'a str, version: i8) -> Self {
        Self {
            path,
            version,
            command: String::from("iqtree"),
            treedir: PathBuf::from("gene-treefiles"),
            parent_dir: PathBuf::from("iqtree-genes"),
        }
    }

    fn get_alignment_paths(&mut self) -> Vec<PathBuf> {
        let pattern = format!("{}/*.nexus", self.path);
        self.get_files(&pattern)
    }

    fn print_genes_paths(&self, paths: &[PathBuf]) -> Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "Alignment path:")?;

        paths
            .iter()
            .for_each(|path| writeln!(handle, "{}", path.to_string_lossy()).unwrap());
        writeln!(handle)?;
        Ok(())
    }

    fn create_tree_files_dir(&mut self) {
        fs::create_dir_all(&self.treedir).expect("CANNOT CREATE DIRECTORY FOR TREE FILES");
    }

    fn par_process_gene_trees(&mut self, paths: &[PathBuf]) {
        self.get_iqtree_version();
        paths
            .par_iter()
            .for_each(|path| self.estimate_gene_tree(path));
    }

    fn estimate_gene_tree(&self, path: &Path) {
        let prefix = path.file_stem().unwrap().to_string_lossy();
        let out = self.call_iqtree(&prefix);
        self.check_process_success(&out, path);
        let files = self.get_iqtree_files(&prefix);
        self.organize_gene_files(&files, &prefix).unwrap();
    }

    fn call_iqtree(&self, prefix: &str) -> Output {
        let mut out = Command::new(&self.command);
        out.arg("-s")
            .arg(&self.path)
            .arg("-T")
            .arg("AUTO")
            .arg("--prefix")
            .arg(prefix)
            .output()
            .expect("FAILED TO RUN IQ-TREE")
    }

    fn get_iqtree_version(&mut self) {
        if self.version == 2 {
            self.command.push('2');
        };
    }

    fn organize_gene_files(&self, files: &[PathBuf], prefix: &str) -> Result<()> {
        let path = self.parent_dir.join(prefix);
        let dir = Path::new(&path);
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

    fn combine_gene_trees(&mut self) {
        let pattern = format!("{}/*.treefile", self.treedir.to_string_lossy());
        let trees = self.get_files(&pattern);
        let fname = "genes.treefiles";
        let file = File::create(&fname).expect("CANNOT CREATE AN ALL GENE TREE FILE");
        let mut treefile = LineWriter::new(file);
        let num_trees = trees.len();
        let msg = format!("Combining {} gene trees into a single file...", num_trees);
        let spin = self.set_spinner();
        spin.set_message(msg);
        trees
            .iter()
            .for_each(|tree| self.write_trees(&mut treefile, tree));
        let finish_msg = format!("Finished combining {} gene trees!", num_trees);
        spin.finish_with_message(finish_msg);
    }

    fn write_trees<W: Write>(&self, treefile: &mut W, tree_path: &Path) {
        let mut content = String::new();
        let mut tree = File::open(tree_path).expect("CANNOT ACCESS TREE FILE");
        tree.read_to_string(&mut content)
            .expect("CANNOT READ TREE FILES");
        writeln!(treefile, "{}", content.trim()).unwrap();
    }
}

struct SpeciesTree<'a> {
    path: &'a Path,
    prefix: String,
    command: String,
    outdir: PathBuf,
}

impl<'a> SpeciesTree<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            prefix: String::from("concat"),
            outdir: PathBuf::from("iqtree-species-tree"),
            command: String::from("iqtree2"),
        }
    }

    fn estimate_species_tree(&mut self) {
        let out = self.call_iqtree();
        self.check_process_success(&out, self.path);
        let files = self.get_iqtree_files(&self.prefix);
        self.organize_species_files(&files)
            .expect("FAILED TO MOVE SPECIES TREE RESULT FILES");
    }

    fn call_iqtree(&mut self) -> Output {
        let mut out = Command::new(&self.command);
        out.arg("-p")
            .arg(&self.path)
            .arg("-B")
            .arg("1000")
            .arg("-T")
            .arg("AUTO")
            .arg("--prefix")
            .arg(&self.prefix)
            .output()
            .expect("FAILED TO RUN IQ-TREE")
    }

    fn organize_species_files(&self, files: &[PathBuf]) -> Result<()> {
        fs::create_dir_all(&self.outdir)?;
        files.iter().for_each(|file| {
            let outdir = self.outdir.join(file);
            let ext = file.extension().unwrap().to_string_lossy();
            if ext != "treefile" {
                fs::rename(file, outdir).expect("CANNOT MOVE IQ-TREE'S RESULT FILES");
            }
        });

        Ok(())
    }
}

struct ConcordFactor<'a> {
    path: &'a Path,
    outdir: PathBuf,
    prefix: String,
    command: String,
}

impl<'a> ConcordFactor<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            outdir: PathBuf::from("iqtree-CF"),
            prefix: String::from("concord"),
            command: String::from("iqtree2"),
        }
    }

    fn estimate_concordance(&mut self) {
        let cores = num_cpus::get_physical();
        let out = self.call_iqtree(cores);
        self.check_process_success(&out, self.path);
        let files = self.get_iqtree_files(&self.prefix);
        self.organize_cf_files(&files)
            .expect("CANNOT MOVE CONCORDANCE FACTOR RESULT FILES");
    }

    fn call_iqtree(&mut self, num_core: usize) -> Output {
        let mut out = Command::new(&self.command);
        out.arg("-t")
            .arg("concat.treefile")
            .arg("--gcf")
            .arg("genes.treefiles")
            .arg("-p")
            .arg(&self.path)
            .arg("--scf")
            .arg("100")
            .arg("-T")
            .arg(num_core.to_string())
            .arg("--prefix")
            .arg(&self.prefix)
            .output()
            .expect("FAILED TO RUN IQ-TREE")
    }

    fn organize_cf_files(&self, files: &[PathBuf]) -> Result<()> {
        fs::create_dir_all(&self.outdir)?;
        files.iter().for_each(|file| {
            let outdir = self.outdir.join(file);
            let ext = file.extension().unwrap().to_string_lossy();
            if ext != "tre" {
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
        let mut genes = GeneTrees::new(path, 2);
        let gene_paths = genes.get_alignment_paths();

        assert_eq!(2, gene_paths.len());
    }

    #[test]
    fn get_iqtree_version_test() {
        let version = 2;
        let path = ".";
        let mut iqtree = GeneTrees::new(&path, version);
        iqtree.get_iqtree_version();
        assert_eq!("iqtree2", iqtree.command);
    }

    #[test]
    #[should_panic]
    fn gene_tree_panic_test() {
        let path = ".";
        build_gene_trees(path, 2);
    }
}
