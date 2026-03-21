use crate::config::Config;
use colored::Colorize;

pub fn run(cfg: &Config) {
    println!("wux commands:\n");

    println!("Built-in:");
    println!("  {:15} {}", "free <port>", "Kill process on port".dimmed());
    println!(
        "  {:15} {}",
        "nuke <path>",
        "Delete file/directory".dimmed()
    );
    println!("  {:15} {}", "whereis <file>", "Find a file anywhere".dimmed());
    println!("  {:15} {}", "info", "Show directory info".dimmed());
    println!();
    println!("  {:15} {}", "config", "Open wux.toml".dimmed());
    println!("  {:15} {}", "help", "Show this help".dimmed());
    println!("  {:15} {}", "list", "List all commands".dimmed());
    println!("  {:15} {}", "update", "Update wux".dimmed());

    if !cfg.commands.custom().is_empty() {
        println!("\nCustom:");
        for (name, meta) in cfg.commands.custom() {
            let run_str = match &meta.run {
                crate::config::CommandRun::Single(s) => s.clone(),
                crate::config::CommandRun::Multiple(v) => v.join(" && "),
            };
            if meta.description.is_empty() {
                println!("  {:15} {}", name, run_str.dimmed());
            } else {
                println!("  {:15} {}", name, meta.description.dimmed());
                println!("               → {}", run_str.dimmed());
            }
        }
    }
}
