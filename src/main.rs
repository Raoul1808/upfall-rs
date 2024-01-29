use tetra::{graphics::{self, Color}, ContextBuilder, State};

struct GameState;

impl GameState {
    fn new(_ctx: &mut tetra::Context) -> tetra::Result<GameState> {
        Ok(GameState)
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb8(100, 149, 237));
        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Upfall", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(GameState::new)
}
