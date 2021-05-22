# myte

![myte-tests](https://github.com/hhandika/myte/workflows/myte-tests/badge.svg)
[![Build Status](https://www.travis-ci.com/hhandika/myte.svg?branch=main)](https://www.travis-ci.com/hhandika/myte)

Phylogenomics tools for tree building. The goal is to simplify phylogenomic tree building.

Current goal:

Estimate species tree, gene trees, gene concordance factor, site concordance factor, and organize resulting files using a single command:

```{Bash}
myte auto -d [gene-alignment-folder]
```

Future goal:

1. Incorporate multi-species coalescence analyses in the pipeline.
2. Other tools useful for tree building.

## Installation

`myte` is a single executable command line app. The executable file will be available in the release [link](https://github.com/hhandika/myte/releases). Copy it the folder that registered in your path variable.

__Comments__: I will update this instruction once the program has complete planned features.

OS support:

1. MacOS
2. Linux
3. Windows-WSL

Dependencies at runtime: [IQ-TREE](http://www.iqtree.org/)

### Compile from source

Download the rust compiler [here](https://www.rust-lang.org/learn/get-started) and follow the installation instruction.

```{Bash}
git clone https://github.com/hhandika/myte
```

```{Bash}
cd myte

cargo build --release
```

Your executable will be available at `/target/release/myte`. Copy it the folder that is registered in your environmental variable.

__Notes__: The program may failed to run in outdated HPC OS due to GLIBC errors. The solution is to compile it to fully static binary using `musl` compiler. See instruction [here](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html). Then, to build the binary `cargo build --release --target x86_64-unknown-linux-musl

## Status of the Code

The code is still at infancy. Working features:

### Build gene trees from a directory of gene alignments

The program will create multiple instances of IQ-TREE to run gene trees estimation in parallel. The program assess available cpu resources in your system and does it sensibly. In a simply words, it won't slow down your computer despite using all your cpu cores. Hence, it can be used on a personal computers without problems.

To generate gene trees:

```{Bash}
myte gene -d [alignment-folder]
```

### Feature planned

1. Allow costum parameters for IQ-TREE.
2. Working auto commands.
