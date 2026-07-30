use super::token::Span;

/// A complete `.incopat` file: a sequence of statements.
#[derive(Debug, Clone)]
pub struct File {
    pub statements: Vec<Statement>,
}

/// A top-level statement in the file.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A search query expression.
    Query(QueryExpr),
    /// A standalone comment line.
    Comment(Comment),
    /// A blank line separator.
    BlankLine,
}

/// A comment node.
#[derive(Debug, Clone)]
pub struct Comment {
    pub text: String,
    pub span: Span,
}

/// A query expression node.
#[derive(Debug, Clone)]
pub enum QueryExpr {
    /// Binary expression: left OP right (AND, OR).
    Binary(BinaryExpr),
    /// Unary NOT expression.
    Not(NotExpr),
    /// Field expression: FIELD=body or FIELD=(body).
    Field(FieldExpr),
    /// Parenthesized group: (expr).
    Group(GroupExpr),
    /// A bare keyword (may include wildcards).
    Keyword(KeywordTerm),
    /// A quoted phrase: "...".
    Quoted(QuotedTerm),
    /// A bracket range expression: [from to to].
    BracketRange(BracketRangeExpr),
    /// A comparison range expression: (value<=field<=value).
    ComparisonRange(ComparisonRangeExpr),
    /// Proximity expression: left (Nw) right.
    Proximity(ProximityExpr),
    /// Frequency expression: term (Nf).
    Frequency(FrequencyExpr),
    /// TREE@ expression.
    TreeAt(TreeAtExpr),
    /// Semantic search expression: R=(value), RAD=(value), RPD=(value).
    SemanticSearch(SemanticSearchExpr),
    /// An error node — unparseable region preserved verbatim.
    Error(ErrorNode),
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<QueryExpr>,
    pub op: BoolOp,
    pub op_span: Span,
    pub right: Box<QueryExpr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct NotExpr {
    pub op_span: Span,
    pub operand: Box<QueryExpr>,
}

#[derive(Debug, Clone)]
pub struct FieldExpr {
    pub field_name: String,
    pub field_span: Span,
    pub equals_span: Span,
    pub body: FieldBody,
}

#[derive(Debug, Clone)]
pub enum FieldBody {
    /// Simple value without parentheses: FIELD=value
    Simple(Box<QueryExpr>),
    /// Parenthesized body: FIELD=(...)
    Parenthesized {
        lparen_span: Span,
        inner: Box<QueryExpr>,
        rparen_span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct GroupExpr {
    pub lparen_span: Span,
    pub inner: Box<QueryExpr>,
    pub rparen_span: Span,
}

#[derive(Debug, Clone)]
pub struct KeywordTerm {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct QuotedTerm {
    pub value: String,
    pub quote_char: char,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BracketRangeExpr {
    pub lbracket_span: Span,
    pub from: String,
    pub from_span: Span,
    pub to_keyword_span: Span,
    pub to: String,
    pub to_span: Span,
    pub rbracket_span: Span,
}

/// Comparison operator in range expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    Lt,
    Lte,
    Gt,
    Gte,
}

impl ComparisonOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComparisonOp::Lt => "<",
            ComparisonOp::Lte => "<=",
            ComparisonOp::Gt => ">",
            ComparisonOp::Gte => ">=",
        }
    }
}

/// A comparison range expression like `(20110101<=pd<=20130101)` or `(pd>20190101)`.
#[derive(Debug, Clone)]
pub struct ComparisonRangeExpr {
    pub lparen_span: Span,
    /// Left value (may be absent for single-sided like `(pd>20190101)`)
    pub left_value: Option<String>,
    pub left_value_span: Option<Span>,
    pub left_op: Option<ComparisonOp>,
    pub left_op_span: Option<Span>,
    /// The field name in the middle.
    pub field_name: String,
    pub field_span: Span,
    /// Right comparison operator.
    pub right_op: Option<ComparisonOp>,
    pub right_op_span: Option<Span>,
    /// Right value (may be absent for single-sided like `(20110101<=pd)`)
    pub right_value: Option<String>,
    pub right_value_span: Option<Span>,
    pub rparen_span: Span,
}

#[derive(Debug, Clone)]
pub struct ProximityExpr {
    pub left: Box<QueryExpr>,
    pub op: String,
    pub op_span: Span,
    pub right: Box<QueryExpr>,
}

#[derive(Debug, Clone)]
pub struct FrequencyExpr {
    pub operand: Box<QueryExpr>,
    pub op: String,
    pub op_span: Span,
}

#[derive(Debug, Clone)]
pub struct TreeAtExpr {
    pub tree_at_span: Span,
    pub operand: Box<QueryExpr>,
}

#[derive(Debug, Clone)]
pub struct SemanticSearchExpr {
    pub keyword: String,
    pub keyword_span: Span,
    pub equals_span: Span,
    pub body: FieldBody,
}

#[derive(Debug, Clone)]
pub struct ErrorNode {
    pub raw_text: String,
    pub span: Span,
}
