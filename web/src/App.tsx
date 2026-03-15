import { Routes, Route } from 'react-router-dom';
import { HomePage } from './pages/Home';
import { DocsLayout } from './components/layout/DocsLayout';
import { InstallationPage } from './pages/docs/Installation';
import { ThemeDevelopmentPage } from './pages/docs/ThemeDevelopment';
import { TappQuickStartPage } from './pages/docs/TappQuickStart';
import { TappDevelopmentPage } from './pages/docs/TappDevelopment';
import { TappUIComponentsPage } from './pages/docs/TappUIComponents';
import { WorktreeModelPage } from './pages/docs/WorktreeModel';
import { DocsIndexPage } from './pages/docs/Index';

export function App() {
  return (
    <Routes>
      <Route path="/" element={<HomePage />} />
      <Route path="/docs" element={<DocsLayout />}>
        <Route index element={<DocsIndexPage />} />
        <Route path="installation" element={<InstallationPage />} />
        <Route path="architecture/worktree" element={<WorktreeModelPage />} />
        <Route path="themes" element={<ThemeDevelopmentPage />} />
        <Route path="tapp" element={<TappQuickStartPage />} />
        <Route path="tapp/development" element={<TappDevelopmentPage />} />
        <Route path="tapp/ui-components" element={<TappUIComponentsPage />} />
      </Route>
    </Routes>
  );
}
