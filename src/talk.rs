use const_format::formatcp;
use log::debug;
use mlua::{Lua, Table, Value};

use crate::error::LuaParseError;

pub trait LuaExt {
    /// Load the default Lua library `talk.lua` into the Lua state.
    fn load_default_lib(&self) -> Result<(), LuaTalkLibError>;

    /// Append additional Lua search path in front of the current package.path.
    fn append_left_additional_path(&self, path: &str) -> Result<(), LuaTalkLibError>;
}

impl LuaExt for Lua {
    fn load_default_lib(&self) -> Result<(), LuaTalkLibError> {
        let loaded_modules: Table = self
            .load("package.loaded")
            .eval()
            .map_err(LuaParseError::Eval)
            .map_err(LuaTalkLibError::Eval)?;

        const LMOD_TALK: &str = "talk";

        let lib_module: Value = self
            .load(crate::assets::lua::lib::talk())
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

    fn append_left_additional_path(&self, path: &str) -> Result<(), LuaTalkLibError> {
        const KEY_PATH: &str = "path";
        let pacakges: Table = self
            .globals()
            .get("package")
            .map_err(LuaTalkLibError::ModuleLoadFailed)?;
        let current_path: String = pacakges
            .get(KEY_PATH)
            .map_err(LuaTalkLibError::GetPackagePathFailed)?;
        let new_path = format!("{path}{current_path}");

        debug!("Lua package path will update to {new_path}");

        pacakges
            .set(KEY_PATH, new_path)
            .map_err(LuaTalkLibError::SetPackagePathFailed)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LuaTalkLibError {
    #[error("Failed to evaluate Lua library code: {0}")]
    Eval(#[source] LuaParseError),

    #[error("Failed to load Lua library module: {0}")]
    ModuleLoadFailed(#[source] mlua::Error),

    #[error("Failed to get global package table: {0}")]
    GetGlobalPackageFailed(#[source] mlua::Error),

    #[error("Failed to get package path: {0}")]
    GetPackagePathFailed(#[source] mlua::Error),

    #[error("Failed to set package path: {0}")]
    SetPackagePathFailed(#[source] mlua::Error),
}
