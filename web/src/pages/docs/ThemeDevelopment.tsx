export function ThemeDevelopmentPage() {
  return (
    <div>
      <h1>Theme Development</h1>
      <p>
        Tyck supports fully customizable themes that let you control colors, fonts, and UI density.
        This guide covers creating, installing, and sharing themes.
      </p>

      <h2>Theme Basics</h2>
      <p>
        Themes are JSON files that define colors, fonts, and other visual properties. Tyck comes with
        built-in light and dark themes, but you can create your own or install community themes.
      </p>

      <h3>Theme Location</h3>
      <p>Custom themes are stored in:</p>
      <ul>
        <li><strong>macOS</strong> — <code>~/.tyck/themes/</code></li>
        <li><strong>Windows</strong> — <code>%APPDATA%\tyck\themes\</code></li>
        <li><strong>Linux</strong> — <code>~/.config/tyck/themes/</code></li>
      </ul>

      <h2>Creating a Theme</h2>
      <h3>Theme Structure</h3>
      <p>
        A theme file has the following structure:
      </p>
      <pre><code>{`{
  "name": "My Theme",
  "type": "dark",
  "colors": {
    "background": "#1a1b26",
    "foreground": "#a9b1d6",
    "accent": "#7aa2f7",
    "surface": "#24283b",
    "border": "#414868",
    
    "editor.background": "#1a1b26",
    "editor.foreground": "#a9b1d6",
    "editor.lineHighlight": "#292e42",
    "editor.selection": "#33467c",
    "editor.cursor": "#c0caf5",
    
    "sidebar.background": "#1f2335",
    "sidebar.foreground": "#787c99",
    
    "panel.background": "#1a1b26",
    "panel.border": "#414868",
    
    "statusBar.background": "#1a1b26",
    "statusBar.foreground": "#787c99",
    
    "button.background": "#7aa2f7",
    "button.foreground": "#1a1b26",
    
    "input.background": "#24283b",
    "input.foreground": "#a9b1d6",
    "input.border": "#414868",
    
    "list.hoverBackground": "#292e42",
    "list.activeBackground": "#33467c",
    
    "syntax.keyword": "#bb9af7",
    "syntax.string": "#9ece6a",
    "syntax.number": "#ff9e64",
    "syntax.function": "#7aa2f7",
    "syntax.comment": "#565f89",
    "syntax.type": "#2ac3de",
    "syntax.variable": "#c0caf5",
    "syntax.constant": "#ff9e64"
  },
  "fonts": {
    "editor": "JetBrains Mono, Menlo, Monaco, monospace",
    "ui": "Inter, system-ui, sans-serif"
  },
  "ui": {
    "fontSize": 13,
    "lineHeight": 1.6,
    "borderRadius": 6
  }
}`}</code></pre>

      <h3>Theme Types</h3>
      <p>
        The <code>type</code> field should be either <code>"light"</code> or <code>"dark"</code>. This helps
        Tyck adjust UI elements appropriately (like scrollbar colors and shadows).
      </p>

      <h2>Color Reference</h2>
      <h3>Base Colors</h3>
      <table>
        <thead>
          <tr>
            <th>Key</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr><td><code>background</code></td><td>Main background color</td></tr>
          <tr><td><code>foreground</code></td><td>Default text color</td></tr>
          <tr><td><code>accent</code></td><td>Primary accent color (buttons, links)</td></tr>
          <tr><td><code>surface</code></td><td>Elevated surface color (cards, dialogs)</td></tr>
          <tr><td><code>border</code></td><td>Border and divider color</td></tr>
        </tbody>
      </table>

      <h3>Editor Colors</h3>
      <table>
        <thead>
          <tr>
            <th>Key</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr><td><code>editor.background</code></td><td>Editor background</td></tr>
          <tr><td><code>editor.foreground</code></td><td>Editor text</td></tr>
          <tr><td><code>editor.lineHighlight</code></td><td>Current line highlight</td></tr>
          <tr><td><code>editor.selection</code></td><td>Selected text background</td></tr>
          <tr><td><code>editor.cursor</code></td><td>Cursor color</td></tr>
          <tr><td><code>editor.lineNumbers</code></td><td>Line number color</td></tr>
          <tr><td><code>editor.activeLineNumber</code></td><td>Active line number color</td></tr>
        </tbody>
      </table>

      <h3>Syntax Highlighting</h3>
      <table>
        <thead>
          <tr>
            <th>Key</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr><td><code>syntax.keyword</code></td><td>Keywords (if, else, fn, etc.)</td></tr>
          <tr><td><code>syntax.string</code></td><td>String literals</td></tr>
          <tr><td><code>syntax.number</code></td><td>Numeric literals</td></tr>
          <tr><td><code>syntax.function</code></td><td>Function names</td></tr>
          <tr><td><code>syntax.comment</code></td><td>Comments</td></tr>
          <tr><td><code>syntax.type</code></td><td>Types and classes</td></tr>
          <tr><td><code>syntax.variable</code></td><td>Variables</td></tr>
          <tr><td><code>syntax.constant</code></td><td>Constants</td></tr>
          <tr><td><code>syntax.operator</code></td><td>Operators (+, -, *, etc.)</td></tr>
        </tbody>
      </table>

      <h2>Installing Themes</h2>
      <h3>From File</h3>
      <ol>
        <li>Download the theme <code>.json</code> file</li>
        <li>Place it in your themes directory (see Theme Location above)</li>
        <li>Restart Tyck or use Command Palette → "Reload Themes"</li>
        <li>Select the theme via Settings → Appearance → Theme</li>
      </ol>

      <h3>From App Store</h3>
      <p>
        Browse and install community themes directly from the Tyck App Store:
      </p>
      <ol>
        <li>Open Command Palette (<code>Cmd+Shift+P</code> / <code>Ctrl+Shift+P</code>)</li>
        <li>Search for "Browse Themes"</li>
        <li>Preview and install themes with one click</li>
      </ol>

      <h2>Switching Themes</h2>
      <p>
        You can switch themes at any time:
      </p>
      <ul>
        <li><strong>Settings UI</strong> — Settings → Appearance → Theme</li>
        <li><strong>Command Palette</strong> — "Preferences: Color Theme"</li>
        <li><strong>Keyboard Shortcut</strong> — <code>Cmd+K Cmd+T</code> / <code>Ctrl+K Ctrl+T</code></li>
      </ul>

      <h2>Publishing Themes</h2>
      <p>
        Share your theme with the community by publishing to the Tyck App Store:
      </p>

      <h3>1. Create a Manifest</h3>
      <pre><code>{`{
  "id": "my-awesome-theme",
  "name": "My Awesome Theme",
  "version": "1.0.0",
  "description": "A beautiful dark theme inspired by...",
  "author": "Your Name",
  "repository": "https://github.com/username/my-theme",
  "type": "theme",
  "theme": "./theme.json"
}`}</code></pre>

      <h3>2. Package Your Theme</h3>
      <pre><code>{`# Directory structure
my-awesome-theme/
├── manifest.json
├── theme.json
├── preview.png   # 1200x800 screenshot
└── README.md`}</code></pre>

      <h3>3. Submit to App Store</h3>
      <pre><code>{`# Using the tapp CLI
tapp publish ./my-awesome-theme`}</code></pre>

      <h2>Tips & Best Practices</h2>
      <ul>
        <li><strong>Consistent contrast</strong> — Ensure text is readable against all backgrounds</li>
        <li><strong>Test syntax highlighting</strong> — Preview with multiple languages</li>
        <li><strong>Include a preview image</strong> — Help users see what they're installing</li>
        <li><strong>Version your themes</strong> — Use semantic versioning for updates</li>
        <li><strong>Credit inspirations</strong> — If based on another theme, mention it</li>
      </ul>

      <h2>Example Themes</h2>
      <p>
        Looking for inspiration? Check out these popular themes:
      </p>
      <ul>
        <li><a href="https://github.com/tyck-dev/themes/tree/main/tokyo-night">Tokyo Night</a></li>
        <li><a href="https://github.com/tyck-dev/themes/tree/main/catppuccin">Catppuccin</a></li>
        <li><a href="https://github.com/tyck-dev/themes/tree/main/nord">Nord</a></li>
        <li><a href="https://github.com/tyck-dev/themes/tree/main/dracula">Dracula</a></li>
      </ul>

      <h2>Next Steps</h2>
      <ul>
        <li><a href="/docs/tapp">Build Tapp extensions</a> — Create apps and tools</li>
        <li><a href="https://github.com/tyck-dev/themes">Browse theme gallery</a> — Find community themes</li>
      </ul>
    </div>
  );
}
