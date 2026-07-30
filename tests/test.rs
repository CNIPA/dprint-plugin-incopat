extern crate dprint_development;
extern crate dprint_plugin_incopat;

use std::path::PathBuf;
use std::sync::Arc;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_incopat::configuration::resolve_config;
use dprint_plugin_incopat::*;

#[test]
fn test_specs() {
    let global_config = GlobalConfiguration::default();

    run_specs(
        &PathBuf::from("./tests/specs"),
        &ParseSpecOptions {
            default_file_name: "file.incopat",
        },
        &RunSpecsOptions {
            fix_failures: false,
            format_twice: true,
        },
        {
            let global_config = global_config.clone();
            Arc::new(move |path, file_text, spec_config| {
                let spec_config: ConfigKeyMap =
                    serde_json::from_value(spec_config.clone().into()).unwrap();
                let config_result = resolve_config(spec_config, &global_config);
                ensure_no_diagnostics(&config_result.diagnostics);

                format_text(&path, &file_text, &config_result.config)
            })
        },
        Arc::new(move |_, _file_text, _spec_config| {
            #[cfg(feature = "tracing")]
            {
                panic!("Tracing not yet implemented");
            }

            #[cfg(not(feature = "tracing"))]
            panic!(
                "\n====\nPlease run with `cargo test --features tracing` to get trace output\n====\n"
            )
        }),
    );
}
