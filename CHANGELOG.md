# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with Tauri backend
- Tapp extension system with WASM runtime (Wasmtime)
- Tapp SDK (`tapp` crate) for building extensions
- Tapp CLI (`tapp-cli`) for extension development workflow
- Extension storage system (session, JSON, SQLite)
- Extension permission system with capability-based security
- Agent integration APIs (context injection, tool registration)
- Example extensions (minimal-app, database-explorer)
- Documentation website scaffolding

### Architecture
- Three-zone UI layout (ContextZone, FocusZone, InsightZone)
- WASM sandbox isolation for extensions
- Message-passing IPC between host and extensions
