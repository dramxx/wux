use crate::config::CommandMeta;
use anyhow::Result;
use colored::Colorize;
use std::io::Write;
use std::process::Command;

pub fn run(name: &str, meta: &CommandMeta, dry_run: bool, skip_prompt: bool) -> Result<()> {
    if dry_run {
        println!(
            "{} Would run custom command '{}':",
            "→".cyan(),
            name.yellow()
        );
        for cmd in meta.run.iter() {
            println!("  > {}", cmd);
        }
        return Ok(());
    }

    if !skip_prompt && !meta.safe {
        let description = if meta.description.is_empty() {
            format!("Run custom command '{}'", name)
        } else {
            meta.description.clone()
        };

        print!("{} {} [y/N]: ", "?".cyan(), description);
        std::io::stdout().flush().ok();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            return Ok(());
        }

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    for cmd in meta.run.iter() {
        println!("{} Running: {}", "→".cyan(), cmd);
        run_single_command(cmd)?;
    }

    println!("{} Done.", "✓".green());
    Ok(())
}

fn run_single_command(cmd: &str) -> Result<()> {
    #[cfg(windows)]
    {
        if cmd.contains(' ') && !cmd.starts_with("cmd") && !cmd.starts_with("powershell") {
            let child = Command::new("cmd").args(["/C", cmd]).spawn()?;

            let output = child.wait_with_output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    eprintln!("{} Error: {}", "✗".red(), stderr.trim());
                }
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                print!("{}", stdout);
            }
        } else {
            let mut parts = cmd.split_whitespace();
            let program = parts.next().unwrap_or(cmd);
            let child = Command::new(program).args(parts).spawn()?;

            let output = child.wait_with_output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    eprintln!("{} Error: {}", "✗".red(), stderr.trim());
                }
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                print!("{}", stdout);
            }
        };
    }

    #[cfg(not(windows))]
    {
        let child = Command::new("sh").args(["-c", cmd]).spawn()?;
        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                eprintln!("{} Error: {}", "✗".red(), stderr.trim());
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            print!("{}", stdout);
        }
    }

    Ok(())
}
