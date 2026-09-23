//! Semantic search (R/RAD/RPD) syntax rules.
//!
//! incoPat imposes special placement rules on the semantic search fields
//! `R`, `RAD` and `RPD`:
//!
//! 1. A semantic search field may appear at most once per query.
//! 2. It must sit at the very top level of the query, at the beginning or
//!    the end (never in the middle).
//! 3. It must be connected to the rest of the query with `and`. An `or`
//!    connector is corrected automatically; a missing (implicit) connector
//!    becomes an explicit `and`.
//! 4. The rest of the query must be wrapped in parentheses unless it is a
//!    single field. The parentheses are added automatically when missing.
//! 5. The semantic search field itself must never be wrapped in a group.
//!    Groups around it are unwrapped automatically.
//!
//! Fixes that do not change the meaning of the query (adding parentheses,
//! removing parentheses, correcting `or` to `and`) are applied
//! automatically. Anything else (duplicate semantic fields, a semantic field
//! in the middle of the chain, a semantic field nested inside `not`, a field
//! value, a proximity/frequency/TREE@ operand, ...) is reported as an error.

use crate::parser::ast::*;
use crate::parser::fields::is_semantic_keyword;
use crate::parser::token::Span;

/// Whether `expr` is a semantic search expression (R/RAD/RPD).
fn is_semantic(expr: &QueryExpr) -> bool {
    matches!(expr, QueryExpr::SemanticSearch(s) if is_semantic_keyword(&s.keyword))
}

/// Whether a semantic search expression exists anywhere in the subtree.
fn semantic_in(expr: &QueryExpr) -> bool {
    match expr {
        QueryExpr::SemanticSearch(s) => is_semantic_keyword(&s.keyword),
        QueryExpr::Binary(b) => semantic_in(&b.left) || semantic_in(&b.right),
        QueryExpr::Not(n) => semantic_in(&n.operand),
        QueryExpr::Group(g) => semantic_in(&g.inner),
        QueryExpr::Field(f) => match &f.body {
            FieldBody::Simple(inner) => semantic_in(inner),
            FieldBody::Parenthesized { inner, .. } => semantic_in(inner),
        },
        QueryExpr::Proximity(p) => semantic_in(&p.left) || semantic_in(&p.right),
        QueryExpr::Frequency(f) => semantic_in(&f.operand),
        QueryExpr::Optional(o) => semantic_in(&o.left) || semantic_in(&o.right),
        QueryExpr::TreeAt(t) => semantic_in(&t.operand),
        _ => false,
    }
}

/// Flatten a binary chain into its operands (nested binaries on both sides).
fn flat_chain<'a>(expr: &'a QueryExpr, out: &mut Vec<&'a QueryExpr>) {
    match expr {
        QueryExpr::Binary(b) => {
            flat_chain(&b.left, out);
            flat_chain(&b.right, out);
        }
        other => out.push(other),
    }
}

/// Position of the semantic expression within the top-level chain.
#[derive(PartialEq, Clone, Copy)]
enum Pos {
    Start,
    End,
}

/// Determine where the semantic expression sits. Errors when it sits in the
/// middle of the chain or appears more than once.
fn compute_position(expr: &QueryExpr) -> Result<Pos, String> {
    match expr {
        QueryExpr::Group(g) => compute_position(&g.inner),
        QueryExpr::SemanticSearch(s) if is_semantic_keyword(&s.keyword) => Ok(Pos::Start),
        _ => {
            let mut parts = Vec::new();
            flat_chain(expr, &mut parts);
            let mut idx = None;
            for (i, part) in parts.iter().enumerate() {
                if semantic_in(part) {
                    if idx.is_some() {
                        return Err("语义检索字段 (R/RAD/RPD) 只能出现一次".to_string());
                    }
                    idx = Some(i);
                }
            }
            match idx {
                None => Err("internal error: semantic search expression not found".to_string()),
                Some(0) => Ok(Pos::Start),
                Some(i) if i == parts.len() - 1 => Ok(Pos::End),
                Some(_) => Err(
                    "语义检索字段 (R/RAD/RPD) 必须位于检索式的开头或结尾，不能出现在中间位置"
                        .to_string(),
                ),
            }
        }
    }
}

/// Find the boolean operator that directly connects the semantic expression
/// to its parent binary node (None when there is no parent binary).
fn parent_connector(expr: &QueryExpr) -> Option<(BoolOp, Span)> {
    match expr {
        QueryExpr::Binary(b) => {
            if is_semantic(&b.left) {
                return Some((b.op, b.op_span));
            }
            if is_semantic(&b.right) {
                return Some((b.op, b.op_span));
            }
            parent_connector(&b.left).or_else(|| parent_connector(&b.right))
        }
        QueryExpr::Group(g) => parent_connector(&g.inner),
        _ => None,
    }
}

