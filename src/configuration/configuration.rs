use dprint_core::configuration::NewLineKind;
use serde::{Deserialize, Serialize};

use super::types::{CaseStyle, OperatorPosition, QuoteStyle};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub line_width: u32,
    pub use_tabs: bool,
    pub indent_width: u8,
    pub new_line_kind: NewLineKind,
    pub operator_position: OperatorPosition,
    pub quote_style: QuoteStyle,
    pub field_case: CaseStyle,
    pub boolean_operator_case: CaseStyle,
    pub ignore_node_comment_text: String,
}
