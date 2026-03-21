use crate::config::get_config_path;
use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn run() -> Result<()> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if !config_path.exists() {
        let default_config = r#"# wux configuration file
# https://github.com/dramxx/wux

[settings]
color = true

# Built-in command safety overrides.
# safe = true  → command runs immediately with no confirmation prompt
# safe = false → user is asked to confirm before the command executes

[commands.free]
safe = true

[commands.nuke]
safe = false
"#;
        fs::write(&config_path, default_config)?;
    }

    println!("{} Opening wux.toml ...", "→".cyan());
    open_in_editor(&config_path)?;

    Ok(())
}

#[cfg(windows)]
fn open_in_editor(path: &PathBuf) -> Result<()> {
    Command::new("notepad.exe").arg(path).spawn()?;

    Ok(())
}

#[cfg(unix)]
fn open_in_editor(path: &PathBuf) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    Command::new(editor).arg(path).spawn()?;

    Ok(())
}
