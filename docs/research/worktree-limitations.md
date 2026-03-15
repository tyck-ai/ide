# Worktree Model Limitations

This document describes known limitations and edge cases in the tyck worktree isolation model.

## Implemented Features

The worktree model provides:

- **Git worktree isolation**: Each agent session runs in a separate git worktree, preventing file conflicts
- **Accurate diff detection**: Only shows actual agent changes, not pre-existing uncommitted changes
- **Three-way merge**: Conflict detection and resolution when the main workspace diverges
- **Stale cleanup**: Automatic cleanup of orphaned worktrees older than 7 days
- **Essential file copying**: `.worktreeinclude` support for copying ignored files (`.env`, credentials)
- **Nested worktree protection**: Prevents creating worktrees from inside other worktrees
- **Session ID validation**: Ensures safe path component usage

## Known Limitations

### 1. `.git` File vs Directory Structure

Git worktrees use a `.git` **file** (not directory) containing a pointer to the main repository's `.git/worktrees/<name>` directory. This can break:

- **Sandbox tools**: Tools like bubblewrap (used by Claude Code) fail when trying to bind-mount `.git/hooks` because `.git` is a file
- **File watchers**: Some watchers expect `.git/` to be a directory

**Impact**: Low for tyck (we don't use bubblewrap sandboxing). The `resolve_git_dir()` helper is provided for any future code that needs to access git internals.

**Workaround**: If sandboxing is added in the future, the sandbox rules must handle the worktree `.git` file structure.

### 2. Database/State Isolation

Git worktrees only provide **file isolation**. Applications with shared state are not isolated:

- **Databases**: Rails apps using `development.sqlite3` will share the same database across worktrees
- **Docker volumes**: Containers mount the same volumes regardless of worktree
- **External services**: APIs, caches, queues are shared

**Mitigation**: Use the `.worktreeinclude` file to copy environment files with worktree-specific configurations. For Rails, consider adding database configuration to automatically namespace by worktree.

### 3. Large File Handling

The `copy_untracked_files()` function copies all untracked files from main to worktree. For repositories with large untracked files (build artifacts, datasets), this can be slow and wasteful.

**Mitigation**: 
- Ensure large generated files are in `.gitignore`
- The `.worktreeinclude` file only copies explicitly listed patterns
- Consider adding a `.worktreeexclude` pattern file in the future

### 4. Symbolic Links

Symbolic links in the repository are handled by git worktree natively, but:

- **Relative symlinks**: May break if they point outside the worktree
- **Absolute symlinks**: Work but point to main repo locations, not worktree copies

**Mitigation**: Avoid relative symlinks that escape the repository root.

### 5. Git Hooks

Git hooks in `.git/hooks/` are shared across all worktrees (they live in the main `.git` directory). This means:

- **Pre-commit hooks**: Run the same for all worktrees
- **Worktree-specific hooks**: Not possible without modifying hook scripts

**Mitigation**: If worktree-specific hook behavior is needed, hooks should check `$GIT_WORK_TREE` and adjust behavior accordingly.

### 6. Multi-Agent Parallelism

The current model supports **one agent per worktree**. For true parallel multi-agent execution (BoN - "bag of N"), additional coordination would be needed:

- **Concurrent worktree creation**: Currently not synchronized
- **Result aggregation**: No mechanism to compare/merge results from multiple parallel agents
- **Resource limits**: No cap on number of concurrent worktrees

**Future consideration**: If BoN support is added, implement a coordinator layer.

### 7. Session Resume and Worktree Association

When resuming a session, the system uses two strategies to find the correct worktree:

1. **Direct lookup**: Match by `provider_session_id` stored in worktree metadata
2. **Pending changes fallback**: Find any worktree for the same `main_cwd` that has uncommitted changes

For new sessions, the system polls every 5 seconds to discover and record the provider's session ID until it's found or the session is closed. The discovery logic is provider-specific:

- **Claude Code**: Scans `~/.claude/projects/<hash>/*.jsonl` for session files whose `cwd` matches the worktree path
- **Codex**: Scans `~/.codex/sessions/` recursively for `rollout-*.jsonl` files with matching `cwd` in `session_meta`

### 8. Resume After Worktree Deletion

If a worktree is manually deleted or cleaned up while an agent session is active:

- The agent will fail to write files
- Session resume may fail if metadata exists but worktree doesn't

**Mitigation**: The stale cleanup process handles orphaned metadata, but in-flight sessions may error.

### 8. Git Version Requirements

The worktree model requires Git 2.17+. The `check_git_version()` command is provided but not automatically enforced at startup.

**Recommendation**: Call `check_git_version()` on app startup and warn users if their Git version is too old.

## Configuration

### `.worktreeinclude` File

Create a `.worktreeinclude` or `.tyck/worktreeinclude` file in your repository root to specify files that should be copied to each worktree despite being gitignored:

```
# Essential environment files
.env
.env.local
.env.development.local

# Rails credentials
config/master.key
config/credentials.yml.enc

# Custom certificates
certs/local.pem
```

If no `.worktreeinclude` file exists, these defaults are copied:
- `.env`
- `.env.local`
- `.env.development`
- `.env.development.local`
- `config/master.key`
- `config/credentials.yml.enc`

## Troubleshooting

### "Cannot create worktree from inside another worktree"

You're trying to create a session while already inside a worktree. Navigate to the main repository first.

### Stale files appearing in review

If you see files that shouldn't be in review, the worktree may have been created before the `initial_untracked` tracking was implemented. Delete the session and create a new one.

### Worktree cleanup fails

If `git worktree remove` fails:

1. The worktree may be locked: `git worktree unlock <path>`
2. There may be uncommitted changes: `git worktree remove --force <path>`
3. Manually delete the directory and run `git worktree prune` in the main repo
