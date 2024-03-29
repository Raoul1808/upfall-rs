use egui_tetra::egui::CtxRef;
use scenes::{Scene, StartScene, Transition};
use tetra::{
    graphics::{self, text::Font, Color, Shader, Texture},
    window, ContextBuilder,
};

mod level;
mod palette;
mod player;
mod scenes;
mod tilemap;
mod util;
mod world;

pub struct Assets {
    pixel: Texture,
    shader: Shader,
    player: Texture,
    spike: Texture,
    tile: Texture,
    door: Texture,
    door_locked: Texture,
    key: Texture,
    portal: Texture,
    pixel_font: (f32, Font),
    pixel_font_small: (f32, Font),
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
            player: Texture::new(ctx, "res/sprites/player.png")?,
            spike: Texture::new(ctx, "res/sprites/spike.png")?,
            tile: Texture::new(ctx, "res/sprites/tile.png")?,
            door: Texture::new(ctx, "res/sprites/door.png")?,
            door_locked: Texture::new(ctx, "res/sprites/door_locked.png")?,
            key: Texture::new(ctx, "res/sprites/key.png")?,
            portal: Texture::new(ctx, "res/sprites/portal.png")?,
            pixel_font: (48., Font::vector(ctx, "res/font/TinyUnicode.ttf", 52.)?),
            pixel_font_small: (36., Font::vector(ctx, "res/font/TinyUnicode.ttf", 32.)?),
        })
    }
}

struct GameState {
    assets: Assets,
    scenes: Vec<Box<dyn Scene>>,
}

impl GameState {
    fn new(ctx: &mut tetra::Context) -> tetra::Result<GameState> {
        Ok(GameState {
            assets: Assets::load(ctx)?,
            scenes: vec![Box::new(StartScene::new())],
        })
    }
}

impl egui_tetra::State for GameState {
    fn event(
        &mut self,
        ctx: &mut tetra::Context,
        _egui_ctx: &CtxRef,
        event: tetra::Event,
    ) -> Result<(), egui_tetra::Error> {
        if let Some(active_scene) = self.scenes.last_mut() {
            active_scene.event(ctx, event)?;
        }
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut tetra::Context,
        egui_ctx: &CtxRef,
    ) -> Result<(), egui_tetra::Error> {
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx, egui_ctx)? {
                Transition::None => {}
                Transition::Push(mut scene) => {
                    scene.update(ctx, egui_ctx)?;
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

    fn ui(&mut self, ctx: &mut tetra::Context, egui_ctx: &CtxRef) -> Result<(), egui_tetra::Error> {
        if let Some(active_scene) = self.scenes.last_mut() {
            active_scene.egui_layout(ctx, egui_ctx)?;
        }
        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut tetra::Context,
        _egui_ctx: &CtxRef,
    ) -> Result<(), egui_tetra::Error> {
        match self.scenes.last_mut() {
            Some(active_scene) => {
                active_scene.draw(ctx, &self.assets)?;
            }
            None => {
                graphics::clear(ctx, Color::BLACK);
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), egui_tetra::Error> {
    ContextBuilder::new("Upfall", 1280, 720)
        .resizable(true)
        .show_mouse(true)
        .build()?
        .run(|ctx| Ok(egui_tetra::StateWrapper::new(GameState::new(ctx)?)))
}
