/// Common errors.

#[derive(Debug, thiserror::Error)]
pub enum LuaParseError {
    #[error("Failed to evaluate Lua code: {0}")]
    Eval(#[source] mlua::Error),

    #[error("Failed to deserialize Lua code: {0}")]
    Deserialize(#[source] mlua::Error),
}
