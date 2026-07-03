use std::sync::OnceLock;

use config::{Config, Environment};
use miette::{IntoDiagnostic, Result, miette};
use serde::Deserialize;

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Initializes the application configuration from environment variables.
/// Should be called only once at the beginning of the program.
pub fn try_init_app_config() -> Result<()> {
    let settings = Config::builder()
        .add_source(Environment::with_prefix("LUATALK").separator("__"))
        .build()
        .into_diagnostic()?;

    let app_config: AppConfig = settings.try_deserialize().into_diagnostic()?;

    APP_CONFIG
        .set(app_config)
        .map_err(|_| miette::miette!("try_init should not be called after initialized"))?;

    Ok(())
}

/// Returns the application configuration.
pub fn app_config() -> &'static AppConfig {
    APP_CONFIG
        .get()
        .or_else(|| {
            panic!(
                "{}",
                miette!("static OnceLock APP_CONFIG has to be initialized")
            )
        })
        .unwrap()
}

/// Application configuration for advanced settings.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    do_lua_input_config: DoLuaInputConfig,
}

/// Configuration for processing Lua input file.
#[derive(Debug, Clone, Deserialize)]
pub struct DoLuaInputConfig {
    /// To disable loading the `talk.lua` module.
    no_default_lib: bool,

    /// Additional Lua search paths which will be appended before the default search paths
    addtional_path: String,
}
