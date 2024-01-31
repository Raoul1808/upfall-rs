use tetra::{
    graphics::{Color, DrawParams},
    input::{self, Key},
    math::Vec2,
};

use crate::{
    level::Level,
    player::Player,
    tilemap::{Facing, Tile, Tilemap},
    world::{World, WorldMode},
    Assets, Transition,
};

#[allow(unused_variables)]
pub trait Scene {
    fn use_shader(&self) -> bool {
        false
    }
    fn clear_color(&self) -> Color;
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition>;
    fn canvas_draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        Ok(())
    }
    fn screen_draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        Ok(())
    }
}

pub struct GameScene {
    world: World,
    color_a: Color,
    color_b: Color,
}

impl GameScene {
    pub fn new(level: Level) -> GameScene {
        GameScene {
            world: World::new(level),
            color_a: Color::BLUE,
            color_b: Color::rgb8(100, 149, 237),
        }
    }
}

impl Scene for GameScene {
    fn use_shader(&self) -> bool {
        true
    }

    fn clear_color(&self) -> Color {
        Color::BLACK
    }

    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Escape) {
            return Ok(Transition::Pop);
        }

        self.world.update(ctx);

        Ok(Transition::None)
    }

    fn canvas_draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        assets
            .shader
            .set_uniform(ctx, "u_color_a", self.color_a.with_alpha(1.));
        assets
            .shader
            .set_uniform(ctx, "u_color_b", self.color_b.with_alpha(1.));
        self.world.draw(ctx, assets);
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
    tile: Tile,
}

impl EditorScene {
    pub fn new() -> EditorScene {
        EditorScene {
            dark_tilemap: Tilemap::new((40, 23), (32., 32.)),
            light_tilemap: Tilemap::new((40, 23), (32., 32.)),
            mode: WorldMode::Dark,
            spawn_pos: Vec2::default(),
            mouse_pos: Vec2::default(),
            facing: Facing::Up,
            tile: Tile::Solid,
        }
    }
}

impl Scene for EditorScene {
    fn clear_color(&self) -> Color {
        Color::rgb8(100, 149, 237)
    }

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

        if input::is_key_pressed(ctx, Key::Up) {
            self.facing = Facing::Up;
            self.tile.set_facing(self.facing);
        }
        if input::is_key_pressed(ctx, Key::Down) {
            self.facing = Facing::Down;
            self.tile.set_facing(self.facing);
        }
        if input::is_key_pressed(ctx, Key::Left) {
            self.facing = Facing::Left;
            self.tile.set_facing(self.facing);
        }
        if input::is_key_pressed(ctx, Key::Right) {
            self.facing = Facing::Right;
            self.tile.set_facing(self.facing);
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
            return Ok(Transition::Push(Box::new(GameScene::new(level))));
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

    fn screen_draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        let (dark_alpha, light_alpha) = match self.mode {
            WorldMode::Dark => (1., 0.33),
            WorldMode::Light => (0.33, 1.),
        };
        self.dark_tilemap.run_for_each_tile(|(x, y), tile| {
            if !matches!(tile, Tile::None) {
                let size = self.dark_tilemap.tile_size();
                let pos = Vec2::new(x as f32, y as f32) * size;
                let hb = tile.hbox(pos, size);
                assets.pixel.draw(
                    ctx,
                    DrawParams::new()
                        .position(Vec2::new(hb.x, hb.y))
                        .scale(Vec2::new(hb.w, hb.h))
                        .color(Color::BLACK.with_alpha(dark_alpha)),
                );
            }
        });
        self.light_tilemap.run_for_each_tile(|(x, y), tile| {
            if !matches!(tile, Tile::None) {
                let size = self.light_tilemap.tile_size();
                let pos = Vec2::new(x as f32, y as f32) * size;
                let hb = tile.hbox(pos, size);
                assets.pixel.draw(
                    ctx,
                    DrawParams::new()
                        .position(Vec2::new(hb.x, hb.y))
                        .scale(Vec2::new(hb.w, hb.h))
                        .color(Color::BLACK.with_alpha(light_alpha)),
                );
            }
        });
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(self.spawn_pos)
                .scale(Vec2::new(Player::PLAYER_SQUARE, Player::PLAYER_SQUARE))
                .color(Color::RED),
        );
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
