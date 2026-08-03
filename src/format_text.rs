use std::path::Path;

use anyhow::Result;
use dprint_core::configuration::NewLineKind;
use dprint_core::formatting::PrintOptions;

use crate::configuration::Configuration;
use crate::generation::context::Context;
use crate::generation::generate::generate;
use crate::normalize::normalize_query;
use crate::parser::ast::{File, Statement};
use crate::parser::parser::Parser;

/// Format a `.incopat` file. Returns `Ok(Some(text))` if changes were made,
/// `Ok(None)` if the text is already formatted, or `Err` on failure.
pub fn format_text(_path: &Path, text: &str, config: &Configuration) -> Result<Option<String>> {
    let parse_result = Parser::new(text).parse();

    // Safety net: never rewrite a file that could not be fully parsed.
    // Otherwise unrecoverable input would be silently discarded and the file
    // would be replaced with a partial (destructive) result.
    if !parse_result.errors.is_empty() {
        let messages: Vec<String> = parse_result
            .errors
            .iter()
            .map(|e| {
                let (line, col) = line_col(text, e.span.start);
                format!("第{}行第{}列: {}", line, col, e.message)
            })
            .collect();
        return Err(anyhow::anyhow!(
            "incoPat 语法错误: {}",
            messages.join("; ")
        ));
    }

    // Enforce the semantic search (R/RAD/RPD) placement rules. Fixable
    // violations are corrected automatically; the rest are reported as errors.
    let mut violations = Vec::new();
    let mut statements = Vec::with_capacity(parse_result.file.statements.len());
    for stmt in &parse_result.file.statements {
        match stmt {
            Statement::Query(expr) => match normalize_query(expr) {
                Ok(normalized) => statements.push(Statement::Query(normalized)),
                Err(msg) => violations.push(msg),
            },
            other => statements.push(other.clone()),
        }
    }
    if !violations.is_empty() {
        return Err(anyhow::anyhow!(
            "incoPat 语法错误: {}",
            violations.join("; ")
        ));
    }
    let file = File { statements };

    let ctx = Context::new(config, text);

    let formatted = dprint_core::formatting::format(
        || generate(&file, &ctx),
        PrintOptions {
            indent_width: config.indent_width,
            max_width: config.line_width,
            use_tabs: config.use_tabs,
            new_line_text: resolve_new_line_kind(text, config.new_line_kind),
        },
    );

    if formatted == text {
        Ok(None)
    } else {
        Ok(Some(formatted))
    }
}

/// Resolve the new line kind to a static string based on configuration and source text.
fn resolve_new_line_kind(text: &str, kind: NewLineKind) -> &'static str {
    match kind {
        NewLineKind::LineFeed => "\n",
        NewLineKind::CarriageReturnLineFeed => "\r\n",
        _ => {
            // Auto or any future variant: detect from source text
            if text.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            }
        }
    }
}

/// Convert a byte offset into a 1-based (line, column) pair for error messages.
fn line_col(text: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in text.char_indices() {
        if i >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use dprint_core::configuration::GlobalConfiguration;

    use super::*;
    use crate::configuration::resolve_config;

    fn format(input: &str) -> Result<Option<String>, String> {
        let config = resolve_config(Default::default(), &GlobalConfiguration::default());
        format_text(Path::new("file.incopat"), input, &config.config)
            .map_err(|e| format!("{:#}", e))
    }

    #[test]
    fn unknown_field_is_preserved_and_formatted() {
        let result = format("apnortt=(清华大学)").unwrap();
        assert_eq!(result.as_deref(), Some("apnortt = (清华大学)\n"));
    }

    #[test]
    fn unknown_field_with_known_fields_formats_together() {
        let result = format("apnortt=(清华大学)\nand tiabc = (压缩机 or compressor)").unwrap();
        assert_eq!(
            result.as_deref(),
            Some(
                "(\n        apnortt = (清华大学)\n    and tiabc = (压缩机 or compressor)\n)\n"
            )
        );
    }

    #[test]
    fn parse_error_returns_err_instead_of_truncating() {
        // Unclosed parenthesis: the parser recovers internally, but rewriting
        // the file with the recovered (partial) result would destroy input.
        let result = format("tiabc = (压缩机\nand des = (比热容)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected ')'"));
    }

    #[test]
    fn error_position_reports_line_and_column() {
        let result = format("ti = (a)\n= (b)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("第2行"));
    }
}
