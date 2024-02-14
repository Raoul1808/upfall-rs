use std::{fs, io, path::Path};

use bincode::Options;
use serde::{Deserialize, Serialize};
use tetra::math::Vec2;

use crate::{palette::Palette, tilemap::Tilemap};

#[derive(Clone, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub author: String,
    pub dark_tilemap: Tilemap,
    pub light_tilemap: Tilemap,
    pub palette: Palette,
    pub spawn_pos: Vec2<f32>,
    pub end_pos: Vec2<f32>,
}

#[derive(Debug)]
pub enum LevelError {
    Io(io::Error),
    Serialization(Box<bincode::ErrorKind>),
    Deserialization(Box<bincode::ErrorKind>),
}

impl Level {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Level, LevelError> {
        let bytes = fs::read(path).map_err(LevelError::Io)?;
        bincode::options()
            .with_varint_encoding()
            .with_big_endian()
            .deserialize(&bytes)
            .map_err(LevelError::Deserialization)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), LevelError> {
        let bytes = bincode::options()
            .with_varint_encoding()
            .with_big_endian()
            .serialize(self)
            .map_err(LevelError::Serialization)?;
        fs::write(path, bytes).map_err(LevelError::Io)
    }
}