/// Extract the semantic search expression from a query, leaving the rest of
/// the expression behind. Groups that contained the semantic expression are
/// unwrapped so that it ends up at the top level.
fn extract_r(expr: &QueryExpr) -> Result<(Option<QueryExpr>, Option<SemanticSearchExpr>), String> {
    match expr {
        QueryExpr::SemanticSearch(s) if is_semantic_keyword(&s.keyword) => {
            // A semantic search value must not itself contain another semantic
            // search expression (e.g. a pasted query merged into the value).
            let contains = match &s.body {
                FieldBody::Simple(inner) => semantic_in(inner),
                FieldBody::Parenthesized { inner, .. } => semantic_in(inner),
            };
            if contains {
                return Err("语义检索字段 (R/RAD/RPD) 只能出现一次".to_string());
            }
            Ok((None, Some(s.clone())))
        }
        QueryExpr::Binary(b) => {
            let (left_rest, left_r) = extract_r(&b.left)?;
            let (right_rest, right_r) = extract_r(&b.right)?;
            if left_r.is_some() && right_r.is_some() {
                return Err("语义检索字段 (R/RAD/RPD) 只能出现一次".to_string());
            }
            let r = left_r.or(right_r);
            let rest = match (left_rest, right_rest) {
                (Some(l), Some(rr)) => Some(QueryExpr::Binary(BinaryExpr {
                    left: Box::new(l),
                    op: b.op,
                    op_span: b.op_span,
                    right: Box::new(rr),
                })),
                (Some(l), None) => Some(l),
                (None, Some(rr)) => Some(rr),
                (None, None) => None,
            };
            Ok((rest, r))
        }
        QueryExpr::Group(g) => {
            let (inner_rest, r) = extract_r(&g.inner)?;
            if r.is_some() {
                // Unwrap the group so the semantic expression is at the top level.
                Ok((inner_rest, r))
            } else {
                Ok((Some(QueryExpr::Group(g.clone())), None))
            }
        }
        QueryExpr::Not(n) => {
            if semantic_in(&n.operand) {
                return Err("语义检索字段 (R/RAD/RPD) 不能被 not 修饰".to_string());
            }
            Ok((Some(expr.clone()), None))
        }
        QueryExpr::Field(f) => {
            let contains = match &f.body {
                FieldBody::Simple(inner) => semantic_in(inner),
                FieldBody::Parenthesized { inner, .. } => semantic_in(inner),
            };
            if contains {
                return Err("语义检索字段 (R/RAD/RPD) 不能出现在普通字段的值中".to_string());
            }
            Ok((Some(expr.clone()), None))
        }
        QueryExpr::Proximity(p) => {
            if semantic_in(&p.left) || semantic_in(&p.right) {
                return Err("语义检索字段 (R/RAD/RPD) 不能用于邻近运算符".to_string());
            }
            Ok((Some(expr.clone()), None))
        }
        QueryExpr::Frequency(f) => {
            if semantic_in(&f.operand) {
                return Err("语义检索字段 (R/RAD/RPD) 不能用于频率运算符".to_string());
            }
            Ok((Some(expr.clone()), None))
        }
        QueryExpr::Optional(o) => {
            if semantic_in(&o.left) || semantic_in(&o.right) {
                return Err("语义检索字段 (R/RAD/RPD) 不能与可选运算符 OPT 连用".to_string());
            }
            Ok((Some(expr.clone()), None))
        }
        QueryExpr::TreeAt(t) => {
            if semantic_in(&t.operand) {
                return Err("语义检索字段 (R/RAD/RPD) 不能用于公司树运算符".to_string());
            }
            Ok((Some(expr.clone()), None))
        }
        _ => Ok((Some(expr.clone()), None)),
    }
}

/// Whether a multi-part binary chain needs to be wrapped in parentheses
/// when placed next to the semantic search expression.
fn needs_wrap(expr: &QueryExpr) -> bool {
    let mut parts = Vec::new();
    flat_chain(expr, &mut parts);
    parts.len() >= 2
}

/// Wrap an atomic value fragment in parentheses (e.g. so synonyms can be
/// listed next to it). Compound expressions and self-delimiting ranges keep
/// their own structure.
fn wrap_fragment(expr: QueryExpr) -> QueryExpr {
    match expr {
        QueryExpr::Group(_)
        | QueryExpr::Proximity(_)
        | QueryExpr::Frequency(_)
        | QueryExpr::BracketRange(_)
        | QueryExpr::ComparisonRange(_) => expr,
        other => QueryExpr::Group(GroupExpr {
            lparen_span: Span::new(0, 0),
            inner: Box::new(other),
            rparen_span: Span::new(0, 0),
        }),
    }
}

