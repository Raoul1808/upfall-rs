use scenes::{GameScene, Scene};
use tetra::{
    graphics::{self, Color, Texture},
    window, ContextBuilder, State,
};

mod player;
mod scenes;
mod tilemap;

pub struct Assets {
    pixel: Texture,
}

impl Assets {
    fn load(ctx: &mut tetra::Context) -> tetra::Result<Assets> {
        Ok(Assets {
            pixel: Texture::from_data(
                ctx,
                1,
                1,
                graphics::TextureFormat::Rgba8,
                &[255, 255, 255, 255],
            )?,
        })
    }
}

pub enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
}

struct GameState {
    assets: Assets,
    scenes: Vec<Box<dyn Scene>>,
}

impl GameState {
    fn new(ctx: &mut tetra::Context) -> tetra::Result<GameState> {
        Ok(GameState {
            assets: Assets::load(ctx)?,
            scenes: vec![Box::new(GameScene::new())],
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut tetra::Context) -> Result<(), tetra::TetraError> {
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx)? {
                Transition::None => {}
                Transition::Push(scene) => {
                    self.scenes.push(scene);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
            },
            None => window::quit(ctx),
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb8(100, 149, 237));
        if let Some(active_scene) = self.scenes.last_mut() {
            active_scene.draw(ctx, &self.assets)?;
        }
        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Upfall", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(GameState::new)
}
