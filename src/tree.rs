use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read, Result, Write};
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;

use ansi_term::Colour::{Red, White};
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

// Executable file name
pub const IQTREE_EXE: &str = "iqtree2";
pub const ASTRAL_EXE: &str = "astral.sh";

// Directories and filenames for species tree estimation
const SPECIES_TREE_PREFIX: &str = "concat";
const SPECIES_TREE_OUTPUT_DIR: &str = "iqtree-species-tree";

// Directories and file name for gene tree estimation
const GENE_TREE_NAME: &str = "genes.treefiles";
const GENE_TREE_OUTPUT_DIR: &str = "iqtree-genes";
const GENE_TREE_DIR: &str = "gene-treefiles";

// Concordance factor estimation
const CONCORD_FACTOR_OUTPUT_DIR: &str = "iqtree-CF";
const CONCORD_FACTOR_PREFIX: &str = "concord";

// Astral msc constant
const ASTRAL_TREE_NAME: &str = "msc_astral.tree";
const ASTRAL_LOG_NAME: &str = "msc_astral.log";

pub fn build_species_tree(path: &str, params: &Option<String>) {
    let dir_path = Path::new(path);
    let mut iqtree = SpeciesTree::new(&dir_path, params);
    iqtree.print_species_info();
    let msg = format!(
        "\x1b[0mIQ-TREE is processing species tree for alignments in {}...",
        path
    );
    let spin = iqtree.set_spinner();
    spin.set_message(msg);
    iqtree.estimate_species_tree();
    spin.abandon_with_message("Finished estimating species tree!\n");
}

pub fn build_gene_trees(path: &str, params: &Option<String>, input_fmt: &InputFmt) {
    let mut genes = GeneTrees::new(path, params, input_fmt);
    let paths = genes.get_alignment_paths();
    assert!(
        paths.len() > 1,
        "Ups... Failed to process file. Less than one alignment found"
    );
    genes.create_tree_files_dir();
    let num_aln = paths.len();
    genes.print_genes_info(&path, num_aln);
    let msg = format!(
        "\x1b[0mIQ-TREE is processing gene trees for {} alignments...",
        num_aln
    );

    let spin = genes.set_spinner();
    spin.set_message(msg);
    genes.par_process_gene_trees(&paths);

    let finish_msg = format!(
        "\x1b[0mFinished estimating gene trees for {} alignments!",
        num_aln
    );
    spin.abandon_with_message(finish_msg);
    genes.combine_gene_trees();
}

pub fn estimate_concordance_factor(path: &str) {
    let dir_path = Path::new(path);
    let mut iqtree = ConcordFactor::new(&dir_path);
    iqtree.print_concord_info();
    let msg = "\x1b[0mIQ-TREE is processing concordance factor...";
    let spin = iqtree.set_spinner();
    spin.set_message(msg);
    iqtree.estimate_concordance();
    spin.abandon_with_message("\x1b[0mFinished estimating concordance factor!\n");
}

pub fn estimate_msc_tree(path: &str) {
    let dir = Path::new(path);
    let mut astral = MSCTree::new(&dir);
    astral.print_msc_info();
    let msg = "\x1b[0mASTRAL is processing MSC tree...";
    let spin = astral.set_spinner();
    spin.set_message(msg);
    astral.estimate_msc_tree();
    spin.abandon_with_message("\x1b[0mFinished estimating MSC tree!\n");
}

trait Commons {
    fn get_files(&self, pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .expect("Failed finding alignment files")
            .filter_map(|ok| ok.ok())
            .collect()
    }

    fn set_spinner(&mut self) -> ProgressBar {
        let spin = ProgressBar::new_spinner();
        spin.enable_steady_tick(150);
        spin.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("🌑🌒🌓🌔🌕🌖🌗🌘")
                .template("{spinner} {msg}"),
        );
        spin
    }

    fn check_process_success(&self, out: &Output, path: &Path) {
        if !out.status.success() {
            log::error!(
                "{}: IQ-TREE failed to process {} (See below).",
                White.on(Red).paint("ERROR"),
                path.to_string_lossy()
            );
            log::error!("{}", std::str::from_utf8(&out.stdout).unwrap());
            log::error!("{}", std::str::from_utf8(&out.stderr).unwrap());
        }
    }
}

pub enum InputFmt {
    Fasta,
    Nexus,
    Phylip,
}

impl Commons for GeneTrees<'_> {}
impl Commons for SpeciesTree<'_> {}
impl Commons for ConcordFactor<'_> {}
impl Commons for MSCTree<'_> {}

struct GeneTrees<'a> {
    path: &'a str,
    params: &'a Option<String>,
    treedir: &'a Path,
    parent_dir: &'a Path,
    input_fmt: &'a InputFmt,
}

impl<'a> GeneTrees<'a> {
    fn new(path: &'a str, params: &'a Option<String>, input_fmt: &'a InputFmt) -> Self {
        Self {
            path,
            params,
            treedir: Path::new(GENE_TREE_DIR),
            parent_dir: Path::new(GENE_TREE_OUTPUT_DIR),
            input_fmt,
        }
    }

