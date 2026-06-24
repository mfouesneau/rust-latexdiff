/// A fast Rust implementation of LaTeX diff tools, combining the functionality of `latexdiff` and `latexpand`.
use regex::Regex;
use similar::{ChangeTag, TextDiff};
use crate::utils::{escape_latex_special_chars, remove_latex_comments, replace_unicode_chars};

const LATEX_DIFF_PREAMBLE: &str = "% rust-latexdiff preamble\n\
\\usepackage[normalem]{ulem}\n\
\\newcommand{\\DIFdel}[1]{{\\color{red}\\sout{#1}}}\n\
\\newcommand{\\DIFadd}[1]{{\\color{blue}\\uline{#1}}}\n\n";

// --- helpers ---

fn insert_preamble_after_documentclass(body: &str) -> String {
    if let Some(pos) = body.find("\\documentclass") {
        if let Some(newline_pos) = body[pos..].find('\n') {
            let split_at = pos + newline_pos + 1;
            return format!(
                "{}{}{}",
                &body[..split_at],
                LATEX_DIFF_PREAMBLE,
                &body[split_at..]
            );
        }
    }
    format!("{}{}", LATEX_DIFF_PREAMBLE, body)
}

#[derive(Debug, Clone)]
struct LaTeXContext {
    in_verbatim: bool,
}

impl LaTeXContext {
    fn new() -> Self {
        Self { in_verbatim: false }
    }
}

fn remove_latex_comments_with_context(line: &str, context: &LaTeXContext) -> String {
    if context.in_verbatim {
        return line.to_string();
    }
    if line.contains("\\begin{verbatim}") || line.contains("\\begin{lstlisting}") {
        return line.to_string();
    }
    remove_latex_comments(line)
}

fn preprocess_latex_text(text: &str) -> String {
    let mut result = String::new();
    let mut context = LaTeXContext::new();

    for line in text.lines() {
        let processed = remove_latex_comments_with_context(line, &context);
        let processed = replace_unicode_chars(&processed);

        if line.contains("\\begin{verbatim}") || line.contains("\\begin{lstlisting}") {
            context.in_verbatim = true;
        }
        if line.contains("\\end{verbatim}") || line.contains("\\end{lstlisting}") {
            context.in_verbatim = false;
        }

        let is_comment_only = line.trim_start().starts_with('%') && processed.trim().is_empty();

        if !is_comment_only {
            result.push_str(&processed);
            result.push('\n');
        }
    }

    result
}

// --- diff logic ---

/// Tokenizer that splits text into words AND whitespace, preserving punctuation
/// as part of words. LaTeX commands (`\foo`) stay atomic.
fn tokenize(text: &str) -> Vec<&str> {
    let re = Regex::new(r"(\s+|\\(?:[a-zA-Z]+\s*|\\.)|[{}]|\w+|[^\w\s\\{}]+)").unwrap();
    re.find_iter(text)
        .map(|m| m.as_str())
        .filter(|s| !s.is_empty())
        .collect()
}

fn is_latex_token(token: &str) -> bool {
    let trimmed = token.trim();
    trimmed.starts_with('\\') || trimmed == "{" || trimmed == "}" || trimmed.is_empty()
}

/// Check whether two texts are effectively the same (taking into account
/// the Unicode replacements we apply during preprocessing).
fn texts_equal(a: &str, b: &str) -> bool {
    preprocess_latex_text(a) == preprocess_latex_text(b)
}

