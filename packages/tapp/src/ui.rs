use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub type UITree = UINode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UINode {
    pub id: String,
    pub node_type: NodeType,
    pub props: HashMap<String, Value>,
    pub children: Vec<UINode>,
}

impl UINode {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            id: generate_id(),
            node_type,
            props: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_prop(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.props.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }

    pub fn with_class(self, class: impl Into<String>) -> Self {
        self.with_prop("class", class.into())
    }

    pub fn with_style(self, style: impl Into<String>) -> Self {
        self.with_prop("style", style.into())
    }

    pub fn with_children(mut self, children: Vec<UINode>) -> Self {
        self.children = children;
        self
    }

    pub fn add_child(mut self, child: UINode) -> Self {
        self.children.push(child);
        self
    }

    pub fn on_click(self, action: impl Into<String>) -> Self {
        self.with_prop("on_click", action.into())
    }

    pub fn on_change(self, action: impl Into<String>) -> Self {
        self.with_prop("on_change", action.into())
    }

    pub fn on_submit(self, action: impl Into<String>) -> Self {
        self.with_prop("on_submit", action.into())
    }

    pub fn draggable(self, draggable: bool) -> Self {
        self.with_prop("draggable", draggable)
    }

    pub fn on_drag_start(self, action: impl Into<String>, data: Value) -> Self {
        self.with_prop("on_drag_start", action.into())
            .with_prop("on_drag_start_data", data)
    }

    pub fn on_drop(self, action: impl Into<String>, data: Value) -> Self {
        self.with_prop("on_drop", action.into())
            .with_prop("on_drop_data", data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // Layout
    View,
    #[serde(rename = "hstack")]
    HStack,
    #[serde(rename = "vstack")]
    VStack,
    Panel,
    Split,
    Tabs,
    Scroll,
    Modal,
    Drawer,

    // Content
    Text,
    Code,
    Markdown,
    Icon,
    Image,
    Badge,
    Avatar,

    // Input
    Button,
    Input,
    #[serde(rename = "textarea")]
    TextArea,
    Select,
    Checkbox,
    Toggle,
    Slider,

    // Data
    List,
    VirtualList,
    Tree,
    Table,
    DataGrid,

    // Feedback
    Toast,
    Progress,
    Spinner,
    Empty,
    Skeleton,
    Alert,
}

use std::sync::atomic::{AtomicU64, Ordering};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Reset the ID counter to 0. Called before each render pass so that
/// node IDs are deterministic and stable across renders.
pub fn reset_id_counter() {
    ID_COUNTER.store(0, Ordering::Relaxed);
}

fn generate_id() -> String {
    let n = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("node_{}", n)
}

pub fn view(children: impl IntoIterator<Item = UINode>) -> UINode {
    UINode::new(NodeType::View).with_children(children.into_iter().collect())
}

pub fn hstack(children: impl IntoIterator<Item = UINode>) -> UINode {
    UINode::new(NodeType::HStack).with_children(children.into_iter().collect())
}

pub fn vstack(children: impl IntoIterator<Item = UINode>) -> UINode {
    UINode::new(NodeType::VStack).with_children(children.into_iter().collect())
}

pub fn panel(title: impl Into<String>) -> PanelBuilder {
    PanelBuilder {
        title: title.into(),
        children: Vec::new(),
    }
}

pub struct PanelBuilder {
    title: String,
    children: Vec<UINode>,
}

impl PanelBuilder {
    pub fn children(self, children: impl IntoIterator<Item = UINode>) -> UINode {
        UINode::new(NodeType::Panel)
            .with_prop("title", self.title)
            .with_children(children.into_iter().collect())
    }
}

pub fn text(content: impl Into<String>) -> UINode {
    UINode::new(NodeType::Text).with_prop("content", content.into())
}

pub fn code(content: impl Into<String>) -> CodeBuilder {
    CodeBuilder {
        content: content.into(),
        language: None,
    }
}

pub struct CodeBuilder {
    content: String,
    language: Option<String>,
}

impl CodeBuilder {
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Code).with_prop("content", self.content);
        if let Some(lang) = self.language {
            node = node.with_prop("language", lang);
        }
        node
    }
}

