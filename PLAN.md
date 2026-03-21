# wux — Development Plan

## What Is wux?

`wux` is a personal CLI utility written in Rust that wraps common but annoyingly verbose
system operations into short, memorable commands. Instead of remembering platform-specific
PowerShell or bash syntax, you type `wux <command> [args]` and it handles the rest.

The binary is placed on PATH at install time and works immediately in any new terminal session.

---

## Project Structure

```
wux/
├── Cargo.toml
├── wux.toml                       # User config — command safety + custom commands
├── src/
│   ├── main.rs                    # Entry point, CLI parsing via clap
│   ├── config.rs                  # Load and parse wux.toml
│   ├── prompt.rs                  # Shared confirmation prompt helper
│   ├── commands/
│   │   ├── mod.rs                 # Command dispatcher
│   │   ├── free.rs                # `wux free <port>` implementation
│   │   ├── nuke.rs                # `wux nuke <path>` implementation
│   │   └── config_cmd.rs          # `wux config` — open wux.toml in editor
│   └── platform.rs                # OS detection + privilege helpers
├── tests/
│   ├── nuke_tests.rs              # Integration tests for nuke command
│   ├── free_tests.rs              # Integration tests for free command
│   └── config_tests.rs            # Config parsing tests
├── install/
│   ├── install.ps1                # Windows installer
│   └── install.sh                 # Unix installer
└── README.md
```

---

## Tech Stack

| Concern | Crate | Why |
|---|---|---|
| CLI argument parsing | `clap` (derive API) | Industry standard, auto-generates help text |
| Config file parsing | `toml` + `serde` | Clean TOML deserialization into Rust structs |
| Process/port inspection | `sysinfo` | Cross-platform, no shell-out needed for process listing |
| Error handling | `anyhow` | Ergonomic error propagation with context messages |
| Terminal output | `colored` | Colored status messages (green OK, red ERROR, yellow WARN) |

---

## Cargo.toml (Dependencies)

```toml
[package]
name = "wux"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "wux"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
sysinfo = "0.30"
anyhow = "1"
colored = "2"
```

---

## CLI Interface (clap structure)

```
wux <COMMAND> [ARGS] [FLAGS]

Commands:
  free <port>       Kill whatever process is sitting on <port>
  nuke <path>       Delete <path> and all its contents recursively
  config            Open wux.toml in your default text editor
  help              Print help for a command

Flags:
  --dry-run         Print what would happen without doing it
  --yes             Skip confirmation prompts (overrides safe = false)
  --version         Print wux version
```

### Examples

```
wux free 80
wux free 3000 --yes
wux nuke ./dist
wux nuke ./node_modules --dry-run
wux nuke ./dist --yes
wux free 8080 --dry-run
wux config
```

---

## Command Implementations

---

### `wux free <port>`

**Goal:** Find the process occupying a given TCP port and kill it.

**File:** `src/commands/free.rs`

**Safety:** `safe = true` by default. Runs without confirmation prompt.

#### Logic Flow

```
1. Parse <port> as u16 — error clearly if not a valid port number
2. Use sysinfo to get all running processes
3. On Windows: shell out to `netstat -ano` and parse for the port + PID
   On Unix:    shell out to `lsof -ti :<port>` or parse /proc/net/tcp
4. If no process found → print "Nothing is running on port <port>" and exit 0
5. If found → print "Found: <process_name> (PID <pid>) on port <port>"
6. If --dry-run → print "Would kill PID <pid>" and exit 0
7. safe = true, so no confirmation prompt — proceed directly
8. Kill the process:
   - Windows: `taskkill /PID <pid> /F`
   - Unix:    `kill -9 <pid>`
9. Confirm: "✓ Killed <process_name> (PID <pid>). Port <port> is now free."
```

#### Windows Port Lookup Detail

PowerShell equivalent being automated:
```powershell
# Step 1 — find PID on port
netstat -ano | findstr :<port>
# Step 2 — kill it
taskkill /PID <pid> /F
```

Parse `netstat -ano` output: lines look like:
```
  TCP    0.0.0.0:80    0.0.0.0:0    LISTENING    1234
```
Split by whitespace, check column 1 ends with `:<port>`, extract last column as PID.

#### Unix Port Lookup Detail

Shell equivalent being automated:
```bash
lsof -ti :<port> | xargs kill -9
```

#### Error Cases to Handle

- Port not a number → `"'abc' is not a valid port number"`
- Port out of range (> 65535) → `"Port must be between 1 and 65535"`
- Nothing on port → `"Nothing is running on port 80"` (exit 0, not an error)
- Process already dead between find and kill → swallow ESRCH, print warning
- Permission denied killing process → `"Permission denied. Try running wux as administrator."`

---

### `wux nuke <path>`