/// Generate a LaTeX diff between two texts.
/// Uses a hybrid approach:
/// - Lines that appear in both texts are copied unchanged (fast path).
/// - Added/removed lines are identified, and a word-level diff is computed
///   for the content of those changed regions so that DIF annotations stay
///   small (per word, not per paragraph).
pub fn generate_diff(
    old_text: &str,
    new_text: &str,
    only_additions: bool,
    only_deletions: bool,
) -> String {
    let old_processed = preprocess_latex_text(old_text);
    let new_processed = preprocess_latex_text(new_text);

    if old_processed.is_empty() && new_processed.is_empty() {
        return String::new();
    }

    // Line-level diff to spot unchanged vs changed groups
    let line_diff = TextDiff::from_lines(&old_processed, &new_processed);

    let mut result = String::new();

    // Collect all changes with their kind
    let changes: Vec<(ChangeTag, String)> = line_diff
        .iter_all_changes()
        .map(|c| (c.tag(), c.value().to_string()))
        .collect();

    let mut i = 0;
    while i < changes.len() {
        let (tag, content) = &changes[i];

        match tag {
            ChangeTag::Equal => {
                result.push_str(content);
                i += 1;
            }
            ChangeTag::Delete => {
                // Collect contiguous Delete + Insert ops
                let mut old_lines_in_group = vec![];
                let mut new_lines_in_group = vec![];
                while i < changes.len() {
                    match &changes[i].0 {
                        ChangeTag::Delete => {
                            old_lines_in_group.push(changes[i].1.clone());
                            i += 1;
                        }
                        ChangeTag::Insert => {
                            new_lines_in_group.push(changes[i].1.clone());
                            i += 1;
                        }
                        _ => break,
                    }
                }
                // If only deletes (no inserts), mark them as deletions
                if !only_additions && new_lines_in_group.is_empty() {
                    let old_text_combined: String = old_lines_in_group.into_iter().collect();
                    result.push_str(&format!(
                        "\\DIFdel{{{}}}\n",
                        escape_latex_special_chars(old_text_combined.trim())
                    ));
                } else if !only_deletions && !old_lines_in_group.is_empty() && !new_lines_in_group.is_empty() {
                    // Both have changes — do fine-grained word diff between
                    // the concatenated old and new texts
                    let old_combined: String = old_lines_in_group.into_iter().collect();
                    let new_combined: String = new_lines_in_group.into_iter().collect();

                    if texts_equal(&old_combined, &new_combined) {
                        result.push_str(&old_combined);
                    } else {
                        let old_tokens = tokenize(&old_combined);
                        let new_tokens = tokenize(&new_combined);
                        let word_diff = TextDiff::from_slices(&old_tokens, &new_tokens);

                        // Collect all word-level changes, then group consecutive
                        // same-tag changes to preserve whitespace within groups
                        let actions: Vec<(ChangeTag, String)> = word_diff
                            .iter_all_changes()
                            .map(|c| (c.tag(), c.value().to_string()))
                            .collect();

                        // Merge Delete and Insert groups: consecutive same-tag ops
                        // are a single group. This keeps whitespace between changed
                        // words inside a single \DIFdel or \DIFadd block.
                        let mut grouped: Vec<(ChangeTag, String)> = Vec::new();
                        for (tag, val) in actions {
                            if let Some((last_tag, ref mut last_val)) = grouped.last_mut() {
                                if *last_tag == tag {
                                    last_val.push_str(&val);
                                    continue;
                                }
                            }
                            grouped.push((tag, val));
                        }

                        for (tag, val) in grouped {
                            match tag {
                                ChangeTag::Equal => {
                                    if !is_latex_token(&val) || !val.trim().is_empty() {
                                        result.push_str(&val);
                                    }
                                }
                                ChangeTag::Delete => {
                                    if !only_additions && !is_latex_token(&val) {
                                        result.push_str(&format!(
                                            "\\DIFdel{{{}}}",
                                            escape_latex_special_chars(&val)
                                        ));
                                    }
                                }
                                ChangeTag::Insert => {
                                    if !only_deletions && !is_latex_token(&val) {
                                        result.push_str(&format!(
                                            "\\DIFadd{{{}}}",
                                            escape_latex_special_chars(&val)
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                // else: only_additions && old_lines_in_group non-empty => skip
            }
            ChangeTag::Insert => {
                // Insert without preceding delete
                let mut add_lines = vec![content.clone()];
                let mut j = i + 1;
                while j < changes.len() {
                    match &changes[j].0 {
                        ChangeTag::Insert => {
                            add_lines.push(changes[j].1.clone());
                            j += 1;
                        }
                        _ => break,
                    }
                }
                i = j;

                if !only_deletions {
                    let combined: String = add_lines.into_iter().collect();
                    result.push_str(&format!(
                        "\\DIFadd{{{}}}\n",
                        escape_latex_special_chars(combined.trim())
                    ));
                    result.push('\n');
                }
            }
        }
    }

    insert_preamble_after_documentclass(&result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_diff() {
        let old_text = "Hello world\nThis is a test";
        let new_text = "Hello universe\nThis is a test\nNew line added";
        let diff = generate_diff(old_text, new_text, false, false);
        assert!(diff.contains("DIFdel"));
        assert!(diff.contains("DIFadd"));
    }

    #[test]
    fn test_latex_escape() {
        let text = "This has & and % comments";
        let escaped = escape_latex_special_chars(text);
        assert!(escaped.contains("\\&"));
        assert!(escaped.contains("\\%"));
    }

    #[test]
    fn test_latex_comment_removal() {
        let text_with_comments =
            "This is text % this is a comment\nMore text\n% Full line comment\nFinal line";
        let processed = preprocess_latex_text(text_with_comments);
        assert!(!processed.contains("this is a comment"));
        assert!(!processed.contains("Full line comment"));
        assert!(processed.contains("This is text"));
        assert!(processed.contains("More text"));
        assert!(processed.contains("Final line"));
    }

    #[test]
    fn test_escaped_percent_preservation() {
        let text = "This has \\% escaped percent and % comment";
        let processed = remove_latex_comments(text);
        assert!(processed.contains("\\%"));
        assert!(!processed.contains("comment"));
        assert_eq!(processed, "This has \\% escaped percent and");
    }

    #[test]
    fn test_diff_ignores_comments() {
        let old_text = "Hello world % old comment\nSecond line";
        let new_text = "Hello world % new comment\nSecond line";
        let diff = generate_diff(old_text, new_text, false, false);
        let body_start = diff.find("Hello world").unwrap_or(0);
        let body = &diff[body_start..];
        assert!(!body.contains("\\DIFdel{"));
        assert!(!body.contains("\\DIFadd{"));
    }

    #[test]
    fn test_verbatim_comment_preservation() {
        let text = "Normal text % comment\n\\begin{verbatim}\nCode with % symbol\n\\end{verbatim}";
        let processed = preprocess_latex_text(text);
        assert!(!processed.contains("comment"));
        assert!(processed.contains("Code with % symbol"));
        assert!(processed.contains("Normal text"));
    }

    #[test]
    fn test_unicode_replacement() {
        let text = "\u{201C}curly quotes\u{201D} and em\u{2014}dash";
        let processed = preprocess_latex_text(text);
        assert!(processed.contains("``curly quotes''"));
        assert!(processed.contains("---dash"));
        assert!(!processed.contains('\u{2014}')); // em dash should be replaced
    }

    #[test]
    fn test_respects_only_flags() {
        let old_text = "apple\nbanana\ncherry";
        let new_text = "apricot\nbanana\nclementine";

        // only_additions: should not show deletions
        let diff_add = generate_diff(old_text, new_text, true, false);
        assert!(!diff_add.contains("DIFdel{"));
        assert!(diff_add.contains("DIFadd"));

        // only_deletions: should not show additions
        let diff_del = generate_diff(old_text, new_text, false, true);
        assert!(!diff_del.contains("DIFadd{"));
        assert!(diff_del.contains("DIFdel"));
    }
}
