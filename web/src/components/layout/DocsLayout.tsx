import { Link, Outlet, useLocation } from 'react-router-dom';
import { Logo } from '../ui';

const navigation = [
  {
    title: 'Getting Started',
    items: [
      { title: 'Installation', href: '/docs/installation' },
    ],
  },
  {
    title: 'Architecture',
    items: [
      { title: 'Worktree Model', href: '/docs/architecture/worktree' },
    ],
  },
  {
    title: 'Customization',
    items: [
      { title: 'Themes', href: '/docs/themes' },
    ],
  },
  {
    title: 'Tapp Development',
    items: [
      { title: 'Quick Start', href: '/docs/tapp' },
      { title: 'Development Guide', href: '/docs/tapp/development' },
      { title: 'UI Components', href: '/docs/tapp/ui-components' },
    ],
  },
];

export function DocsLayout() {
  const location = useLocation();

  return (
    <div className="min-h-screen bg-[var(--color-background)]">
      {/* Top navbar */}
      <nav className="fixed top-0 left-0 right-0 z-50 landing-nav">
        <div className="max-w-[1400px] mx-auto px-6 h-16 flex items-center justify-between">
          <Link to="/" className="flex items-center gap-1.5 group">
            <Logo size={24} />
            <span className="text-lg font-bold text-[var(--color-text)] tracking-tight">tyck</span>
          </Link>

          <div className="hidden md:flex items-center gap-8">
            <Link to="/" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Home</Link>
            <Link to="/docs" className="text-sm text-[var(--color-accent)] font-medium">Docs</Link>
            <a href="https://github.com/tyck-dev/tyck" target="_blank" rel="noopener noreferrer" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">GitHub</a>
          </div>
        </div>
      </nav>

      <div className="pt-16 flex">
        {/* Sidebar */}
        <aside className="docs-sidebar fixed top-16 left-0 w-64 h-[calc(100vh-4rem)] overflow-y-auto bg-[var(--color-surface)] p-6">
          <nav className="space-y-6">
            {navigation.map((section) => (
              <div key={section.title}>
                <h3 className="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-3">
                  {section.title}
                </h3>
                <ul className="space-y-1">
                  {section.items.map((item) => (
                    <li key={item.href}>
                      <Link
                        to={item.href}
                        className={`docs-nav-item ${location.pathname === item.href ? 'active' : ''}`}
                      >
                        {item.title}
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </nav>
        </aside>

        {/* Main content */}
        <main className="flex-1 ml-64 p-8 md:p-12">
          <div className="max-w-3xl mx-auto docs-content">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
