use std::rc::Rc;

use dprint_core::formatting::*;

use crate::configuration::types::*;
use crate::parser::ast::*;
use crate::parser::token::Span;

use super::context::Context;

/// Generate PrintItems IR from a parsed File AST.
pub fn generate(file: &File, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();

    for stmt in file.statements.iter() {
        match stmt {
            Statement::Query(expr) => {
                items.extend(gen_query(expr, ctx));
                items.push_signal(Signal::NewLine);
            }
            Statement::Comment(comment) => {
                items.extend(gen_comment(comment));
                items.push_signal(Signal::NewLine);
            }
            Statement::BlankLine => {
                items.push_signal(Signal::NewLine);
            }
        }
    }

    items
}

// ── Top-level query ──

/// Generate IR for a top-level query expression.
/// Binary chains are always broken into multiple lines.
fn gen_query(expr: &QueryExpr, ctx: &Context) -> PrintItems {
    match expr {
        QueryExpr::Binary(_) => gen_top_level_binary(expr, ctx),
        QueryExpr::Optional(o) => gen_optional(o, ctx),
        QueryExpr::Not(n) => {
            let mut items = PrintItems::new();
            items.push_string(format_not_op(&n.op_span, ctx));
            items.push_string(" ".into());
            items.extend(gen_expr(&n.operand, ctx));
            items
        }
        _ => gen_expr(expr, ctx),
    }
}

/// Generate IR for a top-level optional expression: the required condition
/// keeps its own layout, then `OPT` introduces the parenthesized optional
/// elements on their own lines.
///
/// ```text
/// (
///         ti = (毛衣)
///     and pd = [20100101 to 20250315]
/// )
/// OPT (
///         ab = (编织)
/// )
/// ```
fn gen_optional(expr: &OptionalExpr, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();
    items.extend(gen_query(&expr.left, ctx));
    items.push_signal(Signal::NewLine);
    items.push_string(format_opt_op(&expr.opt_span, ctx));
    items.push_string(" ".into());
    items.extend(gen_query(&expr.right, ctx));
    items
}

// ── Flat part for binary chain flattening ──

struct FlatPart<'a> {
    /// Operator text (None for the first operand).
    op: Option<String>,
    /// The operand expression.
    expr: &'a QueryExpr,
}

/// Recursively flatten a binary expression tree into a linear list of
/// (operator, operand) pairs for formatting.
fn flatten_binary_chain<'a>(expr: &'a QueryExpr, parts: &mut Vec<FlatPart<'a>>, ctx: &Context) {
    match expr {
        QueryExpr::Binary(b) => {
            // Flatten left subtree first
            flatten_binary_chain(&b.left, parts, ctx);

            // Handle implicit AND + NOT → display as "not"
            if b.op == BoolOp::And && b.op_span.start == b.op_span.end {
                if let QueryExpr::Not(not_expr) = b.right.as_ref() {
                    parts.push(FlatPart {
                        op: Some(format_not_op(&not_expr.op_span, ctx)),
                        expr: &not_expr.operand,
                    });
                    return;
                }
            }

            let op_text = format_bool_op(b.op, &b.op_span, ctx);

            // Flatten right subtree and attach our operator to its first element
            let right_start = parts.len();
            flatten_binary_chain(&b.right, parts, ctx);
            if right_start < parts.len() && parts[right_start].op.is_none() {
                parts[right_start].op = Some(op_text);
            }
        }
        QueryExpr::Not(n) => {
            // Standalone NOT at the beginning of a chain
            parts.push(FlatPart {
                op: Some(format_not_op(&n.op_span, ctx)),
                expr: &n.operand,
            });
        }
        other => {
            parts.push(FlatPart {
                op: None,
                expr: other,
            });
        }
    }
}

