# Building & Publishing Tapp Packages

This guide covers how to build and distribute the Tapp CLI and SDK packages to various package registries.

## Package Overview

| Package | Description | Registry |
|---------|-------------|----------|
| `tapp-cli` | CLI tool for building Tapps | crates.io, Homebrew, Scoop, winget |
| `tapp` | Rust SDK for app development | crates.io |
| `tapp-macros` | Procedural macros | crates.io |

## Publishing to crates.io

### Prerequisites

1. **Create crates.io account**: https://crates.io/
2. **Generate API token**: https://crates.io/settings/tokens
3. **Login locally**:
   ```bash
   cargo login <your-token>
   ```

### Preparing Packages

#### 1. Update Version Numbers

Update all `Cargo.toml` files with the new version:

```bash
# packages/tapp-macros/Cargo.toml
# packages/tapp/Cargo.toml
# packages/tapp-cli/Cargo.toml
```

Ensure version consistency:
```toml
# tapp/Cargo.toml
[package]
name = "tapp"
version = "0.2.0"

[dependencies]
tapp-macros = "0.2.0"  # Must match
```

#### 2. Update Cargo.toml Metadata

Ensure all packages have required metadata:

```toml
[package]
name = "tapp-cli"
version = "0.2.0"
edition = "2021"
description = "CLI tool for building Tapp extensions for Tyck IDE"
license = "MIT"
repository = "https://github.com/tyck-dev/tyck"
homepage = "https://tyck.dev"
documentation = "https://docs.tyck.dev/tapp"
readme = "README.md"
keywords = ["tyck", "tapp", "wasm", "extension", "ide"]
categories = ["development-tools", "wasm"]
authors = ["Tyck Team <team@tyck.dev>"]

[package.metadata.docs.rs]
all-features = true
```

#### 3. Create README Files

Each package needs a README.md:

```bash
# packages/tapp/README.md
# packages/tapp-cli/README.md
# packages/tapp-macros/README.md
```

### Publishing Order

Packages must be published in dependency order:

```bash
# 1. Publish tapp-macros first (no dependencies on other tapp packages)
cd packages/tapp-macros
cargo publish

# Wait for crates.io to index (usually 1-2 minutes)
sleep 120

# 2. Publish tapp (depends on tapp-macros)
cd ../tapp
cargo publish

# Wait for indexing
sleep 120

# 3. Publish tapp-cli (standalone, but wait for ecosystem)
cd ../tapp-cli
cargo publish
```

### Dry Run

Test publishing without actually uploading:

```bash
cargo publish --dry-run
```

### Yanking a Version

If you need to remove a broken release:

```bash
cargo yank --version 0.2.0 tapp-cli
```

## Publishing to Homebrew

### Creating a Homebrew Tap

1. **Create tap repository**: `github.com/tyck-dev/homebrew-tap`

2. **Create Formula**:

```ruby
# Formula/tapp.rb
class Tapp < Formula
  desc "CLI tool for building Tapp extensions for Tyck IDE"
  homepage "https://tyck.dev"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tyck-dev/tyck/releases/download/tapp-v0.2.0/tapp-aarch64-apple-darwin.tar.gz"
      sha256 "HASH_HERE"
    else
      url "https://github.com/tyck-dev/tyck/releases/download/tapp-v0.2.0/tapp-x86_64-apple-darwin.tar.gz"
      sha256 "HASH_HERE"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tyck-dev/tyck/releases/download/tapp-v0.2.0/tapp-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "HASH_HERE"
    else
      url "https://github.com/tyck-dev/tyck/releases/download/tapp-v0.2.0/tapp-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "HASH_HERE"
    end
  end

  def install
    bin.install "tapp"
    
    # Install shell completions
    generate_completions_from_executable(bin/"tapp", "completions")
  end

  test do
    assert_match "tapp #{version}", shell_output("#{bin}/tapp --version")
  end
end
```

### Building Release Binaries

