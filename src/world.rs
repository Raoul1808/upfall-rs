use tetra::{
    graphics::{self, BlendState, Color, DrawParams},
    input::{self, Key},
    math::{Rect, Vec2},
};

use crate::{
    level::Level,
    player::Player,
    tilemap::{Tile, Tilemap},
    Assets,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMode {
    Dark,
    Light,
}

impl WorldMode {
    pub fn next(&self) -> WorldMode {
        match *self {
            WorldMode::Dark => WorldMode::Light,
            WorldMode::Light => WorldMode::Dark,
        }
    }

    pub fn switch(&mut self) {
        *self = self.next();
    }
}

pub struct World {
    player: Player,
    dark_tilemap: Tilemap,
    light_tilemap: Tilemap,
    mode: WorldMode,
    spawn_pos: Vec2<f32>,
}

impl World {
    pub fn new(level: Level) -> World {
        let Level {
            dark_tilemap,
            light_tilemap,
            spawn_pos,
        } = level;
        World {
            player: Player::new(spawn_pos),
            dark_tilemap,
            light_tilemap,
            mode: WorldMode::Dark,
            spawn_pos,
        }
    }

    pub fn reset(&mut self) {
        self.player = Player::new(self.spawn_pos);
        self.mode = WorldMode::Dark;
    }

    pub fn update(&mut self, ctx: &mut tetra::Context) {
        if input::is_key_pressed(ctx, Key::X) {
            self.mode.switch();
        }

        let flip_gravity = match self.mode {
            WorldMode::Dark => false,
            WorldMode::Light => true,
        };
        self.player.update(ctx, flip_gravity);

        let tilemap = match self.mode {
            WorldMode::Dark => &self.dark_tilemap,
            WorldMode::Light => &self.light_tilemap,
        };

        let neighbors =
            tilemap.get_neigbor_rects(self.player.position + Player::PLAYER_SQUARE / 2.);
        for tile in &neighbors {
            if matches!(tile.0, Tile::Solid) {
                self.player.solve_collision_y(&tile.1);
                self.player.solve_collision_x(&tile.1);
            }
        }

        self.player.post_update();

        let player_rect = Rect::new(
            self.player.position.x,
            self.player.position.y,
            Player::PLAYER_SQUARE,
            Player::PLAYER_SQUARE,
        );
        let tilemap_rect = tilemap.rect();

        if input::is_key_pressed(ctx, Key::R) || !tilemap_rect.collides_with_rect(player_rect) {
            self.reset();
        }
    }

    pub fn draw(&self, ctx: &mut tetra::Context, assets: &Assets) {
        graphics::set_blend_state(ctx, BlendState::add(false));
        assets.shader.set_uniform(
            ctx,
            "u_circle_radius",
            if self.mode == WorldMode::Light {
                50.0
            } else {
                0.0
            },
        );
        assets.shader.set_uniform(
            ctx,
            "u_circle_pos",
            self.player.position + Player::PLAYER_SQUARE / 2.,
        );
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(self.player.position)
                .scale(Vec2::new(32., 32.))
                .color(Color::WHITE),
        );
        self.dark_tilemap.run_for_each_tile(|(x, y), tile| {
            if matches!(tile, Tile::Solid) {
                let real_x = x as f32 * self.dark_tilemap.tile_width();
                let real_y = y as f32 * self.dark_tilemap.tile_height();
                assets.pixel.draw(
                    ctx,
                    DrawParams::new()
                        .position(Vec2::from((real_x, real_y)))
                        .scale(self.dark_tilemap.tile_size())
                        .color(Color::RED),
                );
            }
        });
        self.light_tilemap.run_for_each_tile(|(x, y), tile| {
            if matches!(tile, Tile::Solid) {
                let real_x = x as f32 * self.light_tilemap.tile_width();
                let real_y = y as f32 * self.light_tilemap.tile_height();
                assets.pixel.draw(
                    ctx,
                    DrawParams::new()
                        .position(Vec2::from((real_x, real_y)))
                        .scale(self.light_tilemap.tile_size())
                        .color(Color::BLUE),
                );
            }
        });
        graphics::reset_blend_state(ctx);
    }
}