**Goal:** Recursively delete a file or directory with no survivors.

**File:** `src/commands/nuke.rs`

**Safety:** `safe = false` by default. Always prompts for confirmation unless `--yes` is passed.

#### Logic Flow

```
1. Resolve <path> to absolute path (relative paths resolved from CWD)
2. Check path exists — if not, print "Nothing at '<path>' — already gone?" and exit 0
3. Stat the path — is it a file or directory?
4. If --dry-run:
   a. If directory: walk and count files/subdirs, print summary
      "Would delete directory: <path> (42 files, 7 subdirectories)"
   b. If file: "Would delete file: <path>"
   c. Exit 0 — no prompt needed for dry run
5. safe = false, so unless --yes is set, prompt:
   a. Directory → "⚠  Do you really want to remove '<path>' and all its contents? [y/N]"
   b. File      → "⚠  Do you really want to delete the file '<path>'? [y/N]"
   If user types anything other than 'y' or 'Y' → print "Aborted." and exit 0
6. Perform deletion:
   a. Directory → std::fs::remove_dir_all(<path>)
   b. File → std::fs::remove_file(<path>)
7. Confirm: "✓ Nuked. <path> is gone."
```

#### Windows Equivalent Being Automated

```powershell
Remove-Item -Recurse -Force ./some-folder
```

#### Unix Equivalent Being Automated

```bash
rm -rf ./some-folder
```

Because `std::fs::remove_dir_all` is cross-platform, no shell-out is needed here —
this one is pure Rust.

#### Error Cases to Handle

- Path does not exist → `"Nothing at '<path>' — already gone?"` (exit 0)
- Path resolves to filesystem root (`/`, `C:\`) → **hard refuse**, print error, never delete
- Path is current working directory → refuse with warning
- Partial deletion failure (permissions mid-tree) → surface `anyhow` error with path context
- Symlink → delete the symlink itself, not what it points to (use `symlink_metadata`)

#### Safety Guard — Root/CWD Check

```rust
// Pseudo-code — implement in nuke.rs
let abs = canonicalize(path)?;
let cwd = std::env::current_dir()?;

if abs == cwd {
    bail!("Refusing to nuke your current working directory.");
}
if abs.parent().is_none() {
    bail!("Refusing to nuke a filesystem root. That would be bad.");
}
```

---

## Config System (`wux.toml`)

The config file lives at `~/.config/wux/wux.toml` (Unix) or `%APPDATA%\wux\wux.toml` (Windows).

### Schema

```toml
[settings]
color = true                  # Colored terminal output

# Built-in command safety overrides.
# safe = true  → command runs immediately with no confirmation prompt
# safe = false → user is asked to confirm before the command executes
[commands.free]
safe = true

[commands.nuke]
safe = false

# Phase 2: user-defined command aliases
# [commands.serve]
# description = "Start dev server"
# run = "npm run dev"
# safe = true
```

### The `safe` Flag — How It Works

Each command has a `safe` boolean in `wux.toml`. The rule is simple:

- `safe = true` → execute immediately, no prompt
- `safe = false` → before executing, ask:
  `"⚠  Do you really want to remove './folder' and all its contents? [y/N]"`
  If the user types anything other than `y` or `Y`, abort and print `"Aborted."`.

The `--yes` flag bypasses the safety prompt regardless of the `safe` setting.
The `--dry-run` flag also bypasses it (nothing is being destroyed).

**Defaults if `wux.toml` is missing or the key is absent:**
- `free` defaults to `safe = true`
- `nuke` defaults to `safe = false`

Never error on a missing config file or missing key — fall back to the hardcoded default.

### Config Loading (`src/config.rs`)

```rust
#[derive(Deserialize, Default)]
pub struct Config {
    pub settings: Settings,
    pub commands: CommandsConfig,
}

#[derive(Deserialize, Default)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub color: bool,
}

#[derive(Deserialize, Default)]
pub struct CommandsConfig {
    pub free: CommandMeta,
    pub nuke: CommandMeta,
}

#[derive(Deserialize)]
pub struct CommandMeta {
    pub safe: bool,
    // Phase 2 fields: run, description, args
}

impl Default for CommandMeta {
    fn default() -> Self {
        // Callers must set the correct per-command default, not a blanket one.
        // Use CommandMeta::for_builtin(name) helper instead.
        Self { safe: true }
    }
}