Create a GitHub Actions workflow:

```yaml
# .github/workflows/release-tapp.yml
name: Release tapp CLI

on:
  push:
    tags:
      - 'tapp-v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build
        run: |
          cd packages/tapp-cli
          cargo build --release --target ${{ matrix.target }}
      
      - name: Package (Unix)
        if: runner.os != 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          tar -czvf tapp-${{ matrix.target }}.tar.gz tapp
      
      - name: Package (Windows)
        if: runner.os == 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          Compress-Archive -Path tapp.exe -DestinationPath tapp-${{ matrix.target }}.zip
      
      - name: Upload Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            target/${{ matrix.target }}/release/tapp-*.tar.gz
            target/${{ matrix.target }}/release/tapp-*.zip

  update-homebrew:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Update Homebrew Formula
        env:
          HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
        run: |
          # Script to update formula with new version and hashes
          ./scripts/update-homebrew.sh
```

### Update Script

```bash
#!/bin/bash
# scripts/update-homebrew.sh

VERSION="${GITHUB_REF#refs/tags/tapp-v}"

# Download and hash each binary
for target in x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  URL="https://github.com/tyck-dev/tyck/releases/download/tapp-v${VERSION}/tapp-${target}.tar.gz"
  SHA=$(curl -sL "$URL" | sha256sum | cut -d' ' -f1)
  echo "${target}: ${SHA}"
done

# Update formula (using sed or a proper templating tool)
# Push to homebrew-tap repo
```

## Publishing to Scoop (Windows)

### Creating a Scoop Bucket

1. **Create bucket repository**: `github.com/tyck-dev/scoop-bucket`

2. **Create Manifest**:

```json
{
  "version": "0.2.0",
  "description": "CLI tool for building Tapp extensions for Tyck IDE",
  "homepage": "https://tyck.dev",
  "license": "MIT",
  "architecture": {
    "64bit": {
      "url": "https://github.com/tyck-dev/tyck/releases/download/tapp-v0.2.0/tapp-x86_64-pc-windows-msvc.zip",
      "hash": "SHA256_HASH_HERE"
    }
  },
  "bin": "tapp.exe",
  "checkver": {
    "github": "https://github.com/tyck-dev/tyck",
    "regex": "tapp-v([\\d.]+)"
  },
  "autoupdate": {
    "architecture": {
      "64bit": {
        "url": "https://github.com/tyck-dev/tyck/releases/download/tapp-v$version/tapp-x86_64-pc-windows-msvc.zip"
      }
    }
  }
}
```

## Publishing to winget (Windows Package Manager)

### Creating a Manifest

1. **Fork**: https://github.com/microsoft/winget-pkgs

2. **Create manifest directory**: `manifests/t/tyck/tapp/0.2.0/`

3. **Create manifest files**:

```yaml
# tyck.tapp.yaml
PackageIdentifier: tyck.tapp
PackageVersion: 0.2.0
PackageLocale: en-US
Publisher: Tyck
PackageName: tapp
License: MIT
ShortDescription: CLI tool for building Tapp extensions for Tyck IDE
Tags:
  - cli
  - rust
  - wasm
  - ide
  - extensions
```

```yaml
# tyck.tapp.installer.yaml
PackageIdentifier: tyck.tapp
PackageVersion: 0.2.0
InstallerType: zip
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/tyck-dev/tyck/releases/download/tapp-v0.2.0/tapp-x86_64-pc-windows-msvc.zip
    InstallerSha256: SHA256_HASH_HERE
    NestedInstallerType: portable
    NestedInstallerFiles:
      - RelativeFilePath: tapp.exe
        PortableCommandAlias: tapp
ManifestType: installer
ManifestVersion: 1.4.0
```

4. **Submit PR** to winget-pkgs repository

## Automated Release Workflow

Complete GitHub Actions workflow for all platforms:

