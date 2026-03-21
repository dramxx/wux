use crate::platform;
use anyhow::Result;
use colored::Colorize;

pub fn run(port: u16, dry_run: bool, safe: bool) -> Result<()> {
    let result = platform::find_pid_on_port(port)?;

    match result {
        Some((pid, process_name)) => {
            println!(
                "{} Found: {} (PID {}) on port {}",
                "→".cyan(),
                process_name.yellow(),
                pid.to_string().yellow(),
                port
            );

            if dry_run {
                println!("{} Would kill PID {}", "→".cyan(), pid);
                return Ok(());
            }

            if safe {
                println!("{} Killing process...", "→".cyan());
            }

            platform::kill_pid(pid)?;

            println!(
                "{} Killed {} (PID {}). Port {} is now free.",
                "✓".green(),
                process_name.green(),
                pid.to_string().green(),
                port
            );
        }
        None => {
            println!("{} Nothing is running on port {}.", "⚠".yellow(), port);
        }
    }

    Ok(())
}
