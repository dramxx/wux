use wux::config::Config;

#[test]
fn nuke_safe_default_is_false() {
    let cfg = wux::config::load().unwrap();
    assert!(!cfg.commands.nuke.safe, "nuke should default to safe=false");
}

#[test]
fn free_safe_default_is_true() {
    let cfg = wux::config::load().unwrap();
    assert!(cfg.commands.free.safe, "free should default to safe=true");
}

#[test]
fn parse_full_config() {
    let toml_str = r#"
[settings]
color = true

[commands.free]
safe = true

[commands.nuke]
safe = false
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.settings.color);
    assert!(config.commands.free.safe);
    assert!(!config.commands.nuke.safe);
}

#[test]
fn config_load_has_correct_defaults() {
    let cfg = wux::config::load().unwrap();
    assert!(cfg.settings.color);
    assert!(cfg.commands.free.safe);
    assert!(!cfg.commands.nuke.safe);
}
