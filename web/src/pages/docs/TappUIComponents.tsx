export function TappUIComponentsPage() {
  return (
    <div>
      <h1>UI Components Reference</h1>
      <p>Complete reference for all UI components available in the Tapp SDK.</p>

      <h2>Layout Components</h2>

      <h3>View</h3>
      <p>Generic container component.</p>
      <pre><code>{`ui::view([
    ui::text("Child 1"),
    ui::text("Child 2"),
])
.with_class("my-container")
.with_style("padding: 16px")`}</code></pre>

      <h3>VStack</h3>
      <p>Vertical stack layout.</p>
      <pre><code>{`ui::vstack([
    ui::text("Top"),
    ui::text("Middle"),
    ui::text("Bottom"),
])`}</code></pre>

      <h3>HStack</h3>
      <p>Horizontal stack layout.</p>
      <pre><code>{`ui::hstack([
    ui::text("Left"),
    ui::text("Center"),
    ui::text("Right"),
])`}</code></pre>

      <h3>Panel</h3>
      <p>Container with title header.</p>
      <pre><code>{`ui::panel("Settings").children([
    ui::text("Panel content"),
])`}</code></pre>

      <h3>Split</h3>
      <p>Resizable split panes.</p>
      <pre><code>{`ui::split(ui::SplitDirection::Horizontal)
    .panel(ui::text("Left pane"), 0.3)  // 30% width
    .panel(ui::text("Right pane"), 0.7) // 70% width
    .min_sizes(vec![100, 200])
    .on_resize("split_resized")
    .build()`}</code></pre>

      <h3>Tabs</h3>
      <p>Tabbed interface.</p>
      <pre><code>{`ui::tabs(vec![
    ui::TabItem::new("tab1", "First Tab", ui::text("Content 1")),
    ui::TabItem::new("tab2", "Second Tab", ui::text("Content 2"))
        .icon("settings")
        .closable(),
])
.active("tab1")
.on_change("tab_changed")
.on_close("tab_closed")
.build()`}</code></pre>

      <h3>Modal</h3>
      <p>Modal dialog overlay.</p>
      <pre><code>{`ui::modal("Confirm Action")
    .content(ui::text("Are you sure?"))
    .footer(ui::hstack([
        ui::button("Cancel").on_click("cancel"),
        ui::button("Confirm").primary().on_click("confirm"),
    ]))
    .large()
    .on_close("close_modal")
    .build()`}</code></pre>

      <h3>Scroll</h3>
      <p>Scrollable container.</p>
      <pre><code>{`ui::scroll(ui::vstack([
    // Many items...
]))
.max_height("500px")
.vertical_only()
.on_scroll("scrolled")`}</code></pre>

      <h2>Content Components</h2>

      <h3>Text</h3>
      <p>Text display.</p>
      <pre><code>{`ui::text("Hello, World!")
    .with_class("text-lg font-bold")`}</code></pre>

      <h3>Code</h3>
      <p>Syntax-highlighted code block.</p>
      <pre><code>{`ui::code("fn main() { println!(\\"Hello\\"); }")
    .language("rust")
    .build()`}</code></pre>

      <h3>Markdown</h3>
      <p>Rendered markdown content.</p>
      <pre><code>{`ui::markdown("# Heading\\n\\nParagraph with **bold** text.")`}</code></pre>

      <h3>Icon</h3>
      <p>Icon display.</p>
      <pre><code>{`ui::icon("folder")
ui::icon("file")
ui::icon("check")
ui::icon("warning")`}</code></pre>

      <h3>Badge</h3>
      <p>Small label/badge.</p>
      <pre><code>{`ui::badge("NEW")
ui::badge("3")`}</code></pre>

      <h2>Input Components</h2>

      <h3>Button</h3>
      <p>Clickable button.</p>
      <pre><code>{`ui::button("Click Me")
    .on_click("button_clicked")

ui::button("Primary")
    .primary()
    .on_click("action")

ui::button("Danger")
    .danger()
    .on_click("delete")

ui::button("Disabled")
    .disabled()
    .build()`}</code></pre>

      <h3>Input</h3>
      <p>Text input field.</p>
      <pre><code>{`ui::input()
    .value(&self.text)
    .placeholder("Enter text...")
    .on_change("text_changed")
    .on_submit("text_submitted")

ui::input()
    .password()
    .placeholder("Password")
    .on_change("password_changed")`}</code></pre>

      <h3>TextArea</h3>
      <p>Multi-line text input.</p>
      <pre><code>{`ui::textarea()
    .value(&self.content)
    .placeholder("Enter description...")
    .rows(5)
    .build()`}</code></pre>

      <h3>Select</h3>
      <p>Dropdown selection.</p>
      <pre><code>{`ui::select(vec![
    ui::SelectOption::new("opt1", "Option 1"),
    ui::SelectOption::new("opt2", "Option 2"),
    ui::SelectOption::new("opt3", "Option 3").disabled(),
])
.value(&self.selected)
.placeholder("Choose an option")
.searchable()
.on_change("selection_changed")

// Multi-select
ui::select(options)
    .multi()
    .selected(self.selected_items.clone())
    .on_change("multi_changed")`}</code></pre>

      <h3>Checkbox</h3>
      <p>Boolean checkbox.</p>
      <pre><code>{`ui::checkbox(self.is_checked)
    .on_change("checkbox_changed")`}</code></pre>

      <h3>Toggle</h3>
      <p>Boolean toggle switch.</p>
      <pre><code>{`ui::toggle(self.is_enabled)
    .on_change("toggle_changed")`}</code></pre>

      <h3>Slider</h3>
      <p>Numeric slider.</p>
      <pre><code>{`ui::slider(self.value)
    .range(0.0, 100.0)
    .step(1.0)
    .on_change("slider_changed")`}</code></pre>

      <h2>Data Components</h2>

      <h3>VirtualList</h3>
      <p>Virtualized list for large datasets.</p>
      <pre><code>{`ui::virtual_list(
    self.items.iter().enumerate().map(|(i, item)| {
        ui::VirtualListItem {
            id: format!("item-{}", i),
            content: ui::hstack([
                ui::text(&item.name),
                ui::badge(&item.status),
            ]),
        }
    }).collect::<Vec<_>>(),
    32 // item height
)
.overscan(5)
.build()`}</code></pre>

      <h3>Table</h3>
      <p>Simple table.</p>
      <pre><code>{`ui::table(vec!["Name", "Email", "Status"])
    .row(vec![
        ui::text("John").into(),
        ui::text("john@example.com").into(),
        ui::badge("Active").into(),
    ])
    .bordered()
    .build()`}</code></pre>

      <h3>DataGrid</h3>
      <p>Full-featured data grid with sorting, filtering, selection.</p>
      <pre><code>{`let columns = vec![
    ui::DataGridColumn::new("name", "Name")
        .width(200)
        .sortable(true),
    ui::DataGridColumn::new("size", "Size")
        .width(100),
    ui::DataGridColumn::new("modified", "Modified")
        .width(150),
];

let rows: Vec<ui::DataGridRow> = self.files.iter().map(|f| {
    ui::DataGridRow::new(&f.id)
        .cell("name", &f.name)
        .cell("size", f.size)
        .cell("modified", &f.modified_at)
}).collect();

ui::data_grid(columns)
    .rows(rows)
    .selectable()
    .striped()
    .on_sort("sort_changed")
    .on_row_click("row_clicked")
    .build()`}</code></pre>

      <h3>Tree</h3>
      <p>Hierarchical tree view.</p>
      <pre><code>{`let nodes = vec![
    ui::TreeNode::new("folder1", "Documents")
        .icon("folder")
        .expanded(true)
        .children(vec![
            ui::TreeNode::new("file1", "report.pdf")
                .icon("file"),
            ui::TreeNode::new("file2", "notes.txt")
                .icon("file"),
        ]),
];

ui::tree(nodes)
    .selected("file1")
    .on_select("node_selected")
    .on_toggle("node_toggled")
    .build()`}</code></pre>

      <h2>Feedback Components</h2>

      <h3>Spinner</h3>
      <p>Loading indicator.</p>
      <pre><code>{`ui::spinner()`}</code></pre>

      <h3>Progress</h3>
      <p>Progress bar.</p>
      <pre><code>{`ui::progress(0.75) // 75%`}</code></pre>

      <h3>Empty</h3>
      <p>Empty state placeholder.</p>
      <pre><code>{`ui::empty("No items found")`}</code></pre>

      <h3>Alert</h3>
      <p>Alert/notification banner.</p>
      <pre><code>{`ui::alert("Operation completed successfully")
    .success()
    .build()

ui::alert("Something went wrong")
    .error()
    .build()

ui::alert("Please review before continuing")
    .warning()
    .build()

ui::alert("This is an informational message")
    .info()
    .build()`}</code></pre>

      <h3>Toast</h3>
      <p>Toast notification (ephemeral).</p>
      <pre><code>{`ui::toast("File saved")
    .success()
    .duration(3000)
    .build()

ui::toast("Error occurred")
    .error()
    .persistent()
    .build()`}</code></pre>

      <h2>Event Handling</h2>

      <h3>Standard Events</h3>
      <table>
        <thead>
          <tr>
            <th>Event</th>
            <th>Components</th>
            <th>Data</th>
          </tr>
        </thead>
        <tbody>
          <tr><td><code>on_click</code></td><td>Button, View</td><td><code>on_click_data</code></td></tr>
          <tr><td><code>on_change</code></td><td>Input, Select, Checkbox, Toggle, Slider</td><td><code>value</code></td></tr>
          <tr><td><code>on_submit</code></td><td>Input, TextArea</td><td><code>value</code></td></tr>
          <tr><td><code>on_focus</code></td><td>Input, TextArea</td><td>-</td></tr>
          <tr><td><code>on_blur</code></td><td>Input, TextArea</td><td>-</td></tr>
        </tbody>
      </table>

      <h3>Drag and Drop Events</h3>
      <pre><code>{`// Draggable element
ui::view([ui::text("Drag me")])
    .draggable(true)
    .on_drag_start("drag_started", json!({ "item_id": 1 }))

// Drop target
ui::view([ui::text("Drop here")])
    .on_drop("item_dropped", json!({ "zone": "target1" }))`}</code></pre>

      <h2>Styling</h2>

      <h3>Classes</h3>
      <pre><code>{`ui::text("Styled text")
    .with_class("text-lg font-bold text-blue-500")`}</code></pre>

      <h3>Inline Styles</h3>
      <pre><code>{`ui::view([...])
    .with_style("padding: 16px; background: #f0f0f0;")`}</code></pre>

      <h3>Common Class Names</h3>
      <table>
        <thead>
          <tr>
            <th>Category</th>
            <th>Classes</th>
          </tr>
        </thead>
        <tbody>
          <tr><td>Text Size</td><td><code>text-xs</code>, <code>text-sm</code>, <code>text-base</code>, <code>text-lg</code>, <code>text-xl</code>, <code>text-2xl</code></td></tr>
          <tr><td>Font Weight</td><td><code>font-normal</code>, <code>font-medium</code>, <code>font-semibold</code>, <code>font-bold</code></td></tr>
          <tr><td>Spacing</td><td><code>p-1</code> through <code>p-8</code>, <code>m-1</code> through <code>m-8</code></td></tr>
          <tr><td>Flex</td><td><code>flex</code>, <code>flex-row</code>, <code>flex-col</code>, <code>justify-center</code>, <code>items-center</code></td></tr>
          <tr><td>Borders</td><td><code>border</code>, <code>border-2</code>, <code>rounded</code>, <code>rounded-lg</code></td></tr>
        </tbody>
      </table>

      <h2>Composition Patterns</h2>

      <h3>Conditional Rendering</h3>
      <pre><code>{`fn render(&self) -> UITree {
    ui::vstack([
        if self.is_loading {
            ui::spinner().into()
        } else {
            ui::text("Content loaded").into()
        },
    ])
}`}</code></pre>

      <h3>List Rendering</h3>
      <pre><code>{`fn render(&self) -> UITree {
    ui::vstack(
        self.items.iter().map(|item| {
            ui::hstack([
                ui::text(&item.name),
                ui::button("Delete")
                    .danger()
                    .on_click("delete")
                    .with_prop("on_click_data", json!({ "id": item.id })),
            ])
        }).collect::<Vec<_>>()
    )
}`}</code></pre>

      <h3>Component Extraction</h3>
      <pre><code>{`impl MyApp {
    fn render_header(&self) -> UINode {
        ui::hstack([
            ui::text("My App").with_class("text-xl font-bold"),
            ui::button("Settings").on_click("open_settings"),
        ])
    }

    fn render_content(&self) -> UINode {
        // ...
    }

    fn render(&self) -> UITree {
        ui::vstack([
            self.render_header(),
            self.render_content(),
        ])
    }
}`}</code></pre>

      <h2>Next Steps</h2>
      <ul>
        <li><a href="/docs/tapp/development">Development Guide</a> — Full development reference</li>
        <li><a href="/docs/tapp">Quick Start</a> — Create your first app</li>
      </ul>
    </div>
  );
}