/// Generate IR for a top-level binary chain with forced line breaks
/// and operator-aligned formatting.
///
/// Output style:
///     first_operand
/// and second_operand
///  or third_operand
///
/// All operands are left-aligned; operators are right-justified within
/// `align_width` characters (= max operator length + 1).
fn gen_top_level_binary(expr: &QueryExpr, ctx: &Context) -> PrintItems {
    let mut parts = Vec::new();
    flatten_binary_chain(expr, &mut parts, ctx);

    if parts.is_empty() {
        return PrintItems::new();
    }
    if parts.len() == 1 {
        return gen_single_part(&parts[0], ctx);
    }

    let align_width = calc_align_width(&parts);
    let mut items = PrintItems::new();

    // First operand. Groups start at the line start without alignment
    // padding so the opening paren hugs the left margin.
    if matches!(parts[0].expr, QueryExpr::Group(_)) {
        items.extend(gen_expr(parts[0].expr, ctx));
    } else {
        emit_aligned_part(&parts[0], align_width, &mut items, ctx);
    }

    // Continuation lines with forced breaks
    for part in &parts[1..] {
        items.push_signal(Signal::NewLine);
        emit_aligned_part(part, align_width, &mut items, ctx);
    }

    items
}

/// Calculate the alignment width for a binary chain.
/// align_width = max(op.len() for all ops) + 1 (for trailing space).
fn calc_align_width(parts: &[FlatPart]) -> usize {
    let max_op_len = parts
        .iter()
        .filter_map(|p| p.op.as_ref())
        .map(|op| op.len())
        .max()
        .unwrap_or(0);
    max_op_len + 1
}

/// Emit a single FlatPart (when the chain has only one element).
fn gen_single_part(part: &FlatPart, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();
    if let Some(ref op) = part.op {
        items.push_string(op.clone());
        items.push_string(" ".into());
    }
    items.extend(gen_expr(part.expr, ctx));
    items
}

/// Emit a part with right-justified operator alignment.
/// If the part has an op, right-justify it within `align_width` chars.
/// If the part has no op (first operand), emit `align_width` spaces as padding.
fn emit_aligned_part(part: &FlatPart, align_width: usize, items: &mut PrintItems, ctx: &Context) {
    if let Some(ref op) = part.op {
        let pad = align_width.saturating_sub(op.len() + 1);
        if pad > 0 {
            items.push_string(" ".repeat(pad));
        }
        items.push_string(op.clone());
        items.push_string(" ".into());
    } else {
        items.push_string(" ".repeat(align_width));
    }
    items.extend(gen_expr(part.expr, ctx));
}

// ── Inner expression generation ──

/// Generate IR for any query expression (inner level).
fn gen_expr(expr: &QueryExpr, ctx: &Context) -> PrintItems {
    match expr {
        QueryExpr::Binary(_) => {
            if ctx.in_field_body {
                gen_inner_binary_chain(expr, ctx)
            } else {
                gen_top_level_binary(expr, ctx)
            }
        }
        QueryExpr::Not(n) => gen_not(n, ctx),
        QueryExpr::Field(f) => gen_field(f, ctx),
        QueryExpr::Group(g) => gen_group(g, ctx),
        QueryExpr::Keyword(k) => gen_keyword(k),
        QueryExpr::Quoted(q) => gen_quoted(q, ctx),
        QueryExpr::BracketRange(r) => gen_bracket_range(r),
        QueryExpr::ComparisonRange(c) => gen_comparison_range(c, ctx),
        QueryExpr::Proximity(p) => gen_proximity(p, ctx),
        QueryExpr::Frequency(f) => gen_frequency(f, ctx),
        QueryExpr::Optional(o) => {
            // Nested optional expressions are rejected by the normalizer; keep
            // a plain inline rendering for safety.
            let mut items = PrintItems::new();
            items.extend(gen_expr(&o.left, ctx));
            items.push_string(" ".into());
            items.push_string(format_opt_op(&o.opt_span, ctx));
            items.push_string(" ".into());
            items.extend(gen_expr(&o.right, ctx));
            items
        }
        QueryExpr::TreeAt(t) => gen_tree_at(t, ctx),
        QueryExpr::SemanticSearch(s) => gen_semantic_search(s, ctx),
        QueryExpr::Error(e) => gen_error(e),
    }
}

