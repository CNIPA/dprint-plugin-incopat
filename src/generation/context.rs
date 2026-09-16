use crate::configuration::Configuration;

/// Context carried during IR generation.
#[allow(dead_code)]
pub struct Context<'a> {
    pub config: &'a Configuration,
    pub source: &'a str,
    /// Track nesting depth: 0 = top-level, > 0 = inside field/group.
    pub depth: usize,
    /// When true, binary chains use adaptive line breaking (SpaceOrNewLine).
    /// Set to true inside field parenthesized bodies.
    pub in_field_body: bool,
    /// Column of the closing paren of the innermost wrapped block inside a
    /// field value (e.g. one side of a `(s)` / `(p)` proximity expression).
    /// Continuation lines of chains inside such a block are indented 4 columns
    /// past that paren instead of relative to `depth`.
    pub block_close_col: Option<usize>,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a Configuration, source: &'a str) -> Self {
        Self {
            config,
            source,
            depth: 0,
            in_field_body: false,
            block_close_col: None,
        }
    }

    #[allow(dead_code)]
    pub fn is_top_level(&self) -> bool {
        self.depth == 0
    }

    /// Return a new context with in_field_body set.
    pub fn with_field_body(&self) -> Context<'a> {
        Context {
            config: self.config,
            source: self.source,
            depth: self.depth,
            in_field_body: true,
            block_close_col: self.block_close_col,
        }
    }

    /// Return a new context for the content of a wrapped block whose closing
    /// paren sits in `close_col`.
    pub fn with_block(&self, close_col: usize) -> Context<'a> {
        Context {
            config: self.config,
            source: self.source,
            depth: self.depth,
            in_field_body: true,
            block_close_col: Some(close_col),
        }
    }

    /// Return a new context nested one level deeper inside a group.
    pub fn with_group(&self) -> Context<'a> {
        Context {
            config: self.config,
            source: self.source,
            depth: self.depth + 1,
            in_field_body: self.in_field_body,
            block_close_col: self.block_close_col,
        }
    }
}
