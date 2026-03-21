# wux

Tired of googling shell syntax for the hundredth time? Same.

`wux` is a personal command toolkit that replaces the stuff you can never remember with commands that actually make sense.

## Install

### Windows

```powershell
git clone https://github.com/dramxx/wux.git
cd wux
.\install\install.ps1
```

### Linux/macOS

```bash
git clone https://github.com/dramxx/wux.git
cd wux
./install/install.sh
```

Restart your terminal. Type `wux help`. Done.

## Commands

**Kill whatever is sitting on a port**

```powershell
# The PowerShell way
Get-Process -Id (Get-NetTCPConnection -LocalPort 3000).OwningProcess | Stop-Process
```

```bash
# The bash way
kill -9 $(lsof -ti :3000)
```

```powershell
# The wux way
wux free 3000
```

**Delete a folder and everything in it**

```powershell
# The PowerShell way
Remove-Item -Recurse -Force .\node_modules
```

```bash
# The bash way
rm -rf ./node_modules
```

```powershell
# The wux way
wux nuke ./node_modules
```

wux will ask before deleting. Use `--yes` to skip the prompt.

**Show folder info**

```powershell
wux info
```

```
📁 C:\DEV\wux
   Size        142.3 MB
   Files       87
   Folders     12
   Largest     target\release\wux.exe     45.2 MB
   Newest      src\commands\free.rs       2 minutes ago
   Oldest      Cargo.toml                 3 days ago
```

## Your own commands

Got things you run every day? Stop typing them.

```powershell
wux config
```

Opens `wux.toml` in Notepad. Close it when done, changes take effect immediately. You can do things like:

```toml
[commands.serve]
run = [
    "cd C:/dev/work/bigproject",
    "npm run dev"
]
description = "Launch Forge"

[commands.bye]
run = ["shutdown /s /t 0"]
description = "Shut down PC"
safe = false
```

Or you need to run parallel processes? Simple .bat file comes to the rescue:

```bat
@echo off
set PROJECT=%1
cd /d %PROJECT%
start "" opencode serve --hostname 127.0.0.1 --port 4096
timeout /t 2
start "" cmd /k "cd /d C:/dev/port-hole/server && npx tsx src/index.ts"
opencode attach http://127.0.0.1:4096
```

And than in wux

```toml
[commands.porthole]
run = ["C:/dev/port-hole/launch.bat C:/dev/myproject"]
description = "Launch port-hole"
```

From now on, it's just `wux serve`, `wux bye` or `wux porthole`.

Run `wux list` to see every command available (including yours).

## Flags

| Flag        | What it does                            |
| ----------- | --------------------------------------- |
| `--yes`     | Skip confirmation prompts               |
| `--dry-run` | Show what would happen without doing it |