/// Generate IR for an inner binary chain using adaptive line breaking.
/// When the chain fits on one line: `a or b or c`
/// When it breaks, operators are right-justified at the indent position:
///    a
/// or b
/// or c
fn gen_inner_binary_chain(expr: &QueryExpr, ctx: &Context) -> PrintItems {
    let mut parts = Vec::new();
    flatten_binary_chain(expr, &mut parts, ctx);

    if parts.is_empty() {
        return PrintItems::new();
    }
    if parts.len() == 1 {
        return gen_single_part(&parts[0], ctx);
    }

    let align_width = calc_align_width(&parts);
    // Wrapped field-value chains are indented 4 columns past the column the
    // value's closing paren sits in: the field's own start column, or the
    // column of the enclosing wrapped block inside a value. Operands start
    // OPERAND_OFFSET columns further right.
    let continuation_indent = match ctx.block_close_col {
        Some(close_col) => close_col + 4,
        None => 8 * ctx.depth + 4,
    };
    let content_col = continuation_indent + OPERAND_OFFSET;
    let mut items = PrintItems::new();

    // First operand — add alignment padding only when already at start of line
    {
        let pad_str: PrintItems = " ".repeat(align_width).into();
        items.push_condition(conditions::if_true(
            "firstAlignPad",
            condition_resolvers::is_start_of_line(),
            pad_str,
        ));
    }
    if let Some(ref op) = parts[0].op {
        items.push_string(op.clone());
        items.push_string(" ".into());
    }
    items.extend(gen_chain_operand(parts[0].expr, content_col, ctx));

    // Continuation parts with adaptive SpaceOrNewLine
    for part in &parts[1..] {
        items.push_signal(Signal::SpaceOrNewLine);
        if let Some(ref op) = part.op {
            // When at start of line (broke): the operand behind the operator
            // always starts in the content column, so keywords stay aligned.
            // Operators narrower than that offset are followed by padding;
            // wider ones (`(sen)`, `(57w)`) shift left instead.
            let layout = continuation_op_layout(op);
            let mut true_path = PrintItems::new();
            true_path.push_string(" ".repeat(continuation_indent.saturating_sub(layout.shift_left)));
            true_path.push_string(op.clone());
            true_path.push_string(" ".repeat(layout.spaces_after));

            // When inline (no break): just op + space
            let mut false_path = PrintItems::new();
            false_path.push_string(op.clone());
            false_path.push_string(" ".into());

            items.push_condition(Condition::new(
                "opAlign",
                ConditionProperties {
                    condition: Rc::new(|context| Some(context.writer_info.is_start_of_line())),
                    true_path: Some(true_path),
                    false_path: Some(false_path),
                },
            ));
        }
        items.extend(gen_chain_operand(part.expr, content_col, ctx));
    }

    items
}

/// Render one operand of an inner binary chain. A proximity expression is
/// wrapped in parentheses so the relation it forms stays visible as one block
/// (`((a or b) (3n)(c or d))`), everything else renders as usual.
fn gen_chain_operand(expr: &QueryExpr, content_col: usize, ctx: &Context) -> PrintItems {
    match expr {
        QueryExpr::Proximity(_) => gen_block(expr, content_col, ctx),
        other => gen_expr(other, ctx),
    }
}

/// Columns between the continuation-operator column of a wrapped chain and the
/// column its operands start in.
const OPERAND_OFFSET: usize = 4;

/// How a continuation operator is laid out on a line it starts.
struct ContinuationOpLayout {
    /// Columns the operator starts to the left of the continuation column.
    shift_left: usize,
    /// Spaces between the operator and the operand behind it.
    spaces_after: usize,
}

/// Lay out a continuation operator so that the operand behind it always starts
/// in the content column (`OPERAND_OFFSET` columns past the continuation
/// column) — keywords stay aligned even for wide operators.
///
/// Operators narrower than the offset (`or`, `and`, `(s)`) start in the
/// continuation column and are padded (`or` → 2 spaces, `and` → 1); operators
/// at least as wide as the offset (`(3w)`, `(sen)`, `(99n)`) are written right
/// against the following operand, and ones wider than the offset shift left so
/// that the parentheses around the connected fragments still line up.
fn continuation_op_layout(op: &str) -> ContinuationOpLayout {
    let width = op.chars().count();
    ContinuationOpLayout {
        shift_left: width.saturating_sub(OPERAND_OFFSET),
        spaces_after: OPERAND_OFFSET.saturating_sub(width),
    }
}

fn gen_not(expr: &NotExpr, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();
    items.push_string(format_not_op(&expr.op_span, ctx));
    items.push_string(" ".into());
    // A negated proximity expression keeps its own parentheses.
    if ctx.in_field_body {
        if let QueryExpr::Proximity(_) = expr.operand.as_ref() {
            items.extend(gen_block(&expr.operand, 8 * ctx.depth + 8, ctx));
            return items;
        }
    }
    items.extend(gen_expr(&expr.operand, ctx));
    items
}