impl From<CodeBuilder> for UINode {
    fn from(builder: CodeBuilder) -> Self {
        builder.build()
    }
}

pub fn markdown(content: impl Into<String>) -> UINode {
    UINode::new(NodeType::Markdown).with_prop("content", content.into())
}

pub fn icon(name: impl Into<String>) -> UINode {
    UINode::new(NodeType::Icon).with_prop("name", name.into())
}

pub fn image(src: impl Into<String>) -> ImageBuilder {
    ImageBuilder {
        src: src.into(),
        alt: None,
    }
}

pub struct ImageBuilder {
    src: String,
    alt: Option<String>,
}

impl ImageBuilder {
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Image).with_prop("src", self.src);
        if let Some(alt) = self.alt {
            node = node.with_prop("alt", alt);
        }
        node
    }
}

impl From<ImageBuilder> for UINode {
    fn from(builder: ImageBuilder) -> Self {
        builder.build()
    }
}

pub fn badge(text: impl Into<String>) -> UINode {
    UINode::new(NodeType::Badge).with_prop("text", text.into())
}

pub fn button(label: impl Into<String>) -> ButtonBuilder {
    ButtonBuilder {
        label: label.into(),
        variant: None,
        disabled: false,
    }
}

pub struct ButtonBuilder {
    label: String,
    variant: Option<String>,
    disabled: bool,
}

impl ButtonBuilder {
    pub fn variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    pub fn primary(self) -> Self {
        self.variant("primary")
    }

    pub fn secondary(self) -> Self {
        self.variant("secondary")
    }

    pub fn danger(self) -> Self {
        self.variant("danger")
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn on_click(self, action: impl Into<String>) -> UINode {
        self.build().on_click(action)
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Button)
            .with_prop("label", self.label)
            .with_prop("disabled", self.disabled);

        if let Some(variant) = self.variant {
            node = node.with_prop("variant", variant);
        }

        node
    }
}

impl From<ButtonBuilder> for UINode {
    fn from(builder: ButtonBuilder) -> Self {
        builder.build()
    }
}

pub fn input() -> InputBuilder {
    InputBuilder {
        value: None,
        placeholder: None,
        input_type: None,
    }
}

pub struct InputBuilder {
    value: Option<String>,
    placeholder: Option<String>,
    input_type: Option<String>,
}

impl InputBuilder {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn password(mut self) -> Self {
        self.input_type = Some("password".to_string());
        self
    }

    pub fn on_change(self, action: impl Into<String>) -> UINode {
        self.build().on_change(action)
    }

    pub fn on_submit(self, action: impl Into<String>) -> UINode {
        self.build().on_submit(action)
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Input);

        if let Some(value) = self.value {
            node = node.with_prop("value", value);
        }
        if let Some(placeholder) = self.placeholder {
            node = node.with_prop("placeholder", placeholder);
        }
        if let Some(input_type) = self.input_type {
            node = node.with_prop("type", input_type);
        }

        node
    }
}

impl From<InputBuilder> for UINode {
    fn from(builder: InputBuilder) -> Self {
        builder.build()
    }
}

pub fn textarea() -> TextAreaBuilder {
    TextAreaBuilder {
        value: None,
        placeholder: None,
        rows: None,
    }
}

pub struct TextAreaBuilder {
    value: Option<String>,
    placeholder: Option<String>,
    rows: Option<u32>,
}

impl TextAreaBuilder {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = Some(rows);
        self
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::TextArea);

        if let Some(value) = self.value {
            node = node.with_prop("value", value);
        }
        if let Some(placeholder) = self.placeholder {
            node = node.with_prop("placeholder", placeholder);
        }
        if let Some(rows) = self.rows {
            node = node.with_prop("rows", rows);
        }

        node
    }
}