    fn get_alignment_paths(&mut self) -> Vec<PathBuf> {
        let pattern = self.get_pattern();
        self.get_files(&pattern)
    }

    fn get_pattern(&mut self) -> String {
        match self.input_fmt {
            InputFmt::Fasta => format!("{}/*.fa*", self.path),
            InputFmt::Nexus => format!("{}/*.nex*", self.path),
            InputFmt::Phylip => format!("{}/*.phy*", self.path),
        }
    }

    fn print_genes_info<P: AsRef<Path>>(&self, path: &P, aln_size: usize) {
        log::info!("{:18}: {}", "Alignment path", path.as_ref().display());
        log::info!("{:18}: {}", "File counts", aln_size);
        log::info!("{:18}: IQ-TREE gene tree estimation", "Analyses");
        log::info!("{:18}: {}\n", "Executable", IQTREE_EXE);
    }

    fn create_tree_files_dir(&mut self) {
        fs::create_dir_all(&self.treedir).expect("Failed creating a directory for treefiles");
    }

    fn par_process_gene_trees(&mut self, paths: &[PathBuf]) {
        paths
            .par_iter()
            .for_each(|path| self.estimate_gene_tree(path));
    }

    fn estimate_gene_tree(&self, path: &Path) {
        let prefix = path.file_stem().unwrap().to_string_lossy();
        let iqtree = Process::new(path, self.params);
        let out = iqtree.run_iqtree(&prefix);
        self.check_process_success(&out, path);
        let files = iqtree.get_iqtree_files(&prefix);
        self.organize_gene_files(&files, &prefix).unwrap();
    }

    fn organize_gene_files(&self, files: &[PathBuf], prefix: &str) -> Result<()> {
        let path = self.parent_dir.join(prefix);
        let dir = Path::new(&path);
        fs::create_dir_all(dir)?;
        files.iter().for_each(|file| {
            let ext = file.extension().unwrap().to_string_lossy();
            if ext == "treefile" {
                let outdir = self.treedir.join(file);
                fs::rename(file, outdir).expect("Failed moving a treefile");
            } else {
                let outdir = dir.join(file);
                fs::rename(file, outdir).expect("Failed moving IQ-TREE files");
            }
        });

        Ok(())
    }

    fn combine_gene_trees(&mut self) {
        let pattern = format!("{}/*.treefile", self.treedir.to_string_lossy());
        let trees = self.get_files(&pattern);
        let file =
            File::create(GENE_TREE_NAME).expect("Failed creating a file to store gene trees");
        let mut treefile = BufWriter::new(file);
        let num_trees = trees.len();
        let msg = format!("Combining {} gene trees into a single file...", num_trees);
        let spin = self.set_spinner();
        spin.set_message(msg);
        trees
            .iter()
            .for_each(|tree| self.write_trees(&mut treefile, tree));
        let finish_msg = format!("Finished combining {} gene trees!\n", num_trees);
        spin.finish_with_message(finish_msg);
    }

    fn write_trees<W: Write>(&self, treefile: &mut W, tree_path: &Path) {
        let mut content = String::new();
        let mut tree = File::open(tree_path).expect("Failed accessing a treefile");
        tree.read_to_string(&mut content)
            .expect("CANNOT READ TREE FILES");
        writeln!(treefile, "{}", content.trim()).unwrap();
    }
}

struct SpeciesTree<'a> {
    path: &'a Path,
    prefix: &'a str,
    params: &'a Option<String>,
    outdir: &'a Path,
}

impl<'a> SpeciesTree<'a> {
    fn new(path: &'a Path, params: &'a Option<String>) -> Self {
        Self {
            path,
            prefix: SPECIES_TREE_PREFIX,
            outdir: Path::new(SPECIES_TREE_OUTPUT_DIR),
            params,
        }
    }

    fn estimate_species_tree(&mut self) {
        let iqtree = Process::new(self.path, self.params);
        let out = iqtree.run_iqtree(&self.prefix);
        self.check_process_success(&out, self.path);
        let files = iqtree.get_iqtree_files(&self.prefix);
        self.organize_species_files(&files)
            .expect("Failed moving species tree files");
    }

    fn print_species_info(&self) {
        log::info!("{:18}: IQ-TREE species tree estimation", "Analyses");
        log::info!("{:18}: {}\n", "Executable", IQTREE_EXE);
    }

    fn organize_species_files(&self, files: &[PathBuf]) -> Result<()> {
        fs::create_dir_all(&self.outdir)?;
        files.iter().for_each(|file| {
            let outdir = self.outdir.join(file);
            let ext = file.extension().unwrap().to_string_lossy();
            if ext != "treefile" {
                fs::rename(file, outdir).expect("Failed moving IQ-TREE files");
            }
        });

        Ok(())
    }
}

struct ConcordFactor<'a> {
    path: &'a Path,
    outdir: &'a Path,
    prefix: &'a str,
}