fn gen_field(expr: &FieldExpr, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();

    let field_name = format_case(&expr.field_name, ctx.config.field_case);
    items.push_string(field_name);
    items.push_string(" = ".into());

    match &expr.body {
        FieldBody::Simple(inner) => {
            items.extend(gen_expr(inner, ctx));
        }
        FieldBody::Parenthesized { inner, .. } => {
            let body_ctx = ctx.with_field_body();
            // Field values use a hanging indent: the first content line sits
            // 8 columns past the closing-paren column (same spacing as group
            // fields), the continuation lines sit 4 columns past it, and the
            // closing paren aligns with the field start.
            let body_indent = 8 * ctx.depth + 8;
            let close_indent = 8 * ctx.depth;

            items.push_string("(".into());
            // Break right after the opening paren when the value doesn't fit
            // on the current line. Unlike SpaceOrNewLine this emits nothing
            // when the value stays on one line.
            items.push_signal(Signal::PossibleNewLine);
            // The body gets its own new-line group so the inner `and`/`or`
            // separators cannot override the break point after `(` — when the
            // value is too wide the printer first breaks after `(`, then the
            // chain breaks internally at the separators.
            items.push_signal(Signal::StartNewLineGroup);

            let mut multiline_condition = Condition::new(
                "multilineFieldBody",
                ConditionProperties {
                    condition: condition_resolvers::is_start_of_line(),
                    true_path: Some(" ".repeat(body_indent).into()),
                    false_path: None,
                },
            );
            let multiline_reference = multiline_condition.create_reference();
            items.push_condition(multiline_condition);
            items.extend(gen_expr(inner, &body_ctx));
            items.push_signal(Signal::FinishNewLineGroup);

            let mut closing_path = PrintItems::new();
            closing_path.push_signal(Signal::NewLine);
            if close_indent > 0 {
                closing_path.push_string(" ".repeat(close_indent));
            }
            closing_path.push_string(")".into());
            items.push_condition(Condition::new(
                "multilineFieldBodyClosingParen",
                ConditionProperties {
                    condition: multiline_reference.create_resolver(),
                    true_path: Some(closing_path),
                    false_path: Some(")".into()),
                },
            ));
        }
    }

    items
}

fn gen_group(expr: &GroupExpr, ctx: &Context) -> PrintItems {
    // A parenthesized proximity expression inside a field value is rendered as
    // an indented block, the same way the generator wraps a bare one.
    if ctx.in_field_body {
        if let QueryExpr::Proximity(_) = expr.inner.as_ref() {
            return gen_block(&expr.inner, 8 * ctx.depth + 8, ctx);
        }
    }

    let mut items = PrintItems::new();
    items.push_string("(".into());

    // Groups inside field values keep the adaptive inline style.
    if ctx.in_field_body {
        let inner_ctx = ctx.with_field_body();
        items.extend(gen_expr(&expr.inner, &inner_ctx));
        items.push_string(")".into());
        return items;
    }

    let mut parts = Vec::new();
    flatten_binary_chain(&expr.inner, &mut parts, ctx);

    // Single-part groups stay on one line.
    if parts.len() <= 1 {
        let inner_ctx = ctx.with_group();
        items.extend(gen_expr(&expr.inner, &inner_ctx));
        items.push_string(")".into());
        return items;
    }

    // Multi-part chains inside a group render one field per line, indented
    // 8 characters per nesting level so the fields align with the fields
    // following the and/or/not connectors.
    let indent_col = 8 * (ctx.depth + 1);
    let group_ctx = ctx.with_group();

    items.push_signal(Signal::NewLine);

    // First field, padded to the indent column.
    match parts[0].op.clone() {
        Some(op) => {
            let pad = indent_col.saturating_sub(op.len() + 1);
            items.push_string(" ".repeat(pad));
            items.push_string(op);
            items.push_string(" ".into());
        }
        None => {
            items.push_string(" ".repeat(indent_col));
        }
    }
    items.extend(gen_expr(parts[0].expr, &group_ctx));

    // Continuation fields with right-justified connectors.
    for part in &parts[1..] {
        items.push_signal(Signal::NewLine);
        let op = part.op.clone().unwrap_or_else(|| "and".to_string());
        let pad = indent_col.saturating_sub(op.len() + 1);
        items.push_string(" ".repeat(pad));
        items.push_string(op);
        items.push_string(" ".into());
        items.extend(gen_expr(part.expr, &group_ctx));
    }

    items.push_signal(Signal::NewLine);
    // The closing paren aligns with the opening paren: 8 chars per level.
    let close_col = 8 * ctx.depth;
    if close_col > 0 {
        items.push_string(" ".repeat(close_col));
    }
    items.push_string(")".into());
    items
}

