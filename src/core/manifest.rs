//! The asset manifest: ONE keyed table through which every model, texture,
//! sound, music track and font is referenced. Chapters never hard-code file
//! paths — they ask for a key (`"model.b612.rose"`), and the manifest says
//! whether that key is a file under `assets/` or a procedural fallback.
//!
//! Sources (later entries override earlier ones):
//! 1. the copies compiled into the binary (`assets/manifest.ron`,
//!    `assets/chapters/<id>.ron`) — the game always starts, even offline;
//! 2. the same files loaded at runtime through the AssetServer — so a player
//!    can drop new files in `assets/` and edit the RON without recompiling.
//!
//! See docs/ASSETS.md for the format and how to swap in your own assets.

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub struct ManifestEntry {
    /// File under `assets/` (glb/gltf, png/jpg, ogg/wav, ttf). `None` → procedural.
    #[serde(default)]
    pub path: Option<String>,
    /// Several files to choose from at random (footsteps …). Overrides `path`.
    #[serde(default)]
    pub variants: Vec<String>,
    /// Procedural generator id used when there is no file (or it fails to load).
    #[serde(default)]
    pub procedural: Option<String>,
    /// Uniform scale applied to the loaded model.
    #[serde(default = "one")]
    pub scale: f32,
    /// Offset (m) applied to the loaded model, in its local frame.
    #[serde(default)]
    pub offset: [f32; 3],
    /// Extra yaw (degrees) applied to the loaded model.
    #[serde(default)]
    pub yaw: f32,
    /// Volume multiplier for sounds.
    #[serde(default = "one")]
    pub volume: f32,
    /// Sounds: loop playback.
    #[serde(default)]
    pub looped: bool,
    /// glTF animation clip names for characters: idle / walk / run / jump.
    #[serde(default)]
    pub anims: HashMap<String, String>,
    /// Free text: what the key is used for.
    #[serde(default)]
    pub note: String,
}

fn one() -> f32 {
    1.0
}

impl Default for ManifestEntry {
    fn default() -> Self {
        ManifestEntry {
            path: None,
            variants: vec![],
            procedural: None,
            scale: 1.0,
            offset: [0.0; 3],
            yaw: 0.0,
            volume: 1.0,
            looped: false,
            anims: HashMap::new(),
            note: String::new(),
        }
    }
}

/// One RON manifest file.
#[derive(Asset, TypePath, Deserialize, Debug, Default, Clone)]
pub struct ManifestFile {
    #[serde(default)]
    pub entries: HashMap<String, ManifestEntry>,
}

impl ManifestFile {
    pub fn parse(text: &str) -> Result<ManifestFile, String> {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str::<ManifestFile>(text)
            .map_err(|e| e.to_string())
    }
}

#[derive(Default, TypePath)]
pub struct ManifestLoader;

impl AssetLoader for ManifestLoader {
    type Asset = ManifestFile;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<ManifestFile, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = String::from_utf8_lossy(&bytes);
        ManifestFile::parse(&text).map_err(std::io::Error::other)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// The merged manifest.
#[derive(Resource, Default, Debug)]
pub struct Manifest {
    pub entries: HashMap<String, ManifestEntry>,
}

impl Manifest {
    pub fn merge(&mut self, file: &ManifestFile) {
        for (k, v) in &file.entries {
            self.entries.insert(k.clone(), v.clone());
        }
    }
    pub fn get(&self, key: &str) -> Option<&ManifestEntry> {
        self.entries.get(key)
    }
    /// File path for a key (first variant or `path`).
    pub fn path(&self, key: &str) -> Option<&str> {
        let e = self.entries.get(key)?;
        e.variants.first().map(|s| s.as_str()).or(e.path.as_deref())
    }
}

/// Manifest files compiled into the binary (the runtime copies override them).
pub const EMBEDDED: &[(&str, &str)] = &[
    ("manifest.ron", include_str!("../../assets/manifest.ron")),
    ("chapters/b612.ron", include_str!("../../assets/chapters/b612.ron")),
    ("chapters/king.ron", include_str!("../../assets/chapters/king.ron")),
    ("chapters/vanity.ron", include_str!("../../assets/chapters/vanity.ron")),
    ("chapters/bar.ron", include_str!("../../assets/chapters/bar.ron")),
    ("chapters/businessman.ron", include_str!("../../assets/chapters/businessman.ron")),
    ("chapters/lamp.ron", include_str!("../../assets/chapters/lamp.ron")),
    ("chapters/geo.ron", include_str!("../../assets/chapters/geo.ron")),
    ("chapters/earth.ron", include_str!("../../assets/chapters/earth.ron")),
];

/// Handles of the runtime manifest files being loaded during Boot.
#[derive(Resource, Default)]
pub struct ManifestLoading {
    pub handles: Vec<Handle<ManifestFile>>,
    pub merged: bool,
}

pub fn embedded_manifest() -> Manifest {
    let mut m = Manifest::default();
    for (name, text) in EMBEDDED {
        match ManifestFile::parse(text) {
            Ok(f) => m.merge(&f),
            Err(e) => error!("manifest {name}: {e}"),
        }
    }
    m
}
