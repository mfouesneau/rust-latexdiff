use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::PathBuf;

mod latexpand;
mod latexdiff;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Expand LaTeX files by including all \\input and \\include files
    Expand {
        /// Input LaTeX file
        input: PathBuf,
        /// Output file (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Keep comments in the expanded output
        #[arg(long)]
        keep_comments: bool,
    },
    /// Compare two LaTeX files and generate a diff
    Diff {
        /// Old version of the LaTeX file
        old: PathBuf,
        /// New version of the LaTeX file
        new: PathBuf,
        /// Output file (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Expand files before diffing
        #[arg(long)]
        expand: bool,
        /// Show only additions (no deletions)
        #[arg(long)]
        only_additions: bool,
        /// Show only deletions (no additions)
        #[arg(long)]
        only_deletions: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Expand { input, output, keep_comments } => {
            let expanded = latexpand::expand_latex_file(&input, keep_comments)
                .context("Failed to expand LaTeX file")?;
            
            if let Some(output_path) = output {
                std::fs::write(&output_path, expanded)
                    .context(format!("Failed to write to {}", output_path.display()))?;
                println!("Expanded file written to: {}", output_path.display());
            } else {
                print!("{}", expanded);
            }
        }
        Commands::Diff { old, new, output, expand, only_additions, only_deletions } => {
            let diff_result = if expand {
                // Expand both files first
                let old_expanded = latexpand::expand_latex_file(&old, true)
                    .context("Failed to expand old file")?;
                let new_expanded = latexpand::expand_latex_file(&new, true)
                    .context("Failed to expand new file")?;
                
                latexdiff::generate_diff(&old_expanded, &new_expanded, only_additions, only_deletions)
            } else {
                let old_content = std::fs::read_to_string(&old)
                    .context(format!("Failed to read {}", old.display()))?;
                let new_content = std::fs::read_to_string(&new)
                    .context(format!("Failed to read {}", new.display()))?;
                
                latexdiff::generate_diff(&old_content, &new_content, only_additions, only_deletions)
            };
            
            if let Some(output_path) = output {
                std::fs::write(&output_path, diff_result)
                    .context(format!("Failed to write to {}", output_path.display()))?;
                println!("Diff written to: {}", output_path.display());
            } else {
                print!("{}", diff_result);
            }
        }
    }

    Ok(())
}
