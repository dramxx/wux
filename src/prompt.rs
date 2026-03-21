use colored::Colorize;
use std::io::{self, Write};

pub fn confirm(prompt: &str) -> bool {
    print!("{} {} ", "?".cyan(), prompt);
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    let input = input.trim();
    input.eq_ignore_ascii_case("y")
}
