# Installing the Tapp CLI

The `tapp` CLI is the primary tool for creating, building, and managing Tapp extensions.

## Prerequisites

### Rust Toolchain
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM target
rustup target add wasm32-wasip2
```

### Optional: wasm-tools
For advanced WASM manipulation:
```bash
cargo install wasm-tools
```

## Installation Methods

### From crates.io (Recommended)
```bash
cargo install tapp-cli
```

### From Homebrew (macOS/Linux)
```bash
brew tap tyck-dev/tap
brew install tapp
```

### From Scoop (Windows)
```powershell
scoop bucket add tyck https://github.com/tyck-dev/scoop-bucket
scoop install tapp
```

### From winget (Windows)
```powershell
winget install tyck.tapp
```

### From Source
```bash
git clone https://github.com/tyck-dev/tyck
cd tyck/packages/tapp-cli
cargo install --path .
```

## Verify Installation

```bash
tapp --version
# tapp 0.1.0

tapp --help
# tapp - CLI for building Tapp extensions
#
# USAGE:
#     tapp <COMMAND>
#
# COMMANDS:
#     init       Create a new Tapp project
#     dev        Start development mode with hot reload
#     build      Build the app for release
#     install    Install an app to Tyck
#     list       List installed apps
#     uninstall  Remove an installed app
#     run        Launch an app in Tyck
#     help       Print help information
```

## Shell Completion

### Bash
```bash
tapp completions bash > ~/.local/share/bash-completion/completions/tapp
```

### Zsh
```bash
tapp completions zsh > ~/.zfunc/_tapp
```

### Fish
```bash
tapp completions fish > ~/.config/fish/completions/tapp.fish
```

### PowerShell
```powershell
tapp completions powershell > $HOME\Documents\PowerShell\Scripts\tapp.ps1
```

## Updating

### crates.io
```bash
cargo install tapp-cli --force
```

### Homebrew
```bash
brew upgrade tapp
```

### Scoop
```powershell
scoop update tapp
```

## Uninstalling

### crates.io
```bash
cargo uninstall tapp-cli
```

### Homebrew
```bash
brew uninstall tapp
```

### Scoop
```powershell
scoop uninstall tapp
```

## Troubleshooting

### "wasm32-wasip2 target not found"
```bash
rustup target add wasm32-wasip2
```

### "tapp command not found"
Ensure `~/.cargo/bin` is in your PATH:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Build fails with linker errors
Install the platform-specific linker:
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install build-essential

# Windows
# Install Visual Studio Build Tools
```
