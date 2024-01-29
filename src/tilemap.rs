use std::cmp::max;

use tetra::math::{Rect, Vec2};

pub struct Tilemap {
    tiles: Vec<Tile>,
    tilemap_size: Vec2<usize>,
    tile_size: Vec2<f32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Tile {
    #[default]
    None,
    Solid,
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

    fn pos_to_index(&self, pos: (usize, usize)) -> usize {
        pos.0 + pos.1 * self.tilemap_size.x
    }

    fn index_to_pos(&self, index: usize) -> (usize, usize) {
        (index % self.tilemap_size.x, index / self.tilemap_size.x)
    }

    pub fn set_tile(&mut self, pos: (usize, usize), tile: Tile) {
        let index = self.pos_to_index(pos);
        if let Some(t) = self.tiles.get_mut(index) {
            *t = tile;
        }
    }

    pub fn get_neigbor_rects(&self, pos: Vec2<f32>) -> Vec<(Tile, Rect<f32, f32>)> {
        let x = (pos.x / self.tile_size.x).round() as usize;
        let y = (pos.y / self.tile_size.y).round() as usize;
        let mut vec = vec![];
        for n in 0..9 {
            let dx = n % 3 - 1 as i32;
            let dy = n / 3 - 1 as i32;
            let ix = max(x as i32 + dx, 0) as usize;
            let iy = max(y as i32 + dy, 0) as usize;
            let fx = (x as i32 + dx) as f32;
            let fy = (y as i32 + dy) as f32;
            let rect = Rect::new(fx as f32 * self.tile_width(), fy * self.tile_height(), self.tile_width(), self.tile_height());
            if let Some(t) = self.tiles.get(self.pos_to_index((ix, iy))) {
                let distance = pos.distance(rect.center());
                vec.push((distance, *t, rect));
            }
        }
        vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        vec.into_iter()
            .map(|(_, t, r)| (t, r))
            .collect()
    }

    pub fn run_for_each_tile<F>(&self, mut f: F)
    where F : FnMut((usize, usize), &Tile)
    {
        self.tiles.iter().enumerate().for_each(|(index, tile)| f(self.index_to_pos(index), tile));
    }
}
