mod lua;

use std::collections::HashMap;

use anyhow::Result;
use log::{error, info, warn};
use obfstr::obfstring;

macro_rules! register_command {
  ($command_handler:expr, $command_name:expr, $command:ty) => {{
    $command_handler.commands.insert($command_name.to_string(), Box::new(<$command>::new()?));
  }};
}

pub trait Command {
  fn new() -> Result<Self>
  where
    Self: Sized;

  fn execute(&self, args: &[String]) -> Result<()>;
}

pub struct CommandHandler {
  commands: HashMap<String, Box<dyn Command>>,
}

impl CommandHandler {
  pub fn new() -> Self {
    Self { commands: HashMap::new() }
  }

  pub fn execute_command(&self, command: &[String]) {
    let (command_name, command_args) = match command.split_first() {
      Some((command_name, command_args)) => (command_name.to_lowercase(), command_args),
      None => {
        warn!("Attempted to execute an empty command.");
        return;
      }
    };

    if self.command_exists(&command_name) {
      match self.commands.get(&command_name).unwrap().execute(command_args) {
        Ok(_) => info!("Executed command {command_name:?} with arguments: {command_args:?}."),
        Err(error) => error!("Execution failed for command {command_name:?} with arguments {command_args:?}: {error:?}"),
      }
    } else {
      warn!("Attempted to execute an unregistered command {command_name:?}.");
    }
  }

  pub fn command_exists(&self, command_name: &str) -> bool {
    self.commands.contains_key(&command_name.to_lowercase())
  }

  pub fn register_commands(&mut self) -> Result<()> {
    register_command!(self, obfstring!("lua"), lua::LuaCommand);

    Ok(())
  }
}
