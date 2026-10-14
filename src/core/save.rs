//! Progress + settings persistence. Mirrors the original's `JourneyProgress`
//! (revealed / visited planet lists) plus `PlayerPrefs`-style settings.
//! Web: localStorage key `lp.bevy.save.v1`; native: ~/.little_prince_bevy/.

use super::i18n::Lang;
use super::platform;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const SAVE_KEY: &str = "lp.bevy.save.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Quality {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TouchMode {
    #[default]
    Auto,
    On,
    Off,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
    /// Look sensitivity multiplier (1.0 = default).
    pub look: f32,
    pub invert_y: bool,
    /// Lock the pointer on click for mouse look (otherwise drag to look).
    pub pointer_lock: bool,
    pub free_travel: bool,
    pub quality: Quality,
    pub touch: TouchMode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            master: 0.8,
            music: 0.7,
            sfx: 0.8,
            look: 1.0,
            invert_y: false,
            pointer_lock: false,
            free_travel: false,
            quality: Quality::Medium,
            touch: TouchMode::Auto,
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveData {
    pub version: u32,
    pub lang: Lang,
    /// Planets whose fog has cleared on the map (JourneyProgress.revealed).
    pub revealed: Vec<String>,
    /// Planets the prince has left (JourneyProgress.visited).
    pub visited: Vec<String>,
    /// Hidden goals found, as `<chapter>:<key>`.
    pub hidden_found: Vec<String>,
    /// Last planet entered (for "continue").
    pub last: Option<String>,
    pub settings: Settings,
    /// Arbitrary per-chapter persistent values (`<chapter>.<name>` → value).
    pub chapter_data: std::collections::BTreeMap<String, String>,
}

impl Default for SaveData {
    fn default() -> Self {
        SaveData {
            version: 1,
            lang: Lang::Zh,
            revealed: Vec::new(),
            visited: Vec::new(),
            hidden_found: Vec::new(),
            last: None,
            settings: Settings::default(),
            chapter_data: Default::default(),
        }
    }
}

impl SaveData {
    pub fn load() -> SaveData {
        platform::storage_get(SAVE_KEY)
            .and_then(|s| ron::from_str::<SaveData>(&s).ok())
            .unwrap_or_default()
    }

    pub fn is_revealed(&self, id: &str) -> bool {
        self.revealed.iter().any(|s| s == id)
    }
    pub fn is_visited(&self, id: &str) -> bool {
        self.visited.iter().any(|s| s == id)
    }
    pub fn reveal(&mut self, id: &str) {
        if !self.is_revealed(id) {
            self.revealed.push(id.to_string());
        }
    }
    pub fn visit(&mut self, id: &str) {
        if !self.is_visited(id) {
            self.visited.push(id.to_string());
        }
    }
    pub fn has_started(&self) -> bool {
        !self.visited.is_empty() || self.last.is_some()
    }
}

/// When false (test mode / `?fresh=1`), nothing is written to storage.
#[derive(Resource)]
pub struct SavePersist(pub bool);

#[derive(Resource, Default)]
pub(crate) struct SaveTimer(f32);

/// Writes the save shortly after it changes.
pub(crate) fn persist_save(
    save: Res<SaveData>,
    persist: Res<SavePersist>,
    mut timer: ResMut<SaveTimer>,
    time: Res<Time<Real>>,
) {
    if save.is_changed() && !save.is_added() {
        timer.0 = 0.4;
    }
    if timer.0 > 0.0 {
        timer.0 -= time.delta_secs();
        if timer.0 <= 0.0 && persist.0 {
            if let Ok(s) = ron::to_string(&*save) {
                platform::storage_set(SAVE_KEY, &s);
            }
        }
    }
}

pub fn clear_save_storage() {
    platform::storage_remove(SAVE_KEY);
}
