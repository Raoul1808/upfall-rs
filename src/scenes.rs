use tetra::{
    graphics::{Color, DrawParams},
    input::{self, Key},
    math::{Rect, Vec2},
};

use crate::{tilemap::{Tile, Tilemap}, Assets};

pub trait Scene {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result;
    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result;
}

#[derive(Debug)]
pub struct Player {
    pub position: Vec2<f32>,
    pub velocity: Vec2<f32>,
}

const PLAYER_SQUARE: f32 = 32.;

impl Player {
    pub fn new() -> Player {
        Player {
            position: Vec2::new(100., 100.),
            velocity: Vec2::default(),
        }
    }

    pub fn update(&mut self, ctx: &mut tetra::Context) {
        const MAX_FALL_SPEED: f32 = 15.;
        const GRAVITY: f32 = 0.7;
        const JUMP_FORCE: f32 = 11.;
        const WALK_SPEED: f32 = 4.;

        let mut x_vel = 0.;
        if input::is_key_down(ctx, Key::Left) {
            x_vel = -WALK_SPEED;
        }
        if input::is_key_down(ctx, Key::Right) {
            x_vel = WALK_SPEED;
        }
        self.velocity.x = x_vel;

        self.velocity.y += GRAVITY;
        if self.velocity.y > MAX_FALL_SPEED {
            self.velocity.y = MAX_FALL_SPEED;
        }

        if input::is_key_pressed(ctx, Key::Space) {
            self.velocity.y = -JUMP_FORCE;
        }
    }

    pub fn solve_collision_y(&mut self, rect: &Rect<f32, f32>) {
        let next_hbox = Rect::new(self.position.x, self.position.y + self.velocity.y, PLAYER_SQUARE, PLAYER_SQUARE);
        if rect.collides_with_rect(next_hbox) {
            if self.velocity.y < 0. {
                self.position.y = rect.y + rect.h;
                self.velocity.y = 0.;
            }
            if self.velocity.y > 0. {
                self.position.y = rect.y - PLAYER_SQUARE;
                self.velocity.y = 0.;
            }
        }
    }
    
    pub fn solve_collision_x(&mut self, rect: &Rect<f32, f32>) {
        let next_hbox = Rect::new(self.position.x + self.velocity.x, self.position.y, PLAYER_SQUARE, PLAYER_SQUARE);
        if rect.collides_with_rect(next_hbox) {
            if self.velocity.x > 0. {
                self.position.x = rect.x - PLAYER_SQUARE;
                self.velocity.x = 0.;
            }
            if self.velocity.x < 0. {
                self.position.x = rect.x + rect.w;
                self.velocity.x = 0.;
            }
        }
    }

    pub fn post_update(&mut self) {
        self.position += self.velocity;
    }
}

pub struct GameScene {
    player: Player,
    tilemap: Tilemap,
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
            tilemap: tilemap,
        }
    }
}

impl Scene for GameScene {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        let m_pos = input::get_mouse_position(ctx);
        if input::is_mouse_button_down(ctx, input::MouseButton::Left) {
            self.tilemap.set_tile_f32(m_pos, Tile::Solid);
        }
        if input::is_mouse_button_down(ctx, input::MouseButton::Right) {
            self.tilemap.set_tile_f32(m_pos, Tile::None);
        }
        self.player.update(ctx);
        
        let neighbors = self.tilemap.get_neigbor_rects(self.player.position + PLAYER_SQUARE / 2.1);
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
                        .color(Color::BLACK)
                );
            }
        });
        Ok(())
    }
}
