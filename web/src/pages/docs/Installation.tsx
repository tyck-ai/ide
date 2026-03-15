export function InstallationPage() {
  return (
    <div>
      <h1>Installation</h1>
      <p>
        Tyck is available for macOS, Windows, and Linux. Choose your platform below to get started.
      </p>

      <h2>System Requirements</h2>
      <table>
        <thead>
          <tr>
            <th>Platform</th>
            <th>Minimum Version</th>
            <th>Architecture</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>macOS</td>
            <td>macOS 11 (Big Sur)</td>
            <td>Apple Silicon, Intel</td>
          </tr>
          <tr>
            <td>Windows</td>
            <td>Windows 10 (1903+)</td>
            <td>x64</td>
          </tr>
          <tr>
            <td>Linux</td>
            <td>glibc 2.31+</td>
            <td>x64</td>
          </tr>
        </tbody>
      </table>

      <h2>macOS</h2>
      <h3>Download</h3>
      <p>
        Download the appropriate DMG file for your Mac:
      </p>
      <ul>
        <li><strong>Apple Silicon (M1/M2/M3)</strong> — <code>Tyck-aarch64.dmg</code></li>
        <li><strong>Intel</strong> — <code>Tyck-x64.dmg</code></li>
      </ul>

      <h3>Install</h3>
      <ol>
        <li>Open the downloaded <code>.dmg</code> file</li>
        <li>Drag Tyck to your Applications folder</li>
        <li>Open Tyck from Applications</li>
        <li>If prompted about an unidentified developer, go to System Preferences → Security & Privacy and click "Open Anyway"</li>
      </ol>

      <h3>Homebrew (Alternative)</h3>
      <pre><code>brew install --cask tyck</code></pre>

      <h2>Windows</h2>
      <h3>Download</h3>
      <p>
        Download the installer or portable version:
      </p>
      <ul>
        <li><strong>Installer</strong> — <code>Tyck-x64.msi</code></li>
        <li><strong>Portable</strong> — <code>Tyck-x64.zip</code></li>
      </ul>

      <h3>Install (MSI)</h3>
      <ol>
        <li>Run the downloaded <code>.msi</code> installer</li>
        <li>Follow the installation wizard</li>
        <li>Launch Tyck from the Start menu</li>
      </ol>

      <h3>Portable</h3>
      <ol>
        <li>Extract the ZIP file to your preferred location</li>
        <li>Run <code>Tyck.exe</code></li>
      </ol>

      <h3>winget (Alternative)</h3>
      <pre><code>winget install tyck.tyck</code></pre>

      <h2>Linux</h2>
      <h3>Download</h3>
      <p>
        Choose the package format for your distribution:
      </p>
      <ul>
        <li><strong>Ubuntu/Debian</strong> — <code>tyck_amd64.deb</code></li>
        <li><strong>Fedora/RHEL</strong> — <code>tyck.x86_64.rpm</code></li>
        <li><strong>Universal</strong> — <code>Tyck.AppImage</code></li>
      </ul>

      <h3>Ubuntu/Debian</h3>
      <pre><code>{`# Install the .deb package
sudo dpkg -i tyck_amd64.deb

# Install dependencies if needed
sudo apt-get install -f`}</code></pre>

      <h3>Fedora/RHEL</h3>
      <pre><code>sudo rpm -i tyck.x86_64.rpm</code></pre>

      <h3>AppImage</h3>
      <pre><code>{`# Make executable
chmod +x Tyck.AppImage

# Run
./Tyck.AppImage`}</code></pre>

      <h2>Build from Source</h2>
      <p>
        If you want to build Tyck from source, you'll need:
      </p>
      <ul>
        <li><a href="https://www.rust-lang.org/tools/install">Rust</a> (latest stable)</li>
        <li><a href="https://nodejs.org/">Node.js</a> (v20+)</li>
        <li><a href="https://pnpm.io/">pnpm</a></li>
      </ul>

      <pre><code>{`# Clone the repository
git clone https://github.com/tyck-dev/tyck.git
cd tyck

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build`}</code></pre>

      <h2>Updating</h2>
      <p>
        Tyck will check for updates automatically and notify you when a new version is available.
        You can also manually check for updates via <strong>Help → Check for Updates</strong>.
      </p>

      <h2>Uninstalling</h2>
      <h3>macOS</h3>
      <p>Drag Tyck from Applications to Trash.</p>

      <h3>Windows</h3>
      <p>Use "Add or Remove Programs" in Windows Settings.</p>

      <h3>Linux</h3>
      <pre><code>{`# Ubuntu/Debian
sudo apt remove tyck

# Fedora/RHEL
sudo rpm -e tyck

# AppImage: Just delete the file`}</code></pre>

      <h2>Next Steps</h2>
      <p>
        Now that you have Tyck installed, explore these guides:
      </p>
      <ul>
        <li><a href="/docs/themes">Customize with themes</a></li>
        <li><a href="/docs/tapp">Build your first Tapp extension</a></li>
      </ul>
    </div>
  );
}
