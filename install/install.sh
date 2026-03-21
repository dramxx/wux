#!/usr/bin/env bash
set -euo pipefail

FORCE=0
if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALL_DIR="$HOME/.wux/bin"
BINARY_PATH="$INSTALL_DIR/wux"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/wux"
CONFIG_PATH="$CONFIG_DIR/wux.toml"
PROFILE_UPDATED=0

printf 'Installing wux...\n'

if [[ ! -f "$REPO_ROOT/target/release/wux" ]]; then
  printf 'Building release binary...\n'
  cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"
fi

if [[ -f "$BINARY_PATH" && "$FORCE" -ne 1 ]]; then
  printf 'wux is already installed. Use --force to reinstall.\n'
  exit 0
fi

mkdir -p "$INSTALL_DIR"
cp "$REPO_ROOT/target/release/wux" "$BINARY_PATH"
chmod +x "$BINARY_PATH"
printf 'Copied wux to %s\n' "$INSTALL_DIR"

ensure_path_in_file() {
  local file="$1"
  local line='export PATH="$HOME/.wux/bin:$PATH"'

  if [[ ! -f "$file" ]]; then
    touch "$file"
  fi

  if ! grep -Fq '$HOME/.wux/bin' "$file"; then
    printf '\n%s\n' "$line" >> "$file"
    PROFILE_UPDATED=1
  fi
}

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  ensure_path_in_file "$HOME/.profile"

  if [[ -f "$HOME/.bashrc" ]]; then
    ensure_path_in_file "$HOME/.bashrc"
  fi

  if [[ -f "$HOME/.zshrc" ]]; then
    ensure_path_in_file "$HOME/.zshrc"
  fi
fi

mkdir -p "$CONFIG_DIR"
if [[ ! -f "$CONFIG_PATH" ]]; then
  cat > "$CONFIG_PATH" <<'EOF'
# wux configuration file

[settings]
color = true

[commands.free]
safe = true

[commands.nuke]
safe = false
EOF

  printf 'Created config file at %s\n' "$CONFIG_PATH"
fi

printf '\nwux installed successfully!\n'
if [[ "$PROFILE_UPDATED" -eq 1 ]]; then
  printf 'Restart your terminal or run: source ~/.profile\n'
fi
printf "Run 'wux help' to get started.\n"
