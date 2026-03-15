# Contributing to Tyck

Thank you for your interest in contributing to Tyck! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- **Rust** 1.75 or later
- **Node.js** 20 or later
- **Tauri CLI** 2.0 or later

### Setup

1. Fork and clone the repository:
```bash
git clone https://github.com/YOUR_USERNAME/tyck.git
cd tyck
```

2. Install the WASM target:
```bash
rustup target add wasm32-wasip2
```

3. Install dependencies:
```bash
# Web dependencies
cd web && npm install && cd ..

# Rust dependencies are handled by Cargo
```

4. Run in development mode:
```bash
cargo tauri dev
```

## Project Structure

| Directory | Description |
|-----------|-------------|
| `src-tauri/` | Rust backend (Tauri application) |
| `src-tauri/src/apps/` | Tapp extension system implementation |
| `src-tauri/src/wasm/` | WASM runtime using Wasmtime |
| `packages/tapp/` | Tapp SDK for building extensions |
| `packages/tapp-cli/` | CLI tool for extension development |
| `packages/tapp-macros/` | Procedural macros for Tapp |
| `packages/examples/` | Example Tapp extensions |
| `web/` | Documentation website |
| `docs/` | Design documents and research |

## Development Workflow

### Making Changes

1. Create a branch for your changes:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes following our code style (see below)

3. Test your changes:
```bash
cargo test
cargo clippy
```

4. Commit with a descriptive message:
```bash
git commit -m "feat: add new UI component for data grids"
```

### Commit Message Format

We use conventional commits:

- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation changes
- `refactor:` — Code refactoring
- `test:` — Adding or updating tests
- `chore:` — Maintenance tasks

### Code Style

**Rust:**
- Follow standard Rust formatting (`cargo fmt`)
- No warnings from `cargo clippy`
- Document public APIs with doc comments

**TypeScript/JavaScript:**
- Use ESLint configuration provided
- Prefer TypeScript over JavaScript

## Areas for Contribution

### Good First Issues

Look for issues labeled `good first issue` — these are specifically chosen for new contributors.

### Priority Areas

1. **UI Components** — Expanding the Tapp component library
2. **Documentation** — Improving guides and API docs
3. **Examples** — Creating sample extensions
4. **Testing** — Increasing test coverage
5. **Platform Support** — Testing on Windows/Linux

### Creating Tapp Extensions

Building example extensions is a great way to contribute:

1. Use `tapp init` to create a new extension
2. Implement useful functionality
3. Submit to `packages/examples/`
4. Include a README explaining the extension

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Fill out the PR template
4. Request review from maintainers

### PR Checklist

- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation updated (if applicable)
- [ ] Commit messages follow convention

## Reporting Issues

### Bug Reports

Include:
- Steps to reproduce
- Expected behavior
- Actual behavior
- OS and version
- Tyck version

### Feature Requests

Describe:
- The problem you're trying to solve
- Proposed solution
- Alternative solutions considered

## Code of Conduct

Be respectful and constructive. We're building something together.

## Questions?

- Open a Discussion on GitHub
- Join our community (links coming soon)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
