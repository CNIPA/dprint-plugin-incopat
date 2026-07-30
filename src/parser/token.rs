/// Byte offset span in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.start..self.span.end]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// A known field code (e.g. ti, ab, ipc).
    FieldCode,
    /// A semantic search keyword (R, RAD, RPD) when followed by `=`.
    SemanticKeyword,
    /// The `=` after a field code or semantic keyword.
    Equals,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// Boolean AND operator.
    And,
    /// Boolean OR operator.
    Or,
    /// Boolean NOT operator.
    Not,
    /// Range keyword `to`.
    To,
    /// Proximity operator: (Nw), (Nn), (s), (p).
    ProximityOp,
    /// Frequency operator: (Nf).
    FrequencyOp,
    /// Comparison operator: <, <=, >, >=.
    ComparisonOp,
    /// `|` pipe separator (used in semantic search multi-values).
    Pipe,
    /// TREE@ operator.
    TreeAt,
    /// A double-quoted string: "...".
    QuotedString,
    /// An unquoted keyword (may contain wildcards *, ?, $).
    Keyword,
    /// A `#` line comment.
    LineComment,
    /// A newline character (\n or \r\n).
    Newline,
    /// Whitespace (spaces, tabs) — not newlines.
    Whitespace,
    /// End of file.
    Eof,
    /// Unrecognized character.
    Error,
}
