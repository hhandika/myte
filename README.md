# myte

![myte-tests](https://github.com/hhandika/myte/workflows/myte-tests/badge.svg)
[![Build Status](https://www.travis-ci.com/hhandika/myte.svg?branch=main)](https://www.travis-ci.com/hhandika/myte)

A tool for phylogenomic tree building. The program estimates species tree, gene trees, gene concordance factor, site concordance factor, and organize resulting files using a single command:

```{Bash}
myte auto -d [gene-alignment-folder]
```

<p align="center">
 <img src="static/interface.png" width="500" >
</p>

## Installation

`myte` is a single executable command line app. The executable file will be available in the release [link](https://github.com/hhandika/myte/releases). Copy it to the folder that is registered in your PATH variable.

OS support:

1. MacOS
2. Linux
3. Windows-WSL

Dependencies:

1. [IQ-TREE](http://www.iqtree.org/)
2. [Astral](https://github.com/smirarab/ASTRAL) (optional)

To check if the app can detect the dependencies:

```Bash
myte check
```

See [segul](https://github.com/hhandika/segul) readme for details instruction on how to install a command line application written in Rust.

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

## Usages

```{Bash}
USAGE:
    myte <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    auto     Auto estimate species tree, gene trees, and gene and site concordance factor
    check    Check dependencies
    deps     Solves dependency issues
    gene     Multi-core gene tree estimation using IQ-Tree
    help     Prints this message or the help of the given subcommand(s)
```

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
