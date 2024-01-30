use scenes::{EditorScene, Scene};
use tetra::{
    graphics::{self, Canvas, Color, Shader, Texture},
    math::Vec2,
    window, ContextBuilder, State,
};

mod level;
mod player;
mod scenes;
mod tilemap;
mod world;

pub struct Assets {
    pixel: Texture,
    shader: Shader,
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
            shader: Shader::from_fragment_file(ctx, "res/shader.frag")?,
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
    canvas: Canvas,
}

impl GameState {
    fn new(ctx: &mut tetra::Context) -> tetra::Result<GameState> {
        Ok(GameState {
            assets: Assets::load(ctx)?,
            scenes: vec![Box::new(EditorScene::new())],
            canvas: Canvas::new(ctx, 1280, 720)?,
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
        match self.scenes.last_mut() {
            Some(active_scene) => {
                graphics::set_canvas(ctx, &self.canvas);
                graphics::clear(ctx, active_scene.clear_color());
                active_scene.canvas_draw(ctx, &self.assets)?;
                graphics::reset_canvas(ctx);
                if active_scene.use_shader() {
                    graphics::set_shader(ctx, &self.assets.shader);
                }
                graphics::clear(ctx, Color::BLACK);
                self.canvas.draw(ctx, Vec2::zero());
                graphics::reset_shader(ctx);
                active_scene.screen_draw(ctx, &self.assets)?;
            }
            None => {
                graphics::clear(ctx, Color::BLACK);
            }
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