fn gen_keyword(term: &KeywordTerm) -> PrintItems {
    let mut items = PrintItems::new();
    items.push_string(term.value.clone());
    items
}

fn gen_quoted(term: &QuotedTerm, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();
    let quote = match ctx.config.quote_style {
        QuoteStyle::Double => '"',
        QuoteStyle::Single => '\'',
        QuoteStyle::Preserve => term.quote_char,
    };
    items.push_string(format!("{}{}{}", quote, term.value, quote));
    items
}

fn gen_bracket_range(expr: &BracketRangeExpr) -> PrintItems {
    let mut items = PrintItems::new();
    items.push_string(format!("[{} to {}]", expr.from, expr.to));
    items
}

/// Generate IR for a comparison range expression.
/// Standardizes to format: (value<=field<=value), (field>value), etc.
/// Field names are formatted according to the field case config.
fn gen_comparison_range(expr: &ComparisonRangeExpr, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();
    items.push_string("(".into());

    let field_name = format_case(&expr.field_name, ctx.config.field_case);

    match (&expr.left_value, &expr.right_value) {
        (Some(left_val), Some(right_val)) => {
            // Double-sided: (value op field op value)
            let left_op = expr.left_op.as_ref().unwrap();
            let right_op = expr.right_op.as_ref().unwrap();
            items.push_string(left_val.clone());
            items.push_string(left_op.as_str().into());
            items.push_string(field_name);
            items.push_string(right_op.as_str().into());
            items.push_string(right_val.clone());
        }
        (Some(left_val), None) => {
            // Left-sided: (value op field)
            let left_op = expr.left_op.as_ref().unwrap();
            items.push_string(left_val.clone());
            items.push_string(left_op.as_str().into());
            items.push_string(field_name);
        }
        (None, Some(right_val)) => {
            // Right-sided: (field op value)
            let right_op = expr.right_op.as_ref().unwrap();
            items.push_string(field_name);
            items.push_string(right_op.as_str().into());
            items.push_string(right_val.clone());
        }
        (None, None) => {
            // Shouldn't happen, but output field name
            items.push_string(field_name);
        }
    }

    items.push_string(")".into());
    items
}

fn gen_proximity(expr: &ProximityExpr, ctx: &Context) -> PrintItems {
    // Inside a field value the fragments on both sides of a proximity operator
    // are rendered as parenthesized blocks so the operator stays visible when
    // the value wraps.
    if ctx.in_field_body {
        return gen_proximity_chain(expr, ctx);
    }

    let mut items = PrintItems::new();
    items.extend(gen_expr(&expr.left, ctx));
    items.push_string(" ".into());
    items.push_string(expr.op.clone());
    items.push_string(" ".into());
    items.extend(gen_expr(&expr.right, ctx));
    items
}

