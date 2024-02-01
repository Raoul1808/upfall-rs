use tetra::{
    graphics::{self, BlendState, Color, DrawParams},
    input::{self, Key},
    math::Vec2,
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

        let neighbors = tilemap.get_neigbor_tile_hboxes(self.player.get_hbox().center());
        let mut spikes = vec![];
        for (tile, rect) in &neighbors {
            match tile {
                Tile::None => continue,
                Tile::Solid => {
                    self.player.solve_collision_y(rect);
                    self.player.solve_collision_x(rect);
                }
                Tile::Spike(_) => {
                    spikes.push(rect);
                }
                Tile::Portal(axis) => {
                    if self.player.can_traverse_portal(rect, *axis) {
                        self.mode.switch();
                    }
                }
            }
        }

        self.player.post_update();

        let player_rect = self.player.get_hbox();
        if spikes.into_iter().any(|s| s.intersects(&player_rect)) {
            self.reset();
            return;
        }
        let tilemap_rect = tilemap.rect();

        if input::is_key_pressed(ctx, Key::R) || !tilemap_rect.intersects(&player_rect) {
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
        let player_hbox = self.player.get_hbox();
        assets
            .shader
            .set_uniform(ctx, "u_circle_pos", player_hbox.center());
        assets.player.draw(
            ctx,
            DrawParams::new()
                .position(player_hbox.top_left())
                .color(Color::WHITE),
        );
        self.dark_tilemap.render_tilemap(ctx, assets, Color::RED);
        self.light_tilemap.render_tilemap(ctx, assets, Color::BLUE);
        graphics::reset_blend_state(ctx);
    }
}
