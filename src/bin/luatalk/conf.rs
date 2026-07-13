use std::sync::{LazyLock, OnceLock};

use config::{Config, Environment};
use getset::Getters;
use log::debug;
use miette::{IntoDiagnostic, Result, miette};
use serde::Deserialize;

use crate::locale::SupportedLang;

static LANG: LazyLock<SupportedLang> = LazyLock::new(|| {
    sys_locale::get_locale()
        .map(|l| l.as_str().into())
        .unwrap_or_default()
});

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn lang() -> SupportedLang {
    *LANG
}

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

    #[getset(get = "pub")]
    #[serde(default)]
    do_json: DoJsonConfig,

    #[getset(get = "pub")]
    #[serde(default)]
    do_typst_compile: DoTypstCompileConfig,
}

/// Configuration for `do <INPUT>` command.
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

/// Configuration for `do <INPUT> json` command,
/// and also the commands which outputs JSON,
/// e.g. `do <INPUT> typst-compile`.
#[derive(Debug, Clone, Default, Deserialize, Getters)]
pub struct DoJsonConfig {
    /// To minify the JSON output.
    /// e.g. `LUATALK__DO_JSON__MINIFY=1`
    #[getset(get = "pub")]
    #[serde(default)]
    minify: bool,
}

/// Configuration for `do <INPUT> typst-compile` command.
#[derive(Debug, Clone, Default, Deserialize, Getters)]
pub struct DoTypstCompileConfig {
    /// The command to run typst-cli. Leave empty to use `typst`.
    /// e.g. `LUATALK__DO_TYPST_COMPILE__TYPST_COMMAND='/path/to/typst-cli-dir/bin/typst'`
    #[getset(get = "pub")]
    #[serde(default)]
    typst_command: String,
}
