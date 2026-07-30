use super::ast::*;
use super::fields::is_field_code;
use super::lexer::Lexer;
use super::token::{Span, Token, TokenKind};

/// Parse error with location information.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Parse result for a .incopat file.
pub struct ParseResult {
    pub file: File,
    pub errors: Vec<ParseError>,
}

/// Parser for incoPat search query syntax.
/// Uses a Pratt parser (precedence climbing) approach.
pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

/// Operator precedence levels (higher = binds tighter).
const PREC_OR: u8 = 1;
const PREC_AND: u8 = 2;
const PREC_PROXIMITY: u8 = 4;

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        Self {
            source,
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Parse the entire file into a File AST node.
    pub fn parse(mut self) -> ParseResult {
        let mut statements = Vec::new();

        self.consume_newlines();

        while !self.at_eof() {
            if self.peek_kind() == TokenKind::Newline {
                let newline_count = self.consume_newlines();
                if newline_count >= 2 {
                    statements.push(Statement::BlankLine);
                }
                continue;
            }

            if self.peek_kind() == TokenKind::LineComment {
                let tok = self.advance();
                statements.push(Statement::Comment(Comment {
                    text: tok.text(self.source).to_string(),
                    span: tok.span,
                }));
                continue;
            }

            match self.parse_query_expr(0) {
                Some(expr) => {
                    statements.push(Statement::Query(expr));
                }
                None => {
                    self.skip_to_next_line();
                }
            }
        }

        ParseResult {
            file: File { statements },
            errors: self.errors,
        }
    }

    // ── Pratt parser core ──

    fn parse_query_expr(&mut self, min_prec: u8) -> Option<QueryExpr> {
        let mut left = self.parse_prefix()?;

        loop {
            let saved = self.pos;
            self.skip_non_blank_newlines();

            if let Some((op, prec, op_span)) = self.peek_infix_op() {
                if prec < min_prec {
                    self.pos = saved;
                    break;
                }
                self.advance(); // consume the operator token

                self.skip_non_blank_newlines();

                let right = match self.parse_query_expr(prec + 1) {
                    Some(r) => r,
                    None => {
                        self.error_at_current("expected expression after operator");
                        break;
                    }
                };

                left = match op {
                    InfixOp::Bool(bool_op) => QueryExpr::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: bool_op,
                        op_span,
                        right: Box::new(right),
                    }),
                    InfixOp::Proximity(op_str) => QueryExpr::Proximity(ProximityExpr {
                        left: Box::new(left),
                        op: op_str,
                        op_span,
                        right: Box::new(right),
                    }),
                };
            } else {
                // Check for implicit AND
                if (self.is_atom_start() || self.peek_kind() == TokenKind::Not) && min_prec <= PREC_AND {
                    let right = match self.parse_query_expr(PREC_AND + 1) {
                        Some(r) => r,
                        None => {
                            self.pos = saved;
                            break;
                        }
                    };
                    let implicit_span = Span::new(
                        left.span().end,
                        right.span().start,
                    );
                    left = QueryExpr::Binary(BinaryExpr {
                        left: Box::new(left),
                        op: BoolOp::And,
                        op_span: implicit_span,
                        right: Box::new(right),
                    });
                } else {
                    self.pos = saved;
                    break;
                }
            }
        }

        // Check for frequency operator (postfix)
        if self.peek_kind() == TokenKind::FrequencyOp {
            let tok = self.advance();
            left = QueryExpr::Frequency(FrequencyExpr {
                operand: Box::new(left),
                op: tok.text(self.source).to_string(),
                op_span: tok.span,
            });
        }

        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<QueryExpr> {
        match self.peek_kind() {
            TokenKind::Not => {
                let tok = self.advance();
                self.skip_non_blank_newlines();
                let operand = self.parse_query_expr(PREC_AND + 1)?;
                Some(QueryExpr::Not(NotExpr {
                    op_span: tok.span,
                    operand: Box::new(operand),
                }))
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Option<QueryExpr> {
        match self.peek_kind() {
            TokenKind::SemanticKeyword => self.parse_semantic_search_expr(),
            TokenKind::FieldCode => self.parse_field_expr(),
            TokenKind::LParen => self.parse_lparen_expr(),
            TokenKind::LBracket => self.parse_bracket_range_expr(),
            TokenKind::QuotedString => self.parse_quoted_term(),
            TokenKind::Keyword => self.parse_keyword_term(),
            TokenKind::TreeAt => self.parse_tree_at_expr(),
            _ => {
                self.error_at_current("unexpected token");
                None
            }
        }
    }

    fn parse_semantic_search_expr(&mut self) -> Option<QueryExpr> {
        let kw_tok = self.advance(); // SemanticKeyword
        let keyword = kw_tok.text(self.source).to_string();
        let keyword_span = kw_tok.span;

        if self.peek_kind() != TokenKind::Equals {
            self.error_at_current("expected '=' after semantic keyword");
            return None;
        }
        let equals_tok = self.advance();

        let body = if self.peek_kind() == TokenKind::LParen {
            let lparen = self.advance();
            self.skip_non_blank_newlines();
            let inner = self.parse_query_expr(0).unwrap_or(QueryExpr::Error(ErrorNode {
                raw_text: String::new(),
                span: Span::new(lparen.span.end, lparen.span.end),
            }));
            self.skip_non_blank_newlines();
            let rparen_span = if self.peek_kind() == TokenKind::RParen {
                self.advance().span
            } else {
                self.error_at_current("expected ')' to close semantic search expression");
                Span::new(self.current_byte_pos(), self.current_byte_pos())
            };
            FieldBody::Parenthesized {
                lparen_span: lparen.span,
                inner: Box::new(inner),
                rparen_span,
            }
        } else {
            let value = self.parse_atom()?;
            FieldBody::Simple(Box::new(value))
        };

        Some(QueryExpr::SemanticSearch(SemanticSearchExpr {
            keyword,
            keyword_span,
            equals_span: equals_tok.span,
            body,
        }))
    }

    fn parse_field_expr(&mut self) -> Option<QueryExpr> {
        let field_tok = self.advance(); // FieldCode
        let field_name = field_tok.text(self.source).to_string();
        let field_span = field_tok.span;

        if self.peek_kind() != TokenKind::Equals {
            self.error_at_current("expected '=' after field code");
            return None;
        }
        let equals_tok = self.advance();

        let body = if self.peek_kind() == TokenKind::LParen {
            let lparen = self.advance();
            self.skip_non_blank_newlines();
            let inner = self.parse_query_expr(0).unwrap_or(QueryExpr::Error(ErrorNode {
                raw_text: String::new(),
                span: Span::new(lparen.span.end, lparen.span.end),
            }));
            self.skip_non_blank_newlines();
            if self.peek_kind() != TokenKind::RParen {
                self.error_at_current("expected ')' to close field expression");
            }
            let rparen_span = if self.peek_kind() == TokenKind::RParen {
                self.advance().span
            } else {
                Span::new(self.current_byte_pos(), self.current_byte_pos())
            };
            FieldBody::Parenthesized {
                lparen_span: lparen.span,
                inner: Box::new(inner),
                rparen_span,
            }
        } else if self.peek_kind() == TokenKind::LBracket {
            let range = self.parse_bracket_range_expr()?;
            FieldBody::Simple(Box::new(range))
        } else {
            let value = self.parse_atom()?;
            FieldBody::Simple(Box::new(value))
        };

        Some(QueryExpr::Field(FieldExpr {
            field_name,
            field_span,
            equals_span: equals_tok.span,
            body,
        }))
    }

    /// Parse `(` which could be a group expression or a comparison range.
    fn parse_lparen_expr(&mut self) -> Option<QueryExpr> {
        // Try to speculatively parse a comparison range expression.
        if let Some(cmp) = self.try_parse_comparison_range() {
            return Some(cmp);
        }
        self.parse_group_expr()
    }

    /// Speculatively try to parse a comparison range expression.
    /// Forms: `(value<=field<=value)`, `(field>value)`, `(value<field)`, etc.
    fn try_parse_comparison_range(&mut self) -> Option<QueryExpr> {
        let saved_pos = self.pos;

        let lparen = self.advance(); // LParen

        // Try form: (value CompOp field CompOp value)
        // or form: (field CompOp value)
        // or form: (value CompOp field)

        // First token could be keyword (value) or field code
        let first_kind = self.peek_kind();
        if first_kind != TokenKind::Keyword && first_kind != TokenKind::FieldCode {
            self.pos = saved_pos;
            return None;
        }
        let first_tok = self.advance();
        let first_text = first_tok.text(self.source).to_string();

        // Next must be a comparison op
        if self.peek_kind() != TokenKind::ComparisonOp {
            self.pos = saved_pos;
            return None;
        }
        let first_op_tok = self.advance();
        let first_op_text = first_op_tok.text(self.source);
        let first_op = match first_op_text {
            "<" => ComparisonOp::Lt,
            "<=" => ComparisonOp::Lte,
            ">" => ComparisonOp::Gt,
            ">=" => ComparisonOp::Gte,
            _ => { self.pos = saved_pos; return None; }
        };

        // Next token
        let second_kind = self.peek_kind();
        if second_kind != TokenKind::Keyword && second_kind != TokenKind::FieldCode {
            self.pos = saved_pos;
            return None;
        }
        let second_tok = self.advance();
        let second_text = second_tok.text(self.source).to_string();

        // Check if there's another comparison op (double-sided range)
        if self.peek_kind() == TokenKind::ComparisonOp {
            let second_op_tok = self.advance();
            let second_op_text = second_op_tok.text(self.source);
            let second_op = match second_op_text {
                "<" => ComparisonOp::Lt,
                "<=" => ComparisonOp::Lte,
                ">" => ComparisonOp::Gt,
                ">=" => ComparisonOp::Gte,
                _ => { self.pos = saved_pos; return None; }
            };

            // Third value
            let third_kind = self.peek_kind();
            if third_kind != TokenKind::Keyword && third_kind != TokenKind::FieldCode {
                self.pos = saved_pos;
                return None;
            }
            let third_tok = self.advance();
            let third_text = third_tok.text(self.source).to_string();

            // Must close with RParen
            if self.peek_kind() != TokenKind::RParen {
                self.pos = saved_pos;
                return None;
            }
            let rparen = self.advance();

            // Double-sided: (value op field op value)
            // The field is in the middle (second_text)
            // Validate it's a field code
            if !is_field_code(&second_text) {
                self.pos = saved_pos;
                return None;
            }

            return Some(QueryExpr::ComparisonRange(ComparisonRangeExpr {
                lparen_span: lparen.span,
                left_value: Some(first_text),
                left_value_span: Some(first_tok.span),
                left_op: Some(first_op),
                left_op_span: Some(first_op_tok.span),
                field_name: second_text,
                field_span: second_tok.span,
                right_op: Some(second_op),
                right_op_span: Some(second_op_tok.span),
                right_value: Some(third_text),
                right_value_span: Some(third_tok.span),
                rparen_span: rparen.span,
            }));
        }

        // Single-sided: check for RParen
        if self.peek_kind() != TokenKind::RParen {
            self.pos = saved_pos;
            return None;
        }
        let rparen = self.advance();

        // Determine which side has the field code
        let first_is_field = is_field_code(&first_text);
        let second_is_field = is_field_code(&second_text);

        if first_is_field && !second_is_field {
            // (field op value)
            return Some(QueryExpr::ComparisonRange(ComparisonRangeExpr {
                lparen_span: lparen.span,
                left_value: None,
                left_value_span: None,
                left_op: None,
                left_op_span: None,
                field_name: first_text,
                field_span: first_tok.span,
                right_op: Some(first_op),
                right_op_span: Some(first_op_tok.span),
                right_value: Some(second_text),
                right_value_span: Some(second_tok.span),
                rparen_span: rparen.span,
            }));
        } else if !first_is_field && second_is_field {
            // (value op field)
            return Some(QueryExpr::ComparisonRange(ComparisonRangeExpr {
                lparen_span: lparen.span,
                left_value: Some(first_text),
                left_value_span: Some(first_tok.span),
                left_op: Some(first_op),
                left_op_span: Some(first_op_tok.span),
                field_name: second_text,
                field_span: second_tok.span,
                right_op: None,
                right_op_span: None,
                right_value: None,
                right_value_span: None,
                rparen_span: rparen.span,
            }));
        }

        // Can't determine which is the field — backtrack
        self.pos = saved_pos;
        None
    }

    fn parse_group_expr(&mut self) -> Option<QueryExpr> {
        let lparen = self.advance(); // LParen
        self.skip_non_blank_newlines();
        let inner = self.parse_query_expr(0).unwrap_or(QueryExpr::Error(ErrorNode {
            raw_text: String::new(),
            span: Span::new(lparen.span.end, lparen.span.end),
        }));
        self.skip_non_blank_newlines();
        if self.peek_kind() != TokenKind::RParen {
            self.error_at_current("expected ')'");
        }
        let rparen_span = if self.peek_kind() == TokenKind::RParen {
            self.advance().span
        } else {
            Span::new(self.current_byte_pos(), self.current_byte_pos())
        };
        Some(QueryExpr::Group(GroupExpr {
            lparen_span: lparen.span,
            inner: Box::new(inner),
            rparen_span,
        }))
    }

    fn parse_bracket_range_expr(&mut self) -> Option<QueryExpr> {
        let lbracket = self.advance(); // LBracket
        let from_tok = self.expect_keyword_or_quoted("expected range start value")?;
        let from = from_tok.text(self.source).to_string();
        let from_span = from_tok.span;

        if self.peek_kind() != TokenKind::To {
            self.error_at_current("expected 'to' in range expression");
            return None;
        }
        let to_kw = self.advance();

        let to_tok = self.expect_keyword_or_quoted("expected range end value")?;
        let to = to_tok.text(self.source).to_string();
        let to_span = to_tok.span;

        if self.peek_kind() != TokenKind::RBracket {
            self.error_at_current("expected ']' to close range");
        }
        let rbracket_span = if self.peek_kind() == TokenKind::RBracket {
            self.advance().span
        } else {
            Span::new(self.current_byte_pos(), self.current_byte_pos())
        };

        Some(QueryExpr::BracketRange(BracketRangeExpr {
            lbracket_span: lbracket.span,
            from,
            from_span,
            to_keyword_span: to_kw.span,
            to,
            to_span,
            rbracket_span,
        }))
    }

    fn parse_quoted_term(&mut self) -> Option<QueryExpr> {
        let tok = self.advance();
        let raw = tok.text(self.source);
        let quote_char = raw.chars().next().unwrap_or('"');
        let value = if raw.len() >= 2 {
            raw[1..raw.len() - 1].to_string()
        } else {
            raw.to_string()
        };
        Some(QueryExpr::Quoted(QuotedTerm {
            value,
            quote_char,
            span: tok.span,
        }))
    }

    fn parse_keyword_term(&mut self) -> Option<QueryExpr> {
        let tok = self.advance();
        Some(QueryExpr::Keyword(KeywordTerm {
            value: tok.text(self.source).to_string(),
            span: tok.span,
        }))
    }

    fn parse_tree_at_expr(&mut self) -> Option<QueryExpr> {
        let tree_at_tok = self.advance(); // TREE@
        let operand = self.parse_atom()?;
        Some(QueryExpr::TreeAt(TreeAtExpr {
            tree_at_span: tree_at_tok.span,
            operand: Box::new(operand),
        }))
    }

    // ── Helper methods ──

    fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn peek_token(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn at_eof(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos.min(self.tokens.len() - 1)].clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn current_byte_pos(&self) -> usize {
        self.peek_token().span.start
    }

    fn skip_non_blank_newlines(&mut self) {
        while self.peek_kind() == TokenKind::Newline {
            let next_pos = self.pos + 1;
            if next_pos < self.tokens.len() && self.tokens[next_pos].kind == TokenKind::Newline {
                break;
            }
            self.advance();
        }
    }

    fn consume_newlines(&mut self) -> usize {
        let mut count = 0;
        while self.peek_kind() == TokenKind::Newline {
            self.advance();
            count += 1;
        }
        count
    }

    fn skip_to_next_line(&mut self) {
        while !self.at_eof() && self.peek_kind() != TokenKind::Newline {
            self.advance();
        }
        if self.peek_kind() == TokenKind::Newline {
            self.advance();
        }
    }

    fn is_atom_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::SemanticKeyword
                | TokenKind::FieldCode
                | TokenKind::Keyword
                | TokenKind::QuotedString
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::TreeAt
        )
    }

    fn peek_infix_op(&self) -> Option<(InfixOp, u8, Span)> {
        let tok = self.peek_token();
        match &tok.kind {
            TokenKind::And => Some((InfixOp::Bool(BoolOp::And), PREC_AND, tok.span)),
            TokenKind::Or => Some((InfixOp::Bool(BoolOp::Or), PREC_OR, tok.span)),
            TokenKind::ProximityOp => {
                let op_str = tok.text(self.source).to_string();
                Some((InfixOp::Proximity(op_str), PREC_PROXIMITY, tok.span))
            }
            _ => None,
        }
    }

    fn expect_keyword_or_quoted(&mut self, msg: &str) -> Option<Token> {
        match self.peek_kind() {
            TokenKind::Keyword | TokenKind::QuotedString => Some(self.advance()),
            _ => {
                self.error_at_current(msg);
                None
            }
        }
    }

    fn error_at_current(&mut self, message: &str) {
        let span = self.peek_token().span;
        self.errors.push(ParseError {
            message: message.to_string(),
            span,
        });
    }
}

enum InfixOp {
    Bool(BoolOp),
    Proximity(String),
}

// ── Span helpers for QueryExpr ──

impl QueryExpr {
    pub fn span(&self) -> Span {
        match self {
            QueryExpr::Binary(e) => Span::new(e.left.span().start, e.right.span().end),
            QueryExpr::Not(e) => Span::new(e.op_span.start, e.operand.span().end),
            QueryExpr::Field(e) => {
                let end = match &e.body {
                    FieldBody::Simple(inner) => inner.span().end,
                    FieldBody::Parenthesized { rparen_span, .. } => rparen_span.end,
                };
                Span::new(e.field_span.start, end)
            }
            QueryExpr::Group(e) => Span::new(e.lparen_span.start, e.rparen_span.end),
            QueryExpr::Keyword(e) => e.span,
            QueryExpr::Quoted(e) => e.span,
            QueryExpr::BracketRange(e) => Span::new(e.lbracket_span.start, e.rbracket_span.end),
            QueryExpr::ComparisonRange(e) => Span::new(e.lparen_span.start, e.rparen_span.end),
            QueryExpr::Proximity(e) => Span::new(e.left.span().start, e.right.span().end),
            QueryExpr::Frequency(e) => Span::new(e.operand.span().start, e.op_span.end),
            QueryExpr::TreeAt(e) => Span::new(e.tree_at_span.start, e.operand.span().end),
            QueryExpr::SemanticSearch(e) => {
                let end = match &e.body {
                    FieldBody::Simple(inner) => inner.span().end,
                    FieldBody::Parenthesized { rparen_span, .. } => rparen_span.end,
                };
                Span::new(e.keyword_span.start, end)
            }
            QueryExpr::Error(e) => e.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> ParseResult {
        Parser::new(input).parse()
    }

    fn assert_no_errors(result: &ParseResult) {
        if !result.errors.is_empty() {
            panic!("expected no parse errors, got: {:?}", result.errors);
        }
    }

    #[test]
    fn simple_keyword() {
        let result = parse("汽车");
        assert_no_errors(&result);
        assert_eq!(result.file.statements.len(), 1);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Keyword(k)) => assert_eq!(k.value, "汽车"),
            other => panic!("expected keyword, got {:?}", other),
        }
    }

    #[test]
    fn field_with_simple_value() {
        let result = parse("ti=汽车");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Field(f)) => {
                assert_eq!(f.field_name, "ti");
                match &f.body {
                    FieldBody::Simple(inner) => match inner.as_ref() {
                        QueryExpr::Keyword(k) => assert_eq!(k.value, "汽车"),
                        other => panic!("expected keyword, got {:?}", other),
                    },
                    _ => panic!("expected simple body"),
                }
            }
            other => panic!("expected field expr, got {:?}", other),
        }
    }

    #[test]
    fn field_with_parenthesized_or() {
        let result = parse("tiab=(空调 or 蒸发器)");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Field(f)) => {
                assert_eq!(f.field_name, "tiab");
                match &f.body {
                    FieldBody::Parenthesized { inner, .. } => match inner.as_ref() {
                        QueryExpr::Binary(b) => assert_eq!(b.op, BoolOp::Or),
                        other => panic!("expected binary, got {:?}", other),
                    },
                    _ => panic!("expected parenthesized body"),
                }
            }
            other => panic!("expected field expr, got {:?}", other),
        }
    }

    #[test]
    fn binary_and_or_precedence() {
        let result = parse("ti=a and ab=b or ipc=c");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Binary(outer)) => {
                assert_eq!(outer.op, BoolOp::Or);
                match outer.left.as_ref() {
                    QueryExpr::Binary(inner) => assert_eq!(inner.op, BoolOp::And),
                    other => panic!("expected inner AND, got {:?}", other),
                }
            }
            other => panic!("expected binary, got {:?}", other),
        }
    }

    #[test]
    fn not_operator() {
        let result = parse("ti=a not ab=b");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Binary(b)) => {
                assert_eq!(b.op, BoolOp::And);
                match b.right.as_ref() {
                    QueryExpr::Not(_) => {}
                    other => panic!("expected NOT, got {:?}", other),
                }
            }
            other => panic!("expected binary with NOT, got {:?}", other),
        }
    }

    #[test]
    fn bracket_range() {
        let result = parse("pd=[20110101 to 20130101]");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Field(f)) => {
                assert_eq!(f.field_name, "pd");
                match &f.body {
                    FieldBody::Simple(inner) => match inner.as_ref() {
                        QueryExpr::BracketRange(r) => {
                            assert_eq!(r.from, "20110101");
                            assert_eq!(r.to, "20130101");
                        }
                        other => panic!("expected bracket range, got {:?}", other),
                    },
                    _ => panic!("expected simple body"),
                }
            }
            other => panic!("expected field, got {:?}", other),
        }
    }

    #[test]
    fn comparison_range_double() {
        let result = parse("(20110101<=pd<=20130101)");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::ComparisonRange(c)) => {
                assert_eq!(c.left_value.as_deref(), Some("20110101"));
                assert_eq!(c.left_op, Some(ComparisonOp::Lte));
                assert_eq!(c.field_name, "pd");
                assert_eq!(c.right_op, Some(ComparisonOp::Lte));
                assert_eq!(c.right_value.as_deref(), Some("20130101"));
            }
            other => panic!("expected comparison range, got {:?}", other),
        }
    }

    #[test]
    fn comparison_range_single_field_first() {
        let result = parse("(pd>20190101)");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::ComparisonRange(c)) => {
                assert_eq!(c.field_name, "pd");
                assert_eq!(c.right_op, Some(ComparisonOp::Gt));
                assert_eq!(c.right_value.as_deref(), Some("20190101"));
                assert!(c.left_value.is_none());
            }
            other => panic!("expected comparison range, got {:?}", other),
        }
    }

    #[test]
    fn proximity_operator() {
        let result = parse("data (2w) line");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Proximity(p)) => {
                assert_eq!(p.op, "(2w)");
            }
            other => panic!("expected proximity, got {:?}", other),
        }
    }

    #[test]
    fn frequency_operator() {
        let result = parse("tiab=(\"机器人\" (3f))");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Field(f)) => match &f.body {
                FieldBody::Parenthesized { inner, .. } => match inner.as_ref() {
                    QueryExpr::Frequency(freq) => assert_eq!(freq.op, "(3f)"),
                    other => panic!("expected frequency, got {:?}", other),
                },
                _ => panic!("expected parenthesized"),
            },
            other => panic!("expected field, got {:?}", other),
        }
    }

    #[test]
    fn semantic_search_r() {
        let result = parse("R=(CN101850473B)");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::SemanticSearch(s)) => {
                assert_eq!(s.keyword, "R");
            }
            other => panic!("expected semantic search, got {:?}", other),
        }
    }

    #[test]
    fn semantic_search_with_and() {
        let result = parse("R=(CN101850473B) AND tiab=(发动机)");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Binary(b)) => {
                assert_eq!(b.op, BoolOp::And);
                match b.left.as_ref() {
                    QueryExpr::SemanticSearch(s) => assert_eq!(s.keyword, "R"),
                    other => panic!("expected semantic search, got {:?}", other),
                }
            }
            other => panic!("expected binary, got {:?}", other),
        }
    }

    #[test]
    fn tree_at_expression() {
        let result = parse("ap=(TREE@\"清华大学\")");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Field(f)) => {
                assert_eq!(f.field_name, "ap");
                match &f.body {
                    FieldBody::Parenthesized { inner, .. } => match inner.as_ref() {
                        QueryExpr::TreeAt(t) => match t.operand.as_ref() {
                            QueryExpr::Quoted(q) => assert_eq!(q.value, "清华大学"),
                            other => panic!("expected quoted, got {:?}", other),
                        },
                        other => panic!("expected tree_at, got {:?}", other),
                    },
                    _ => panic!("expected parenthesized body"),
                }
            }
            other => panic!("expected field, got {:?}", other),
        }
    }

    #[test]
    fn comment_standalone() {
        let result = parse("# this is a comment\nti=test");
        assert_no_errors(&result);
        assert_eq!(result.file.statements.len(), 2);
        match &result.file.statements[0] {
            Statement::Comment(c) => assert_eq!(c.text, "# this is a comment"),
            other => panic!("expected comment, got {:?}", other),
        }
    }

    #[test]
    fn blank_line_between_queries() {
        let result = parse("ti=a\n\nti=b");
        assert_no_errors(&result);
        assert_eq!(result.file.statements.len(), 3);
        assert!(matches!(result.file.statements[0], Statement::Query(_)));
        assert!(matches!(result.file.statements[1], Statement::BlankLine));
        assert!(matches!(result.file.statements[2], Statement::Query(_)));
    }

    #[test]
    fn multiline_query() {
        let result = parse("ti=a\nand ab=b");
        assert_no_errors(&result);
        match &result.file.statements[0] {
            Statement::Query(QueryExpr::Binary(b)) => assert_eq!(b.op, BoolOp::And),
            other => panic!("expected binary AND for multiline, got {:?}", other),
        }
    }
}
