use tetra::math::Vec2;

use crate::tilemap::Tilemap;

pub struct Level {
    pub tilemap: Tilemap,
    pub spawn_pos: Vec2<f32>,
}
