use egui_tetra::egui::CtxRef;
use tetra::{
    graphics::{
        self,
        scaling::{ScalingMode, ScreenScaler},
        Camera, Color,
    },
    input::{self, Key},
    math::Vec2,
    Event,
};

use crate::{
    level::{Level, LevelPack},
    palette::PaletteSystem,
    world::World,
    Assets, Scene,
};

use super::Transition;

pub struct GameScene {
    world: World,
    camera: Camera,
    scaler: ScreenScaler,
    palette_system: PaletteSystem,
    level_pack: LevelPack,
    current_level: usize,
    playtest: bool,
}

impl GameScene {
    const INNER_SIZE: Vec2<i32> = Vec2::new(640, 360);
    pub fn new(ctx: &mut tetra::Context, level: Level) -> tetra::Result<GameScene> {
        let palette = level.palette;
        let level_pack = LevelPack {
            levels: vec![level.clone()],
            ..Default::default()
        };
        Ok(GameScene {
            world: World::new(level),
            camera: Camera::new(Self::INNER_SIZE.x as f32, Self::INNER_SIZE.y as f32),
            scaler: ScreenScaler::with_window_size(
                ctx,
                Self::INNER_SIZE.x,
                Self::INNER_SIZE.y,
                ScalingMode::ShowAll,
            )?,
            palette_system: PaletteSystem::new(palette),
            level_pack,
            current_level: 0,
            playtest: true,
        })
    }

    pub fn with_pack(ctx: &mut tetra::Context, pack: LevelPack) -> tetra::Result<GameScene> {
        let first_level = &pack.levels[0];
        let palette = first_level.palette;
        Ok(GameScene {
            world: World::new(first_level.clone()),
            camera: Camera::new(Self::INNER_SIZE.x as f32, Self::INNER_SIZE.y as f32),
            scaler: ScreenScaler::with_window_size(
                ctx,
                Self::INNER_SIZE.x,
                Self::INNER_SIZE.y,
                ScalingMode::ShowAll,
            )?,
            palette_system: PaletteSystem::new(palette),
            level_pack: pack,
            current_level: 0,
            playtest: false,
        })
    }
}

impl Scene for GameScene {
    fn event(&mut self, _ctx: &mut tetra::Context, event: tetra::Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut tetra::Context,
        _egui_ctx: &CtxRef,
    ) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Escape) {
            return Ok(Transition::Pop);
        }

        let dt = tetra::time::get_delta_time(ctx).as_secs_f32();

        self.world.update(ctx);
        if self.world.win() {
            match self.playtest {
                true => {
                    self.world.reset();
                }
                false => {
                    self.current_level += 1;
                    if self.current_level == self.level_pack.levels.len() {
                        return Ok(Transition::Pop);
                    }
                    let next_level = self.level_pack.levels[self.current_level].clone();
                    self.palette_system.change_palette(next_level.palette);
                    self.world = World::new(next_level);
                }
            }
        }
        self.camera.position = self.world.player_pos();
        let world_rect = self.world.get_world_rect();
        let cam_rect = self.camera.visible_rect();
        if cam_rect.left() < world_rect.left() {
            self.camera.position.x += world_rect.left() - cam_rect.left();
        }
        if cam_rect.right() > world_rect.right() {
            self.camera.position.x += world_rect.right() - cam_rect.right();
        }
        if cam_rect.top() < world_rect.top() {
            self.camera.position.y += world_rect.top() - cam_rect.top();
        }
        if cam_rect.bottom() > world_rect.bottom() {
            self.camera.position.y += world_rect.bottom() - cam_rect.bottom();
        }
        self.camera.update();
        self.palette_system.update(dt);
        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        assets
            .shader
            .set_uniform(ctx, "u_color_a", self.palette_system.dark());
        assets
            .shader
            .set_uniform(ctx, "u_color_b", self.palette_system.light());
        assets.shader.set_uniform(
            ctx,
            "u_circle_offset",
            self.camera.visible_rect().top_left(),
        );
        assets
            .shader
            .set_uniform(ctx, "u_resolution", Self::INNER_SIZE.as_());
        assets
            .shader
            .set_uniform(ctx, "u_scale_factor", self.scaler.scale_factor());
        let scale_offset = self.scaler.unproject(Vec2::zero());
        assets
            .shader
            .set_uniform(ctx, "u_scale_offset", scale_offset);
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::set_transform_matrix(ctx, self.camera.as_matrix());
        graphics::clear(ctx, Color::BLACK);
        self.world.draw(ctx, assets);
        graphics::reset_transform_matrix(ctx);
        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);
        graphics::set_shader(ctx, &assets.shader);
        self.scaler.draw(ctx);
        graphics::reset_shader(ctx);
        Ok(())
    }
}
