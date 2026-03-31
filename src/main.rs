use colored::Colorize;
use wux::commands;
use wux::config;

fn main() -> anyhow::Result<()> {
    let cfg = config::load()?;

    let args: Vec<String> = std::env::args().collect();

    let mut dry_run = false;
    let mut yes = false;

    let filtered_args: Vec<&String> = args
        .iter()
        .filter(|a| match a.as_str() {
            "--dry-run" => {
                dry_run = true;
                false
            }
            "-y" | "--yes" => {
                yes = true;
                false
            }
            _ => true,
        })
        .collect();

    let cmd = filtered_args.get(1).map(|s| s.as_str());

    match cmd {
        Some("free") => {
            let port: u16 = filtered_args
                .get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(80);
            let safe = cfg.commands.free.safe;
            commands::free::run(port, dry_run, safe || yes)
        }
        Some("nuke") => {
            let path = filtered_args.get(2).map(|s| s.as_str()).unwrap_or_default();
            let safe = cfg.commands.nuke.safe;
            commands::nuke::run(path, dry_run, safe || yes)
        }
        Some("config") => commands::config_cmd::run(),
        Some("list") => {
            commands::list::run(&cfg);
            Ok(())
        }
        Some("update") => commands::update::run(),
        Some("info") => commands::info::run(),
        Some("whereis") => {
            let file_name = filtered_args.get(2).map(|s| s.as_str()).unwrap_or_default();
            commands::whereis::run(file_name)
        }
        Some("dockersafe") => commands::docker::dockersafe(),
        Some("dockerrun") => commands::docker::dockerrun(),
        Some("help") | None => {
            print_help(&cfg);
            Ok(())
        }
        Some(c) => {
            if let Some(meta) = cfg.commands.custom().get(c) {
                commands::custom::run(c, meta, dry_run, yes || meta.safe)
            } else {
                println!("Unknown command: {}", c.red());
                println!("Run 'wux help' or 'wux list' for available commands.");
                Ok(())
            }
        }
    }
}

fn print_help(cfg: &config::Config) {
    println!("wux - Your personal command toolkit\n");
    println!("Commands:");
    println!("  free <port>    Kill process on port");
    println!("  nuke <path>    Delete file/directory");
    println!("  whereis <file> Find a file anywhere on the filesystem");
    println!("  info           Show directory info");
    println!("  dockersafe     Spin up a read-only, no-network container");
    println!("  dockerrun      Spin up a writable, networked container");
    println!();
    println!("  config         Open wux.toml");
    println!("  list           List all commands");
    println!("  update         Update wux");
    println!("  help           Show this help\n");

    if !cfg.commands.custom().is_empty() {
        println!("Custom commands:");
        for name in cfg.commands.custom().keys() {
            println!("  {}", name);
        }
    }
}
