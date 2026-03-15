export function WorktreeModelPage() {
  return (
    <div>
      <h1>Worktree Model</h1>
      <p>
        Tyck uses a git worktree-based isolation model that enables multiple AI agents to work on the same
        codebase simultaneously without conflicts. This architecture is fundamental to how Tyck supports
        agent-agnostic development.
      </p>

      <h2>The Problem</h2>
      <p>
        Traditional IDEs have a single working directory. When you run an AI coding agent, it makes changes
        directly to your files. This creates several issues:
      </p>
      <ul>
        <li><strong>No isolation</strong> — Agent changes mix with your uncommitted work</li>
        <li><strong>No review</strong> — Changes are applied immediately, making it hard to review before accepting</li>
        <li><strong>No parallelism</strong> — You can't run multiple agents simultaneously</li>
        <li><strong>Unclear diffs</strong> — Hard to distinguish what the agent changed vs. your prior edits</li>
      </ul>

      <h2>The Solution: Worktree Isolation</h2>
      <p>
        Tyck creates a separate <a href="https://git-scm.com/docs/git-worktree">git worktree</a> for each
        agent session. This provides complete file isolation while sharing the same git history.
      </p>

      <pre><code>{`# Your workspace structure
~/projects/my-app/                    # Main workspace (your edits)
~/.tyck/worktrees/
├── my-app-session-abc123/            # Claude Code session
├── my-app-session-def456/            # Codex session
└── my-app-session-ghi789/            # Another parallel session`}</code></pre>

      <h3>How It Works</h3>
      <ol>
        <li><strong>Session Creation</strong> — When you start an agent session, Tyck creates a new worktree branched from your current state</li>
        <li><strong>Agent Execution</strong> — The agent runs in its isolated worktree, making all changes there</li>
        <li><strong>Review</strong> — Tyck shows you a clean diff of only what the agent changed</li>
        <li><strong>Accept/Reject</strong> — You review changes file-by-file, accepting or rejecting each one</li>
        <li><strong>Merge</strong> — Accepted changes are merged back to your main workspace</li>
        <li><strong>Cleanup</strong> — The worktree is removed (or kept for session resume)</li>
      </ol>

      <h2>Multi-Agent Support</h2>
      <p>
        Because each agent runs in its own worktree, you can run multiple agents in parallel:
      </p>

      <pre><code>{`┌─────────────────────────────────────────────────────────────────────┐
│                         Main Workspace                               │
│                     (your uncommitted changes)                       │
└─────────────────────────────────────────────────────────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              ▼                   ▼                   ▼
┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
│   Worktree A        │ │   Worktree B        │ │   Worktree C        │
│   Claude Code       │ │   Codex             │ │   Aider             │
│   "Add auth system" │ │   "Fix bug #123"    │ │   "Refactor tests"  │
└─────────────────────┘ └─────────────────────┘ └─────────────────────┘
              │                   │                   │
              └───────────────────┼───────────────────┘
                                  ▼
                    ┌─────────────────────────┐
                    │     Review & Merge      │
                    │   (in Tyck Review UI)   │
                    └─────────────────────────┘`}</code></pre>

      <h3>Agent-Agnostic Design</h3>
      <p>
        The worktree model is agent-agnostic by design. Tyck doesn't need to know the internals of each agent — 
        it only needs to:
      </p>
      <ul>
        <li>Create an isolated directory for the agent to work in</li>
        <li>Track which files the agent modified</li>
        <li>Present a diff when the session completes</li>
      </ul>
      <p>
        This means any CLI-based agent that can work in a directory works with Tyck: Claude Code, Codex,
        Aider, Continue, Cursor's agent, or your own custom tools.
      </p>

      <h2>Clean Diffs</h2>
      <p>
        A key benefit of worktree isolation is <strong>accurate diff detection</strong>. When reviewing
        agent changes, you only see what the agent actually changed — not your pre-existing uncommitted work.
      </p>

      <h3>Without Worktree Isolation</h3>
      <pre><code>{`# Your prior uncommitted changes + agent changes = confusing diff
- function old() {}        # Was this the agent or you?
+ function new() {}        # Was this the agent or you?
+ function agentAdded() {} # Only this was the agent`}</code></pre>

      <h3>With Worktree Isolation</h3>
      <pre><code>{`# Only shows agent changes — your uncommitted work is in main workspace
+ function agentAdded() {}  # Clear: this is from the agent`}</code></pre>

      <h2>Session Resume</h2>
      <p>
        Worktrees persist across Tyck restarts. If you close Tyck while an agent session is running,
        you can resume it later:
      </p>
      <ul>
        <li>The worktree retains all uncommitted agent changes</li>
        <li>Session metadata is stored in <code>~/.tyck/sessions/</code></li>
        <li>Tyck discovers the agent's session ID by scanning provider-specific metadata files</li>
      </ul>

      <h3>Provider Session Discovery</h3>
      <p>
        To resume sessions, Tyck needs to map worktrees back to agent sessions:
      </p>
      <table>
        <thead>
          <tr>
            <th>Agent</th>
            <th>Session Discovery</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Claude Code</td>
            <td>Scans <code>~/.claude/projects/&lt;hash&gt;/*.jsonl</code> for matching <code>cwd</code></td>
          </tr>
          <tr>
            <td>Codex</td>
            <td>Scans <code>~/.codex/sessions/</code> for <code>rollout-*.jsonl</code> with matching <code>cwd</code></td>
          </tr>
          <tr>
            <td>Custom</td>
            <td>Provider interface can be implemented for any agent</td>
          </tr>
        </tbody>
      </table>

      <h2>Conflict Resolution</h2>
      <p>
        If your main workspace changes while an agent is working (you edited a file, or merged a PR),
        Tyck detects this and offers three-way merge:
      </p>
      <ol>
        <li><strong>Base</strong> — The state when the session started</li>
        <li><strong>Main</strong> — Your current main workspace state</li>
        <li><strong>Agent</strong> — The agent's changes in the worktree</li>
      </ol>
      <p>
        Tyck shows conflicts in the review UI, letting you resolve them before merging.
      </p>
      <h3>Agent-Assisted Conflict Resolution</h3>
      <p>
        When conflicts are detected, Tyck automatically pulls the conflicting changes into the agent's worktree.
        This allows the agent to resolve the conflicts for you — just ask it to fix the merge conflicts
        and it will see the conflict markers in context and propose a resolution.
      </p>

      <h2>Configuration</h2>
      <h3>.worktreeinclude</h3>
      <p>
        Some files are gitignored but essential for the project to run (like <code>.env</code> files).
        Create a <code>.worktreeinclude</code> file to specify which ignored files should be copied to worktrees:
      </p>

      <pre><code>{`# .worktreeinclude
.env
.env.local
.env.development.local
config/master.key
config/credentials.yml.enc`}</code></pre>

      <p>If no <code>.worktreeinclude</code> exists, Tyck uses sensible defaults for common environment files.</p>

      <h2>Requirements</h2>
      <ul>
        <li><strong>Git 2.17+</strong> — Required for worktree support</li>
        <li><strong>Git repository</strong> — The project must be a git repo (worktrees are a git feature)</li>
      </ul>

      <h2>Limitations</h2>
      <h3>Shared State</h3>
      <p>
        Git worktrees only isolate <em>files</em>. External state is shared:
      </p>
      <ul>
        <li><strong>Databases</strong> — SQLite files, PostgreSQL, etc. are shared across worktrees</li>
        <li><strong>Docker</strong> — Containers and volumes are shared</li>
        <li><strong>External services</strong> — APIs, caches, queues are shared</li>
      </ul>
      <p>
        For database isolation, consider using worktree-specific database names in your <code>.env</code> files.
      </p>

      <h3>Git Hooks</h3>
      <p>
        Git hooks live in the main <code>.git/hooks/</code> directory and are shared across worktrees.
        Worktree-specific hook behavior requires hooks to check <code>$GIT_WORK_TREE</code>.
      </p>

      <h2>Next Steps</h2>
      <ul>
        <li><a href="/docs/installation">Install Tyck</a> to try the worktree model</li>
        <li><a href="/docs/tapp">Build extensions</a> with the Tapp framework</li>
      </ul>
    </div>
  );
}
