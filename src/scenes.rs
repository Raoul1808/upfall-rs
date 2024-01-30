use tetra::{
    graphics::{Color, DrawParams},
    input::{self, Key},
    math::Vec2,
};

use crate::{
    level::Level, player::Player, tilemap::{Tile, Tilemap}, world::World, Assets, Transition
};

pub trait Scene {
    fn clear_color(&self) -> Color;
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result;
}

pub struct GameScene {
    world: World,
}

impl GameScene {
    pub fn new(level: Level) -> GameScene {
        GameScene {
            world: World::new(level),
        }
    }
}

impl Scene for GameScene {
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

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        self.world.draw(ctx, assets);
        Ok(())
    }
}

pub struct EditorScene {
    tilemap: Tilemap,
    spawn_pos: Vec2<f32>,
    mouse_pos: Vec2<f32>,
}

impl EditorScene {
    pub fn new() -> EditorScene {
        EditorScene {
            tilemap: Tilemap::new((40, 23), (32., 32.)),
            spawn_pos: Vec2::default(),
            mouse_pos: Vec2::default(),
        }
    }
}

impl Scene for EditorScene {
    fn clear_color(&self) -> Color {
        Color::rgb8(100, 149, 237)
    }

    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition> {
        self.mouse_pos = input::get_mouse_position(ctx);
        if input::is_mouse_button_down(ctx, input::MouseButton::Left) {
            self.tilemap.set_tile_f32(self.mouse_pos, Tile::Solid);            
        }

        if input::is_mouse_button_down(ctx, input::MouseButton::Right) {
            self.tilemap.set_tile_f32(self.mouse_pos, Tile::None);
        }

        if input::is_mouse_button_down(ctx, input::MouseButton::Middle) {
            self.spawn_pos = self.tilemap.snap(self.mouse_pos);
        }

        if input::is_key_pressed(ctx, Key::Enter) {
            let level = Level {
                tilemap: self.tilemap.clone(),
                spawn_pos: self.spawn_pos.clone(),
            };
            return Ok(Transition::Push(Box::new(GameScene::new(level))))
        }

        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        self.tilemap.run_for_each_tile(|(x, y), tile| {
            if !matches!(tile, Tile::Solid) {
                return;
            }
            assets.pixel.draw(
                ctx,
                DrawParams::new()
                .position(Vec2::new(
                    x as f32 * self.tilemap.tile_width(),
                    y as f32 * self.tilemap.tile_height(),
                ))
                .scale(self.tilemap.tile_size())
                .color(Color::BLACK)
            );
        });
        assets.pixel.draw(
            ctx, 
            DrawParams::new()
                .position(self.spawn_pos)
                .scale(Vec2::new(Player::PLAYER_SQUARE, Player::PLAYER_SQUARE))
                .color(Color::RED)
        );
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(self.tilemap.snap(self.mouse_pos))
                .scale(self.tilemap.tile_size())
                .color(Color::WHITE.with_alpha(1./3.))
        );
        Ok(())
    }
}
