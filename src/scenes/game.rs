use egui_tetra::egui::CtxRef;
use tetra::{
    graphics::{
        self,
        scaling::{ScalingMode, ScreenScaler},
        text::Text,
        Camera, Color, DrawParams,
    },
    input::{self, Key},
    math::Vec2,
    window, Event,
};

use crate::{
    level::{Level, LevelPack},
    palette::PaletteSystem,
    world::World,
    Assets, Scene,
};

use super::Transition;

#[derive(Default)]
struct LevelLabel {
    name: String,
    author: String,
    name_text: Option<Text>,
    author_text: Option<Text>,
    name_offset: Vec2<f32>,
    author_offset: Vec2<f32>,
    name_position: Vec2<f32>,
    author_position: Vec2<f32>,
    text_timer: f32,
    dirty_offset: bool,
    new_text: bool,
    screen_size: (i32, i32),
}

impl LevelLabel {
    const TEXT_SHOW_TIME: f32 = 3.;
    const STROKE_WIDTH: f32 = 2.;

    pub fn new(ctx: &mut tetra::Context, name: &str, author: &str) -> Self {
        let mut label = Self::default();
        label.update_screen_size(ctx);
        label.set(name, author);
        label
    }

    pub fn set(&mut self, name: &str, author: &str) {
        self.name = name.into();
        self.author = format!("Created by {}", author);
        self.text_timer = 0.;
        self.dirty_offset = true;
        self.new_text = true;
    }

    pub fn update_screen_size(&mut self, ctx: &mut tetra::Context) {
        let size = window::get_size(ctx);
        if self.screen_size != size {
            self.screen_size = size;
            self.dirty_offset = true;
        }
    }

    fn update_offsets(&mut self, ctx: &mut tetra::Context) {
        // Lots of unwrapping. This is bad practice, but I don't really have another choice
        let name = self.name_text.as_mut().unwrap();
        let author = self.author_text.as_mut().unwrap();
        let name_bounds = name.get_bounds(ctx).unwrap();
        let author_bounds = author.get_bounds(ctx).unwrap();
        self.name_offset = Vec2::new(name_bounds.center().x, name_bounds.bottom());
        self.author_offset = Vec2::new(author_bounds.center().x, author_bounds.bottom());
        self.name_position = Vec2::new(
            self.screen_size.0 as f32 / 2.,
            self.screen_size.1 as f32 - 50.,
        );
        self.author_position = Vec2::new(
            self.screen_size.0 as f32 / 2.,
            self.screen_size.1 as f32 - 25.,
        );
    }

    pub fn update_timer(&mut self, dt: f32) {
        self.text_timer += dt;
    }

    fn draw_text(
        ctx: &mut tetra::Context,
        text: &mut Text,
        position: Vec2<f32>,
        origin: Vec2<f32>,
        stroke: f32,
        text_color: Color,
        stroke_color: Color,
    ) {
        for i in 0..9 {
            let x = (i % 3) - 1;
            let y = (i / 3) - 1;
            if x == 0 && y == 0 {
                continue;
            }
            text.draw(
                ctx,
                DrawParams::new()
                    .position(position - Vec2::new(x as f32 * stroke, y as f32 * stroke))
                    .origin(origin)
                    .color(stroke_color),
            );
        }
        text.draw(
            ctx,
            DrawParams::new()
                .position(position)
                .origin(origin)
                .color(text_color),
        );
    }

    pub fn draw(
        &mut self,
        ctx: &mut tetra::Context,
        assets: &Assets,
        dark_color: &Color,
        light_color: &Color,
    ) {
        if self.new_text {
            let mut name = Text::new(&self.name, assets.pixel_font.1.clone());
            let mut author = Text::new(&self.author, assets.pixel_font_small.1.clone());
            name.get_bounds(ctx);
            author.get_bounds(ctx);
            self.name_text = Some(name);
            self.author_text = Some(author);
            self.new_text = false;
        }
        if self.dirty_offset {
            self.update_offsets(ctx);
            self.dirty_offset = false;
        }
        if self.text_timer >= Self::TEXT_SHOW_TIME {
            return;
        }
        Self::draw_text(
            ctx,
            self.name_text.as_mut().unwrap(),
            self.name_position,
            self.name_offset,
            Self::STROKE_WIDTH,
            *light_color,
            *dark_color,
        );
        Self::draw_text(
            ctx,
            self.author_text.as_mut().unwrap(),
            self.author_position,
            self.author_offset,
            Self::STROKE_WIDTH,
            *light_color,
            *dark_color,
        );
    }
}

pub struct GameScene {
    world: World,
    camera: Camera,
    scaler: ScreenScaler,
    palette_system: PaletteSystem,
    level_pack: LevelPack,
    current_level: usize,
    playtest: bool,
    label: LevelLabel,
}

impl GameScene {
    const INNER_SIZE: Vec2<i32> = Vec2::new(640, 360);
    pub fn new(ctx: &mut tetra::Context, level: Level) -> tetra::Result<GameScene> {
        let mut scene = GameScene::with_pack(
            ctx,
            LevelPack {
                levels: vec![level],
                ..Default::default()
            },
        )?;
        scene.playtest = false;
        Ok(scene)
    }

    pub fn with_pack(ctx: &mut tetra::Context, pack: LevelPack) -> tetra::Result<GameScene> {
        let first_level = &pack.levels[0];
        let palette = first_level.palette;
        let label = LevelLabel::new(ctx, &first_level.name, &first_level.author);
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
            label,
        })
    }
}

impl Scene for GameScene {
    fn event(&mut self, ctx: &mut tetra::Context, event: tetra::Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
            self.label.update_screen_size(ctx);
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
        self.label.update_timer(dt);
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
                    self.label.set(&next_level.name, &next_level.author);
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
        if !self.playtest {
            self.label.draw(
                ctx,
                assets,
                self.palette_system.dark(),
                self.palette_system.light(),
            );
        }
        Ok(())
    }
}
