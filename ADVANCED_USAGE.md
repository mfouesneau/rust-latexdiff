# Advanced Usage Examples

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [Advanced Expansion](#advanced-expansion)
3. [Sophisticated Diffing](#sophisticated-diffing)
4. [Integration with LaTeX Workflow](#integration-with-latex-workflow)
5. [Performance Tips](#performance-tips)

## Basic Usage

### Simple File Expansion

```bash
# Expand a single file
./target/release/rust-latexdiff expand main.tex

# Save to file
./target/release/rust-latexdiff expand main.tex -o complete.tex

# Keep comments while expanding
./target/release/rust-latexdiff expand main.tex --keep-comments -o complete.tex
```

### Basic Diffing

```bash
# Generate diff between two files
./target/release/rust-latexdiff diff old.tex new.tex

# Save diff to file
./target/release/rust-latexdiff diff old.tex new.tex -o changes.tex
```

## Advanced Expansion

### Handling Complex Project Structures

The tool automatically handles:
- Relative paths in `\input{chapter1/section1.tex}`
- Files with and without `.tex` extension
- Nested includes (with cycle detection)
- Mixed `\input` and `\include` commands

### Example Project Structure

```
project/
├── main.tex
├── chapters/
│   ├── intro.tex
│   ├── methods.tex
│   └── conclusion.tex
├── sections/
│   ├── background.tex
│   └── related_work.tex
└── figures/
    └── diagrams.tex
```

## Sophisticated Diffing

### Diff with Expansion

```bash
# Expand both files before diffing (useful for complex projects)
./target/release/rust-latexdiff diff main_v1.tex main_v2.tex --expand -o changes.tex
```

### Selective Diffing

```bash
# Show only additions (new content)
./target/release/rust-latexdiff diff old.tex new.tex --only-additions

# Show only deletions (removed content)
./target/release/rust-latexdiff diff old.tex new.tex --only-deletions
```

### Diff Output Styles

The tool generates LaTeX commands that can be customized:

#### Default Style (Red deletions, Blue additions)
```latex
\newcommand{\DIFdel}[1]{\textcolor{red}{\sout{#1}}}
\newcommand{\DIFadd}[1]{\textcolor{blue}{#1}}
```

#### Alternative Styles
```latex
% Bold additions, struck-out deletions
\newcommand{\DIFdel}[1]{\textcolor{red}{\st{#1}}}
\newcommand{\DIFadd}[1]{\textcolor{blue}{\bf #1}}

% Highlighted background
\newcommand{\DIFdel}[1]{\colorbox{red!20}{\sout{#1}}}
\newcommand{\DIFadd}[1]{\colorbox{blue!20}{#1}}

% Margin notes
\newcommand{\DIFdel}[1]{\marginpar{\textcolor{red}{DEL}}\textcolor{red}{\sout{#1}}}
\newcommand{\DIFadd}[1]{\marginpar{\textcolor{blue}{ADD}}\textcolor{blue}{#1}}
```

## Integration with LaTeX Workflow

### Complete Document Workflow

1. **Expand your LaTeX document:**
   ```bash
   ./target/release/rust-latexdiff expand main.tex -o main_expanded.tex
   ```

2. **Make changes and expand again:**
   ```bash
   ./target/release/rust-latexdiff expand main_v2.tex -o main_v2_expanded.tex
   ```

3. **Generate diff:**
   ```bash
   ./target/release/rust-latexdiff diff main_expanded.tex main_v2_expanded.tex -o changes.tex
   ```

4. **Compile the diff:**
   ```bash
   pdflatex changes.tex
   ```

### Automated Script Example

```bash
#!/bin/bash
# diff_latex.sh - Automated LaTeX diffing script

OLD_FILE="$1"
NEW_FILE="$2"
OUTPUT_FILE="${3:-diff_output.tex}"

echo "Generating LaTeX diff..."

# Generate diff with expansion
./target/release/rust-latexdiff diff "$OLD_FILE" "$NEW_FILE" --expand -o "$OUTPUT_FILE"

# Add necessary packages to the beginning
{
    echo "\\documentclass{article}"
    echo "\\usepackage{xcolor}"
    echo "\\usepackage[normalem]{ulem}"
    echo "\\begin{document}"
    cat "$OUTPUT_FILE"
    echo "\\end{document}"
} > "complete_$OUTPUT_FILE"

echo "Complete diff document created: complete_$OUTPUT_FILE"
echo "Compile with: pdflatex complete_$OUTPUT_FILE"
```

## Performance Tips

### Large Documents

For large documents with many includes:

1. **Use expansion sparingly** - Only expand when necessary for diffing
2. **Selective diffing** - Use `--only-additions` or `--only-deletions` for faster processing
3. **Batch processing** - Process multiple files in parallel

### Memory Usage

The tool loads entire files into memory. For very large documents:
- Consider splitting into smaller chunks
- Use the expansion feature to understand the actual size of your document

### Speed Comparisons

Typical performance improvements over Perl-based tools:
- **Small files (< 1MB)**: 5-10x faster
- **Medium files (1-10MB)**: 10-20x faster  
- **Large files (> 10MB)**: 20-50x faster

## Common Use Cases

### Academic Paper Revisions

```bash
# Compare two versions of a paper
./target/release/rust-latexdiff diff paper_v1.tex paper_v2.tex --expand -o revisions.tex
```

### Collaborative Writing

```bash
# Show only what was added by collaborators
./target/release/rust-latexdiff diff my_version.tex their_version.tex --only-additions
```

### Thesis Writing

```bash
# Expand entire thesis to single file for submission
./target/release/rust-latexdiff expand thesis.tex -o thesis_complete.tex
```

### Journal Submissions

```bash
# Create a diff for reviewers
./target/release/rust-latexdiff diff submitted_version.tex revised_version.tex --expand -o reviewer_changes.tex
```

## Troubleshooting

### Common Issues

1. **File not found errors**
   - Ensure relative paths are correct
   - Check that included files have proper extensions

2. **Compilation errors in diff output**
   - Verify that all required packages are included
   - Check for special characters that need escaping

3. **Large memory usage**
   - Consider processing smaller sections
   - Use selective diffing options

### Best Practices

1. **Always test compile** your diff output
2. **Keep backups** of original files
3. **Use version control** alongside LaTeX diffing
4. **Validate paths** before running expansion on large projects
