use egui_tetra::egui;
use tetra::graphics::{self, Color};

use super::{EditorScene, Scene, Transition};

pub struct StartScene {
    editor: bool,
    quit: bool,
}

impl StartScene {
    pub fn new() -> Self {
        Self {
            editor: false,
            quit: false,
        }
    }
}

impl Scene for StartScene {
    fn update(
        &mut self,
        ctx: &mut tetra::Context,
        _egui_ctx: &egui_tetra::egui::CtxRef,
    ) -> tetra::Result<Transition> {
        if self.editor {
            self.editor = false;
            return Ok(Transition::Push(Box::new(EditorScene::new(ctx))));
        }
        if self.quit {
            return Ok(Transition::Pop);
        }
        Ok(Transition::None)
    }

    fn egui_layout(
        &mut self,
        _ctx: &mut tetra::Context,
        egui_ctx: &egui_tetra::egui::CtxRef,
    ) -> Result<(), egui_tetra::Error> {
        egui::Window::new("Upfall").show(egui_ctx, |ui| {
            ui.heading("PLACEHOLDER MENU");
            ui.label("This menu will not be present in the final 1.0 version of the game, as a proper menu system is still being worked on.");
            ui.separator();
            if ui.button("Editor").clicked() {
                self.editor = true;
            }
            if ui.button("Quit").clicked() {
                self.quit = true;
            }
        });
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context, _assets: &crate::Assets) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);
        Ok(())
    }
}