/// Wrap the operand of a frequency operator. Atomic values get parentheses so
/// synonyms can be listed next to them, and relations (proximity / boolean)
/// keep their own parentheses so `(Nf)` still applies to the whole relation
/// instead of only to the value directly in front of it.
fn wrap_frequency_operand(expr: QueryExpr) -> QueryExpr {
    match expr {
        // Already delimited, or nesting that stays as written.
        QueryExpr::Group(_)
        | QueryExpr::BracketRange(_)
        | QueryExpr::ComparisonRange(_)
        | QueryExpr::Frequency(_) => expr,
        QueryExpr::Proximity(_) | QueryExpr::Binary(_) | QueryExpr::Not(_) => {
            QueryExpr::Group(GroupExpr {
                lparen_span: Span::new(0, 0),
                inner: Box::new(expr),
                rparen_span: Span::new(0, 0),
            })
        }
        other => wrap_fragment(other),
    }
}

/// Ensure `expr` is parenthesized, keeping an existing group as is.
fn ensure_group(expr: QueryExpr) -> QueryExpr {
    match expr {
        QueryExpr::Group(_) => expr,
        other => QueryExpr::Group(GroupExpr {
            lparen_span: Span::new(0, 0),
            inner: Box::new(other),
            rparen_span: Span::new(0, 0),
        }),
    }
}

/// Whether the subtree contains an optional (`OPT`) expression.
fn contains_optional(expr: &QueryExpr) -> bool {
    match expr {
        QueryExpr::Optional(_) => true,
        QueryExpr::Binary(b) => contains_optional(&b.left) || contains_optional(&b.right),
        QueryExpr::Not(n) => contains_optional(&n.operand),
        QueryExpr::Group(g) => contains_optional(&g.inner),
        QueryExpr::Field(f) => match &f.body {
            FieldBody::Simple(inner) => contains_optional(inner),
            FieldBody::Parenthesized { inner, .. } => contains_optional(inner),
        },
        QueryExpr::Proximity(p) => contains_optional(&p.left) || contains_optional(&p.right),
        QueryExpr::Frequency(f) => contains_optional(&f.operand),
        QueryExpr::TreeAt(t) => contains_optional(&t.operand),
        _ => false,
    }
}

/// The elements listed after `OPT` may only be joined by `or`: no `and` /
/// `not`, no proximity or frequency relations, and no nesting.
fn optional_side_violation(expr: &QueryExpr) -> Option<String> {
    const ONLY_OR: &str = "可选运算符 OPT 限定的检索要素之间只能用 or 连接";
    match expr {
        QueryExpr::Optional(_) => Some("可选运算符 OPT 不支持嵌套使用".to_string()),
        QueryExpr::Binary(b) if b.op == BoolOp::And => Some(ONLY_OR.to_string()),
        QueryExpr::Not(_) => Some(ONLY_OR.to_string()),
        QueryExpr::Proximity(_) => Some("可选运算符 OPT 内不支持邻近运算符".to_string()),
        QueryExpr::Frequency(_) => Some("可选运算符 OPT 内不支持频率运算符".to_string()),
        QueryExpr::Binary(b) => {
            optional_side_violation(&b.left).or_else(|| optional_side_violation(&b.right))
        }
        QueryExpr::Group(g) => optional_side_violation(&g.inner),
        QueryExpr::Field(f) => match &f.body {
            FieldBody::Simple(inner) => optional_side_violation(inner),
            FieldBody::Parenthesized { inner, .. } => optional_side_violation(inner),
        },
        _ => None,
    }
}

/// Enforce the optional operator (`OPT`) rules on the whole tree.
fn validate_optional(expr: &QueryExpr) -> Result<(), String> {
    match expr {
        QueryExpr::Optional(o) => {
            if contains_optional(&o.left) || contains_optional(&o.right) {
                return Err("可选运算符 OPT 不支持嵌套使用".to_string());
            }
            if let Some(message) = optional_side_violation(&o.right) {
                return Err(message);
            }
            validate_optional(&o.left)?;
            validate_optional(&o.right)
        }
        QueryExpr::Binary(b) => {
            validate_optional(&b.left)?;
            validate_optional(&b.right)
        }
        QueryExpr::Not(n) => validate_optional(&n.operand),
        QueryExpr::Group(g) => validate_optional(&g.inner),
        QueryExpr::Field(f) => match &f.body {
            FieldBody::Simple(inner) => validate_optional(inner),
            FieldBody::Parenthesized { inner, .. } => validate_optional(inner),
        },
        QueryExpr::Proximity(p) => {
            validate_optional(&p.left)?;
            validate_optional(&p.right)
        }
        QueryExpr::Frequency(f) => validate_optional(&f.operand),
        QueryExpr::TreeAt(t) => validate_optional(&t.operand),
        _ => Ok(()),
    }
}

/// Move a frequency operator that follows a whole field expression into the
/// field's value: `tiabc = (压缩机) (9f)` becomes `tiabc = ((压缩机) (9f))`.
fn move_frequency_into_field(fe: &FieldExpr, op: &str, op_span: Span) -> QueryExpr {
    let body = match &fe.body {
        FieldBody::Simple(inner) | FieldBody::Parenthesized { inner, .. } => {
            FieldBody::Parenthesized {
                lparen_span: Span::new(0, 0),
                inner: Box::new(QueryExpr::Frequency(FrequencyExpr {
                    operand: inner.clone(),
                    op: op.to_string(),
                    op_span,
                })),
                rparen_span: Span::new(0, 0),
            }
        }
    };
    QueryExpr::Field(FieldExpr {
        field_name: fe.field_name.clone(),
        field_span: fe.field_span,
        equals_span: fe.equals_span,
        body,
    })
}

