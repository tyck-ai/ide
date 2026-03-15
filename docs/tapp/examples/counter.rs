//! Counter Example
//! 
//! The simplest possible Tapp - demonstrates basic state and UI.

use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct Counter {
    count: i32,
}

impl App for Counter {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "increment" => {
                self.count += 1;
                Ok(Response::render())
            }
            "decrement" => {
                self.count -= 1;
                Ok(Response::render())
            }
            "reset" => {
                self.count = 0;
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::panel("Counter").children([
            ui::text(format!("{}", self.count))
                .with_class("text-4xl font-bold text-center"),
            ui::hstack([
                ui::button("−")
                    .on_click("decrement")
                    .with_class("px-4 py-2"),
                ui::button("Reset")
                    .on_click("reset")
                    .with_class("px-4 py-2"),
                ui::button("+")
                    .primary()
                    .on_click("increment")
                    .with_class("px-4 py-2"),
            ])
        ])
    }
}

// Expose the count to agents
#[tapp::tools]
impl Counter {
    #[tool(description = "Get the current counter value")]
    fn get_count(&self, _args: serde_json::Value) -> ToolResult {
        ToolResult::json(serde_json::json!({ "count": self.count }))
    }

    #[tool(description = "Set the counter to a specific value")]
    fn set_count(&mut self, args: serde_json::Value) -> ToolResult {
        if let Some(value) = args.get("value").and_then(|v| v.as_i64()) {
            self.count = value as i32;
            ToolResult::ok()
        } else {
            ToolResult::error("Missing 'value' parameter")
        }
    }
}
