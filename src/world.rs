use tetra::{
    graphics::{Color, DrawParams},
    input::{self, Key},
    math::{Rect, Vec2},
};

use crate::{
    level::Level,
    player::Player,
    tilemap::{Tile, Tilemap},
    Assets,
};

pub struct World {
    player: Player,
    tilemap: Tilemap,
    spawn_pos: Vec2<f32>,
    flip_gravity: bool,
}

impl World {
    pub fn new(level: Level) -> World {
        let Level { tilemap, spawn_pos } = level;
        World {
            player: Player::new(spawn_pos),
            tilemap,
            spawn_pos,
            flip_gravity: false,
        }
    }

    pub fn reset(&mut self) {
        self.player = Player::new(self.spawn_pos);
        self.flip_gravity = false;
    }

    pub fn update(&mut self, ctx: &mut tetra::Context) {
        if input::is_key_pressed(ctx, Key::X) {
            self.flip_gravity = !self.flip_gravity;
        }
        self.player.update(ctx, self.flip_gravity);

        let neighbors = self
            .tilemap
            .get_neigbor_rects(self.player.position + Player::PLAYER_SQUARE / 2.1);
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
        let tilemap_rect = self.tilemap.rect();

        if input::is_key_pressed(ctx, Key::R) || !tilemap_rect.collides_with_rect(player_rect) {
            self.reset();
        }
    }

    pub fn draw(&self, ctx: &mut tetra::Context, assets: &Assets) {
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(self.player.position)
                .scale(Vec2::new(32., 32.))
                .color(Color::RED),
        );
        self.tilemap.run_for_each_tile(|(x, y), tile| {
            if matches!(tile, Tile::Solid) {
                let real_x = x as f32 * self.tilemap.tile_width();
                let real_y = y as f32 * self.tilemap.tile_height();
                assets.pixel.draw(
                    ctx,
                    DrawParams::new()
                        .position(Vec2::from((real_x, real_y)))
                        .scale(self.tilemap.tile_size())
                        .color(Color::WHITE),
                );
            }
        });
    }
}
