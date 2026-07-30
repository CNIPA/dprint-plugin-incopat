use std::path::Path;

use anyhow::Result;
use dprint_core::configuration::NewLineKind;
use dprint_core::formatting::PrintOptions;

use crate::configuration::Configuration;
use crate::generation::context::Context;
use crate::generation::generate::generate;
use crate::parser::parser::Parser;

/// Format a `.incopat` file. Returns `Ok(Some(text))` if changes were made,
/// `Ok(None)` if the text is already formatted, or `Err` on failure.
pub fn format_text(_path: &Path, text: &str, config: &Configuration) -> Result<Option<String>> {
    let parse_result = Parser::new(text).parse();

    let ctx = Context::new(config, text);

    let formatted = dprint_core::formatting::format(
        || generate(&parse_result.file, &ctx),
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
