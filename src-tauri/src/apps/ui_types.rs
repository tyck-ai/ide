use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
            id: uuid::Uuid::new_v4().to_string(),
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
        self.props.insert(key.into(), serde_json::to_value(value).unwrap_or(Value::Null));
        self
    }

    pub fn with_children(mut self, children: Vec<UINode>) -> Self {
        self.children = children;
        self
    }

    pub fn add_child(mut self, child: UINode) -> Self {
        self.children.push(child);
        self
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGridRow {
    pub id: String,
    pub cells: HashMap<String, Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNodeData {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub children: Vec<TreeNodeData>,
    pub expanded: bool,
    pub selectable: bool,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualListItem {
    pub id: String,
    pub content: UINode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabItemData {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub closable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOptionData {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawerPosition {
    Left,
    Right,
    Top,
    Bottom,
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

pub type UITree = UINode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEvent {
    pub component_id: String,
    pub event_type: UIEventType,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UIEventType {
    Click,
    DoubleClick,
    Change,
    Submit,
    Focus,
    Blur,
    KeyDown,
    KeyUp,
    DragStart,
    DragEnd,
    DragOver,
    Drop,
    Scroll,
    Resize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    name: String,
    data: HashMap<String, Value>,
}

impl Action {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: HashMap::new(),
        }
    }

    pub fn with_data(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.data.insert(key.into(), serde_json::to_value(value).unwrap_or(Value::Null));
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, serde_json::Error> {
        let value = self.data.get(key).cloned().unwrap_or(Value::Null);
        serde_json::from_value(value)
    }

    pub fn get_optional<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: ResponseStatus,
    pub data: Option<Value>,
    pub render: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ok,
    Error,
    NotFound,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            status: ResponseStatus::Ok,
            data: None,
            render: false,
        }
    }

    pub fn render() -> Self {
        Self {
            status: ResponseStatus::Ok,
            data: None,
            render: true,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: Some(Value::String(message.into())),
            render: false,
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: ResponseStatus::NotFound,
            data: None,
            render: false,
        }
    }

    pub fn with_data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }
}
