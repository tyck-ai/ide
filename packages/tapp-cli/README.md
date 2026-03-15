# tapp-cli

Command-line tool for building Tapp extensions for Tyck IDE.

## Installation

### From crates.io
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

## Usage

### Create a new project
```bash
tapp init my-app
cd my-app
```

### Start development mode
```bash
tapp dev
```

This watches for file changes and hot-reloads your app.

### Build for release
```bash
tapp build
```

### Install to Tyck
```bash
tapp install ./manifest.json
```

### List installed apps
```bash
tapp list
```

### Uninstall an app
```bash
tapp uninstall my-app
```

## Commands

| Command | Description |
|---------|-------------|
| `init <name>` | Create a new Tapp project |
| `dev` | Start development mode with hot reload |
| `build` | Build the app for release |
| `install <manifest>` | Install an app to Tyck |
| `list` | List installed apps |
| `uninstall <id>` | Remove an installed app |
| `run <id>` | Launch an app in Tyck |
| `completions <shell>` | Generate shell completions |

## Project Structure

Running `tapp init` creates:

```
my-app/
├── Cargo.toml       # Rust dependencies
├── src/
│   └── lib.rs       # App implementation
├── manifest.json    # App metadata & permissions
└── assets/
    └── icon.svg     # App icon
```

## Prerequisites

- Rust toolchain with wasm32-wasip2 target
- Tyck IDE

```bash
rustup target add wasm32-wasip2
```

## Documentation

- [Installation Guide](https://docs.tyck.dev/tapp/installation)
- [Quick Start](https://docs.tyck.dev/tapp/quick-start)
- [Development Guide](https://docs.tyck.dev/tapp/development)

## License

MIT
