import { Link } from 'react-router-dom';
import { Logo } from '../ui';

export function Navbar() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 landing-nav">
      <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
        <Link to="/" className="flex items-center gap-1.5 group">
          <Logo size={24} />
          <span className="text-lg font-bold text-[var(--color-text)] tracking-tight">tyck</span>
        </Link>

        <div className="hidden md:flex items-center gap-8">
          <a href="#features" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Features</a>
          <a href="#download" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Download</a>
          <Link to="/docs" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">Docs</Link>
          <a href="https://github.com/tyck-dev/tyck" target="_blank" rel="noopener noreferrer" className="text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">GitHub</a>
        </div>

        <div className="flex items-center gap-3">
          <Link
            to="/docs"
            className="text-sm font-medium bg-[var(--color-text)] text-white px-4 py-2 rounded-lg hover:opacity-90 transition-opacity"
          >
            Get Started
          </Link>
        </div>
      </div>
    </nav>
  );
}