/// Generate IR for a proximity expression (`(s)`, `(p)`, `(Nw)`, `(Nn)`) inside
/// a field value.
///
/// The two fragments are parenthesized blocks sitting in the field value's
/// content column. They stay on one line with the operator when everything
/// fits (`(fragment) (s) (fragment)`). Otherwise the operator starts its own
/// line, in the continuation column, so both blocks' parentheses line up:
///
/// ```text
/// (
///         fragment ...
///     or continuation ...
/// )
/// (s) (
///         fragment ...
/// )
/// ```
fn gen_proximity_chain(expr: &ProximityExpr, ctx: &Context) -> PrintItems {
    // The fragments sit in the content column of the field value, or of the
    // enclosing wrapped block (when the whole relation is itself parenthesized).
    let block_col = match ctx.block_close_col {
        Some(close_col) => close_col + 8,
        None => 8 * ctx.depth + 8,
    };
    let continuation_indent = block_col - OPERAND_OFFSET;
    let mut items = PrintItems::new();

    // Remember the line the first fragment starts on so a wrapped first
    // fragment can push the operator onto its own line.
    let start_line = LineNumber::new("proximityStartLine");
    items.push_info(Info::LineNumber(start_line));
    items.extend(gen_operand_block(&expr.left, block_col, ctx));
    let mid_line = LineNumber::new("proximityMidLine");
    items.push_info(Info::LineNumber(mid_line));

    let mut break_path = PrintItems::new();
    break_path.push_signal(Signal::NewLine);
    items.push_condition(Condition::new(
        "proximityOpBreak",
        ConditionProperties {
            condition: Rc::new(move |context| {
                let start = context.resolved_line_number(start_line)?;
                let mid = context.resolved_line_number(mid_line)?;
                Some(mid > start)
            }),
            true_path: Some(break_path),
            // Not wrapped yet: leave a break point behind (with the usual
            // separating space) so a wrapped second fragment also moves the
            // operator onto its own line.
            false_path: Some(Signal::SpaceOrNewLine.into()),
        },
    ));

    // The operator itself: on a line it starts it is laid out so the second
    // block begins in the same column as the first one, whatever the width of
    // the operator (`(s)`, `(3w)`, `(sen)`, ...).
    let layout = continuation_op_layout(&expr.op);
    let mut op_on_new_line = PrintItems::new();
    op_on_new_line.push_string(" ".repeat(continuation_indent.saturating_sub(layout.shift_left)));
    op_on_new_line.push_string(expr.op.clone());
    op_on_new_line.push_string(" ".repeat(layout.spaces_after));
    let mut op_inline = PrintItems::new();
    op_inline.push_string(expr.op.clone());
    op_inline.push_string(" ".into());
    items.push_condition(Condition::new(
        "proximityOpAlign",
        ConditionProperties {
            condition: Rc::new(|context| Some(context.writer_info.is_start_of_line())),
            true_path: Some(op_on_new_line),
            false_path: Some(op_inline),
        },
    ));

    items.extend(gen_operand_block(&expr.right, block_col, ctx));
    items
}

/// Render one side of a `(s)` / `(p)` proximity expression as a parenthesized
/// block sitting in `block_col`.
fn gen_operand_block(operand: &QueryExpr, block_col: usize, ctx: &Context) -> PrintItems {
    let inner = match operand {
        // The normalizer already wrapped value fragments; render the block
        // around the fragment itself instead of nesting another pair of parens.
        QueryExpr::Group(group) => group.inner.as_ref(),
        // Self-delimiting ranges keep their own delimiters.
        QueryExpr::BracketRange(_) | QueryExpr::ComparisonRange(_) => return gen_expr(operand, ctx),
        other => other,
    };
    gen_block(inner, block_col, ctx)
}

/// Render `inner` as a parenthesized block whose parens hug the content while
/// it fits (`(content)`) and move onto their own lines when it wraps:
///
/// ```text
/// (
///         content ...
///     or continuation ...
/// )
/// ```
///
/// `open_col` is the column the opening paren sits in; the closing paren is
/// aligned with it and the content is indented relative to it.
fn gen_block(inner: &QueryExpr, open_col: usize, ctx: &Context) -> PrintItems {
    let body_indent = open_col + 8;
    let body_ctx = ctx.with_block(open_col);
    let mut items = PrintItems::new();

    // The block is wrapped in two nested new line groups: the outer one keeps
    // the block's break point (right after the opening paren) shallower than
    // the content's, the inner one keeps it deeper than an enclosing `(s)`
    // operator's break point. Wrapping content therefore breaks at the
    // operator first, then at the paren, then inside the fragments.
    items.push_signal(Signal::StartNewLineGroup);
    items.push_string("(".into());
    items.push_signal(Signal::PossibleNewLine);
    items.push_signal(Signal::StartNewLineGroup);

    let mut multiline_condition = Condition::new(
        "multilineBlockBody",
        ConditionProperties {
            condition: condition_resolvers::is_start_of_line(),
            true_path: Some(" ".repeat(body_indent).into()),
            false_path: None,
        },
    );
    let multiline_reference = multiline_condition.create_reference();
    items.push_condition(multiline_condition);
    items.extend(gen_expr(inner, &body_ctx));
    items.push_signal(Signal::FinishNewLineGroup);
    items.push_signal(Signal::FinishNewLineGroup);

    let mut closing_path = PrintItems::new();
    closing_path.push_signal(Signal::NewLine);
    if open_col > 0 {
        closing_path.push_string(" ".repeat(open_col));
    }
    closing_path.push_string(")".into());
    items.push_condition(Condition::new(
        "multilineBlockClosingParen",
        ConditionProperties {
            condition: multiline_reference.create_resolver(),
            true_path: Some(closing_path),
            false_path: Some(")".into()),
        },
    ));
    items
}