impl CommandMeta {
    /// Returns the correct default for each built-in command by name.
    pub fn default_for(name: &str) -> Self {
        match name {
            "nuke" => Self { safe: false },
            _      => Self { safe: true },
        }
    }
}
```

Config is loaded once at startup. If the file does not exist, defaults are used silently —
never error on missing config.

---

### `wux config`

**Goal:** Open `wux.toml` in the user's default text editor so they can tweak settings
and add custom commands without hunting for the file path.

**File:** `src/commands/config_cmd.rs`

**Naming note:** `config` is preferred over `settings` — it's the standard convention in
CLI tools (git config, npm config, cargo config) and implies "the file that configures
this tool" rather than a settings UI.

#### Logic Flow

```
1. Resolve config file path (~/.config/wux/wux.toml or %APPDATA%\wux\wux.toml)
2. If file does not exist → create it with default contents first, then open
3. Open the file:
   - Windows: `notepad.exe <path>`  (reliable, always present)
   - Unix:    respect $EDITOR env var, fall back to `nano`, then `vi`
4. Print: "→ Opening wux.toml ..."
5. Do NOT wait for the editor to close — spawn detached so the terminal is free
```

#### Windows Detail

```rust
std::process::Command::new("notepad.exe")
    .arg(&config_path)
    .spawn()?;
```

Notepad is always present on Windows and opens fast. No need to respect `$EDITOR` on
Windows — this is a personal tool and notepad is the right default for a TOML file.

---



All output goes to stdout. Errors go to stderr.

```
✓  Success — green
✗  Error — red  
→  Info/action — default color
⚠  Warning — yellow
?  Prompt — cyan
```

Examples:
```
→ Found: node.exe (PID 18234) on port 3000
? Kill it? [y/N]: y
✓ Killed node.exe (PID 18234). Port 3000 is now free.
```

```
→ Scanning ./node_modules ...
? Delete directory './node_modules' and ALL its contents? [y/N]: y
✓ Nuked. ./node_modules is gone.
```

```
⚠  Nothing is running on port 80.
```

---

## Platform Handling (`src/platform.rs`)

```rust
pub enum Os { Windows, Unix }

pub fn current_os() -> Os {
    if cfg!(target_os = "windows") { Os::Windows } else { Os::Unix }
}

pub fn find_pid_on_port(port: u16) -> anyhow::Result<Option<(u32, String)>>
// Returns (pid, process_name) or None

pub fn kill_pid(pid: u32) -> anyhow::Result<()>
// Calls taskkill on Windows, kill -9 on Unix
```

All platform branching lives in this module. Command implementations in `commands/`
call platform functions and never `#[cfg(windows)]` themselves.

---

## main.rs Skeleton

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wux", about = "Your personal command toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Print what would happen without doing it
    #[arg(long, global = true)]
    dry_run: bool,

    /// Skip confirmation prompts (overrides safe = false)
    #[arg(long, short = 'y', global = true)]
    yes: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Kill whatever process is sitting on <port>
    Free { port: u16 },
    /// Delete <path> and all its contents recursively
    Nuke { path: String },
    /// Open wux.toml in your default text editor
    Config,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = config::load()?;

    match cli.command {
        Commands::Free { port } => {
            let safe = config.commands.free.safe;
            commands::free::run(port, cli.dry_run, safe || cli.yes)
        }
        Commands::Nuke { path } => {
            let safe = config.commands.nuke.safe;
            commands::nuke::run(&path, cli.dry_run, safe || cli.yes)
        }
        Commands::Config => {
            commands::config_cmd::run()
        }
    }
}
```

---

## Install Flow

### Windows (`install/install.ps1`)

1. `cargo build --release`
2. Copy `target/release/wux.exe` to `C:\Users\<user>\.wux\bin\`
3. Add that directory to user `PATH` via registry
4. Create default `%APPDATA%\wux\wux.toml` if not present
5. Print: `wux installed. Open a new terminal and run 'wux --version' to confirm.`

> **Note:** No PowerShell `$PROFILE` injection needed — once `wux.exe` is on PATH,
> it is available in every terminal session automatically.

### Unix (`install/install.sh`)

1. `cargo build --release`
2. Copy binary to `~/.local/bin/wux` (or `/usr/local/bin/` with sudo)
3. Create `~/.config/wux/wux.toml` if not present
4. Print install confirmation

---

## Build Phases

### Phase 1 — Core (build this first)
- [ ] Scaffold Rust project, Cargo.toml with all deps
- [ ] `main.rs` with clap CLI structure (`free`, `nuke`, `config`)
- [ ] `platform.rs` with OS detection stubs
- [ ] `config.rs` with `safe` flag per command + defaults
- [ ] `prompt.rs` shared confirmation helper (reads stdin, handles y/N)
- [ ] `wux nuke <path>` — full implementation with safety guards + `safe` prompt
- [ ] `wux free <port>` — full implementation, Windows + Unix, no prompt (safe)
- [ ] `wux config` — opens `wux.toml` in notepad (Windows) or $EDITOR (Unix)
- [ ] Output formatting with `colored`
- [ ] `--dry-run` and `--yes` flags wired through all commands
- [ ] Tests (see Testing section below)
- [ ] `install.ps1` for Windows
- [ ] `README.md` (see README section below)

### Phase 2 — Config-driven custom commands (next)
- [ ] `[commands.*]` section in `wux.toml` with `run`, `description`, `safe` fields
- [ ] Custom command dispatcher (reads `run = "..."`, executes in shell)
- [ ] `wux list` — print all available built-in + custom commands
- [ ] `wux add` — interactive prompt to append a new entry to `wux.toml`

### Phase 3 — Quality of life
- [ ] Shell completions (PowerShell + bash + zsh) via `clap_complete`
- [ ] `wux update` — self-update by pulling latest binary from GitHub releases
- [ ] Man page generation

---

## Testing

Write tests in `tests/`. Use `#[cfg(test)]` unit tests inside source files for pure logic,
and integration tests in `tests/` for file system and process behavior.

