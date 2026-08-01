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
}

impl<'a> Context<'a> {
    pub fn new(config: &'a Configuration, source: &'a str) -> Self {
        Self {
            config,
            source,
            depth: 0,
            in_field_body: false,
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
        }
    }

    /// Return a new context nested one level deeper inside a group.
    pub fn with_group(&self) -> Context<'a> {
        Context {
            config: self.config,
            source: self.source,
            depth: self.depth + 1,
            in_field_body: self.in_field_body,
        }
    }
}
