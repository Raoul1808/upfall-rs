use tetra::{
    graphics::{
        self,
        scaling::{ScalingMode, ScreenScaler},
        Camera, Color, DrawParams,
    },
    input::{self, Key},
    math::Vec2,
    Event,
};

use crate::{
    level::Level,
    tilemap::{Axis, Facing, Tile, Tilemap},
    world::{World, WorldMode},
    Assets, Transition,
};

#[allow(unused_variables)]
pub trait Scene {
    fn event(&mut self, ctx: &mut tetra::Context, event: tetra::Event) -> tetra::Result {
        Ok(())
    }
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result;
}

pub struct GameScene {
    world: World,
    color_a: Color,
    color_b: Color,
    camera: Camera,
    scaler: ScreenScaler,
    dt: f32,
}

impl GameScene {
    const INNER_SIZE: Vec2<i32> = Vec2::new(640, 360);
    pub fn new(ctx: &mut tetra::Context, level: Level) -> tetra::Result<GameScene> {
        Ok(GameScene {
            world: World::new(level),
            color_a: Color::BLUE,
            color_b: Color::rgb8(100, 149, 237),
            camera: Camera::new(Self::INNER_SIZE.x as f32, Self::INNER_SIZE.y as f32),
            scaler: ScreenScaler::with_window_size(
                ctx,
                Self::INNER_SIZE.x,
                Self::INNER_SIZE.y,
                ScalingMode::ShowAll,
            )?,
            dt: 0.,
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

    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Escape) {
            return Ok(Transition::Pop);
        }

        self.dt += tetra::time::get_delta_time(ctx).as_secs_f32();
        self.world.update(ctx);
        self.camera.position = Self::INNER_SIZE.as_() / 2.;
        self.camera.update();
        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        assets
            .shader
            .set_uniform(ctx, "u_color_a", self.color_a.with_alpha(1.));
        assets
            .shader
            .set_uniform(ctx, "u_color_b", self.color_b.with_alpha(1.));
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

pub struct EditorScene {
    dark_tilemap: Tilemap,
    light_tilemap: Tilemap,
    mode: WorldMode,
    spawn_pos: Vec2<f32>,
    mouse_pos: Vec2<f32>,
    facing: Facing,
    axis: Axis,
    tile: Tile,
    tilemap_size: Vec2<usize>,
}

impl EditorScene {
    pub const TILEMAP_MIN_X: usize = 40;
    pub const TILEMAP_MIN_Y: usize = 23;

    pub fn new() -> EditorScene {
        let tilemap_size = (80, 45);
        EditorScene {
            dark_tilemap: Tilemap::new(tilemap_size, (16., 16.)),
            light_tilemap: Tilemap::new(tilemap_size, (16., 16.)),
            mode: WorldMode::Dark,
            spawn_pos: Vec2::default(),
            mouse_pos: Vec2::default(),
            facing: Facing::Up,
            axis: Axis::Horizontal,
            tile: Tile::Solid,
            tilemap_size: tilemap_size.into(),
        }
    }
}

impl Scene for EditorScene {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition> {
        self.mouse_pos = input::get_mouse_position(ctx);
        if input::is_key_pressed(ctx, Key::Tab) {
            self.mode.switch();
        }
        if input::is_key_pressed(ctx, Key::Num1) {
            println!("solid!");
            self.tile = Tile::Solid;
        }
        if input::is_key_pressed(ctx, Key::Num2) {
            println!("spike!");
            self.tile = Tile::Spike(self.facing);
        }
        if input::is_key_pressed(ctx, Key::Num3) {
            println!("portal!");
            self.tile = Tile::Portal(self.axis);
        }

        if input::is_key_pressed(ctx, Key::Up) {
            self.facing = Facing::Up;
            self.axis = Axis::Vertical;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }
        if input::is_key_pressed(ctx, Key::Down) {
            self.facing = Facing::Down;
            self.axis = Axis::Vertical;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }
        if input::is_key_pressed(ctx, Key::Left) {
            self.facing = Facing::Left;
            self.axis = Axis::Horizontal;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }
        if input::is_key_pressed(ctx, Key::Right) {
            self.facing = Facing::Right;
            self.axis = Axis::Horizontal;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }

        if input::is_key_down(ctx, Key::LeftShift) {
            if input::is_key_pressed(ctx, Key::A) && self.tilemap_size.x > Self::TILEMAP_MIN_X {
                self.tilemap_size.x -= 1;
                self.dark_tilemap.resize(self.tilemap_size);
                self.light_tilemap.resize(self.tilemap_size);
                println!("Tilemap is now size {}", self.tilemap_size);
            }
            if input::is_key_pressed(ctx, Key::D) {
                self.tilemap_size.x += 1;
                self.dark_tilemap.resize(self.tilemap_size);
                self.light_tilemap.resize(self.tilemap_size);
                println!("Tilemap is now size {}", self.tilemap_size);
            }
            if input::is_key_pressed(ctx, Key::W) && self.tilemap_size.y > Self::TILEMAP_MIN_Y {
                self.tilemap_size.y -= 1;
                self.dark_tilemap.resize(self.tilemap_size);
                self.light_tilemap.resize(self.tilemap_size);
                println!("Tilemap is now size {}", self.tilemap_size);
            }
            if input::is_key_pressed(ctx, Key::S) {
                self.tilemap_size.y += 1;
                self.dark_tilemap.resize(self.tilemap_size);
                self.light_tilemap.resize(self.tilemap_size);
                println!("Tilemap is now size {}", self.tilemap_size);
            }
        }

        let tilemap = match self.mode {
            WorldMode::Dark => &mut self.dark_tilemap,
            WorldMode::Light => &mut self.light_tilemap,
        };

        if input::is_mouse_button_down(ctx, input::MouseButton::Left) {
            tilemap.set_tile_f32(self.mouse_pos, self.tile);
        }

        if input::is_mouse_button_down(ctx, input::MouseButton::Right) {
            tilemap.set_tile_f32(self.mouse_pos, Tile::None);
        }

        if input::is_mouse_button_down(ctx, input::MouseButton::Middle) {
            self.spawn_pos = tilemap.snap(self.mouse_pos);
        }

        if input::is_key_pressed(ctx, Key::Enter) {
            let level = Level {
                dark_tilemap: self.dark_tilemap.clone(),
                light_tilemap: self.light_tilemap.clone(),
                spawn_pos: self.spawn_pos,
            };
            return Ok(Transition::Push(Box::new(GameScene::new(ctx, level)?)));
        }

        if input::is_key_down(ctx, Key::LeftCtrl) {
            if input::is_key_pressed(ctx, Key::S) {
                let level = Level {
                    dark_tilemap: self.dark_tilemap.clone(),
                    light_tilemap: self.light_tilemap.clone(),
                    spawn_pos: self.spawn_pos,
                };
                let res = level.save("level.umdx");
                println!("{:?}", res.err());
            }

            if input::is_key_pressed(ctx, Key::O) {
                match Level::load("level.umdx") {
                    Ok(l) => {
                        self.dark_tilemap = l.dark_tilemap;
                        self.light_tilemap = l.light_tilemap;
                        self.spawn_pos = l.spawn_pos;
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
        }

        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);
        let tilemap_rect = self.dark_tilemap.rect();
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(tilemap_rect.top_left())
                .scale(tilemap_rect.bottom_right())
                .color(Color::rgb8(100, 149, 237)),
        );
        let (dark_alpha, light_alpha) = match self.mode {
            WorldMode::Dark => (1., 0.33),
            WorldMode::Light => (0.33, 1.),
        };
        self.dark_tilemap
            .render_tilemap(ctx, assets, Color::WHITE.with_alpha(dark_alpha));
        self.light_tilemap
            .render_tilemap(ctx, assets, Color::BLACK.with_alpha(light_alpha));
        assets
            .player
            .draw(ctx, DrawParams::new().position(self.spawn_pos));
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(self.dark_tilemap.snap(self.mouse_pos))
                .scale(self.dark_tilemap.tile_size())
                .color(Color::WHITE.with_alpha(1. / 3.)),
        );
        Ok(())
    }
}
