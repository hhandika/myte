use std::fs;
use std::fs::File;
use std::io::{self, BufWriter, Read, Result, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;

use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

const IQTREE_EXE: &str = "iqtree2";
const ASTRAL_EXE: &str = "astral";
const GENE_TREE_NAME: &str = "genes.treefiles";
const ASTRAL_TREE_NAME: &str = "msc_astral.tree";
const GENE_IQTREE_DIR: &str = "iqtree-genes";
const GENE_TREE_DIR: &str = "gene-treefiles";

pub fn build_species_tree(path: &str, params: &Option<String>) {
    let dir_path = Path::new(path);
    let mut iqtree = SpeciesTree::new(&dir_path, params);
    let msg = format!(
        "\x1b[0mIQ-TREE is processing species tree for alignments in {}...",
        path
    );
    let spin = iqtree.set_spinner();
    spin.set_message(msg);
    iqtree.estimate_species_tree();
    spin.abandon_with_message("Finished estimating species tree!");
}

pub fn build_gene_trees(path: &str, params: &Option<String>, input_fmt: &InputFmt) {
    let mut genes = GeneTrees::new(path, params, input_fmt);
    let paths = genes.get_alignment_paths();
    assert!(
        paths.len() > 1,
        "LESS THAN ONE ALIGNMENT FOUND FOR GENE TREE ANALYSES"
    );

    genes.create_tree_files_dir();
    genes.print_genes_paths(&path).unwrap();

    let num_alignments = paths.len();
    let msg = format!(
        "\x1b[0mIQ-TREE is processing gene trees for {} alignments...",
        num_alignments
    );

    let spin = genes.set_spinner();
    spin.set_message(msg);
    genes.par_process_gene_trees(&paths);

    let finish_msg = format!(
        "\x1b[0mFinished estimating gene trees for {} alignments!",
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
    spin.abandon_with_message("\x1b[0mFinished estimating concordance factor!");
}

pub fn estimate_msc_tree(path: &str) {
    let dir = Path::new(path);
    let mut astral = MSCTree::new(&dir);
    let msg = "\x1b[0mASTRAL is processing MSC tree...";
    let spin = astral.set_spinner();
    spin.set_message(msg);
    astral.estimate_msc_tree();
    spin.abandon_with_message("\x1b[0mFinished estimating MSC tree!");
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
            parent_dir: Path::new(GENE_IQTREE_DIR),
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

    fn print_genes_paths<P: AsRef<Path>>(&self, path: &P) -> Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "Alignment path: {}\n", path.as_ref().display())?;
        Ok(())
    }

    fn create_tree_files_dir(&mut self) {
        fs::create_dir_all(&self.treedir).expect("CANNOT CREATE DIRECTORY FOR TREE FILES");
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
    params: &'a Option<String>,
    outdir: PathBuf,
}

impl<'a> SpeciesTree<'a> {
    fn new(path: &'a Path, params: &'a Option<String>) -> Self {
        Self {
            path,
            prefix: String::from("concat"),
            outdir: PathBuf::from("iqtree-species-tree"),
            params,
        }
    }

    fn estimate_species_tree(&mut self) {
        let iqtree = Process::new(self.path, self.params);
        let out = iqtree.run_iqtree(&self.prefix);
        self.check_process_success(&out, self.path);
        let files = iqtree.get_iqtree_files(&self.prefix);
        self.organize_species_files(&files)
            .expect("FAILED TO MOVE SPECIES TREE RESULT FILES");
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
}

impl<'a> ConcordFactor<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            outdir: PathBuf::from("iqtree-CF"),
            prefix: String::from("concord"),
        }
    }

    fn estimate_concordance(&mut self) {
        let iqtree = Process::new(self.path, &None);
        let out = iqtree.run_iqtree_concord(&self.prefix);
        self.check_process_success(&out, self.path);
        let files = iqtree.get_iqtree_files(&self.prefix);
        self.organize_cf_files(&files)
            .expect("CANNOT MOVE CONCORDANCE FACTOR RESULT FILES");
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

struct MSCTree<'a> {
    path: &'a Path,
    astral_out: String,
}

impl<'a> MSCTree<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            astral_out: String::from("astral.log"),
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

    fn write_astral_output(&self, out: &Output) {
        let mut asral_log = File::create(&self.astral_out).expect("CANNOT WRITE ASTRAL OUTPUT");
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
        out.output().expect("Failed to run IQ-Tree")
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
            .expect("Failed to run IQ-Tree concordance factors")
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
