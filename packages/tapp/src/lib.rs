//! # Tapp SDK
//!
//! SDK for building Tapp extensions for Tyck IDE.
//!
//! ## Quick Start
//!
//! ```rust
//! use tapp::prelude::*;
//!
//! #[tapp::app]
//! #[derive(Default)]
//! pub struct MyApp {
//!     counter: u32,
//! }
//!
//! impl App for MyApp {
//!     fn init(&mut self, _ctx: &Context) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     fn shutdown(&mut self) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     fn handle(&mut self, action: Action) -> Result<Response> {
//!         match action.name() {
//!             "increment" => {
//!                 self.counter += 1;
//!                 Ok(Response::render())
//!             }
//!             _ => Ok(Response::not_found())
//!         }
//!     }
//!
//!     fn render(&self) -> UITree {
//!         ui::vstack([
//!             ui::text(format!("Count: {}", self.counter)),
//!             ui::button("Increment").on_click("increment"),
//!         ])
//!     }
//! }
//! ```

mod app;
mod action;
pub mod ui;
mod storage;
mod agent;
mod tools;
mod hooks;
mod context;
mod error;

pub mod __internal;

pub use app::{App, TappToolProvider, TappHookProvider};
pub use action::{Action, Response};
pub use ui::{UITree, UINode, NodeType};
pub use storage::Storage;
pub use agent::{AgentSession, SpawnOptions};
pub use tools::{ToolDefinition, ToolResult, ToolParameter};
pub use hooks::{HookType, HookResult, HookRegistration};
pub use context::Context;
pub use error::{Error, Result};

pub use tapp_macros::{app, tools, hooks, tool, hook, TappToolDefault, TappHookDefault};

#[cfg(target_arch = "wasm32")]
pub use wit_bindgen;

pub mod prelude {
    pub use crate::{
        App, Action, Response, UITree, UINode, NodeType,
        Storage, AgentSession, SpawnOptions,
        ToolDefinition, ToolResult, ToolParameter,
        HookType, HookResult, HookRegistration,
        Context, Error, Result,
        app, tools, hooks, tool, hook,
    };
    pub use crate::ui;
}