/// Rebuild the query in canonical form: `R and (rest)` or `(rest) and R`.
fn rebuild(
    r: SemanticSearchExpr,
    rest: Option<QueryExpr>,
    pos: Pos,
    connector: Option<(BoolOp, Span)>,
) -> QueryExpr {
    // The connector must be `and`. An existing `and` keeps its span so the
    // configured case can be preserved; anything else (`or`, implicit or
    // missing) is normalized to a plain `and`.
    let (op, op_span) = match connector {
        Some((BoolOp::And, span)) => (BoolOp::And, span),
        _ => (BoolOp::And, Span::new(0, 0)),
    };

    let r_expr = QueryExpr::SemanticSearch(r);
    let Some(rest_expr) = rest else {
        return r_expr;
    };

    let rest_expr = if needs_wrap(&rest_expr) {
        QueryExpr::Group(GroupExpr {
            lparen_span: Span::new(0, 0),
            inner: Box::new(rest_expr),
            rparen_span: Span::new(0, 0),
        })
    } else {
        rest_expr
    };

    match pos {
        Pos::Start => QueryExpr::Binary(BinaryExpr {
            left: Box::new(r_expr),
            op,
            op_span,
            right: Box::new(rest_expr),
        }),
        Pos::End => QueryExpr::Binary(BinaryExpr {
            left: Box::new(rest_expr),
            op,
            op_span,
            right: Box::new(r_expr),
        }),
    }
}

/// Apply the semantic search (R/RAD/RPD) placement rules to a top-level query.
/// Returns the normalized query, or an error describing the violation.
pub fn normalize_query(expr: &QueryExpr) -> Result<QueryExpr, String> {
    // Collapse redundant parentheses and normalize field values first so the
    // semantic rules see a clean tree.
    let collapsed = collapse_parens(expr);
    validate_optional(&collapsed)?;
    let (rest, r) = extract_r(&collapsed)?;
    let Some(r) = r else {
        return Ok(wrap_top_level(collapsed));
    };
    let pos = compute_position(&collapsed)?;
    let connector = parent_connector(&collapsed);
    Ok(rebuild(r, rest, pos, connector))
}

// ── Parenthesis collapsing & field value normalization ──

/// Collapse redundant parentheses and normalize field values.
///
/// - A group around a single atom is removed entirely.
/// - A group around a multi-part chain keeps exactly one layer.
/// - Field values are always parenthesized (with redundant inner group
///   layers removed), except for self-delimiting bracket/comparison ranges.
fn collapse_parens(expr: &QueryExpr) -> QueryExpr {
    match expr {
        QueryExpr::Group(g) => {
            let inner_c = collapse_parens(&g.inner);
            match inner_c {
                QueryExpr::Group(_) => inner_c,
                QueryExpr::Binary(_) => QueryExpr::Group(GroupExpr {
                    lparen_span: g.lparen_span,
                    inner: Box::new(inner_c),
                    rparen_span: g.rparen_span,
                }),
                other => other,
            }
        }
        QueryExpr::Binary(b) => QueryExpr::Binary(BinaryExpr {
            left: Box::new(collapse_parens(&b.left)),
            op: b.op,
            op_span: b.op_span,
            right: Box::new(collapse_parens(&b.right)),
        }),
        QueryExpr::Not(n) => QueryExpr::Not(NotExpr {
            op_span: n.op_span,
            operand: Box::new(collapse_parens(&n.operand)),
        }),
        QueryExpr::Field(f) => QueryExpr::Field(FieldExpr {
            field_name: f.field_name.clone(),
            field_span: f.field_span,
            equals_span: f.equals_span,
            body: normalize_field_body(&f.body),
        }),
        QueryExpr::SemanticSearch(s) => QueryExpr::SemanticSearch(SemanticSearchExpr {
            keyword: s.keyword.clone(),
            keyword_span: s.keyword_span,
            equals_span: s.equals_span,
            body: normalize_field_body(&s.body),
        }),
        QueryExpr::Proximity(p) => {
            let left_c = collapse_parens(&p.left);
            let right_c = collapse_parens(&p.right);
            // 邻近运算符(同句 (s)、同段 (p)、词距 (Nw)/(Nn))前后的值片断都加括号,
            // 既方便并列近义词,也让格式化后两侧片断成块、运算符单独可见。
            let left_c = wrap_fragment(left_c);
            let right_c = wrap_fragment(right_c);
            QueryExpr::Proximity(ProximityExpr {
                left: Box::new(left_c),
                op: p.op.clone(),
                op_span: p.op_span,
                right: Box::new(right_c),
            })
        }
        QueryExpr::Frequency(f) => {
            let operand_c = collapse_parens(&f.operand);
            // 频率运算符 (Nf) 只允许紧跟值,不能跟在完整的"字段名=值"之后。
            // 若它直接跟在字段表达式后(可能带一层括号),将其移入字段值内部。
            if let QueryExpr::Field(fe) = &operand_c {
                return collapse_parens(&move_frequency_into_field(fe, &f.op, f.op_span));
            }
            QueryExpr::Frequency(FrequencyExpr {
                operand: Box::new(wrap_frequency_operand(operand_c)),
                op: f.op.clone(),
                op_span: f.op_span,
            })
        }
        QueryExpr::TreeAt(t) => QueryExpr::TreeAt(TreeAtExpr {
            tree_at_span: t.tree_at_span,
            operand: Box::new(collapse_parens(&t.operand)),
        }),
        QueryExpr::Optional(o) => {
            let left = collapse_parens(&o.left);
            let right = collapse_parens(&o.right);
            // The required condition keeps its parentheses when it lists several
            // elements; the optional elements are always parenthesized.
            let left = if needs_wrap(&left) {
                ensure_group(left)
            } else {
                left
            };
            QueryExpr::Optional(OptionalExpr {
                left: Box::new(left),
                opt_span: o.opt_span,
                right: Box::new(ensure_group(right)),
            })
        }
        other => other.clone(),
    }
}