fn gen_frequency(expr: &FrequencyExpr, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();
    items.extend(gen_expr(&expr.operand, ctx));
    items.push_string(" ".into());
    items.push_string(expr.op.clone());
    items
}

fn gen_tree_at(expr: &TreeAtExpr, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();
    items.push_string("TREE@".into());
    items.extend(gen_expr(&expr.operand, ctx));
    items
}

/// Generate IR for a semantic search expression (R, RAD, RPD).
/// The keyword is always output in UPPERCASE.
fn gen_semantic_search(expr: &SemanticSearchExpr, ctx: &Context) -> PrintItems {
    let mut items = PrintItems::new();

    // Semantic keywords are always uppercase
    items.push_string(expr.keyword.to_uppercase());
    items.push_string(" = ".into());

    match &expr.body {
        FieldBody::Simple(inner) => {
            items.extend(gen_expr(inner, ctx));
        }
        FieldBody::Parenthesized { inner, .. } => {
            items.push_string("(".into());
            let body_ctx = ctx.with_field_body();
            items.extend(gen_expr(inner, &body_ctx));
            items.push_string(")".into());
        }
    }

    items
}

fn gen_comment(comment: &Comment) -> PrintItems {
    let mut items = PrintItems::new();
    items.push_string(comment.text.clone());
    items
}

fn gen_error(error: &ErrorNode) -> PrintItems {
    let mut items = PrintItems::new();
    items.push_string(error.raw_text.clone());
    items
}

// ── Formatting helpers ──

fn format_bool_op(op: BoolOp, op_span: &Span, ctx: &Context) -> String {
    if ctx.config.boolean_operator_case == CaseStyle::Preserve && op_span.len() > 0 {
        return ctx.source[op_span.start..op_span.end].to_string();
    }
    let text = match op {
        BoolOp::And => "and",
        BoolOp::Or => "or",
    };
    format_case(text, ctx.config.boolean_operator_case)
}

/// Preserve the case of the `OPT` keyword as written (`opt`/`OPT`).
fn format_opt_op(opt_span: &Span, ctx: &Context) -> String {
    if opt_span.len() > 0 {
        return ctx.source[opt_span.start..opt_span.end].to_string();
    }
    "OPT".to_string()
}

fn format_not_op(not_span: &Span, ctx: &Context) -> String {
    if ctx.config.boolean_operator_case == CaseStyle::Preserve && not_span.len() > 0 {
        return ctx.source[not_span.start..not_span.end].to_string();
    }
    format_case("not", ctx.config.boolean_operator_case)
}

