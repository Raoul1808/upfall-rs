use std::cmp::max;

use serde::{Deserialize, Serialize};
use tetra::{
    graphics::{Color, DrawParams},
    math::{Rect, Vec2},
};

use crate::Assets;

#[derive(Clone, Serialize, Deserialize)]
pub struct Tilemap {
    tiles: Vec<Tile>,
    tilemap_size: Vec2<usize>,
    tile_size: Vec2<f32>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Tile {
    #[default]
    None,
    Solid,
    Spike(Facing),
    Portal(Axis),
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Facing {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Axis {
    #[default]
    Vertical,
    Horizontal,
}

impl Tile {
    pub fn hbox(&self, pos: Vec2<f32>, size: Vec2<f32>) -> Rect<f32, f32> {
        const SPIKE_FRONT_GAP: f32 = 9. / 16.;
        const SPIKE_THICKNESS: f32 = 1. - SPIKE_FRONT_GAP;
        const SPIKE_SIDE_GAP: f32 = 1. / 16.;
        const SPIKE_LENGTH: f32 = 14. / 16.;
        match *self {
            Tile::None => Rect::default(),
            Tile::Solid | Tile::Portal(_) => Rect::new(pos.x, pos.y, size.x, size.y),
            Tile::Spike(dir) => match dir {
                Facing::Up => Rect::new(
                    pos.x + size.x * SPIKE_SIDE_GAP,
                    pos.y + size.y * SPIKE_FRONT_GAP,
                    size.x * SPIKE_LENGTH,
                    size.y * SPIKE_THICKNESS,
                ),
                Facing::Down => Rect::new(
                    pos.x + size.x * SPIKE_SIDE_GAP,
                    pos.y,
                    size.x * SPIKE_LENGTH,
                    size.y * SPIKE_THICKNESS,
                ),
                Facing::Left => Rect::new(
                    pos.x + size.x * SPIKE_FRONT_GAP,
                    pos.y + size.y * SPIKE_SIDE_GAP,
                    size.x * SPIKE_THICKNESS,
                    size.y * SPIKE_LENGTH,
                ),
                Facing::Right => Rect::new(
                    pos.x,
                    pos.y + size.y * SPIKE_SIDE_GAP,
                    size.x * SPIKE_THICKNESS,
                    size.y * SPIKE_LENGTH,
                ),
            },
        }
    }

    pub fn set_facing(&mut self, facing: Facing) {
        if let Tile::Spike(ref mut f) = *self {
            *f = facing;
        }
    }

    pub fn set_axis(&mut self, axis: Axis) {
        if let Tile::Portal(ref mut a) = *self {
            *a = axis;
        }
    }
}

impl Tilemap {
    pub fn new(map_size: (usize, usize), tile_size: (f32, f32)) -> Tilemap {
        Tilemap {
            tiles: vec![Tile::None; map_size.0 * map_size.1],
            tilemap_size: Vec2::from(map_size),
            tile_size: Vec2::from(tile_size),
        }
    }

    pub fn tile_width(&self) -> f32 {
        self.tile_size.x
    }

    pub fn tile_height(&self) -> f32 {
        self.tile_size.y
    }

    pub fn tile_size(&self) -> Vec2<f32> {
        self.tile_size
    }

    pub fn rect(&self) -> Rect<f32, f32> {
        Rect::new(
            0.,
            0.,
            self.tilemap_size.x as f32 * self.tile_width(),
            self.tilemap_size.y as f32 * self.tile_height(),
        )
    }

    fn pos_to_index(&self, pos: (usize, usize)) -> usize {
        pos.0 + pos.1 * self.tilemap_size.x
    }

    fn index_to_pos(&self, index: usize) -> (usize, usize) {
        (index % self.tilemap_size.x, index / self.tilemap_size.x)
    }

    pub fn snap(&self, pos: Vec2<f32>) -> Vec2<f32> {
        Vec2::new(
            (pos.x / self.tile_width()).trunc() * self.tile_width(),
            (pos.y / self.tile_height()).trunc() * self.tile_height(),
        )
    }

    pub fn set_tile_f32(&mut self, pos: Vec2<f32>, tile: Tile) {
        if pos.x < 0. || pos.y < 0. {
            return;
        }
        let x = (pos.x / self.tile_width()).trunc() as usize;
        let y = (pos.y / self.tile_width()).trunc() as usize;
        self.set_tile_usize((x, y), tile);
    }

    pub fn set_tile_usize(&mut self, pos: (usize, usize), tile: Tile) {
        let index = self.pos_to_index(pos);
        if let Some(t) = self.tiles.get_mut(index) {
            *t = tile;
        }
    }

    pub fn get_neigbor_tile_hboxes(&self, pos: Vec2<f32>) -> Vec<(Tile, Rect<f32, f32>)> {
        let x = (pos.x / self.tile_width()).trunc() as usize;
        let y = (pos.y / self.tile_height()).trunc() as usize;
        let mut vec = vec![];
        for n in 0..9 {
            let dx = n % 3 - 1_i32;
            let dy = n / 3 - 1_i32;
            let ix = max(x as i32 + dx, 0) as usize;
            let iy = max(y as i32 + dy, 0) as usize;
            if let Some(t) = self.tiles.get(self.pos_to_index((ix, iy))) {
                let fx = (x as i32 + dx) as f32;
                let fy = (y as i32 + dy) as f32;
                let pos = Vec2::new(fx, fy) * self.tile_size;
                let hbox = t.hbox(pos, self.tile_size);
                let distance = pos.distance(hbox.center());
                vec.push((distance, *t, hbox));
            }
        }
        vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        vec.into_iter().map(|(_, t, r)| (t, r)).collect()
    }

    pub fn run_for_each_tile<F>(&self, mut f: F)
    where
        F: FnMut((usize, usize), &Tile),
    {
        self.tiles
            .iter()
            .enumerate()
            .for_each(|(index, tile)| f(self.index_to_pos(index), tile));
    }

    pub fn render_tilemap(&self, ctx: &mut tetra::Context, assets: &Assets, color: Color) {
        self.run_for_each_tile(|(x, y), tile| match tile {
            Tile::None => {}
            Tile::Solid | Tile::Portal(_) => {
                let pos = Vec2::new(x as f32, y as f32) * self.tile_size;
                assets
                    .tile
                    .draw(ctx, DrawParams::new().position(pos).color(color));
            }
            Tile::Spike(dir) => {
                use std::f32::consts::PI;
                let offset = self.tile_size() / 2.;
                let pos = Vec2::new(x as f32, y as f32) * self.tile_size + offset;
                let rot = match dir {
                    Facing::Right => 0.,
                    Facing::Left => PI,
                    Facing::Up => -PI / 2.,
                    Facing::Down => PI / 2.,
                };
                assets.spike.draw(
                    ctx,
                    DrawParams::new()
                        .position(pos)
                        .rotation(rot)
                        .origin(offset)
                        .color(color),
                );
            }
        });
    }
}