/// Normalize a field body: the value is always parenthesized and redundant
/// inner group layers are removed. Bracket and comparison ranges are
/// self-delimiting and stay unwrapped.
fn normalize_field_body(body: &FieldBody) -> FieldBody {
    match body {
        FieldBody::Simple(inner) => match inner.as_ref() {
            QueryExpr::BracketRange(_) | QueryExpr::ComparisonRange(_) => {
                FieldBody::Simple(Box::new(collapse_parens(inner)))
            }
            _ => FieldBody::Parenthesized {
                lparen_span: Span::new(0, 0),
                inner: Box::new(collapse_parens(inner)),
                rparen_span: Span::new(0, 0),
            },
        },
        FieldBody::Parenthesized { inner, .. } => {
            let inner_c = collapse_parens(inner);
            // The field body parens already delimit the value, so any inner
            // group layer is redundant.
            let inner_c = match inner_c {
                QueryExpr::Group(g) => *g.inner,
                other => other,
            };
            FieldBody::Parenthesized {
                lparen_span: Span::new(0, 0),
                inner: Box::new(inner_c),
                rparen_span: Span::new(0, 0),
            }
        }
    }
}

/// Wrap a multi-field top-level chain in a single paren layer so the query
/// can be spliced with other query fragments without ambiguity. Single-field
/// queries are left unwrapped.
fn wrap_top_level(expr: QueryExpr) -> QueryExpr {
    let mut parts = Vec::new();
    flat_chain(&expr, &mut parts);
    if parts.len() >= 2 {
        QueryExpr::Group(GroupExpr {
            lparen_span: Span::new(0, 0),
            inner: Box::new(expr),
            rparen_span: Span::new(0, 0),
        })
    } else {
        expr
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::configuration::builder::ConfigurationBuilder;
    use crate::configuration::types::*;
    use crate::format_text::format_text;

    fn format(input: &str) -> String {
        let config = ConfigurationBuilder::new().build();
        format_text(Path::new("test.incopat"), input, &config)
            .unwrap()
            .unwrap_or_else(|| input.to_string())
    }

    fn format_with(input: &str, build: impl FnOnce(&mut ConfigurationBuilder)) -> String {
        let mut builder = ConfigurationBuilder::new();
        build(&mut builder);
        let config = builder.build();
        format_text(Path::new("test.incopat"), input, &config)
            .unwrap()
            .unwrap_or_else(|| input.to_string())
    }

    fn format_err(input: &str) -> String {
        let config = ConfigurationBuilder::new().build();
        format_text(Path::new("test.incopat"), input, &config)
            .err()
            .map(|e| e.to_string())
            .expect("expected an error")
    }

    // ── Legal forms are formatted as-is ──

    #[test]
    fn r_alone() {
        assert_eq!(format("R=(CN101850473B)"), "R = (CN101850473B)\n");
    }

    #[test]
    fn r_at_end_single_field_rest() {
        assert_eq!(
            format("des = (比热容) and R = (一种空调水系统水容量自测工具)"),
            "    des = (比热容)\nand R = (一种空调水系统水容量自测工具)\n"
        );
    }

    #[test]
    fn r_at_start_single_field_rest() {
        assert_eq!(
            format("R = (一种空调水系统水容量自测工具) and des = (比热容)"),
            "    R = (一种空调水系统水容量自测工具)\nand des = (比热容)\n"
        );
    }

    #[test]
    fn r_at_end_wrapped_rest_kept() {
        assert_eq!(
            format(
                "(tiabc = (压缩机 or compressor) and des = (比热容)) and R = (一种空调水系统水容量自测工具)"
            ),
            "(\n        tiabc = (压缩机 or compressor)\n    and des = (比热容)\n)\nand R = (一种空调水系统水容量自测工具)\n"
        );
    }

    #[test]
    fn r_at_start_wrapped_rest_kept() {
        assert_eq!(
            format(
                "R = (一种空调水系统水容量自测工具) and (tiabc = (压缩机 or compressor) and des = (比热容))"
            ),
            "    R = (一种空调水系统水容量自测工具)\nand (\n        tiabc = (压缩机 or compressor)\n    and des = (比热容)\n)\n"
        );
    }

    // ── Automatic fixes ──

    #[test]
    fn missing_parens_added_around_rest() {
        assert_eq!(
            format(
                "tiabc = (压缩机 or compressor) or des = (比热容) and R = (一种空调水系统水容量自测工具)"
            ),
            "(\n        tiabc = (压缩机 or compressor)\n    or  des = (比热容)\n)\nand R = (一种空调水系统水容量自测工具)\n"
        );
    }

    #[test]
    fn missing_parens_added_r_at_start() {
        assert_eq!(
            format(
                "R = (一种空调水系统水容量自测工具) and tiabc = (压缩机 or compressor) and des = (比热容)"
            ),
            "    R = (一种空调水系统水容量自测工具)\nand (\n        tiabc = (压缩机 or compressor)\n    and des = (比热容)\n)\n"
        );
    }

    #[test]
    fn outer_group_unwrapped() {
        assert_eq!(
            format(
                "((tiabc = (压缩机 or compressor) and des = (比热容)) and R = (一种空调水系统水容量自测工具))"
            ),
            "(\n        tiabc = (压缩机 or compressor)\n    and des = (比热容)\n)\nand R = (一种空调水系统水容量自测工具)\n"
        );
    }

    #[test]
    fn semantic_in_group_unwrapped() {
        assert_eq!(format("(R = (CN101850473B))"), "R = (CN101850473B)\n");
    }

    #[test]
    fn or_connector_corrected_at_start() {
        assert_eq!(
            format("R = (一种空调水系统水容量自测工具) or tiabc = (压缩机)"),
            "    R = (一种空调水系统水容量自测工具)\nand tiabc = (压缩机)\n"
        );
    }

    #[test]
    fn or_connector_corrected_at_end() {
        assert_eq!(
            format("tiabc = (压缩机) or R = (一种空调水系统水容量自测工具)"),
            "    tiabc = (压缩机)\nand R = (一种空调水系统水容量自测工具)\n"
        );
    }

    #[test]
    fn implicit_and_becomes_explicit() {
        assert_eq!(
            format("R = (一种空调水系统水容量自测工具) tiabc = (压缩机)"),
            "    R = (一种空调水系统水容量自测工具)\nand tiabc = (压缩机)\n"
        );
    }

    // ── RAD / RPD follow the same rules ──

    #[test]
    fn rad_wrapped_rest() {
        assert_eq!(
            format("RAD = (x) and tiabc = (y) and des = (z)"),
            "    RAD = (x)\nand (\n        tiabc = (y)\n    and des = (z)\n)\n"
        );
    }

    #[test]
    fn rpd_wrapped_rest_at_end() {
        assert_eq!(
            format("tiabc = (y) or des = (z) and RPD = (x)"),
            "(\n        tiabc = (y)\n    or  des = (z)\n)\nand RPD = (x)\n"
        );
    }

    // ── Illegal forms are reported ──

    #[test]
    fn duplicate_semantic_errors() {
        let msg = format_err("R = (a) and R = (b)");
        assert!(msg.contains("只能出现一次"), "unexpected: {}", msg);
    }

    #[test]
    fn r_and_rad_together_error() {
        let msg = format_err("R = (a) and RAD = (b)");
        assert!(msg.contains("只能出现一次"), "unexpected: {}", msg);
    }

    #[test]
    fn r_in_middle_errors() {
        let msg = format_err("tiabc = (a) and R = (b) and des = (c)");
        assert!(msg.contains("开头或结尾"), "unexpected: {}", msg);
    }

    #[test]
    fn r_nested_in_another_r_value_errors() {
        // 一个 R 的值里又嵌入了另一个语义字段(粘贴合并错误)
        let msg = format_err(
            "R = (一种 and (tiabc = (压缩机) and class = (F25B39/02))\nand R = (一种空调水系统水容量自测工具))",
        );
        assert!(msg.contains("只能出现一次"), "unexpected: {}", msg);
    }

    #[test]
    fn r_under_not_errors() {
        let msg = format_err("not R = (a)");
        assert!(msg.contains("not"), "unexpected: {}", msg);
    }

    #[test]
    fn r_in_field_body_errors() {
        let msg = format_err("tiabc = (R = (a))");
        assert!(msg.contains("普通字段"), "unexpected: {}", msg);
    }

    #[test]
    fn r_under_proximity_errors() {
        let msg = format_err("R = (a) (2w) 空调");
        assert!(msg.contains("邻近运算符"), "unexpected: {}", msg);
    }

    // ── Optional operator (OPT) ──

    #[test]
    fn optional_operator_requires_parentheses() {
        assert_eq!(
            format("ti=(毛衣) and pd=[20200101 to 20241231] OPT ab=编织"),
            "(\n        ti = (毛衣)\n    and pd = [20200101 to 20241231]\n)\nOPT (ab = (编织))\n"
        );
        assert_eq!(
            format("ti=(毛衣) OPT (ab=(编织 or 针织))"),
            "ti = (毛衣)\nOPT (ab = (编织 or 针织))\n"
        );
        assert_eq!(
            format("ti=(毛衣) OPT (ab=编织 or 针织)"),
            "ti = (毛衣)\nOPT (\n        ab = (编织)\n    or  针织\n)\n"
        );
    }

    #[test]
    fn optional_operator_with_r_errors() {
        let msg = format_err("R = (缝合) OPT (ab=编织)");
        assert!(msg.contains("不能与可选运算符 OPT 连用"), "unexpected: {}", msg);
    }

    #[test]
    fn optional_side_only_allows_or() {
        let msg = format_err("ti=(毛衣) OPT (ab=(编织 and 针织))");
        assert!(msg.contains("只能用 or 连接"), "unexpected: {}", msg);
        let msg = format_err("ti=(毛衣) OPT (ab=(编织 not 针织))");
        assert!(msg.contains("只能用 or 连接"), "unexpected: {}", msg);
    }

    #[test]
    fn optional_operator_does_not_nest() {
        let msg = format_err("(ti=x) OPT ((ab=y) OPT (cd=z))");
        assert!(msg.contains("不支持嵌套使用"), "unexpected: {}", msg);
    }

    #[test]
    fn optional_side_rejects_relations() {
        let msg = format_err("ti=(毛衣) OPT (ab=(编织 (3n) 针织))");
        assert!(msg.contains("不支持邻近运算符"), "unexpected: {}", msg);
        let msg = format_err("ti=(毛衣) OPT ((ab=(编织)) (3f))");
        assert!(msg.contains("不支持频率运算符"), "unexpected: {}", msg);
    }

    // ── Frequency operator forms ──

    #[test]
    fn frequency_up_to_100_recognized() {
        assert_eq!(format("tiab=(\"机器人\"(100f))"), "tiab = ((\"机器人\") (100f))\n");
    }

    #[test]
    fn frequency_combines_with_boolean_operators() {
        assert_eq!(
            format("tiabc=(\"机器人\"(3f) AND \"遥感\"(3f))"),
            "tiabc = ((\"机器人\") (3f) and (\"遥感\") (3f))\n"
        );
    }

    #[test]
    fn frequency_on_group_and_relation() {
        assert_eq!(
            format("des=((机器人 OR 红外)(3f))"),
            "des = ((机器人 or 红外) (3f))\n"
        );
        assert_eq!(
            format("ti=((clectronic(3n)vehicle)(3f))"),
            "ti = (((clectronic) (3n) (vehicle)) (3f))\n"
        );
    }

    // ── Misc ──

    #[test]
    fn uppercase_connector_preserved_when_configured() {
        let result = format_with("R = (a) AND tiabc = (b)", |b| {
            b.boolean_operator_case(CaseStyle::Preserve);
        });
        assert_eq!(result, "    R = (a)\nAND tiabc = (b)\n");
    }

    // ── Field values are always parenthesized ──

    #[test]
    fn field_value_always_parenthesized() {
        assert_eq!(format("TI=汽车"), "ti = (汽车)\n");
    }

    #[test]
    fn field_value_extra_parens_removed() {
        assert_eq!(format("TI=((压缩机))"), "ti = (压缩机)\n");
    }

    #[test]
    fn field_quoted_value_parenthesized() {
        assert_eq!(format("TI=\"air condition\""), "ti = (\"air condition\")\n");
    }

    #[test]
    fn semantic_value_always_parenthesized() {
        assert_eq!(format("R=CN101850473B"), "R = (CN101850473B)\n");
    }

    #[test]
    fn bracket_range_value_stays_unwrapped() {
        assert_eq!(
            format("pd=[20200101 to 20241231]"),
            "pd = [20200101 to 20241231]\n"
        );
    }

    #[test]
    fn field_body_inner_group_collapsed() {
        assert_eq!(format("ti=((a or b))"), "ti = (a or b)\n");
    }

    // ── Top-level grouping for splicing ──

    #[test]
    fn top_level_multi_field_wrapped() {
        assert_eq!(
            format("ti = (a) and ab = (b)"),
            "(\n        ti = (a)\n    and ab = (b)\n)\n"
        );
    }

    #[test]
    fn top_level_single_field_not_wrapped() {
        assert_eq!(format("ti = (a)"), "ti = (a)\n");
    }

    #[test]
    fn multi_layer_parens_collapsed_to_one() {
        assert_eq!(
            format("((ti = (a) and ab = (b)))"),
            "(\n        ti = (a)\n    and ab = (b)\n)\n"
        );
    }

    #[test]
    fn single_field_all_parens_removed() {
        assert_eq!(format("((ti = (a)))"), "ti = (a)\n");
    }

    #[test]
    fn redundant_parens_in_part_collapsed() {
        assert_eq!(
            format("ti = (a) and ((ab = (b) or ipc = (c)))"),
            "(\n        ti = (a)\n   and  (\n                ab = (b)\n            or  ipc = (c)\n        )\n)\n"
        );
    }

    // ── Semantic search + collapsing ──

    #[test]
    fn semantic_rest_parens_collapsed() {
        assert_eq!(
            format("R = (x) and ((tiabc = (y) or des = (z)))"),
            "    R = (x)\nand (\n        tiabc = (y)\n    or  des = (z)\n)\n"
        );
    }

    #[test]
    fn semantic_single_rest_all_parens_removed() {
        assert_eq!(
            format("R = (x) and ((tiabc = (y)))"),
            "    R = (x)\nand tiabc = (y)\n"
        );
    }

    // ── Proximity (s)/(p) & frequency fragments are parenthesized ──

    #[test]
    fn sentence_op_fragments_parenthesized_in_field() {
        assert_eq!(
            format("des = (温差 (s) (第二 or second))"),
            "des = ((温差) (s) (第二 or second))\n"
        );
    }

    #[test]
    fn sentence_op_fragments_parenthesized_top_level() {
        assert_eq!(format("温差 (s) 蒸发器"), "(温差) (s) (蒸发器)\n");
    }

    #[test]
    fn paragraph_op_fragments_parenthesized() {
        assert_eq!(format("空调 (p) 蒸发器"), "(空调) (p) (蒸发器)\n");
    }

    #[test]
    fn frequency_fragment_parenthesized() {
        assert_eq!(format("机器人 (3f)"), "(机器人) (3f)\n");
    }

    #[test]
    fn frequency_after_field_moved_into_value() {
        assert_eq!(
            format("tiabc = (压缩机 or compressor) (9f)"),
            "tiabc = ((压缩机 or compressor) (9f))\n"
        );
    }

    #[test]
    fn frequency_after_field_simple_value_moved_into_value() {
        assert_eq!(format("ti = 压缩机 (3f)"), "ti = ((压缩机) (3f))\n");
    }

    #[test]
    fn frequency_after_grouped_field_moved_into_value() {
        assert_eq!(
            format("(tiabc = (压缩机)) (9f)"),
            "tiabc = ((压缩机) (9f))\n"
        );
    }

    #[test]
    fn frequency_inside_value_stays_in_value() {
        assert_eq!(format("des = ((比热容) (7f))"), "des = ((比热容) (7f))\n");
    }

    #[test]
    fn r_preserved_when_following_frequency_query() {
        // 回归:格式化产物必须可再次解析且稳定,否则 dprint 重复格式化会丢 R
        let input = "(\n        (tiabc = (压缩机 or compressor) (9f))\n    and des = ((比热容) (7f))\n    and des = ((温差) (s) (第二 or second))\n    and pd = [20010101 to 20260801]\n    and ipc = (F25B39/02)\n    and fi = (F25B39/02)\n    and class = (F25B39/02)\n)\nR = (一种空调水系统水容量检测方法)";
        let config = ConfigurationBuilder::new().build();
        let pass1 = format_text(Path::new("t.incopat"), input, &config)
            .unwrap()
            .unwrap();
        assert!(
            pass1.contains("and R = (一种空调水系统水容量检测方法)"),
            "R 被删除:\n{}",
            pass1
        );
        assert!(pass1.contains("tiabc = ((压缩机 or compressor) (9f))"));
        // 第二次格式化必须无变化
        let pass2 = format_text(Path::new("t.incopat"), &pass1, &config).unwrap();
        assert!(
            pass2.is_none(),
            "输出不稳定:\n{}",
            pass2.unwrap_or_default()
        );
    }

    #[test]
    fn word_op_fragments_wrapped() {
        // 词距运算符 (Nw)/(Nn) 的两侧片断与 (s)/(p) 一样加括号
        assert_eq!(format("data (2w) line"), "(data) (2w) (line)\n");
        assert_eq!(format("data (3n) line"), "(data) (3n) (line)\n");
    }

    #[test]
    fn already_grouped_fragment_not_double_wrapped() {
        // 已是组/复合表达式的操作数不再额外包裹(链中的组按层级规则多行显示)
        assert_eq!(
            format("a (s) (b or c)"),
            "(a) (s) (\n        b\n    or  c\n)\n"
        );
    }
}
