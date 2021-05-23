# myte

![myte-tests](https://github.com/hhandika/myte/workflows/myte-tests/badge.svg)
[![Build Status](https://www.travis-ci.com/hhandika/myte.svg?branch=main)](https://www.travis-ci.com/hhandika/myte)

A tool to simplify phylogemics data analyses.

Current goal:

Estimate species tree, gene trees, gene concordance factor, site concordance factor, and organize resulting files using a single command:

```{Bash}
myte auto -d [gene-alignment-folder]
```

Future goals:

1. Incorporate multi-species coalescence analyses in the pipeline.
2. Other tools useful for genomic tree estimation. The goal is to take advantage of Rust performance (identical to C/C++) to re-write typical genomic tasks that are inefficient to do in interpreted languages, such as Python, R, Perl, etc.

## Installation

`myte` is a single executable command line app. The executable file will be available in the release [link](https://github.com/hhandika/myte/releases). Copy it to the folder that is registered in your PATH variable.

__Comments__: I will update this instruction once the program has complete planned features.

OS support:

1. MacOS
2. Linux
3. Windows-WSL

Dependencies: 

1. [IQ-TREE](http://www.iqtree.org/)

### Compile from source

Download the rust compiler [here](https://www.rust-lang.org/learn/get-started) and follow the installation instruction.

```{Bash}
git clone https://github.com/hhandika/myte
```

```{Bash}
cd myte

cargo build --release
```

Your executable will be available at `/target/release/myte`. Copy it to the folder that is registered in your PATH variable.

__Notes__: The program may failed to run in outdated HPC OS due to GLIBC errors. The solution is to compile it to fully static binary using `musl` compiler. See instruction [here](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html). Then, to build the binary `cargo build --release --target x86_64-unknown-linux-musl`

## Status of the Code

The code is still at infancy. Working features:

### Auto estimate species and gene trees and concordance factors

```{Bash}
myte auto -d [alignment-folder]
```

### Build gene trees from a directory of gene alignments

The program will create multiple instances of IQ-TREE to run gene tree estimation in parallel. The program assess available cpu resources in your system and does it sensibly. In a simple word, it won't slow down your computer despite using all your cpu cores. Hence, it can be used on a personal computer without interferring your other work.

To generate gene trees:

```{Bash}
myte gene -d [alignment-folder]
```

### Feature planned

1. Allow costum parameters for IQ-TREE.
2. Neater terminal output.
