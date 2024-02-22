use std::cmp::{max, min};

use derive_more::Display;
use serde::{Deserialize, Serialize};
use tetra::{
    graphics::{Color, DrawParams, Rectangle},
    math::Vec2,
};

use crate::Assets;

#[derive(Clone, Serialize, Deserialize)]
pub struct Tilemap {
    tiles: Vec<Tile>,
    tilemap_size: Vec2<usize>,
    tile_size: Vec2<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    #[default]
    None,
    Solid,
    Spike(Facing),
    Portal(Axis),
    Key,
    Spring(Facing),
}

impl Tile {
    pub fn type_str(&self) -> &str {
        match self {
            Tile::None => "None",
            Tile::Solid => "Solid",
            Tile::Spike(_) => "Spike",
            Tile::Portal(_) => "Portal",
            Tile::Key => "Key",
            Tile::Spring(_) => "Spring",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Facing {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Axis {
    #[default]
    Vertical,
    Horizontal,
}

impl Tile {
    pub fn hbox(&self, pos: Vec2<f32>, size: Vec2<f32>) -> Rectangle {
        const SPIKE_FRONT_GAP: f32 = 9. / 16.;
        const SPIKE_THICKNESS: f32 = 1. - SPIKE_FRONT_GAP;
        const SPIKE_SIDE_GAP: f32 = 1. / 16.;
        const SPIKE_LENGTH: f32 = 14. / 16.;
        match *self {
            Tile::None => Rectangle::default(),
            Tile::Solid | Tile::Portal(_) => Rectangle::new(pos.x, pos.y, size.x, size.y),
            Tile::Spike(dir) | Tile::Spring(dir) => match dir {
                Facing::Up => Rectangle::new(
                    pos.x + size.x * SPIKE_SIDE_GAP,
                    pos.y + size.y * SPIKE_FRONT_GAP,
                    size.x * SPIKE_LENGTH,
                    size.y * SPIKE_THICKNESS,
                ),
                Facing::Down => Rectangle::new(
                    pos.x + size.x * SPIKE_SIDE_GAP,
                    pos.y,
                    size.x * SPIKE_LENGTH,
                    size.y * SPIKE_THICKNESS,
                ),
                Facing::Left => Rectangle::new(
                    pos.x + size.x * SPIKE_FRONT_GAP,
                    pos.y + size.y * SPIKE_SIDE_GAP,
                    size.x * SPIKE_THICKNESS,
                    size.y * SPIKE_LENGTH,
                ),
                Facing::Right => Rectangle::new(
                    pos.x,
                    pos.y + size.y * SPIKE_SIDE_GAP,
                    size.x * SPIKE_THICKNESS,
                    size.y * SPIKE_LENGTH,
                ),
            },
            Tile::Key => Rectangle::new(pos.x + 4., pos.y + 4., size.x - 8., size.y - 8.),
        }
    }

    #[allow(dead_code)]
    pub fn set_facing(&mut self, facing: Facing) {
        if let Tile::Spike(ref mut f) = *self {
            *f = facing;
        }
    }

    #[allow(dead_code)]
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

    pub fn size(&self) -> Vec2<usize> {
        self.tilemap_size
    }

    pub fn rect(&self) -> Rectangle {
        Rectangle::new(
            0.,
            0.,
            self.tilemap_size.x as f32 * self.tile_width(),
            self.tilemap_size.y as f32 * self.tile_height(),
        )
    }

    pub fn keys_amount(&self) -> usize {
        self.tiles.iter().filter(|t| matches!(t, Tile::Key)).count()
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

    pub fn get_neigbor_tile_hboxes(&self, pos: Vec2<f32>) -> Vec<(Tile, Rectangle)> {
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
            Tile::Solid => {
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
            Tile::Portal(axis) => {
                use std::f32::consts::PI;
                let rot = match axis {
                    Axis::Horizontal => 0.,
                    Axis::Vertical => PI / 2.,
                };
                let offset = self.tile_size() / 2.;
                let pos = Vec2::new(x as f32, y as f32) * self.tile_size() + offset;
                assets.portal.draw(
                    ctx,
                    DrawParams::new()
                        .position(pos)
                        .rotation(rot)
                        .origin(offset)
                        .color(color),
                )
            }
            Tile::Key => {
                let pos = Vec2::new(x as f32, y as f32) * self.tile_size;
                assets
                    .key
                    .draw(ctx, DrawParams::new().position(pos).color(color));
            }
            Tile::Spring(_) => {
                let rect = tile.hbox(
                    Vec2::new(x as f32 * self.tile_width(), y as f32 * self.tile_height()),
                    self.tile_size(),
                );
                assets.pixel.draw(
                    ctx,
                    DrawParams::new()
                        .position(rect.top_left())
                        .scale(rect.bottom_right() - rect.top_left())
                        .color(color),
                );
            }
        });
    }

    pub fn resize(&mut self, new_size: Vec2<usize>) {
        if self.tilemap_size == new_size {
            return;
        }
        let new_total_size = new_size.x * new_size.y;
        let old_total_size = self.tilemap_size.x * self.tilemap_size.y;
        let mut new_map = vec![Tile::None; new_size.x * new_size.y];
        let mut index = 0;
        let mut x = 0;
        let mut y = 0;
        while (x + 1) * (y + 1) < min(new_total_size, old_total_size) {
            new_map[index] = self.tiles[y * self.tilemap_size.x + x];
            index += 1;
            x += 1;
            if x >= min(self.tilemap_size.x, new_size.x) {
                x = 0;
                y += 1;
                if self.tilemap_size.x < new_size.x {
                    index = new_size.x * y;
                }
            }
        }
        self.tilemap_size = new_size;
        self.tiles = new_map;
    }
}

#[cfg(test)]
mod test {
    use super::{Tile, Tilemap};

    fn make_vec_from_str(flat_tilemap: &str) -> Vec<Tile> {
        let mut vec = Vec::with_capacity(flat_tilemap.len());
        flat_tilemap.chars().for_each(|c| match c {
            'O' => {
                vec.push(Tile::None);
            }
            'X' => {
                vec.push(Tile::Solid);
            }
            _ => {
                panic!("Invalid character for test tilemap creation");
            }
        });
        vec
    }

    #[test]
    fn resize_tilemap() {
        let mut tilemap = Tilemap::new((3, 3), (1., 1.));
        tilemap.tiles[4] = Tile::Solid;
        tilemap.tiles[8] = Tile::Solid;
        let expected_tiles = make_vec_from_str("OOOOXOOOX");
        assert_eq!(tilemap.tiles, expected_tiles);

        tilemap.resize((2, 3).into());
        let expected_tiles = make_vec_from_str("OOOXOO");
        assert_eq!(tilemap.tiles, expected_tiles);

        tilemap.resize((4, 4).into());
        let expected_tiles = make_vec_from_str("OOOOOXOOOOOOOOOO");
        assert_eq!(tilemap.tiles, expected_tiles);
    }
}