impl From<TextAreaBuilder> for UINode {
    fn from(builder: TextAreaBuilder) -> Self {
        builder.build()
    }
}

pub fn checkbox(checked: bool) -> UINode {
    UINode::new(NodeType::Checkbox).with_prop("checked", checked)
}

pub fn toggle(enabled: bool) -> UINode {
    UINode::new(NodeType::Toggle).with_prop("enabled", enabled)
}

pub fn list<I, F>(items: I, render: F) -> UINode
where
    I: IntoIterator,
    F: Fn(I::Item) -> UINode,
{
    let children: Vec<UINode> = items.into_iter().map(render).collect();
    UINode::new(NodeType::List).with_children(children)
}

pub fn spinner() -> UINode {
    UINode::new(NodeType::Spinner)
}

pub fn progress(value: f64) -> UINode {
    UINode::new(NodeType::Progress).with_prop("value", value)
}

pub fn empty(message: impl Into<String>) -> UINode {
    UINode::new(NodeType::Empty).with_prop("message", message.into())
}

pub fn alert(message: impl Into<String>) -> AlertBuilder {
    AlertBuilder {
        message: message.into(),
        variant: None,
    }
}

pub struct AlertBuilder {
    message: String,
    variant: Option<String>,
}

impl AlertBuilder {
    pub fn info(mut self) -> Self {
        self.variant = Some("info".to_string());
        self
    }

    pub fn warning(mut self) -> Self {
        self.variant = Some("warning".to_string());
        self
    }

    pub fn error(mut self) -> Self {
        self.variant = Some("error".to_string());
        self
    }

    pub fn success(mut self) -> Self {
        self.variant = Some("success".to_string());
        self
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Alert).with_prop("message", self.message);

        if let Some(variant) = self.variant {
            node = node.with_prop("variant", variant);
        }

        node
    }
}

