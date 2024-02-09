use egui_tetra::egui::CtxRef;

use crate::Assets;

#[allow(unused_variables)]
pub trait Scene {
    fn event(&mut self, ctx: &mut tetra::Context, event: tetra::Event) -> tetra::Result {
        Ok(())
    }
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result;
    fn egui_layout(
        &mut self,
        ctx: &mut tetra::Context,
        egui_ctx: &CtxRef,
    ) -> Result<(), egui_tetra::Error> {
        Ok(())
    }
}

pub enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
}

mod editor;
mod game;

pub use editor::EditorScene;
pub use game::GameScene;
