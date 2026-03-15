use tapp::prelude::*;

#[tapp::app]
#[derive(Default, tapp::TappToolDefault, tapp::TappHookDefault)]
pub struct MinimalApp {
    counter: u32,
}

impl App for MinimalApp {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "increment" => {
                self.counter += 1;
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::vstack([
            ui::text(&format!("Count: {}", self.counter)),
            ui::button("Increment").on_click("increment"),
        ])
    }
}