impl From<AlertBuilder> for UINode {
    fn from(builder: AlertBuilder) -> Self {
        builder.build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualListItem {
    pub id: String,
    pub content: UINode,
}

pub fn virtual_list<I>(items: I, item_height: u32) -> VirtualListBuilder
where
    I: IntoIterator<Item = VirtualListItem>,
{
    let items: Vec<VirtualListItem> = items.into_iter().collect();
    VirtualListBuilder {
        total_items: items.len(),
        item_height,
        items,
        overscan: 5,
    }
}

pub fn virtual_list_lazy(total_items: usize, item_height: u32) -> VirtualListBuilder {
    VirtualListBuilder {
        total_items,
        item_height,
        items: Vec::new(),
        overscan: 5,
    }
}

pub struct VirtualListBuilder {
    total_items: usize,
    item_height: u32,
    items: Vec<VirtualListItem>,
    overscan: usize,
}

impl VirtualListBuilder {
    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    pub fn with_visible_items(mut self, items: Vec<VirtualListItem>) -> Self {
        self.items = items;
        self
    }

    pub fn build(self) -> UINode {
        let children: Vec<UINode> = self.items.into_iter().map(|item| {
            UINode::new(NodeType::View)
                .with_id(item.id)
                .add_child(item.content)
        }).collect();

        UINode::new(NodeType::VirtualList)
            .with_prop("total_items", self.total_items)
            .with_prop("item_height", self.item_height)
            .with_prop("overscan", self.overscan)
            .with_children(children)
    }
}

impl From<VirtualListBuilder> for UINode {
    fn from(builder: VirtualListBuilder) -> Self {
        builder.build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGridColumn {
    pub id: String,
    pub header: String,
    pub width: Option<u32>,
    pub sortable: bool,
    pub resizable: bool,
}

impl DataGridColumn {
    pub fn new(id: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width: None,
            sortable: true,
            resizable: true,
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGridRow {
    pub id: String,
    pub cells: HashMap<String, Value>,
}

impl DataGridRow {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            cells: HashMap::new(),
        }
    }

    pub fn cell(mut self, column_id: impl Into<String>, value: impl Serialize) -> Self {
        self.cells.insert(
            column_id.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataGridSort {
    pub column: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

pub fn data_grid(columns: Vec<DataGridColumn>) -> DataGridBuilder {
    DataGridBuilder {
        columns,
        rows: Vec::new(),
        sort: None,
        filter: None,
        selectable: false,
        selected_rows: Vec::new(),
        row_height: 32,
        header_height: 40,
        striped: false,
    }
}

pub struct DataGridBuilder {
    columns: Vec<DataGridColumn>,
    rows: Vec<DataGridRow>,
    sort: Option<DataGridSort>,
    filter: Option<String>,
    selectable: bool,
    selected_rows: Vec<String>,
    row_height: u32,
    header_height: u32,
    striped: bool,
}

impl DataGridBuilder {
    pub fn rows(mut self, rows: Vec<DataGridRow>) -> Self {
        self.rows = rows;
        self
    }

    pub fn sort(mut self, column: impl Into<String>, direction: SortDirection) -> Self {
        self.sort = Some(DataGridSort {
            column: column.into(),
            direction,
        });
        self
    }

    pub fn filter(mut self, query: impl Into<String>) -> Self {
        self.filter = Some(query.into());
        self
    }

    pub fn selectable(mut self) -> Self {
        self.selectable = true;
        self
    }

    pub fn selected(mut self, row_ids: Vec<String>) -> Self {
        self.selected_rows = row_ids;
        self
    }

    pub fn row_height(mut self, height: u32) -> Self {
        self.row_height = height;
        self
    }

    pub fn header_height(mut self, height: u32) -> Self {
        self.header_height = height;
        self
    }

    pub fn striped(mut self) -> Self {
        self.striped = true;
        self
    }

    pub fn on_sort(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_sort", action.into())
    }

    pub fn on_filter(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_filter", action.into())
    }

    pub fn on_row_click(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_row_click", action.into())
    }

    pub fn on_selection_change(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_selection_change", action.into())
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::DataGrid)
            .with_prop("columns", &self.columns)
            .with_prop("rows", &self.rows)
            .with_prop("selectable", self.selectable)
            .with_prop("selected_rows", &self.selected_rows)
            .with_prop("row_height", self.row_height)
            .with_prop("header_height", self.header_height)
            .with_prop("striped", self.striped);

        if let Some(sort) = self.sort {
            node = node.with_prop("sort", sort);
        }
        if let Some(filter) = self.filter {
            node = node.with_prop("filter", filter);
        }

        node
    }
}

impl From<DataGridBuilder> for UINode {
    fn from(builder: DataGridBuilder) -> Self {
        builder.build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub selectable: bool,
    pub data: Option<Value>,
}

impl TreeNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
            selectable: true,
            data: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }
}

pub fn tree(nodes: Vec<TreeNode>) -> TreeBuilder {
    TreeBuilder {
        nodes,
        selected: None,
        multi_select: false,
        selected_nodes: Vec::new(),
        show_icons: true,
        indent_size: 20,
        expandable: true,
    }
}

pub struct TreeBuilder {
    nodes: Vec<TreeNode>,
    selected: Option<String>,
    multi_select: bool,
    selected_nodes: Vec<String>,
    show_icons: bool,
    indent_size: u32,
    expandable: bool,
}

impl TreeBuilder {
    pub fn selected(mut self, node_id: impl Into<String>) -> Self {
        self.selected = Some(node_id.into());
        self
    }

    pub fn multi_select(mut self) -> Self {
        self.multi_select = true;
        self
    }

    pub fn selected_nodes(mut self, node_ids: Vec<String>) -> Self {
        self.selected_nodes = node_ids;
        self
    }

    pub fn hide_icons(mut self) -> Self {
        self.show_icons = false;
        self
    }

    pub fn indent_size(mut self, size: u32) -> Self {
        self.indent_size = size;
        self
    }

    pub fn non_expandable(mut self) -> Self {
        self.expandable = false;
        self
    }

    pub fn on_select(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_select", action.into())
    }

    pub fn on_toggle(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_toggle", action.into())
    }

    pub fn on_double_click(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_double_click", action.into())
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Tree)
            .with_prop("nodes", &self.nodes)
            .with_prop("multi_select", self.multi_select)
            .with_prop("selected_nodes", &self.selected_nodes)
            .with_prop("show_icons", self.show_icons)
            .with_prop("indent_size", self.indent_size)
            .with_prop("expandable", self.expandable);

        if let Some(selected) = self.selected {
            node = node.with_prop("selected", selected);
        }

        node
    }
}

impl From<TreeBuilder> for UINode {
    fn from(builder: TreeBuilder) -> Self {
        builder.build()
    }
}

pub fn table(headers: Vec<impl Into<String>>) -> TableBuilder {
    TableBuilder {
        headers: headers.into_iter().map(Into::into).collect(),
        rows: Vec::new(),
        bordered: false,
        hoverable: true,
        compact: false,
    }
}

pub struct TableBuilder {
    headers: Vec<String>,
    rows: Vec<Vec<UINode>>,
    bordered: bool,
    hoverable: bool,
    compact: bool,
}

impl TableBuilder {
    pub fn row(mut self, cells: Vec<impl Into<UINode>>) -> Self {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<impl Into<UINode> + Clone>>) -> Self {
        self.rows = rows.into_iter()
            .map(|row| row.into_iter().map(|c| c.into()).collect())
            .collect();
        self
    }

    pub fn bordered(mut self) -> Self {
        self.bordered = true;
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn no_hover(mut self) -> Self {
        self.hoverable = false;
        self
    }

    pub fn build(self) -> UINode {
        let header_row = UINode::new(NodeType::View)
            .with_class("table-header")
            .with_children(
                self.headers.into_iter()
                    .map(|h| UINode::new(NodeType::Text).with_prop("content", h))
                    .collect()
            );

        let body_rows: Vec<UINode> = self.rows.into_iter()
            .map(|row| {
                UINode::new(NodeType::View)
                    .with_class("table-row")
                    .with_children(row)
            })
            .collect();

        let mut children = vec![header_row];
        children.extend(body_rows);

        UINode::new(NodeType::Table)
            .with_prop("bordered", self.bordered)
            .with_prop("hoverable", self.hoverable)
            .with_prop("compact", self.compact)
            .with_children(children)
    }
}

impl From<TableBuilder> for UINode {
    fn from(builder: TableBuilder) -> Self {
        builder.build()
    }
}

pub fn tabs(items: Vec<TabItem>) -> TabsBuilder {
    TabsBuilder {
        items,
        active: None,
        variant: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub content: UINode,
    pub closable: bool,
}

impl TabItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, content: UINode) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            content,
            closable: false,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }
}

pub struct TabsBuilder {
    items: Vec<TabItem>,
    active: Option<String>,
    variant: Option<String>,
}

impl TabsBuilder {
    pub fn active(mut self, tab_id: impl Into<String>) -> Self {
        self.active = Some(tab_id.into());
        self
    }

    pub fn pills(mut self) -> Self {
        self.variant = Some("pills".to_string());
        self
    }

    pub fn underline(mut self) -> Self {
        self.variant = Some("underline".to_string());
        self
    }

    pub fn on_change(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_change", action.into())
    }

    pub fn on_close(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_close", action.into())
    }

    pub fn build(self) -> UINode {
        let children: Vec<UINode> = self.items.into_iter()
            .map(|item| {
                let mut tab = UINode::new(NodeType::View)
                    .with_id(&item.id)
                    .with_prop("label", item.label)
                    .with_prop("closable", item.closable)
                    .add_child(item.content);

                if let Some(icon) = item.icon {
                    tab = tab.with_prop("icon", icon);
                }

                tab
            })
            .collect();

        let mut node = UINode::new(NodeType::Tabs).with_children(children);

        if let Some(active) = self.active {
            node = node.with_prop("active", active);
        }
        if let Some(variant) = self.variant {
            node = node.with_prop("variant", variant);
        }

        node
    }
}

impl From<TabsBuilder> for UINode {
    fn from(builder: TabsBuilder) -> Self {
        builder.build()
    }
}

pub fn split(direction: SplitDirection) -> SplitBuilder {
    SplitBuilder {
        direction,
        children: Vec::new(),
        sizes: Vec::new(),
        min_sizes: Vec::new(),
        resizable: true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

pub struct SplitBuilder {
    direction: SplitDirection,
    children: Vec<UINode>,
    sizes: Vec<f64>,
    min_sizes: Vec<u32>,
    resizable: bool,
}

impl SplitBuilder {
    pub fn panel(mut self, content: impl Into<UINode>, size: f64) -> Self {
        self.children.push(content.into());
        self.sizes.push(size);
        self
    }

    pub fn min_sizes(mut self, mins: Vec<u32>) -> Self {
        self.min_sizes = mins;
        self
    }

    pub fn fixed(mut self) -> Self {
        self.resizable = false;
        self
    }

    pub fn on_resize(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_resize", action.into())
    }

    pub fn build(self) -> UINode {
        UINode::new(NodeType::Split)
            .with_prop("direction", &self.direction)
            .with_prop("sizes", &self.sizes)
            .with_prop("min_sizes", &self.min_sizes)
            .with_prop("resizable", self.resizable)
            .with_children(self.children)
    }
}

impl From<SplitBuilder> for UINode {
    fn from(builder: SplitBuilder) -> Self {
        builder.build()
    }
}

pub fn modal(title: impl Into<String>) -> ModalBuilder {
    ModalBuilder {
        title: title.into(),
        content: None,
        footer: None,
        closable: true,
        size: None,
    }
}

pub struct ModalBuilder {
    title: String,
    content: Option<UINode>,
    footer: Option<UINode>,
    closable: bool,
    size: Option<String>,
}

impl ModalBuilder {
    pub fn content(mut self, content: impl Into<UINode>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<UINode>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn not_closable(mut self) -> Self {
        self.closable = false;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = Some("sm".to_string());
        self
    }

    pub fn large(mut self) -> Self {
        self.size = Some("lg".to_string());
        self
    }

    pub fn full(mut self) -> Self {
        self.size = Some("full".to_string());
        self
    }

    pub fn on_close(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_close", action.into())
    }

    pub fn build(self) -> UINode {
        let mut children = Vec::new();

        if let Some(content) = self.content {
            children.push(UINode::new(NodeType::View)
                .with_class("modal-body")
                .add_child(content));
        }

        if let Some(footer) = self.footer {
            children.push(UINode::new(NodeType::View)
                .with_class("modal-footer")
                .add_child(footer));
        }

        let mut node = UINode::new(NodeType::Modal)
            .with_prop("title", self.title)
            .with_prop("closable", self.closable)
            .with_children(children);

        if let Some(size) = self.size {
            node = node.with_prop("size", size);
        }

        node
    }
}

impl From<ModalBuilder> for UINode {
    fn from(builder: ModalBuilder) -> Self {
        builder.build()
    }
}

pub fn drawer(position: DrawerPosition) -> DrawerBuilder {
    DrawerBuilder {
        position,
        title: None,
        content: None,
        size: None,
        closable: true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawerPosition {
    Left,
    Right,
    Top,
    Bottom,
}

pub struct DrawerBuilder {
    position: DrawerPosition,
    title: Option<String>,
    content: Option<UINode>,
    size: Option<u32>,
    closable: bool,
}

impl DrawerBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, content: impl Into<UINode>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn size(mut self, size: u32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn not_closable(mut self) -> Self {
        self.closable = false;
        self
    }

    pub fn on_close(self, action: impl Into<String>) -> UINode {
        self.build().with_prop("on_close", action.into())
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Drawer)
            .with_prop("position", &self.position)
            .with_prop("closable", self.closable);

        if let Some(title) = self.title {
            node = node.with_prop("title", title);
        }
        if let Some(size) = self.size {
            node = node.with_prop("size", size);
        }
        if let Some(content) = self.content {
            node = node.add_child(content);
        }

        node
    }
}

impl From<DrawerBuilder> for UINode {
    fn from(builder: DrawerBuilder) -> Self {
        builder.build()
    }
}

pub fn slider(value: f64) -> SliderBuilder {
    SliderBuilder {
        value,
        min: 0.0,
        max: 100.0,
        step: 1.0,
        disabled: false,
    }
}

pub struct SliderBuilder {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
}

impl SliderBuilder {
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn on_change(self, action: impl Into<String>) -> UINode {
        self.build().on_change(action)
    }

    pub fn build(self) -> UINode {
        UINode::new(NodeType::Slider)
            .with_prop("value", self.value)
            .with_prop("min", self.min)
            .with_prop("max", self.max)
            .with_prop("step", self.step)
            .with_prop("disabled", self.disabled)
    }
}

impl From<SliderBuilder> for UINode {
    fn from(builder: SliderBuilder) -> Self {
        builder.build()
    }
}

pub fn select(options: Vec<SelectOption>) -> SelectBuilder {
    SelectBuilder {
        options,
        value: None,
        placeholder: None,
        disabled: false,
        searchable: false,
        multi: false,
        selected: Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

pub struct SelectBuilder {
    options: Vec<SelectOption>,
    value: Option<String>,
    placeholder: Option<String>,
    disabled: bool,
    searchable: bool,
    multi: bool,
    selected: Vec<String>,
}

impl SelectBuilder {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn searchable(mut self) -> Self {
        self.searchable = true;
        self
    }

    pub fn multi(mut self) -> Self {
        self.multi = true;
        self
    }

    pub fn selected(mut self, values: Vec<String>) -> Self {
        self.selected = values;
        self
    }

    pub fn on_change(self, action: impl Into<String>) -> UINode {
        self.build().on_change(action)
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Select)
            .with_prop("options", &self.options)
            .with_prop("disabled", self.disabled)
            .with_prop("searchable", self.searchable)
            .with_prop("multi", self.multi)
            .with_prop("selected", &self.selected);

        if let Some(value) = self.value {
            node = node.with_prop("value", value);
        }
        if let Some(placeholder) = self.placeholder {
            node = node.with_prop("placeholder", placeholder);
        }

        node
    }
}

impl From<SelectBuilder> for UINode {
    fn from(builder: SelectBuilder) -> Self {
        builder.build()
    }
}

pub fn skeleton() -> SkeletonBuilder {
    SkeletonBuilder {
        variant: None,
        width: None,
        height: None,
        count: 1,
    }
}

pub struct SkeletonBuilder {
    variant: Option<String>,
    width: Option<String>,
    height: Option<String>,
    count: u32,
}

impl SkeletonBuilder {
    pub fn text(mut self) -> Self {
        self.variant = Some("text".to_string());
        self
    }

    pub fn circle(mut self) -> Self {
        self.variant = Some("circle".to_string());
        self
    }

    pub fn rect(mut self) -> Self {
        self.variant = Some("rect".to_string());
        self
    }

    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<String>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Skeleton)
            .with_prop("count", self.count);

        if let Some(variant) = self.variant {
            node = node.with_prop("variant", variant);
        }
        if let Some(width) = self.width {
            node = node.with_prop("width", width);
        }
        if let Some(height) = self.height {
            node = node.with_prop("height", height);
        }

        node
    }
}

impl From<SkeletonBuilder> for UINode {
    fn from(builder: SkeletonBuilder) -> Self {
        builder.build()
    }
}

pub fn scroll(content: impl Into<UINode>) -> ScrollBuilder {
    ScrollBuilder {
        content: content.into(),
        horizontal: true,
        vertical: true,
        max_height: None,
    }
}

pub struct ScrollBuilder {
    content: UINode,
    horizontal: bool,
    vertical: bool,
    max_height: Option<String>,
}

impl ScrollBuilder {
    pub fn vertical_only(mut self) -> Self {
        self.horizontal = false;
        self
    }

    pub fn horizontal_only(mut self) -> Self {
        self.vertical = false;
        self
    }

    pub fn max_height(mut self, height: impl Into<String>) -> Self {
        self.max_height = Some(height.into());
        self
    }

    pub fn on_scroll(self, action: impl Into<String>) -> UINode {
        let node: UINode = self.into();
        node.with_prop("on_scroll", action.into())
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Scroll)
            .with_prop("horizontal", self.horizontal)
            .with_prop("vertical", self.vertical)
            .add_child(self.content);

        if let Some(max_height) = self.max_height {
            node = node.with_prop("max_height", max_height);
        }

        node
    }
}

impl From<ScrollBuilder> for UINode {
    fn from(builder: ScrollBuilder) -> Self {
        builder.build()
    }
}

pub fn avatar(src: impl Into<String>) -> AvatarBuilder {
    AvatarBuilder {
        src: Some(src.into()),
        name: None,
        size: None,
    }
}

pub fn avatar_initials(name: impl Into<String>) -> AvatarBuilder {
    AvatarBuilder {
        src: None,
        name: Some(name.into()),
        size: None,
    }
}

pub struct AvatarBuilder {
    src: Option<String>,
    name: Option<String>,
    size: Option<String>,
}

impl AvatarBuilder {
    pub fn small(mut self) -> Self {
        self.size = Some("sm".to_string());
        self
    }

    pub fn medium(mut self) -> Self {
        self.size = Some("md".to_string());
        self
    }

    pub fn large(mut self) -> Self {
        self.size = Some("lg".to_string());
        self
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Avatar);

        if let Some(src) = self.src {
            node = node.with_prop("src", src);
        }
        if let Some(name) = self.name {
            node = node.with_prop("name", name);
        }
        if let Some(size) = self.size {
            node = node.with_prop("size", size);
        }

        node
    }
}

impl From<AvatarBuilder> for UINode {
    fn from(builder: AvatarBuilder) -> Self {
        builder.build()
    }
}

pub fn toast(message: impl Into<String>) -> ToastBuilder {
    ToastBuilder {
        message: message.into(),
        variant: None,
        duration: None,
        dismissable: true,
    }
}

pub struct ToastBuilder {
    message: String,
    variant: Option<String>,
    duration: Option<u32>,
    dismissable: bool,
}

impl ToastBuilder {
    pub fn info(mut self) -> Self {
        self.variant = Some("info".to_string());
        self
    }

    pub fn success(mut self) -> Self {
        self.variant = Some("success".to_string());
        self
    }

    pub fn warning(mut self) -> Self {
        self.variant = Some("warning".to_string());
        self
    }

    pub fn error(mut self) -> Self {
        self.variant = Some("error".to_string());
        self
    }

    pub fn duration(mut self, ms: u32) -> Self {
        self.duration = Some(ms);
        self
    }

    pub fn persistent(mut self) -> Self {
        self.dismissable = false;
        self
    }

    pub fn build(self) -> UINode {
        let mut node = UINode::new(NodeType::Toast)
            .with_prop("message", self.message)
            .with_prop("dismissable", self.dismissable);

        if let Some(variant) = self.variant {
            node = node.with_prop("variant", variant);
        }
        if let Some(duration) = self.duration {
            node = node.with_prop("duration", duration);
        }

        node
    }
}

impl From<ToastBuilder> for UINode {
    fn from(builder: ToastBuilder) -> Self {
        builder.build()
    }
}
