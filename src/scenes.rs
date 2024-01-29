use tetra::{
    graphics::{Color, DrawParams},
    input,
    math::Vec2,
};

use crate::{
    player::Player,
    tilemap::{Tile, Tilemap},
    Assets,
};

pub trait Scene {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result;
    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result;
}

pub struct GameScene {
    player: Player,
    tilemap: Tilemap,
    mouse_pos: Vec2<f32>,
}

impl GameScene {
    pub fn new() -> GameScene {
        let mut tilemap = Tilemap::new((40, 23), (32., 32.));
        tilemap.set_tile_usize((2, 5), Tile::Solid);
        tilemap.set_tile_usize((3, 5), Tile::Solid);
        tilemap.set_tile_usize((4, 5), Tile::Solid);
        tilemap.set_tile_usize((5, 5), Tile::Solid);
        tilemap.set_tile_usize((6, 5), Tile::Solid);
        tilemap.set_tile_usize((7, 5), Tile::Solid);
        tilemap.set_tile_usize((7, 4), Tile::Solid);
        GameScene {
            player: Player::new(),
            tilemap,
            mouse_pos: Vec2::default(),
        }
    }
}

impl Scene for GameScene {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        self.mouse_pos = input::get_mouse_position(ctx);
        if input::is_mouse_button_down(ctx, input::MouseButton::Left) {
            self.tilemap.set_tile_f32(self.mouse_pos, Tile::Solid);
        }
        if input::is_mouse_button_down(ctx, input::MouseButton::Right) {
            self.tilemap.set_tile_f32(self.mouse_pos, Tile::None);
        }
        self.player.update(ctx);

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

        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
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
                        .color(Color::BLACK),
                );
            }
        });
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(self.tilemap.snap(self.mouse_pos))
                .scale(self.tilemap.tile_size())
                .color(Color::WHITE.with_alpha(0.3)),
        );
        Ok(())
    }
}
