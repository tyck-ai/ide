import { Link } from 'react-router-dom';
import { Logo } from '../ui';

export function Footer() {
  return (
    <footer className="border-t border-[var(--color-border)] py-12 px-6">
      <div className="max-w-6xl mx-auto">
        <div className="flex flex-col md:flex-row items-start justify-between gap-8 mb-10">
          <div>
            <Link to="/" className="flex items-center gap-1.5 mb-3">
              <Logo size={22} />
              <span className="text-base font-bold text-[var(--color-text)] tracking-tight">tyck</span>
            </Link>
            <p className="text-sm text-[var(--color-text-muted)] max-w-xs leading-relaxed">
              Agent-agnostic code editor built for the future of software development.
            </p>
          </div>

          <div className="flex gap-16">
            <div>
              <p className="text-sm font-medium text-[var(--color-text)] mb-3">Product</p>
              <ul className="space-y-2">
                <li><a href="#features" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Features</a></li>
                <li><a href="#download" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Download</a></li>
                <li><Link to="/docs" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Documentation</Link></li>
              </ul>
            </div>
            <div>
              <p className="text-sm font-medium text-[var(--color-text)] mb-3">Developers</p>
              <ul className="space-y-2">
                <li><Link to="/docs/themes" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Themes</Link></li>
                <li><Link to="/docs/tapp" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Tapp SDK</Link></li>
                <li><a href="https://github.com/tyck-dev/tyck" target="_blank" rel="noopener noreferrer" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">GitHub</a></li>
              </ul>
            </div>
            <div>
              <p className="text-sm font-medium text-[var(--color-text)] mb-3">Community</p>
              <ul className="space-y-2">
                <li><a href="https://discord.gg/tyck" target="_blank" rel="noopener noreferrer" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Discord</a></li>
                <li><a href="https://twitter.com/tyck_dev" target="_blank" rel="noopener noreferrer" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Twitter</a></li>
              </ul>
            </div>
          </div>
        </div>

        <div className="border-t border-[var(--color-border)] pt-6 flex flex-col md:flex-row items-center justify-between gap-4">
          <p className="text-xs text-[var(--color-text-muted)]">
            &copy; {new Date().getFullYear()} Tyck. All rights reserved.
          </p>
          <p className="text-xs text-[var(--color-text-muted)]">
            Built with Tauri, SvelteKit & Rust
          </p>
        </div>
      </div>
    </footer>
  );
}
