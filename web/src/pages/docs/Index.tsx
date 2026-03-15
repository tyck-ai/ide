import { Link } from 'react-router-dom';

function IconDownload() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" x2="12" y1="15" y2="3" />
    </svg>
  );
}

function IconGitBranch() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="6" x2="6" y1="3" y2="15" />
      <circle cx="18" cy="6" r="3" />
      <circle cx="6" cy="18" r="3" />
      <path d="M18 9a9 9 0 0 1-9 9" />
    </svg>
  );
}

function IconPalette() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="13.5" cy="6.5" r="0.5" fill="currentColor" />
      <circle cx="17.5" cy="10.5" r="0.5" fill="currentColor" />
      <circle cx="8.5" cy="7.5" r="0.5" fill="currentColor" />
      <circle cx="6.5" cy="12.5" r="0.5" fill="currentColor" />
      <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.555C21.965 6.012 17.461 2 12 2z" />
    </svg>
  );
}

function IconPuzzle() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M19.439 7.85c-.049.322.059.648.289.878l1.568 1.568c.47.47.706 1.087.706 1.704s-.235 1.233-.706 1.704l-1.611 1.611a.98.98 0 0 1-.837.276c-.47-.07-.802-.48-.968-.925a2.501 2.501 0 1 0-3.214 3.214c.446.166.855.497.925.968a.979.979 0 0 1-.276.837l-1.61 1.61a2.404 2.404 0 0 1-1.705.707 2.402 2.402 0 0 1-1.704-.706l-1.568-1.568a1.026 1.026 0 0 0-.878-.29c-.493.074-.84.504-1.02.968a2.5 2.5 0 1 1-3.237-3.237c.464-.18.894-.527.967-1.02a1.026 1.026 0 0 0-.289-.878l-1.568-1.568A2.402 2.402 0 0 1 1.998 12c0-.617.236-1.234.706-1.704L4.315 8.685a.98.98 0 0 1 .837-.276c.47.07.802.48.968.925a2.501 2.501 0 1 0 3.214-3.214c-.446-.166-.855-.497-.925-.968a.979.979 0 0 1 .276-.837l1.61-1.61a2.404 2.404 0 0 1 1.705-.707c.618 0 1.234.236 1.704.706l1.568 1.568c.23.23.556.338.878.29.493-.074.84-.504 1.02-.968a2.5 2.5 0 1 1 3.237 3.237c-.464.18-.894.527-.967 1.02Z" />
    </svg>
  );
}

function IconArrowRight() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="5" x2="19" y1="12" y2="12" />
      <polyline points="12 5 19 12 12 19" />
    </svg>
  );
}

const sections = [
  {
    title: 'Installation',
    description: 'Get Tyck up and running on macOS, Windows, or Linux.',
    href: '/docs/installation',
    icon: <IconDownload />,
    color: '#7BC9A0',
  },
  {
    title: 'Worktree Model',
    description: 'Learn how Tyck isolates agent sessions using git worktrees.',
    href: '/docs/architecture/worktree',
    icon: <IconGitBranch />,
    color: '#C9B87B',
  },
  {
    title: 'Themes',
    description: 'Customize the look and feel of Tyck with custom themes.',
    href: '/docs/themes',
    icon: <IconPalette />,
    color: '#7BA8C9',
  },
  {
    title: 'Tapp Development',
    description: 'Build powerful extensions using the Tapp SDK.',
    href: '/docs/tapp',
    icon: <IconPuzzle />,
    color: '#A07BC9',
  },
];

export function DocsIndexPage() {
  return (
    <div>
      <h1>Documentation</h1>
      <p>
        Welcome to the Tyck documentation. Tyck is an agent-agnostic code editor — use Claude Code,
        Codex, Cursor, Copilot, or any CLI agent you prefer. Here you'll find everything you need
        to install, customize, and extend Tyck.
      </p>

      <div className="grid grid-cols-1 gap-4 mt-8">
        {sections.map((section) => (
          <Link
            key={section.href}
            to={section.href}
            className="group flex items-start gap-4 p-6 rounded-xl border border-[var(--color-border)] hover:border-[var(--color-border-focus)] transition-all duration-200 hover:shadow-md"
          >
            <div
              className="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0"
              style={{ backgroundColor: `color-mix(in srgb, ${section.color} 12%, transparent)`, color: section.color }}
            >
              {section.icon}
            </div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-[var(--color-text)] mb-1 flex items-center gap-2">
                {section.title}
                <span className="opacity-0 group-hover:opacity-100 transition-opacity">
                  <IconArrowRight />
                </span>
              </h3>
              <p className="text-[var(--color-text-secondary)] text-sm leading-relaxed m-0">
                {section.description}
              </p>
            </div>
          </Link>
        ))}
      </div>

      <h2>Quick Links</h2>
      <ul>
        <li><Link to="/docs/architecture/worktree">Worktree Model</Link> — How Tyck enables multi-agent development</li>
        <li><Link to="/docs/tapp">Tapp Quick Start</Link> — Create your first Tapp in 5 minutes</li>
        <li><Link to="/docs/tapp/ui-components">UI Components</Link> — Browse available UI components</li>
        <li><Link to="/docs/themes">Theme Development</Link> — Create and share custom themes</li>
      </ul>

      <h2>Community</h2>
      <p>
        Join the Tyck community to get help, share your extensions, and connect with other developers.
      </p>
      <ul>
        <li><a href="https://github.com/tyck-dev/tyck" target="_blank" rel="noopener noreferrer">GitHub</a> — Report issues and contribute</li>
        <li><a href="https://discord.gg/tyck" target="_blank" rel="noopener noreferrer">Discord</a> — Chat with the community</li>
        <li><a href="https://twitter.com/tyck_dev" target="_blank" rel="noopener noreferrer">Twitter</a> — Follow for updates</li>
      </ul>
    </div>
  );
}
