use tetra::{
    graphics::{self, Camera, Color, DrawParams},
    input::{self, Key},
    math::Vec2,
    Event,
};

use crate::{
    level::Level,
    tilemap::{Axis, Facing, Tile, Tilemap},
    world::WorldMode,
    Assets,
};

use super::{GameScene, Scene, Transition};

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
    camera: Camera,
}

impl EditorScene {
    pub const TILEMAP_MIN_X: usize = 40;
    pub const TILEMAP_MIN_Y: usize = 23;

    pub fn new(ctx: &mut tetra::Context) -> EditorScene {
        let tilemap_size = (80, 45);
        let mut camera = Camera::with_window_size(ctx);
        camera.position = camera.visible_rect().bottom_right();
        camera.update();
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
            camera,
        }
    }
}

impl Scene for EditorScene {
    fn event(&mut self, _ctx: &mut tetra::Context, event: tetra::Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.camera.set_viewport_size(width as f32, height as f32);
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition> {
        self.mouse_pos = self.camera.mouse_position(ctx);
        let shift =
            input::is_key_down(ctx, Key::LeftShift) || input::is_key_down(ctx, Key::RightShift);
        let ctrl =
            input::is_key_down(ctx, Key::LeftCtrl) || input::is_key_down(ctx, Key::RightCtrl);
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

        if shift && !ctrl {
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

            const CAMERA_MOVE: f32 = 5.;
            const SCALE_FACTOR: f32 = 1.5;
            if input::is_key_down(ctx, Key::Left) {
                self.camera.position.x -= CAMERA_MOVE;
            }
            if input::is_key_down(ctx, Key::Right) {
                self.camera.position.x += CAMERA_MOVE;
            }
            if input::is_key_down(ctx, Key::Up) {
                self.camera.position.y -= CAMERA_MOVE;
            }
            if input::is_key_down(ctx, Key::Down) {
                self.camera.position.y += CAMERA_MOVE;
            }
            if input::is_key_pressed(ctx, Key::Equals) {
                self.camera.scale *= SCALE_FACTOR;
            }
            if input::is_key_pressed(ctx, Key::Minus) {
                self.camera.scale /= SCALE_FACTOR;
                if self.camera.scale.x <= 1. {
                    self.camera.scale = Vec2::one();
                }
            }
        }

        let tilemap = match self.mode {
            WorldMode::Dark => &mut self.dark_tilemap,
            WorldMode::Light => &mut self.light_tilemap,
        };

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Left) {
            tilemap.set_tile_f32(self.mouse_pos, self.tile);
        }

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Right) {
            tilemap.set_tile_f32(self.mouse_pos, Tile::None);
        }

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Middle) {
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

        if ctrl && !shift {
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

        self.camera.update();
        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);
        graphics::set_transform_matrix(ctx, self.camera.as_matrix());
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
        if self.dark_tilemap.rect().contains_point(self.mouse_pos) {
            assets.pixel.draw(
                ctx,
                DrawParams::new()
                    .position(self.dark_tilemap.snap(self.mouse_pos))
                    .scale(self.dark_tilemap.tile_size())
                    .color(Color::WHITE.with_alpha(1. / 3.)),
            );
        }
        graphics::reset_transform_matrix(ctx);
        Ok(())
    }
}