impl<'a> ConcordFactor<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            outdir: Path::new(CONCORD_FACTOR_OUTPUT_DIR),
            prefix: CONCORD_FACTOR_PREFIX,
        }
    }

    fn estimate_concordance(&mut self) {
        let iqtree = Process::new(self.path, &None);
        let out = iqtree.run_iqtree_concord(&self.prefix);
        self.check_process_success(&out, self.path);
        let files = iqtree.get_iqtree_files(&self.prefix);
        self.organize_cf_files(&files)
            .expect("Failed moving concordance factor files");
    }

    fn print_concord_info(&self) {
        log::info!(
            "{:18}: IQ-TREE gene and site concordance factors",
            "Analyses"
        );
        log::info!("{:18}: {}\n", "Executable", IQTREE_EXE);
    }

    fn organize_cf_files(&self, files: &[PathBuf]) -> Result<()> {
        fs::create_dir_all(&self.outdir)?;
        files.iter().for_each(|file| {
            let outdir = self.outdir.join(file);
            let ext = file.extension().unwrap().to_string_lossy();
            if ext != "tre" {
                fs::rename(file, outdir).expect("Failed moving IQ-TREE files");
            }
        });

        Ok(())
    }
}

struct MSCTree<'a> {
    path: &'a Path,
    astral_out: &'a str,
}

impl<'a> MSCTree<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            astral_out: ASTRAL_LOG_NAME,
        }
    }

    fn estimate_msc_tree(&self) {
        let astral = Process::new(self.path, &None);
        let out = astral.run_astral();
        self.check_process_success(&out, self.path);
        if out.status.success() {
            self.write_astral_output(&out);
        }
    }

    fn print_msc_info(&self) {
        log::info!("{:18}: Astral MSC", "Analyses");
        log::info!("{:18}: {}\n", "Executable", ASTRAL_EXE);
    }

    fn write_astral_output(&self, out: &Output) {
        let mut asral_log = File::create(&self.astral_out).expect("Failed writing Astral log");
        write!(asral_log, "{}", str::from_utf8(&out.stderr).unwrap()).unwrap();
    }
}

impl Commons for Process<'_> {}

struct Process<'a> {
    path: &'a Path,
    params: &'a Option<String>,
}

impl<'a> Process<'a> {
    fn new(path: &'a Path, params: &'a Option<String>) -> Self {
        Self { path, params }
    }

    fn run_iqtree(&self, prefix: &str) -> Output {
        let mut out = Command::new(IQTREE_EXE);
        out.arg("-s").arg(self.path).arg("--prefix").arg(prefix);
        self.get_thread_num(&mut out);
        self.get_iqtree_params(&mut out);
        out.output().expect("Failed to run IQ-TREE")
    }

    fn run_iqtree_concord(&self, prefix: &str) -> Output {
        let cores = num_cpus::get_physical();
        let mut out = Command::new(IQTREE_EXE);
        out.arg("-t")
            .arg("concat.treefile")
            .arg("--gcf")
            .arg(GENE_TREE_NAME)
            .arg("-p")
            .arg(&self.path)
            .arg("--scf")
            .arg("100")
            .arg("-T")
            .arg(cores.to_string())
            .arg("--prefix")
            .arg(prefix)
            .output()
            .expect("Failed to run IQ-TREE concordance factors")
    }

    fn run_astral(&self) -> Output {
        let mut out = Command::new(ASTRAL_EXE);
        out.arg("-i")
            .arg(GENE_TREE_NAME)
            .arg("-o")
            .arg(ASTRAL_TREE_NAME)
            .output()
            .expect("Failed to run Astral")
    }

    fn get_iqtree_params(&self, out: &mut Command) {
        match self.params {
            Some(param) => {
                let params: Vec<&str> = param.split_whitespace().collect();
                params.iter().for_each(|param| {
                    out.arg(param);
                });
            }
            None => {
                out.arg("-B").arg("1000");
            }
        }
    }

    fn get_iqtree_files(&self, prefix: &str) -> Vec<PathBuf> {
        let pattern = format!("{}.*", prefix);
        self.get_files(&pattern)
    }

    fn get_thread_num(&self, out: &mut Command) {
        if self.params.is_none() {
            out.arg("-T").arg("1");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const INPUT_FMT: InputFmt = InputFmt::Nexus;

    #[test]
    fn get_gene_paths_test() {
        let path = "test_files";
        let mut genes = GeneTrees::new(path, &None, &INPUT_FMT);
        let gene_paths = genes.get_alignment_paths();

        assert_eq!(2, gene_paths.len());
    }

    #[test]
    #[should_panic]
    fn gene_tree_panic_test() {
        let path = ".";
        build_gene_trees(path, &None, &INPUT_FMT);
    }

    #[test]
    fn get_genetree_fname_test() {
        let name = "genes.treefiles";
        assert_eq!(name, GENE_TREE_NAME);
    }

    #[test]
    fn get_astral_fname_test() {
        let name = "msc_astral.tree";
        assert_eq!(name, ASTRAL_TREE_NAME);
    }
}
