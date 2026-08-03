use super::fields::is_semantic_keyword;
use super::token::{Span, Token, TokenKind};

/// Lexer for incoPat search query syntax.
pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize, // index into chars
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            source,
            chars,
            pos: 0,
        }
    }

    /// Tokenize the entire input into a Vec of tokens.
    /// Whitespace tokens are excluded; Newline tokens are preserved.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok.kind == TokenKind::Whitespace {
                continue;
            }
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        if self.pos >= self.chars.len() {
            let byte_pos = self.source.len();
            return Token::new(TokenKind::Eof, Span::new(byte_pos, byte_pos));
        }

        let (byte_start, ch) = self.chars[self.pos];

        match ch {
            // Newlines
            '\n' => {
                self.pos += 1;
                Token::new(TokenKind::Newline, Span::new(byte_start, byte_start + 1))
            }
            '\r' => {
                self.pos += 1;
                let end = if self.peek_char() == Some('\n') {
                    self.pos += 1;
                    byte_start + 2
                } else {
                    byte_start + 1
                };
                Token::new(TokenKind::Newline, Span::new(byte_start, end))
            }

            // Whitespace (not newlines)
            ' ' | '\t' => self.read_whitespace(byte_start),

            // Comment: # ...
            '#' => self.read_line_comment(byte_start),

            // Structural
            '=' => {
                self.pos += 1;
                Token::new(TokenKind::Equals, Span::new(byte_start, byte_start + 1))
            }
            ')' => {
                self.pos += 1;
                Token::new(TokenKind::RParen, Span::new(byte_start, byte_start + 1))
            }
            '[' => {
                self.pos += 1;
                Token::new(TokenKind::LBracket, Span::new(byte_start, byte_start + 1))
            }
            ']' => {
                self.pos += 1;
                Token::new(TokenKind::RBracket, Span::new(byte_start, byte_start + 1))
            }
            '|' => {
                self.pos += 1;
                Token::new(TokenKind::Pipe, Span::new(byte_start, byte_start + 1))
            }

            // Comparison operators
            '<' | '>' => self.read_comparison_op(byte_start),

            // LParen or proximity/frequency operator
            '(' => self.read_lparen_or_operator(byte_start),

            // Quoted string
            '"' => self.read_quoted_string(byte_start, '"'),
            '\'' => self.read_quoted_string(byte_start, '\''),

            // Word: field code, boolean operator, keyword, or TREE@
            _ if is_word_start(ch) => self.read_word(byte_start),

            // Any other character — treat as keyword character
            _ => self.read_keyword_other(byte_start),
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).map(|&(_, c)| c)
    }

    /// Look ahead past optional spaces/tabs to check if a specific char follows.
    /// Does NOT advance the position.
    fn peek_past_spaces_is(&self, target: char) -> bool {
        let mut i = self.pos;
        while let Some(&(_, ch)) = self.chars.get(i) {
            if ch == ' ' || ch == '\t' {
                i += 1;
            } else {
                return ch == target;
            }
        }
        false
    }

    fn byte_pos(&self) -> usize {
        self.chars
            .get(self.pos)
            .map(|&(b, _)| b)
            .unwrap_or(self.source.len())
    }

    fn read_whitespace(&mut self, byte_start: usize) -> Token {
        self.pos += 1;
        while let Some(&(_, ch)) = self.chars.get(self.pos) {
            if ch == ' ' || ch == '\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Token::new(TokenKind::Whitespace, Span::new(byte_start, self.byte_pos()))
    }

    fn read_line_comment(&mut self, byte_start: usize) -> Token {
        self.pos += 1; // skip '#'
        while let Some(&(_, ch)) = self.chars.get(self.pos) {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.pos += 1;
        }
        Token::new(TokenKind::LineComment, Span::new(byte_start, self.byte_pos()))
    }

    fn read_quoted_string(&mut self, byte_start: usize, quote: char) -> Token {
        self.pos += 1; // skip opening quote
        while let Some(&(_, ch)) = self.chars.get(self.pos) {
            self.pos += 1;
            if ch == quote {
                return Token::new(
                    TokenKind::QuotedString,
                    Span::new(byte_start, self.byte_pos()),
                );
            }
            if ch == '\\' {
                if self.pos < self.chars.len() {
                    self.pos += 1;
                }
            }
        }
        // Unterminated quote
        Token::new(TokenKind::Error, Span::new(byte_start, self.byte_pos()))
    }

    fn read_comparison_op(&mut self, byte_start: usize) -> Token {
        self.pos += 1; // skip '<' or '>'
        if self.peek_char() == Some('=') {
            self.pos += 1;
        }
        Token::new(TokenKind::ComparisonOp, Span::new(byte_start, self.byte_pos()))
    }

    /// Disambiguate `(` as either a proximity/frequency operator or a plain LParen.
    /// Matches patterns: `(Nw)`, `(Nn)`, `(s)`, `(p)`, `(Nf)` where N is 0-2 digits.
    fn read_lparen_or_operator(&mut self, byte_start: usize) -> Token {
        // Save position for potential backtrack
        let saved_pos = self.pos;

        self.pos += 1; // skip '('

        // Try to match proximity/frequency operator pattern
        let mut digit_count = 0;

        // Read optional digits (0-2)
        while digit_count < 2 {
            if let Some(&(_, ch)) = self.chars.get(self.pos) {
                if ch.is_ascii_digit() {
                    self.pos += 1;
                    digit_count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Read the operator letter
        if let Some(&(_, ch)) = self.chars.get(self.pos) {
            let lower = ch.to_ascii_lowercase();
            if matches!(lower, 'w' | 'n' | 's' | 'p' | 'f') {
                self.pos += 1;
                // Check for closing ')'
                if self.peek_char() == Some(')') {
                    self.pos += 1;
                    let byte_end = self.byte_pos();
                    let kind = if lower == 'f' {
                        TokenKind::FrequencyOp
                    } else {
                        TokenKind::ProximityOp
                    };
                    return Token::new(kind, Span::new(byte_start, byte_end));
                }
            }
        }

        // Not a proximity/frequency op — backtrack and return LParen
        self.pos = saved_pos + 1;
        Token::new(TokenKind::LParen, Span::new(byte_start, byte_start + 1))
    }

    fn read_word(&mut self, byte_start: usize) -> Token {
        self.pos += 1;

        while let Some(&(_, ch)) = self.chars.get(self.pos) {
            if is_word_continue(ch) {
                self.pos += 1;
            } else {
                break;
            }
        }

        let byte_end = self.byte_pos();
        let word = &self.source[byte_start..byte_end];
        let upper = word.to_ascii_uppercase();

        // Check for TREE@ — the word "TREE" followed by '@'
        if upper == "TREE" && self.peek_char() == Some('@') {
            self.pos += 1; // consume '@'
            let byte_end = self.byte_pos();
            return Token::new(TokenKind::TreeAt, Span::new(byte_start, byte_end));
        }

        // Lookahead for '=' — skip optional whitespace to detect field codes
        // This handles both `field=value` and `field = value`
        let has_equals_ahead = self.peek_past_spaces_is('=');

        // Check if this is a semantic keyword (R, RAD, RPD) followed by '='
        if has_equals_ahead && is_semantic_keyword(word) {
            return Token::new(TokenKind::SemanticKeyword, Span::new(byte_start, byte_end));
        }

        // Any word followed by '=' is treated as a field code. Known codes are
        // validated by the generator; unknown ones (e.g. official fields that
        // are missing from our list) are preserved structurally instead of
        // being dropped as unparseable input.
        if has_equals_ahead {
            return Token::new(TokenKind::FieldCode, Span::new(byte_start, byte_end));
        }

        // Check for boolean operators (case-insensitive)
        match upper.as_str() {
            "AND" => Token::new(TokenKind::And, Span::new(byte_start, byte_end)),
            "OR" => Token::new(TokenKind::Or, Span::new(byte_start, byte_end)),
            "NOT" => Token::new(TokenKind::Not, Span::new(byte_start, byte_end)),
            "TO" => Token::new(TokenKind::To, Span::new(byte_start, byte_end)),
            _ => Token::new(TokenKind::Keyword, Span::new(byte_start, byte_end)),
        }
    }

    /// Read characters that don't start a word but may be part of a keyword
    /// (CJK characters, special symbols, wildcards, etc.)
    fn read_keyword_other(&mut self, byte_start: usize) -> Token {
        self.pos += 1;
        while let Some(&(_, ch)) = self.chars.get(self.pos) {
            if is_keyword_continue(ch) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Token::new(TokenKind::Keyword, Span::new(byte_start, self.byte_pos()))
    }
}

/// Characters that can start a "word" — ASCII letters, digits, or wildcard chars.
fn is_word_start(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '*' || ch == '?'
        || is_cjk_or_extended(ch)
}

/// Characters that can continue a "word".
fn is_word_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || ch == '_'
        || ch == '*'
        || ch == '?'
        || ch == '$' // wildcard: 0 or 1 char (incoPat uses $ instead of #)
        || ch == '.'
        || ch == '/'
        || ch == '-'
        || is_special_char(ch)
        || is_cjk_or_extended(ch)
}

/// Characters that can continue a keyword (non-word-start chars like CJK etc.)
fn is_keyword_continue(ch: char) -> bool {
    !matches!(
        ch,
        ' ' | '\t' | '\n' | '\r' | '(' | ')' | '[' | ']' | '=' | '"' | '\'' | '<' | '>' | '|'
    ) && !ch.is_ascii_whitespace()
    && ch != '#' // # at start of a new token is a comment
}

/// Check if a character is CJK or other extended Unicode that can appear in keywords.
fn is_cjk_or_extended(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{3000}'..='\u{303F}' // CJK Symbols and Punctuation
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{AC00}'..='\u{D7AF}' // Hangul Syllables
    )
}

/// Special characters supported by incoPat as part of keywords.
fn is_special_char(ch: char) -> bool {
    matches!(ch,
        '℃' | '℉' | '%' | '±' | '°' | '™' | '®'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::token::TokenKind::*;

    fn lex(input: &str) -> Vec<(TokenKind, &str)> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Eof)
            .map(|t| (t.kind.clone(), t.text(input)))
            .collect()
    }

    #[test]
    fn simple_field_expression() {
        let result = lex("ti=汽车");
        assert_eq!(
            result,
            vec![
                (FieldCode, "ti"),
                (Equals, "="),
                (Keyword, "汽车"),
            ]
        );
    }

    #[test]
    fn field_with_parens_and_or() {
        let result = lex("tiab=(空调 or evaporator)");
        assert_eq!(
            result,
            vec![
                (FieldCode, "tiab"),
                (Equals, "="),
                (LParen, "("),
                (Keyword, "空调"),
                (Or, "or"),
                (Keyword, "evaporator"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn boolean_operators() {
        let result = lex("a AND b OR c NOT d");
        assert_eq!(
            result,
            vec![
                (Keyword, "a"),
                (And, "AND"),
                (Keyword, "b"),
                (Or, "OR"),
                (Keyword, "c"),
                (Not, "NOT"),
                (Keyword, "d"),
            ]
        );
    }

    #[test]
    fn case_insensitive_operators() {
        let result = lex("a and b or c not d");
        assert_eq!(
            result,
            vec![
                (Keyword, "a"),
                (And, "and"),
                (Keyword, "b"),
                (Or, "or"),
                (Keyword, "c"),
                (Not, "not"),
                (Keyword, "d"),
            ]
        );
    }

    #[test]
    fn quoted_string() {
        let result = lex("ti=\"air condition\"");
        assert_eq!(
            result,
            vec![
                (FieldCode, "ti"),
                (Equals, "="),
                (QuotedString, "\"air condition\""),
            ]
        );
    }

    #[test]
    fn bracket_range_expression() {
        let result = lex("pd=[20110101 to 20130101]");
        assert_eq!(
            result,
            vec![
                (FieldCode, "pd"),
                (Equals, "="),
                (LBracket, "["),
                (Keyword, "20110101"),
                (To, "to"),
                (Keyword, "20130101"),
                (RBracket, "]"),
            ]
        );
    }

    #[test]
    fn proximity_w_operator() {
        let result = lex("car (w) engine");
        assert_eq!(
            result,
            vec![
                (Keyword, "car"),
                (ProximityOp, "(w)"),
                (Keyword, "engine"),
            ]
        );
    }

    #[test]
    fn proximity_numbered_w() {
        let result = lex("car (2w) engine");
        assert_eq!(
            result,
            vec![
                (Keyword, "car"),
                (ProximityOp, "(2w)"),
                (Keyword, "engine"),
            ]
        );
    }

    #[test]
    fn proximity_n_operator() {
        let result = lex("电 (2n) 机");
        assert_eq!(
            result,
            vec![
                (Keyword, "电"),
                (ProximityOp, "(2n)"),
                (Keyword, "机"),
            ]
        );
    }

    #[test]
    fn proximity_s_operator() {
        let result = lex("a (s) b");
        assert_eq!(
            result,
            vec![
                (Keyword, "a"),
                (ProximityOp, "(s)"),
                (Keyword, "b"),
            ]
        );
    }

    #[test]
    fn proximity_p_operator() {
        let result = lex("a (p) b");
        assert_eq!(
            result,
            vec![
                (Keyword, "a"),
                (ProximityOp, "(p)"),
                (Keyword, "b"),
            ]
        );
    }

    #[test]
    fn frequency_operator() {
        let result = lex("\"机器人\" (3f)");
        assert_eq!(
            result,
            vec![
                (QuotedString, "\"机器人\""),
                (FrequencyOp, "(3f)"),
            ]
        );
    }

    #[test]
    fn paren_not_proximity() {
        // (abc) is a parenthesized group, NOT a proximity op
        let result = lex("(abc)");
        assert_eq!(
            result,
            vec![
                (LParen, "("),
                (Keyword, "abc"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn paren_with_number_not_proximity() {
        // (2) is parenthesized, not a proximity op
        let result = lex("(2)");
        assert_eq!(
            result,
            vec![
                (LParen, "("),
                (Keyword, "2"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn comparison_operators() {
        let result = lex("(20110101<=pd<=20130101)");
        assert_eq!(
            result,
            vec![
                (LParen, "("),
                (Keyword, "20110101"),
                (ComparisonOp, "<="),
                (Keyword, "pd"),
                (ComparisonOp, "<="),
                (Keyword, "20130101"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn comparison_gt() {
        let result = lex("(pd>20190101)");
        assert_eq!(
            result,
            vec![
                (LParen, "("),
                (Keyword, "pd"),
                (ComparisonOp, ">"),
                (Keyword, "20190101"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn semantic_keyword_r() {
        let result = lex("R=(CN101850473B)");
        assert_eq!(
            result,
            vec![
                (SemanticKeyword, "R"),
                (Equals, "="),
                (LParen, "("),
                (Keyword, "CN101850473B"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn semantic_keyword_rad() {
        let result = lex("RAD=(CN1325248C)");
        assert_eq!(
            result,
            vec![
                (SemanticKeyword, "RAD"),
                (Equals, "="),
                (LParen, "("),
                (Keyword, "CN1325248C"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn pipe_in_semantic() {
        let result = lex("R=(CN101850473B|CN1872361B|引擎)");
        assert_eq!(
            result,
            vec![
                (SemanticKeyword, "R"),
                (Equals, "="),
                (LParen, "("),
                (Keyword, "CN101850473B"),
                (Pipe, "|"),
                (Keyword, "CN1872361B"),
                (Pipe, "|"),
                (Keyword, "引擎"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn tree_at_operator() {
        let result = lex("ap=(TREE@\"清华大学\")");
        assert_eq!(
            result,
            vec![
                (FieldCode, "ap"),
                (Equals, "="),
                (LParen, "("),
                (TreeAt, "TREE@"),
                (QuotedString, "\"清华大学\""),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn line_comment() {
        let result = lex("# this is a comment\nti=test");
        assert_eq!(
            result,
            vec![
                (LineComment, "# this is a comment"),
                (Newline, "\n"),
                (FieldCode, "ti"),
                (Equals, "="),
                (Keyword, "test"),
            ]
        );
    }

    #[test]
    fn wildcard_keywords() {
        let result = lex("electr* ?otor m$tor");
        assert_eq!(
            result,
            vec![
                (Keyword, "electr*"),
                (Keyword, "?otor"),
                (Keyword, "m$tor"),
            ]
        );
    }

    #[test]
    fn ipc_classification_code() {
        let result = lex("ipc=(A61K31/04)");
        assert_eq!(
            result,
            vec![
                (FieldCode, "ipc"),
                (Equals, "="),
                (LParen, "("),
                (Keyword, "A61K31/04"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn hyphenated_field() {
        let result = lex("ti-cn=(发动机)");
        assert_eq!(
            result,
            vec![
                (FieldCode, "ti-cn"),
                (Equals, "="),
                (LParen, "("),
                (Keyword, "发动机"),
                (RParen, ")"),
            ]
        );
    }

    #[test]
    fn empty_input() {
        let result = lex("");
        assert_eq!(result, vec![]);
    }

    #[test]
    fn whitespace_only() {
        let result = lex("   \t  ");
        assert_eq!(result, vec![]);
    }

    #[test]
    fn complex_real_world_query() {
        let result = lex("tiab=(空调 or \"air condition\" or 空气调节) and ti=(蒸发器 or evaporator)");
        assert_eq!(result.len(), 17);
        assert_eq!(result[0], (FieldCode, "tiab"));
        assert_eq!(result[1], (Equals, "="));
        assert_eq!(result[2], (LParen, "("));
        assert_eq!(result[3], (Keyword, "空调"));
        assert_eq!(result[4], (Or, "or"));
        assert_eq!(result[5], (QuotedString, "\"air condition\""));
        assert_eq!(result[6], (Or, "or"));
        assert_eq!(result[7], (Keyword, "空气调节"));
        assert_eq!(result[8], (RParen, ")"));
        assert_eq!(result[9], (And, "and"));
        assert_eq!(result[10], (FieldCode, "ti"));
        assert_eq!(result[11], (Equals, "="));
        assert_eq!(result[12], (LParen, "("));
        assert_eq!(result[13], (Keyword, "蒸发器"));
        assert_eq!(result[14], (Or, "or"));
        assert_eq!(result[15], (Keyword, "evaporator"));
        assert_eq!(result[16], (RParen, ")"));
    }
}
