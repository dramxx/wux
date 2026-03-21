use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub commands: CommandsConfig,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub color: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { color: true }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Default)]
pub struct CommandsConfig {
    pub free: BuiltinMeta,
    pub nuke: BuiltinMeta,
    #[serde(flatten, default)]
    custom: std::collections::HashMap<String, CommandMeta>,
}

impl CommandsConfig {
    pub fn custom(&self) -> &std::collections::HashMap<String, CommandMeta> {
        &self.custom
    }

    pub fn take_custom(&mut self) -> std::collections::HashMap<String, CommandMeta> {
        std::mem::take(&mut self.custom)
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct BuiltinMeta {
    #[serde(default)]
    pub safe: bool,
}

#[derive(Deserialize, Clone)]
pub struct CommandMeta {
    pub run: CommandRun,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_safe_true")]
    pub safe: bool,
}

fn default_safe_true() -> bool {
    true
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum CommandRun {
    Single(String),
    Multiple(Vec<String>),
}

impl CommandRun {
    pub fn iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            CommandRun::Single(s) => Box::new(vec![s.as_str()].into_iter()),
            CommandRun::Multiple(v) => Box::new(v.iter().map(|s| s.as_str())),
        }
    }
}

fn config_path() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string()))
        .join("wux")
        .join("wux.toml")
}

fn is_reserved_name(name: &str) -> bool {
    matches!(
        name,
        "free" | "nuke" | "config" | "help" | "list" | "update"
    )
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path).context("Failed to read config file")?;

    let mut config: Config = toml::from_str(&content).unwrap_or_else(|e| {
        eprintln!("{} Failed to parse config: {}", "⚠".yellow(), e);
        Config::default()
    });

    let custom = config.commands.take_custom();
    let mut filtered = std::collections::HashMap::new();
    for (name, meta) in custom {
        if is_reserved_name(&name) {
            eprintln!(
                "{} Warning: '{}' is a reserved command name. Skipping.",
                "⚠".yellow(),
                name
            );
            continue;
        }
        filtered.insert(name, meta);
    }
    config.commands.custom = filtered;

    let root: toml::Value = content
        .parse()
        .unwrap_or_else(|_| toml::Value::Boolean(false));
    let free_has_explicit_safe = root
        .get("commands")
        .and_then(|v| v.get("free"))
        .and_then(|v| v.get("safe"))
        .is_some();
    if !free_has_explicit_safe {
        config.commands.free = BuiltinMeta { safe: true };
    }

    Ok(config)
}

pub fn get_config_path() -> PathBuf {
    config_path()
}
