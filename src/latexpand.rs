use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;
use anyhow::{Result, Context};

/// Expand a LaTeX file by recursively including all \input and \include files
pub fn expand_latex_file(file_path: &Path, keep_comments: bool) -> Result<String> {
    let base_dir = file_path.parent().unwrap_or(Path::new("."));
    let content = fs::read_to_string(file_path)
        .context(format!("Failed to read file: {}", file_path.display()))?;
    
    expand_latex_content(&content, base_dir, keep_comments)
}

fn expand_latex_content(content: &str, base_dir: &Path, keep_comments: bool) -> Result<String> {
    let mut result = String::new();
    let mut processed_files = std::collections::HashSet::new();
    
    expand_recursive(content, base_dir, keep_comments, &mut processed_files, &mut result)?;
    
    Ok(result)
}

fn expand_recursive(
    content: &str,
    base_dir: &Path,
    keep_comments: bool,
    processed_files: &mut std::collections::HashSet<PathBuf>,
    result: &mut String,
) -> Result<()> {
    // Regex patterns for \input and \include commands
    let input_regex = Regex::new(r"\\input\s*\{([^}]+)\}").unwrap();
    let include_regex = Regex::new(r"\\include\s*\{([^}]+)\}").unwrap();
    let comment_regex = Regex::new(r"%.*$").unwrap();
    
    for line in content.lines() {
        let mut processed_line = line.to_string();
        
        // Remove comments if not keeping them
        if !keep_comments {
            processed_line = comment_regex.replace(&processed_line, "").to_string();
        }
        
        // Check for \input commands
        if let Some(captures) = input_regex.captures(&processed_line) {
            let filename = &captures[1];
            let file_path = resolve_latex_file_path(base_dir, filename);
            
            if let Some(path) = file_path {
                if !processed_files.contains(&path) {
                    processed_files.insert(path.clone());
                    
                    match fs::read_to_string(&path) {
                        Ok(included_content) => {
                            result.push_str(&format!("% BEGIN INCLUDED FILE: {}\n", path.display()));
                            expand_recursive(&included_content, path.parent().unwrap_or(base_dir), 
                                           keep_comments, processed_files, result)?;
                            result.push_str(&format!("% END INCLUDED FILE: {}\n", path.display()));
                            continue;
                        }
                        Err(_) => {
                            // If file can't be read, keep the original command
                            result.push_str(&format!("{}\n", line));
                        }
                    }
                } else {
                    // File already processed, skip to avoid infinite loops
                    result.push_str(&format!("% ALREADY INCLUDED: {}\n", filename));
                    continue;
                }
            } else {
                // File not found, keep original command
                result.push_str(&format!("{}\n", line));
            }
        }
        // Check for \include commands
        else if let Some(captures) = include_regex.captures(&processed_line) {
            let filename = &captures[1];
            let file_path = resolve_latex_file_path(base_dir, filename);
            
            if let Some(path) = file_path {
                if !processed_files.contains(&path) {
                    processed_files.insert(path.clone());
                    
                    match fs::read_to_string(&path) {
                        Ok(included_content) => {
                            result.push_str("\\clearpage\n"); // \include implies \clearpage
                            result.push_str(&format!("% BEGIN INCLUDED FILE: {}\n", path.display()));
                            expand_recursive(&included_content, path.parent().unwrap_or(base_dir), 
                                           keep_comments, processed_files, result)?;
                            result.push_str(&format!("% END INCLUDED FILE: {}\n", path.display()));
                            result.push_str("\\clearpage\n");
                            continue;
                        }
                        Err(_) => {
                            // If file can't be read, keep the original command
                            result.push_str(&format!("{}\n", line));
                        }
                    }
                } else {
                    // File already processed, skip to avoid infinite loops
                    result.push_str(&format!("% ALREADY INCLUDED: {}\n", filename));
                    continue;
                }
            } else {
                // File not found, keep original command
                result.push_str(&format!("{}\n", line));
            }
        } else {
            // Regular line, add to result
            result.push_str(&format!("{}\n", processed_line));
        }
    }
    
    Ok(())
}

fn resolve_latex_file_path(base_dir: &Path, filename: &str) -> Option<PathBuf> {
    // Try different extensions and paths
    let extensions = ["", ".tex", ".latex"];
    
    for ext in &extensions {
        let full_filename = format!("{}{}", filename, ext);
        let path = base_dir.join(&full_filename);
        
        if path.exists() {
            return Some(path);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_basic_expansion() {
        let temp_dir = tempdir().unwrap();
        let main_file = temp_dir.path().join("main.tex");
        let included_file = temp_dir.path().join("chapter1.tex");
        
        fs::write(&main_file, r"
\documentclass{article}
\begin{document}
\input{chapter1}
\end{document}
        ").unwrap();
        
        fs::write(&included_file, r"
\chapter{First Chapter}
This is the first chapter.
        ").unwrap();
        
        let result = expand_latex_file(&main_file, true).unwrap();
        assert!(result.contains("First Chapter"));
        assert!(result.contains("BEGIN INCLUDED FILE"));
    }
}
