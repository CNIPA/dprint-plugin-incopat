use dprint_core::configuration::{ConfigKeyMap, ConfigKeyValue, GlobalConfiguration, NewLineKind};

use super::*;
use super::types::*;

#[derive(Default)]
pub struct ConfigurationBuilder {
    pub(super) config: ConfigKeyMap,
    global_config: Option<GlobalConfiguration>,
}

impl ConfigurationBuilder {
    pub fn new() -> ConfigurationBuilder {
        Default::default()
    }

    pub fn build(&self) -> Configuration {
        if let Some(global_config) = &self.global_config {
            resolve_config(self.config.clone(), global_config).config
        } else {
            resolve_config(self.config.clone(), &GlobalConfiguration::default()).config
        }
    }

    pub fn global_config(&mut self, global_config: GlobalConfiguration) -> &mut Self {
        self.global_config = Some(global_config);
        self
    }

    pub fn line_width(&mut self, value: u32) -> &mut Self {
        self.insert("lineWidth", (value as i32).into())
    }

    pub fn use_tabs(&mut self, value: bool) -> &mut Self {
        self.insert("useTabs", value.into())
    }

    pub fn indent_width(&mut self, value: u8) -> &mut Self {
        self.insert("indentWidth", (value as i32).into())
    }

    pub fn new_line_kind(&mut self, value: NewLineKind) -> &mut Self {
        self.insert("newLineKind", value.to_string().into())
    }

    pub fn operator_position(&mut self, value: OperatorPosition) -> &mut Self {
        self.insert("operatorPosition", value.to_string().into())
    }

    pub fn quote_style(&mut self, value: QuoteStyle) -> &mut Self {
        self.insert("quoteStyle", value.to_string().into())
    }

    pub fn field_case(&mut self, value: CaseStyle) -> &mut Self {
        self.insert("fieldCase", value.to_string().into())
    }

    pub fn boolean_operator_case(&mut self, value: CaseStyle) -> &mut Self {
        self.insert("booleanOperatorCase", value.to_string().into())
    }

    pub fn ignore_node_comment_text(&mut self, value: &str) -> &mut Self {
        self.insert("ignoreNodeCommentText", value.into())
    }

    #[cfg(test)]
    pub(super) fn get_inner_config(&self) -> ConfigKeyMap {
        self.config.clone()
    }

    fn insert(&mut self, name: &str, value: ConfigKeyValue) -> &mut Self {
        self.config.insert(String::from(name), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use dprint_core::configuration::{GlobalConfiguration, NewLineKind, resolve_global_config, ConfigKeyMap};
    use super::*;

    #[test]
    fn check_all_values_set() {
        let mut config = ConfigurationBuilder::new();
        config
            .line_width(100)
            .use_tabs(false)
            .indent_width(2)
            .new_line_kind(NewLineKind::LineFeed)
            .operator_position(OperatorPosition::Before)
            .quote_style(QuoteStyle::Double)
            .field_case(CaseStyle::Lowercase)
            .boolean_operator_case(CaseStyle::Lowercase)
            .ignore_node_comment_text("incopat-ignore");

        let inner_config = config.get_inner_config();
        assert_eq!(inner_config.len(), 9);
        let diagnostics = resolve_config(inner_config, &GlobalConfiguration::default()).diagnostics;
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn handle_global_config() {
        let mut global_config = ConfigKeyMap::new();
        global_config.insert(String::from("lineWidth"), 90.into());
        global_config.insert(String::from("useTabs"), true.into());
        let global_config = resolve_global_config(&mut global_config).config;
        let config = ConfigurationBuilder::new()
            .global_config(global_config)
            .build();
        assert_eq!(config.line_width, 90);
        assert_eq!(config.use_tabs, true);
    }
}