fn format_case(text: &str, case: CaseStyle) -> String {
    match case {
        CaseStyle::Lowercase => text.to_lowercase(),
        CaseStyle::Uppercase => text.to_uppercase(),
        CaseStyle::Preserve => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::configuration::builder::ConfigurationBuilder;
    use crate::configuration::types::*;
    use crate::format_text::format_text;
    use std::path::Path;

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

    // ── Basic formatting ──

    #[test]
    fn simple_keyword() {
        assert_eq!(format("汽车"), "汽车\n");
    }

    #[test]
    fn field_lowercase() {
        assert_eq!(format("TI=汽车"), "ti = (汽车)\n");
    }

    #[test]
    fn field_uppercase() {
        let result = format_with("ti=汽车", |b| {
            b.field_case(CaseStyle::Uppercase);
        });
        assert_eq!(result, "TI = (汽车)\n");
    }

    #[test]
    fn field_with_parenthesized_body() {
        assert_eq!(format("TI=(空调 or 蒸发器)"), "ti = (空调 or 蒸发器)\n");
    }

    // ── Top-level binary breaking ──

    #[test]
    fn top_level_and_breaks() {
        assert_eq!(
            format("TI=空调 and AB=蒸发器"),
            "(\n        ti = (空调)\n    and ab = (蒸发器)\n)\n"
        );
    }

    #[test]
    fn top_level_or_breaks() {
        assert_eq!(
            format("TI=空调 or AB=蒸发器"),
            "(\n        ti = (空调)\n     or ab = (蒸发器)\n)\n"
        );
    }

    #[test]
    fn top_level_chain_three_parts() {
        assert_eq!(
            format("TI=a AND AB=b OR IPC=c"),
            "(\n        ti = (a)\n    and ab = (b)\n     or ipc = (c)\n)\n"
        );
    }

    #[test]
    fn top_level_not_in_chain() {
        assert_eq!(
            format("TI=a not AB=b"),
            "(\n        ti = (a)\n    not ab = (b)\n)\n"
        );
    }

    // ── Boolean operator case ──

    #[test]
    fn boolean_operator_uppercase() {
        let result = format_with("TI=空调 and AB=蒸发器", |b| {
            b.boolean_operator_case(CaseStyle::Uppercase);
        });
        assert_eq!(result, "(\n        ti = (空调)\n    AND ab = (蒸发器)\n)\n");
    }

    // ── Inner binary (adaptive breaking) ──

    #[test]
    fn inner_or_chain_fits_single_line() {
        assert_eq!(format("ti=(空调 or 蒸发器)"), "ti = (空调 or 蒸发器)\n");
    }

    // ── Bracket range expression ──

    #[test]
    fn bracket_range_expression() {
        assert_eq!(
            format("pd=[20200101 to 20241231]"),
            "pd = [20200101 to 20241231]\n"
        );
    }

    // ── Comparison range expression ──

    #[test]
    fn comparison_range_double() {
        assert_eq!(
            format("(20110101<=PD<=20130101)"),
            "(20110101<=pd<=20130101)\n"
        );
    }

    #[test]
    fn comparison_range_single() {
        assert_eq!(format("(PD>20190101)"), "(pd>20190101)\n");
    }

    // ── Quoted strings ──

    #[test]
    fn quoted_string_preserve() {
        assert_eq!(format("TI=\"air condition\""), "ti = (\"air condition\")\n");
    }

    #[test]
    fn quoted_string_single() {
        let result = format_with("TI=\"air condition\"", |b| {
            b.quote_style(QuoteStyle::Single);
        });
        assert_eq!(result, "ti = ('air condition')\n");
    }

    // ── Proximity & frequency ──

    #[test]
    fn proximity_operator() {
        assert_eq!(format("data (2w) line"), "(data) (2w) (line)\n");
    }

    #[test]
    fn frequency_operator() {
        assert_eq!(
            format("tiab=(\"机器人\" (3f))"),
            "tiab = ((\"机器人\") (3f))\n"
        );
    }

    // ── TREE@ ──

    #[test]
    fn tree_at_expression() {
        assert_eq!(
            format("ap=(TREE@\"清华大学\")"),
            "ap = (TREE@\"清华大学\")\n"
        );
    }

    // ── Semantic search ──

    #[test]
    fn semantic_search_r() {
        assert_eq!(format("R=(CN101850473B)"), "R = (CN101850473B)\n");
    }

    #[test]
    fn semantic_search_rad_with_and() {
        assert_eq!(
            format("RAD=(CN101850473B) AND tiab=(发动机)"),
            "    RAD = (CN101850473B)\nand tiab = (发动机)\n"
        );
    }

    // ── Comments & blank lines ──

    #[test]
    fn comment_preserved() {
        assert_eq!(
            format("# this is a comment\nti=test"),
            "# this is a comment\nti = (test)\n"
        );
    }

    #[test]
    fn blank_line_between_queries() {
        assert_eq!(format("ti=a\n\nti=b"), "ti = (a)\n\nti = (b)\n");
    }

    // ── Complex real-world query ──

    #[test]
    fn complex_query() {
        let input = "ti=(空调 or \"air condition\" or 空气调节) and tiab=(蒸发器 or evaporator)";
        let result = format(input);
        assert_eq!(
            result,
            "(\n        ti = (空调 or \"air condition\" or 空气调节)\n    and tiab = (蒸发器 or evaporator)\n)\n"
        );
    }

    // ── Idempotency ──

    #[test]
    fn already_formatted_returns_none() {
        let formatted = "(\n        ti = (空调)\n    and ab = (蒸发器)\n)\n";
        let config = ConfigurationBuilder::new().build();
        let result = format_text(Path::new("test.incopat"), formatted, &config).unwrap();
        assert!(
            result.is_none(),
            "already formatted text should return None"
        );
    }
}
