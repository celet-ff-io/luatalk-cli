use const_format::formatcp;
use log::debug;
use mlua::{Lua, Table, Value};

use crate::error::LuaParseError;

pub trait LuaTalkExt {
    /// Load the default Lua library `talk.lua` into the Lua state.
    fn load_default_lib(&self) -> Result<(), LuaTalkLibError>;
}

impl LuaTalkExt for Lua {
    fn load_default_lib(&self) -> Result<(), LuaTalkLibError> {
        let loaded_modules: Table = self
            .load("package.loaded")
            .eval()
            .map_err(LuaParseError::Eval)
            .map_err(LuaTalkLibError::Eval)?;

        const LMOD_TALK: &str = "talk";

        let lib_module: Value = self
            .load(include_str!("../assets/lua/lib/talk.lua"))
            .set_name(formatcp!("{LMOD_TALK}.lua"))
            .eval()
            .map_err(LuaParseError::Eval)
            .map_err(LuaTalkLibError::Eval)?;

        loaded_modules
            .set(LMOD_TALK, lib_module)
            .map_err(LuaTalkLibError::ModuleLoadFailed)?;

        debug!("`{LMOD_TALK}` loaded");

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LuaTalkLibError {
    #[error("Failed to evaluate Lua library code: {0}")]
    Eval(#[source] LuaParseError),

    #[error("Failed to load Lua library module: {0}")]
    ModuleLoadFailed(#[source] mlua::Error),
}
