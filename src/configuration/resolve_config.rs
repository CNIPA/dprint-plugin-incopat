use super::Configuration;
use super::types::*;
use dprint_core::configuration::*;

pub fn resolve_config(
    config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
    let mut diagnostics = Vec::new();
    let mut config = config;

    let resolved_config = Configuration {
        line_width: get_value(
            &mut config,
            "lineWidth",
            global_config
                .line_width
                .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.line_width),
            &mut diagnostics,
        ),
        use_tabs: get_value(
            &mut config,
            "useTabs",
            global_config
                .use_tabs
                .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.use_tabs),
            &mut diagnostics,
        ),
        indent_width: get_value(
            &mut config,
            "indentWidth",
            global_config.indent_width.unwrap_or(2),
            &mut diagnostics,
        ),
        new_line_kind: get_value(
            &mut config,
            "newLineKind",
            global_config
                .new_line_kind
                .unwrap_or(RECOMMENDED_GLOBAL_CONFIGURATION.new_line_kind),
            &mut diagnostics,
        ),
        operator_position: get_value(
            &mut config,
            "operatorPosition",
            OperatorPosition::Before,
            &mut diagnostics,
        ),
        quote_style: get_value(
            &mut config,
            "quoteStyle",
            QuoteStyle::Double,
            &mut diagnostics,
        ),
        field_case: get_value(
            &mut config,
            "fieldCase",
            CaseStyle::Lowercase,
            &mut diagnostics,
        ),
        boolean_operator_case: get_value(
            &mut config,
            "booleanOperatorCase",
            CaseStyle::Lowercase,
            &mut diagnostics,
        ),
        ignore_node_comment_text: get_value(
            &mut config,
            "ignoreNodeCommentText",
            String::from("incopat-ignore"),
            &mut diagnostics,
        ),
    };

    diagnostics.extend(get_unknown_property_diagnostics(config));

    ResolveConfigurationResult {
        config: resolved_config,
        diagnostics,
    }
}

#[cfg(test)]
mod test {
    use dprint_core::configuration::{ConfigKeyMap, GlobalConfiguration};
    use super::resolve_config;
    use super::super::types::*;

    #[test]
    fn defaults() {
        let result = resolve_config(ConfigKeyMap::new(), &GlobalConfiguration::default());
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.config.indent_width, 2);
        assert_eq!(result.config.line_width, 120);
        assert_eq!(result.config.use_tabs, false);
        assert_eq!(result.config.operator_position, OperatorPosition::Before);
        assert_eq!(result.config.quote_style, QuoteStyle::Double);
        assert_eq!(result.config.field_case, CaseStyle::Lowercase);
        assert_eq!(result.config.boolean_operator_case, CaseStyle::Lowercase);
        assert_eq!(result.config.ignore_node_comment_text, "incopat-ignore");
    }

    #[test]
    fn unknown_property_diagnostic() {
        let mut config = ConfigKeyMap::new();
        config.insert("unknownProp".to_string(), true.into());
        let result = resolve_config(config, &GlobalConfiguration::default());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].property_name, "unknownProp");
    }
}
