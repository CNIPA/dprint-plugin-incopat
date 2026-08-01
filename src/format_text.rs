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
