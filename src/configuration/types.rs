use dprint_core::configuration::*;
use dprint_core::generate_str_to_from;
use serde::{Deserialize, Serialize};

/// Where to place boolean operators when breaking to a new line.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperatorPosition {
    /// Place operator at the beginning of the new line (default).
    Before,
    /// Place operator at the end of the previous line.
    After,
    /// Keep the operator position as-is in the source.
    Preserve,
}

generate_str_to_from![
    OperatorPosition,
    [Before, "before"],
    [After, "after"],
    [Preserve, "preserve"]
];

/// Quote style for quoted phrases.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteStyle {
    Double,
    Single,
    Preserve,
}

generate_str_to_from![
    QuoteStyle,
    [Double, "double"],
    [Single, "single"],
    [Preserve, "preserve"]
];

/// Case style for field codes and boolean operators.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaseStyle {
    Lowercase,
    Uppercase,
    Preserve,
}

generate_str_to_from![
    CaseStyle,
    [Lowercase, "lowercase"],
    [Uppercase, "uppercase"],
    [Preserve, "preserve"]
];
