export type NodeType =
  // Layout
  | 'view'
  | 'hstack'
  | 'vstack'
  | 'panel'
  | 'split'
  | 'tabs'
  | 'scroll'
  | 'modal'
  | 'drawer'
  // Content
  | 'text'
  | 'code'
  | 'markdown'
  | 'icon'
  | 'image'
  | 'badge'
  | 'avatar'
  // Input
  | 'button'
  | 'input'
  | 'textarea'
  | 'select'
  | 'checkbox'
  | 'toggle'
  | 'slider'
  // Data
  | 'list'
  | 'virtual_list'
  | 'tree'
  | 'table'
  | 'data_grid'
  // Feedback
  | 'toast'
  | 'progress'
  | 'spinner'
  | 'empty'
  | 'skeleton'
  | 'alert';

export interface UINode {
  id: string;
  node_type: NodeType;
  props: Record<string, unknown>;
  children: UINode[];
}

export type UITree = UINode;

export interface UIEvent {
  component_id: string;
  event_type: UIEventType;
  data?: unknown;
}

export type UIEventType =
  | 'click'
  | 'double_click'
  | 'change'
  | 'submit'
  | 'focus'
  | 'blur'
  | 'key_down'
  | 'key_up'
  | 'drag_start'
  | 'drag_end'
  | 'drag_over'
  | 'drop'
  | 'scroll'
  | 'resize';

export interface AppInfo {
  id: string;
  name: string;
  version: string;
  description?: string;
  enabled: boolean;
  running: boolean;
  layout: LayoutMode;
}

export type LayoutMode = 'full' | 'sidebar' | 'panel' | 'modal';
