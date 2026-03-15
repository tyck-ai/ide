# UI Components Reference

Complete reference for all UI components available in the Tapp SDK.

## Layout Components

### View
Generic container component.

```rust
ui::view([
    ui::text("Child 1"),
    ui::text("Child 2"),
])
.with_class("my-container")
.with_style("padding: 16px")
```

### VStack
Vertical stack layout.

```rust
ui::vstack([
    ui::text("Top"),
    ui::text("Middle"),
    ui::text("Bottom"),
])
```

### HStack
Horizontal stack layout.

```rust
ui::hstack([
    ui::text("Left"),
    ui::text("Center"),
    ui::text("Right"),
])
```

### Panel
Container with title header.

```rust
ui::panel("Settings").children([
    ui::text("Panel content"),
])
```

### Split
Resizable split panes.

```rust
ui::split(ui::SplitDirection::Horizontal)
    .panel(ui::text("Left pane"), 0.3)  // 30% width
    .panel(ui::text("Right pane"), 0.7) // 70% width
    .min_sizes(vec![100, 200])
    .on_resize("split_resized")
    .build()
```

### Tabs
Tabbed interface.

```rust
ui::tabs(vec![
    ui::TabItem::new("tab1", "First Tab", ui::text("Content 1")),
    ui::TabItem::new("tab2", "Second Tab", ui::text("Content 2"))
        .icon("settings")
        .closable(),
])
.active("tab1")
.on_change("tab_changed")
.on_close("tab_closed")
.build()
```

### Modal
Modal dialog overlay.

```rust
ui::modal("Confirm Action")
    .content(ui::text("Are you sure?"))
    .footer(ui::hstack([
        ui::button("Cancel").on_click("cancel"),
        ui::button("Confirm").primary().on_click("confirm"),
    ]))
    .large()
    .on_close("close_modal")
    .build()
```

### Drawer
Slide-in panel.

```rust
ui::drawer(ui::DrawerPosition::Right)
    .title("Details")
    .content(ui::text("Drawer content"))
    .size(400)
    .on_close("close_drawer")
    .build()
```

### Scroll
Scrollable container.

```rust
ui::scroll(ui::vstack([
    // Many items...
]))
.max_height("500px")
.vertical_only()
.on_scroll("scrolled")
```

## Content Components

### Text
Text display.

```rust
ui::text("Hello, World!")
    .with_class("text-lg font-bold")
```

### Code
Syntax-highlighted code block.

```rust
ui::code("fn main() { println!(\"Hello\"); }")
    .language("rust")
    .build()
```

### Markdown
Rendered markdown content.

```rust
ui::markdown("# Heading\n\nParagraph with **bold** text.")
```

### Icon
Icon display.

```rust
ui::icon("folder")
ui::icon("file")
ui::icon("check")
ui::icon("warning")
```

### Image
Image display.

```rust
ui::image("/path/to/image.png")
    .alt("Description")
    .build()
```

### Badge
Small label/badge.

```rust
ui::badge("NEW")
ui::badge("3")
```

### Avatar
User avatar.

```rust
// With image
ui::avatar("/path/to/avatar.jpg")
    .large()
    .build()

// With initials
ui::avatar_initials("John Doe")
    .small()
    .build()
```

## Input Components

### Button
Clickable button.

```rust
ui::button("Click Me")
    .on_click("button_clicked")

ui::button("Primary")
    .primary()
    .on_click("action")

ui::button("Danger")
    .danger()
    .on_click("delete")

ui::button("Disabled")
    .disabled()
    .build()
```

### Input
Text input field.

```rust
ui::input()
    .value(&self.text)
    .placeholder("Enter text...")
    .on_change("text_changed")
    .on_submit("text_submitted")

ui::input()
    .password()
    .placeholder("Password")
    .on_change("password_changed")
```

### TextArea
Multi-line text input.

```rust
ui::textarea()
    .value(&self.content)
    .placeholder("Enter description...")
    .rows(5)
    .build()
```

### Select
Dropdown selection.

```rust
ui::select(vec![
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
    .on_change("multi_changed")
```

### Checkbox
Boolean checkbox.

```rust
ui::checkbox(self.is_checked)
    .on_change("checkbox_changed")
```

### Toggle
Boolean toggle switch.

```rust
ui::toggle(self.is_enabled)
    .on_change("toggle_changed")
```

### Slider
Numeric slider.

```rust
ui::slider(self.value)
    .range(0.0, 100.0)
    .step(1.0)
    .on_change("slider_changed")
```

## Data Components

### List
Simple list.

