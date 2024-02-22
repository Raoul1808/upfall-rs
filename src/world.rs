use core::fmt;

use keyframe::{ease, functions::EaseOutQuart, num_traits::clamp};
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

struct OthersideAnimator {
    size_anim_progress: f32,
    full_circle_size: f32,
    circle_size: f32,
    mode: WorldMode,
    size_passive_accumulator: f32,
}

impl OthersideAnimator {
    const SIZE_PASSIVE_SIN_MOD: f32 = 5.;
    const CIRCLE_GROW_TIME: f32 = 0.25;
    const CIRCLE_SHRINK_TIME: f32 = 0.125;
    const CIRCLE_SIZE: f32 = 75.;

    pub fn new() -> Self {
        Self {
            size_anim_progress: 0.,
            full_circle_size: 0.,
            circle_size: 0.,
            mode: WorldMode::Dark,
            size_passive_accumulator: 0.,
        }
    }

    pub fn world_change(&mut self, world: WorldMode) {
        self.mode = world;
    }

    pub fn update(&mut self, dt: f32) {
        let cond = match self.mode {
            WorldMode::Dark => self.size_anim_progress > 0.,
            WorldMode::Light => self.size_anim_progress < 1.,
        };
        if cond {
            match self.mode {
                WorldMode::Dark => {
                    self.size_anim_progress -= dt / Self::CIRCLE_SHRINK_TIME;
                }
                WorldMode::Light => {
                    self.size_anim_progress += dt / Self::CIRCLE_GROW_TIME;
                }
            }
            let t = clamp(self.size_anim_progress, 0., 1.);
            self.circle_size = ease(EaseOutQuart, 0., self.full_circle_size, t);
        } else if self.mode == WorldMode::Light {
            self.circle_size = self.full_circle_size;
        }
        self.size_passive_accumulator += dt;
        self.full_circle_size =
            Self::CIRCLE_SIZE + self.size_passive_accumulator.sin() * Self::SIZE_PASSIVE_SIN_MOD;
    }

    pub fn get_circle_size(&self) -> f32 {
        self.circle_size
    }
}

pub struct World {
    player: Player,
    dark_tilemap: Tilemap,
    light_tilemap: Tilemap,
    mode: WorldMode,
    spawn_pos: Vec2<f32>,
    end_rect: Rectangle,
    dark_keys: Vec<(usize, usize)>,
    light_keys: Vec<(usize, usize)>,
    keys_amount: usize,
    got_keys: usize,
    win: bool,
    otherside: OthersideAnimator,
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
        let keys_amount = dark_tilemap.keys_amount() + light_tilemap.keys_amount();
        World {
            player: Player::new(spawn_pos),
            dark_tilemap,
            light_tilemap,
            mode: WorldMode::Dark,
            spawn_pos,
            end_rect: Rectangle::new(end_pos.x, end_pos.y, tile_size.x, tile_size.y),
            win: false,
            dark_keys: Vec::new(),
            light_keys: Vec::new(),
            keys_amount,
            got_keys: 0,
            otherside: OthersideAnimator::new(),
        }
    }

    pub fn reset(&mut self) {
        self.win = false;
        self.player = Player::new(self.spawn_pos);
        self.mode = WorldMode::Dark;
        self.otherside = OthersideAnimator::new();
        for pos in &self.dark_keys {
            self.dark_tilemap.set_tile_usize(*pos, Tile::Key);
        }
        for pos in &self.light_keys {
            self.light_tilemap.set_tile_usize(*pos, Tile::Key);
        }
        self.got_keys = 0;
    }

    pub fn player_pos(&self) -> Vec2<f32> {
        self.player.get_hbox().center()
    }

    pub fn get_world_rect(&self) -> Rectangle {
        self.dark_tilemap.rect()
    }

    pub fn update(&mut self, ctx: &mut tetra::Context) {
        let dt = tetra::time::get_delta_time(ctx).as_secs_f32();
        if self.win {
            return;
        }
        self.player.update(ctx);

        let (tilemap, keys) = match self.mode {
            WorldMode::Dark => (&mut self.dark_tilemap, &mut self.dark_keys),
            WorldMode::Light => (&mut self.light_tilemap, &mut self.light_keys),
        };

        let neighbors = tilemap.get_neigbor_tile_hboxes(self.player.get_hbox().center());
        let mut spikes = vec![];
        let mut springs = vec![];
        let mut collected_keys = vec![];
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
                        self.otherside.world_change(self.mode);
                    }
                }
                Tile::Key => {
                    collected_keys.push(rect);
                }
                Tile::Spring(dir) => {
                    springs.push((rect, dir));
                }
            }
        }

        self.player.post_update();
        self.otherside.update(dt);

        let player_rect = self.player.get_hbox();
        if self.got_keys == self.keys_amount && player_rect.intersects(&self.end_rect) {
            self.win = true;
            return;
        }
        collected_keys.into_iter().for_each(|k| {
            if player_rect.intersects(k) {
                let coords = k.top_left() / tilemap.tile_size();
                let coords = (coords.x as usize, coords.y as usize);
                keys.push(coords);
                tilemap.set_tile_usize(coords, Tile::None);
                self.got_keys += 1;
            }
        });
        if spikes.into_iter().any(|s| s.intersects(&player_rect)) {
            self.reset();
            return;
        }
        springs.into_iter().for_each(|(rect, dir)| {
            if player_rect.intersects(rect) {
                self.player.on_spring(*dir);
            }
        });
        let tilemap_rect = tilemap.rect();

        if input::is_key_pressed(ctx, Key::R) || !tilemap_rect.intersects(&player_rect) {
            self.reset();
        }
    }

    pub fn draw(&self, ctx: &mut tetra::Context, assets: &Assets) {
        graphics::set_blend_state(ctx, BlendState::add(false));
        assets
            .shader
            .set_uniform(ctx, "u_circle_radius", self.otherside.get_circle_size());
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
        let door = if self.got_keys == self.keys_amount {
            &assets.door
        } else {
            &assets.door_locked
        };
        door.draw(ctx, DrawParams::new().position(self.end_rect.top_left()));
        self.dark_tilemap.render_tilemap(ctx, assets, Color::RED);
        self.light_tilemap.render_tilemap(ctx, assets, Color::BLUE);
        graphics::reset_blend_state(ctx);
    }

    pub fn win(&self) -> bool {
        self.win
    }
}
