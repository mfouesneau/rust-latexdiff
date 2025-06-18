# Rust LaTeXDiff

A fast Rust implementation of LaTeX diff tools, combining the functionality of `latexdiff` and `latexpand`.

## Features

- **LaTeX Expansion**: Recursively expand `\input` and `\include` commands to create a single LaTeX file
- **LaTeX Diffing**: Generate visual diffs between two LaTeX files with colored additions and deletions
- **Fast Performance**: Written in Rust for speed and reliability
- **Command Line Interface**: Easy to use CLI with multiple options

## Installation

### From binaries

Download the latest release from the [Releases page](https://github.com/mfouesneau/rust-latexdiff/releases)

### From Source
To build from source, ensure you have [Rust](https://www.rust-lang.org/tools/install) installed, then clone the repository and build:

```bash
cargo build --release
```

## Usage

### Expand LaTeX Files

Expand a LaTeX file by including all `\input` and `\include` files:

```bash
rust-latexdiff expand input.tex -o expanded.tex
```

Options:
- `--keep-comments`: Keep comments in the expanded output
- `-o, --output`: Specify output file (defaults to stdout)

### Generate LaTeX Diffs

Compare two LaTeX files and generate a diff:

```bash
rust-latexdiff diff old.tex new.tex -o diff.tex
```

Options:
- `--expand`: Expand files before diffing
- `--only-additions`: Show only additions (no deletions)
- `--only-deletions`: Show only deletions (no additions)
- `-o, --output`: Specify output file (defaults to stdout)

## Examples

### Basic Diff
```bash
# Generate a diff between two LaTeX files
rust-latexdiff diff paper_v1.tex paper_v2.tex -o changes.tex
```

### Expand and Diff
```bash
# Expand both files first, then generate diff
rust-latexdiff diff paper_v1.tex paper_v2.tex --expand -o changes.tex
```

### Expand Only
```bash
# Create a single file with all includes expanded
rust-latexdiff expand main.tex -o complete.tex
```

## How It Works

### LaTeX Expansion
- Recursively processes `\input{filename}` and `\include{filename}` commands
- Automatically handles `.tex` extension resolution
- Prevents infinite loops by tracking processed files
- Preserves directory structure for relative paths

### LaTeX Diffing
- Uses advanced diffing algorithms to identify changes
- Wraps deletions in `\DIFdel{}` commands (colored red with strikethrough)
- Wraps additions in `\DIFadd{}` commands (colored blue)
- Handles LaTeX special characters properly
- Generates compilable LaTeX output

## Performance

This Rust implementation is significantly faster than the original Perl-based tools, especially for large documents with many includes.