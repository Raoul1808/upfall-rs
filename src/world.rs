use core::fmt;

use tetra::{
    graphics::{self, BlendState, Color, DrawParams, Rectangle},
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

impl fmt::Display for WorldMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorldMode::Dark => write!(f, "Dark"),
            WorldMode::Light => write!(f, "Light"),
        }
    }
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
    end_rect: Rectangle,
    win: bool,
}

impl World {
    pub fn new(level: Level) -> World {
        let Level {
            dark_tilemap,
            light_tilemap,
            spawn_pos,
            end_pos,
            ..
        } = level;
        let tile_size = dark_tilemap.tile_size();
        World {
            player: Player::new(spawn_pos),
            dark_tilemap,
            light_tilemap,
            mode: WorldMode::Dark,
            spawn_pos,
            end_rect: Rectangle::new(end_pos.x, end_pos.y, tile_size.x, tile_size.y),
            win: false,
        }
    }

    pub fn reset(&mut self) {
        self.player = Player::new(self.spawn_pos);
        self.mode = WorldMode::Dark;
    }

    pub fn player_pos(&self) -> Vec2<f32> {
        self.player.get_hbox().center()
    }

    pub fn get_world_rect(&self) -> Rectangle {
        self.dark_tilemap.rect()
    }

    pub fn update(&mut self, ctx: &mut tetra::Context) {
        if self.win {
            return;
        }
        self.player.update(ctx);

        if self.player.get_hbox().intersects(&self.end_rect) {
            // win!!
            self.win = true;
            return;
        }

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
                        self.player.on_world_change(self.mode);
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
                .position(player_hbox.center())
                .origin(Vec2::one() * 8.)
                .color(Color::WHITE)
                .scale(Vec2::new(
                    if self.player.flip_horizontal() {
                        -1.0
                    } else {
                        1.0
                    },
                    if self.player.flip_vertical() {
                        -1.0
                    } else {
                        1.0
                    },
                )),
        );
        assets
            .door
            .draw(ctx, DrawParams::new().position(self.end_rect.top_left()));
        self.dark_tilemap.render_tilemap(ctx, assets, Color::RED);
        self.light_tilemap.render_tilemap(ctx, assets, Color::BLUE);
        graphics::reset_blend_state(ctx);
    }

    pub fn win(&self) -> bool {
        self.win
    }
}
