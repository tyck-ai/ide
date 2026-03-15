# Tapp Examples

Complete, working examples of Tapp extensions.

## Basic Examples

### [Counter](./counter.rs)
The simplest possible Tapp - a counter with increment/decrement buttons.

### [Todo List](./todo-list.rs)
A full-featured todo list with persistence.

### [File Browser](./file-browser.rs)
Browse workspace files with a tree view.

## Agent Integration

### [Database Explorer](./database-explorer.rs)
Expose database tools for AI agents.

### [Code Review Assistant](./code-review.rs)
Hooks to enrich agent context with code review capabilities.

## Advanced UI

### [Data Grid](./data-grid.rs)
Large dataset handling with sorting and filtering.

### [Dashboard](./dashboard.rs)
Multiple panels with tabs and modals.

## Running Examples

1. Copy the example to a new project:
   ```bash
   tapp init my-example
   cp docs/tapp/examples/counter.rs my-example/src/lib.rs
   ```

2. Build and run:
   ```bash
   cd my-example
   tapp dev
   ```
