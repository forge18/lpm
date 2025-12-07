use lpm::core::LpmResult;
use lpm::core::path::find_project_root;
use lpm::path_setup::{LuaRunner, RunOptions};
use std::env;

pub fn run(command: Vec<String>) -> LpmResult<()> {
    if command.is_empty() {
        return Err(lpm::core::LpmError::Package(
            "No command provided".to_string(),
        ));
    }

    let current_dir = env::current_dir()?;
    // Verify we're in a project (for error checking)
    find_project_root(&current_dir)?;

    // Join command parts into a single command string
    let command_str = command.join(" ");

    // Execute command with automatic path setup
    let exit_code = LuaRunner::exec_command(&command_str, RunOptions::default())?;
    std::process::exit(exit_code);
}
