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
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Level, LevelError> {
        let bytes = fs::read(path).map_err(LevelError::Io)?;
        bincode::options()
            .with_varint_encoding()
            .with_big_endian()
            .deserialize(&bytes)
            .map_err(LevelError::Deserialization)
    }

    pub fn save_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LevelError> {
        let bytes = bincode::options()
            .with_varint_encoding()
            .with_big_endian()
            .serialize(self)
            .map_err(LevelError::Serialization)?;
        fs::write(path, bytes).map_err(LevelError::Io)
    }
}

#[derive(Default)]
pub struct LevelPack {
    pub name: String,
    pub levels: Vec<Level>,
}

impl LevelPack {
    pub fn from_directory<P: AsRef<Path>>(path: P) -> Result<LevelPack, LevelError> {
        let mut levels = vec![];
        let entries: std::io::Result<Vec<fs::DirEntry>> = fs::read_dir(path.as_ref())
            .map_err(LevelError::Io)?
            .into_iter()
            .collect();
        let mut entries = entries.map_err(LevelError::Io)?;
        entries.sort_by_key(|e| e.path());
        for entry in &entries {
            let ft = entry.file_type().map_err(LevelError::Io)?;
            if ft.is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "umdx" {
                        let level = Level::load_file(&path)?;
                        levels.push(level);
                    }
                }
            }
        }
        Ok(LevelPack {
            levels,
            name: path
                .as_ref()
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or("Unnamed Pack".into()),
        })
    }

    pub fn get_packs_in_directory<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<LevelPack>> {
        let mut packs = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(p) = LevelPack::from_directory(entry.path()) {
                    packs.push(p);
                }
            }
        }
        Ok(packs)
    }
}