### `tests/config_tests.rs`

```
- parse_full_config: load a complete wux.toml string, assert all fields correct
- parse_missing_safe_key: omit [commands.nuke] safe field, assert it defaults to false
- parse_empty_file: empty TOML file loads with all defaults, no panic
- parse_missing_file: config::load() on nonexistent path returns Ok with defaults
- nuke_safe_default_is_false: CommandMeta::default_for("nuke").safe == false
- free_safe_default_is_true: CommandMeta::default_for("free").safe == true
```

### `tests/nuke_tests.rs`

```
- nuke_deletes_file: create temp file, run nuke with --yes, assert file gone
- nuke_deletes_directory: create temp dir with nested files, run nuke --yes, assert gone
- nuke_dry_run_does_not_delete: run with --dry-run, assert path still exists after
- nuke_aborts_on_nonexistent_path: path doesn't exist, exits 0 with friendly message
- nuke_refuses_cwd: pass current dir as path, assert hard refusal error
- nuke_refuses_root_windows: pass "C:\\" on Windows, assert hard refusal
- nuke_refuses_root_unix: pass "/" on Unix, assert hard refusal
- nuke_prompt_abort: safe=false, --yes not set, simulate 'n' input, assert nothing deleted
```

Use `tempfile` crate for creating temporary files/dirs in tests.
Add to `[dev-dependencies]` in Cargo.toml: `tempfile = "3"`.

### `tests/free_tests.rs`

```
- free_invalid_port_string: pass "abc" as port, assert error message
- free_dry_run_no_kill: dry-run on any port, assert no process is killed
- free_nothing_on_port: query a port guaranteed to be empty, assert friendly "nothing running" message
```

Note: tests that actually kill processes are skipped in CI (mark with `#[ignore]`).

---

## README.md

**The implementing model must write `README.md` as the final step after all code is complete.**

The README should be user-oriented, not developer-oriented. It should cover:

### Sections to include

1. **What is wux** — one paragraph, plain English. What problem it solves.

2. **Installation**
   - Prerequisites: Rust + Cargo installed
   - Clone the repo
   - Run `install.ps1` (Windows) or `install.sh` (Unix)
   - Verify: `wux --version`

3. **Commands**
   - Short table: command, what it does, example
   - `wux free <port>` — kills the process on that port
   - `wux nuke <path>` — deletes the folder/file recursively (prompts first)
   - `wux config` — opens your config file in a text editor

4. **Safety prompts**
   - Explain that some commands (like `nuke`) ask for confirmation
   - Show how `--yes` skips the prompt: `wux nuke ./dist --yes`
   - Show how to change `safe` in `wux.toml`

5. **Adding custom commands** *(Phase 2 placeholder — write the section, mark it "coming soon")*

6. **Configuration reference**
   - Annotated example of `wux.toml` with every field explained

Keep the README concise. No walls of text. Use short examples over long explanations.

---

## Notes for the Implementing Model

- Target Windows primarily. Unix support is a bonus — implement it, but test on Windows first.
- The `sysinfo` crate is the preferred way to get process info. Only shell out to
  `netstat` / `lsof` for port-to-PID mapping where `sysinfo` doesn't expose socket info.
- Never `unwrap()` — use `?` and `anyhow::bail!()` for all error paths.
- The `safe` flag in `wux.toml` is the single source of truth for whether a command prompts.
  `--yes` overrides it. `--dry-run` bypasses it. Nothing else should.
- The confirmation prompt lives in `prompt.rs` as a shared helper — do not duplicate it
  in each command file.
- The nuke root-guard is non-negotiable. Refuse loudly if the target resolves to `/` or `C:\`.
- Write `README.md` last, after all code is working. It should read like a human wrote it,
  not like generated documentation.
- Keep the binary small and startup fast. No async runtime needed — everything is synchronous.
