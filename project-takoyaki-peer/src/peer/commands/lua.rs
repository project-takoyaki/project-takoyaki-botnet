use anyhow::{anyhow, Result};
use mlua::Lua;
use obfstr::obfstr;

use super::Command;

pub struct LuaCommand {
  lua: Lua,
}

impl Command for LuaCommand {
  fn new() -> Result<Self> {
    let lua = Lua::new();

    Ok(Self { lua })
  }

  fn execute(&self, args: &[String]) -> Result<()> {
    let script = args.get(0).ok_or_else(|| anyhow!("{}", obfstr!("Missing required argument: 'script'")))?;

    self.lua.load(script).exec()?;

    Ok(())
  }
}
