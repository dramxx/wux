use crate::config::get_config_path;
use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if !config_path.exists() {
        let default_config = r#"# wux configuration file
# https://github.com/yourusername/wux

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

fn open_in_editor(path: &PathBuf) -> Result<()> {
    std::process::Command::new("notepad.exe")
        .arg(path)
        .spawn()?;

    Ok(())
}
