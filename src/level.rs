use std::{
    fs::{self, File},
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use bincode::Options;
use serde::{Deserialize, Serialize};
use tetra::math::Vec2;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

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
    Zip(zip::result::ZipError),
}

impl Level {
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Level, LevelError> {
        let bytes = fs::read(path).map_err(LevelError::Io)?;
        Self::load_bytes(&bytes)
    }

    pub fn load_bytes(bytes: &[u8]) -> Result<Level, LevelError> {
        bincode::options()
            .with_varint_encoding()
            .with_big_endian()
            .deserialize(bytes)
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
    pub location: PathBuf,
    pub is_zip: bool,
}

impl LevelPack {
    pub fn from_directory<P: AsRef<Path>>(path: P) -> Result<LevelPack, LevelError> {
        let mut levels = vec![];
        let entries: std::io::Result<Vec<fs::DirEntry>> = fs::read_dir(path.as_ref())
            .map_err(LevelError::Io)?
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
            location: path.as_ref().to_path_buf(),
            is_zip: false,
        })
    }

    pub fn from_zip_file<P: AsRef<Path>>(path: P) -> Result<LevelPack, LevelError> {
        let f = File::open(&path).map_err(LevelError::Io)?;
        let reader = BufReader::new(f);
        let mut zip = ZipArchive::new(reader).map_err(LevelError::Zip)?;
        let mut files = vec![];
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).map_err(LevelError::Zip)?;
            let path = {
                match file.enclosed_name() {
                    Some(p) => (*p).to_path_buf(),
                    None => continue,
                }
            };
            if path.extension().is_some_and(|e| e == "umdx") {
                let mut buf = vec![];
                let _ = file.read_to_end(&mut buf).map_err(LevelError::Io)?;
                let bytes = buf.clone();
                files.push((path.to_path_buf(), bytes));
            }
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut levels = vec![];
        for (_, bytes) in files {
            let level = Level::load_bytes(&bytes)?;
            levels.push(level);
        }
        let name = path
            .as_ref()
            .file_stem()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or("Unnamed Pack".into());
        Ok(LevelPack {
            name,
            levels,
            location: path.as_ref().to_path_buf(),
            is_zip: true,
        })
    }

    pub fn get_packs_in_directory<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<LevelPack>> {
        let mut packs = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(p) = LevelPack::from_directory(entry.path()) {
                    if !p.levels.is_empty() {
                        println!(
                            "Loaded pack {} from directory {} with {} levels",
                            p.name,
                            entry.path().display(),
                            p.levels.len()
                        );
                        packs.push(p);
                    }
                }
            }
            if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|e| e == "zip")
            {
                if let Ok(p) = LevelPack::from_zip_file(entry.path()) {
                    if !p.levels.is_empty() {
                        println!(
                            "Loaded pack {} from file {} with {} levels",
                            p.name,
                            entry.path().display(),
                            p.levels.len()
                        );
                        packs.push(p);
                    }
                }
            }
        }
        Ok(packs)
    }

    pub fn make_zip_from_dir<P: AsRef<Path>>(dir: P) -> Result<LevelPack, LevelError> {
        assert!(!dir.as_ref().is_file());
        let files: Vec<PathBuf> = fs::read_dir(&dir)
            .map_err(LevelError::Io)?
            .filter_map(Result::ok)
            .map(|f| f.path())
            .collect();
        dbg!(&files);
        let zip_path = dir.as_ref().with_extension("zip");
        dbg!(&zip_path);
        {
            let file = File::create(&zip_path).map_err(LevelError::Io)?;
            let mut writer = ZipWriter::new(file);
            for file in &files {
                let filename = file.file_name().unwrap();
                writer
                    .start_file(
                        filename.to_string_lossy().to_string(),
                        FileOptions::default(),
                    )
                    .map_err(LevelError::Zip)?;
                let file_bytes = fs::read(file).map_err(LevelError::Io)?;
                writer.write_all(&file_bytes).map_err(LevelError::Io)?;
            }
            writer.finish().map_err(LevelError::Zip)?;
            drop(writer);
        }
        // Load back the pack to make sure the process worked
        let pack = LevelPack::from_zip_file(zip_path)?;
        Ok(pack)
    }
}
