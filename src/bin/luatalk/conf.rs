use std::sync::OnceLock;

use config::{Config, Environment};
use getset::Getters;
use log::debug;
use miette::{IntoDiagnostic, Result, miette};
use serde::Deserialize;

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Initializes the application configuration from environment variables.
/// Should be called only once at the beginning of the program.
pub fn try_init_app_config() -> Result<()> {
    debug!("Initializing application configuration from environment variables...");

    let settings = Config::builder()
        .add_source(Environment::with_prefix("LUATALK").separator("__"))
        .build()
        .into_diagnostic()?;

    let app_config: AppConfig = settings.try_deserialize().into_diagnostic()?;

    debug!("Loading config: {:#?}", app_config);

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
/// Specified with environment variables with prefix `LUATALK__`.
#[derive(Debug, Clone, Deserialize, Getters)]
pub struct AppConfig {
    #[getset(get = "pub")]
    #[serde(default)]
    do_lua: DoLuaConfig,
}

/// Configuration for processing Lua input file.
#[derive(Debug, Clone, Default, Deserialize, Getters)]
pub struct DoLuaConfig {
    /// To disable loading the `talk.lua` module.
    /// e.g. `LUATALK__DO_LUA__NO_DEFAULT_LIB=1`
    #[getset(get = "pub")]
    #[serde(default)]
    no_default_lib: bool,

    /// Additional Lua search paths which will be appended before the default search paths.
    /// e.g. `LUATALK__DO_LUA__ADDITIONAL_PATH='/path/to/lib/?.lua;/path/to/lib/?/init.lua;'`
    #[getset(get = "pub")]
    #[serde(default)]
    additional_path: String,
}
