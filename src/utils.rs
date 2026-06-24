/// Shared LaTeX utilities used by both `latexpand` and `latexdiff`.

/// Remove LaTeX comments from a line, handling escaped `\%` correctly.
pub fn remove_latex_comments(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                result.push(ch);
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '%' || next_ch == '\\' {
                        result.push(chars.next().unwrap());
                    }
                }
            }
            '%' => break, // Comment starts here
            _ => result.push(ch),
        }
    }

    result.trim_end().to_string()
}

/// Replace problematic Unicode characters with LaTeX equivalents.
pub fn replace_unicode_chars(text: &str) -> String {
    text.replace('\u{2014}', "---")   // em dash —
        .replace('\u{2013}', "--")    // en dash –
        .replace('\u{2018}', "`")     // left single quote ‘
        .replace('\u{2019}', "'")     // right single quote ’
        .replace('\u{201C}', "``")    // left double quote “
        .replace('\u{201D}', "''")    // right double quote ”
        .replace('\u{142}', "\\l{}")  // Polish ł
}

pub fn escape_latex_special_chars(text: &str) -> String {
    // Only escape characters that are unconditionally problematic in LaTeX text mode.
    // Math, commands, braces, etc. are left intact.
    text.replace('&', "\\&").replace('%', "\\%")
}