```yaml
# .github/workflows/release-tapp-full.yml
name: Release Tapp CLI (All Platforms)

on:
  push:
    tags:
      - 'tapp-v*'
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release'
        required: true

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: tapp ${{ github.ref_name }}
          draft: false
          prerelease: false

  build-and-upload:
    needs: create-release
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: tapp
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: tapp
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: tapp
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact: tapp
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: tapp.exe

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install musl-tools (Linux musl)
        if: matrix.target == 'x86_64-unknown-linux-musl'
        run: sudo apt-get install -y musl-tools
      
      - name: Build
        run: |
          cd packages/tapp-cli
          cargo build --release --target ${{ matrix.target }}
      
      - name: Create Archive (Unix)
        if: runner.os != 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          tar -czvf tapp-${{ matrix.target }}.tar.gz ${{ matrix.artifact }}
          sha256sum tapp-${{ matrix.target }}.tar.gz > tapp-${{ matrix.target }}.tar.gz.sha256
      
      - name: Create Archive (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          cd target/${{ matrix.target }}/release
          Compress-Archive -Path ${{ matrix.artifact }} -DestinationPath tapp-${{ matrix.target }}.zip
          (Get-FileHash tapp-${{ matrix.target }}.zip -Algorithm SHA256).Hash > tapp-${{ matrix.target }}.zip.sha256
      
      - name: Upload Release Asset (Unix)
        if: runner.os != 'Windows'
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: target/${{ matrix.target }}/release/tapp-${{ matrix.target }}.tar.gz
          asset_name: tapp-${{ matrix.target }}.tar.gz
          asset_content_type: application/gzip
      
      - name: Upload Release Asset (Windows)
        if: runner.os == 'Windows'
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: target/${{ matrix.target }}/release/tapp-${{ matrix.target }}.zip
          asset_name: tapp-${{ matrix.target }}.zip
          asset_content_type: application/zip

  publish-crates:
    needs: build-and-upload
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cd packages/tapp-macros && cargo publish --allow-dirty
          sleep 60
          cd ../tapp && cargo publish --allow-dirty
          sleep 60
          cd ../tapp-cli && cargo publish --allow-dirty

  update-package-managers:
    needs: build-and-upload
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Update Homebrew
        env:
          HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
        run: ./scripts/update-homebrew.sh
      
      - name: Update Scoop
        env:
          SCOOP_BUCKET_TOKEN: ${{ secrets.SCOOP_BUCKET_TOKEN }}
        run: ./scripts/update-scoop.sh
```

## Version Management

### Semantic Versioning

Follow semver for all packages:
- **MAJOR**: Breaking API changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes

### Keeping Versions in Sync

Use a script to update all versions:

```bash
#!/bin/bash
# scripts/bump-version.sh

NEW_VERSION=$1

# Update all Cargo.toml files
sed -i '' "s/^version = .*/version = \"$NEW_VERSION\"/" packages/tapp/Cargo.toml
sed -i '' "s/^version = .*/version = \"$NEW_VERSION\"/" packages/tapp-macros/Cargo.toml
sed -i '' "s/^version = .*/version = \"$NEW_VERSION\"/" packages/tapp-cli/Cargo.toml

# Update dependency versions
sed -i '' "s/tapp-macros = .*/tapp-macros = \"$NEW_VERSION\"/" packages/tapp/Cargo.toml

# Create git tag
git add -A
git commit -m "Bump version to $NEW_VERSION"
git tag "tapp-v$NEW_VERSION"
git push origin main --tags
```

## Changelog

Maintain a CHANGELOG.md for each package:

```markdown
# Changelog

## [0.2.0] - 2026-03-14

### Added
- Hot reload support for development
- VirtualList component for large lists
- DataGrid component with sorting/filtering

### Changed
- Improved error messages in CLI
- Faster WASM compilation

### Fixed
- Memory leak in tool execution
- UI flicker on rapid updates

## [0.1.0] - 2026-02-01

Initial release.
```