```rust
ui::list(
    self.items.iter(),
    |item| ui::text(&item.name)
)
```

### VirtualList
Virtualized list for large datasets.

```rust
ui::virtual_list(
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
.build()
```

### Table
Simple table.

```rust
ui::table(vec!["Name", "Email", "Status"])
    .row(vec![
        ui::text("John").into(),
        ui::text("john@example.com").into(),
        ui::badge("Active").into(),
    ])
    .row(vec![
        ui::text("Jane").into(),
        ui::text("jane@example.com").into(),
        ui::badge("Pending").into(),
    ])
    .bordered()
    .build()
```

### DataGrid
Full-featured data grid with sorting, filtering, selection.

```rust
let columns = vec![
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
    .selected(self.selected_rows.clone())
    .sort("name", ui::SortDirection::Ascending)
    .striped()
    .on_sort("sort_changed")
    .on_row_click("row_clicked")
    .on_selection_change("selection_changed")
    .build()
```

### Tree
Hierarchical tree view.

```rust
let nodes = vec![
    ui::TreeNode::new("folder1", "Documents")
        .icon("folder")
        .expanded(true)
        .children(vec![
            ui::TreeNode::new("file1", "report.pdf")
                .icon("file"),
            ui::TreeNode::new("file2", "notes.txt")
                .icon("file"),
            ui::TreeNode::new("folder2", "Projects")
                .icon("folder")
                .children(vec![
                    ui::TreeNode::new("file3", "project.json")
                        .icon("file"),
                ]),
        ]),
];

ui::tree(nodes)
    .selected("file1")
    .on_select("node_selected")
    .on_toggle("node_toggled")
    .on_double_click("node_opened")
    .build()
```

## Feedback Components

### Spinner
Loading indicator.

```rust
ui::spinner()
```

### Progress
Progress bar.

```rust
ui::progress(0.75) // 75%
```

### Empty
Empty state placeholder.

```rust
ui::empty("No items found")
```

### Alert
Alert/notification banner.

```rust
ui::alert("Operation completed successfully")
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
    .build()
```

### Toast
Toast notification (ephemeral).

```rust
ui::toast("File saved")
    .success()
    .duration(3000)
    .build()

ui::toast("Error occurred")
    .error()
    .persistent()
    .build()
```

### Skeleton
Loading placeholder.

```rust
ui::skeleton()
    .text()
    .width("200px")
    .count(3)
    .build()

ui::skeleton()
    .circle()
    .width("48px")
    .height("48px")
    .build()

ui::skeleton()
    .rect()
    .width("100%")
    .height("200px")
    .build()
```

## Event Handling

### Standard Events

| Event | Components | Data |
|-------|------------|------|
| `on_click` | Button, View | `on_click_data` |
| `on_change` | Input, Select, Checkbox, Toggle, Slider | `value` |
| `on_submit` | Input, TextArea | `value` |
| `on_focus` | Input, TextArea | - |
| `on_blur` | Input, TextArea | - |

### Drag and Drop Events

```rust
// Draggable element
ui::view([ui::text("Drag me")])
    .draggable(true)
    .on_drag_start("drag_started", json!({ "item_id": 1 }))

// Drop target
ui::view([ui::text("Drop here")])
    .on_drop("item_dropped", json!({ "zone": "target1" }))
```

### Scroll Events

```rust
ui::scroll(content)
    .on_scroll("scroll_position_changed")
```

## Styling

### Classes
```rust
ui::text("Styled text")
    .with_class("text-lg font-bold text-blue-500")
```

### Inline Styles
```rust
ui::view([...])
    .with_style("padding: 16px; background: #f0f0f0;")
```

### Common Class Names

| Category | Classes |
|----------|---------|
| **Text Size** | `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`, `text-2xl` |
| **Font Weight** | `font-normal`, `font-medium`, `font-semibold`, `font-bold` |
| **Spacing** | `p-{1-8}`, `m-{1-8}`, `px-{1-8}`, `py-{1-8}` |
| **Flex** | `flex`, `flex-row`, `flex-col`, `justify-center`, `items-center` |
| **Colors** | `text-{color}`, `bg-{color}`, `border-{color}` |
| **Borders** | `border`, `border-2`, `rounded`, `rounded-lg` |

## Composition Patterns

### Conditional Rendering
```rust
fn render(&self) -> UITree {
    ui::vstack([
        if self.is_loading {
            ui::spinner().into()
        } else {
            ui::text("Content loaded").into()
        },
    ])
}
```

### List Rendering
```rust
fn render(&self) -> UITree {
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
}
```

### Component Extraction
```rust
impl MyApp {
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
}
```
