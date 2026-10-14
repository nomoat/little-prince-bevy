//! Launch parameters (debug URL params, like the original's WebPlatform query
//! handling): `?chapter=<id>&test=1&lang=en&free=1&map=1`.

use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, Clone, Default)]
pub struct LaunchParams {
    /// `?chapter=<id>`: skip title/map and enter this chapter directly.
    pub chapter: Option<String>,
    /// `?test=1`: scripted autoplay; prints `[test:ok <id>]` / `[test:fail <id>] reason`.
    pub test: bool,
    /// `?lang=en|zh` overrides the saved language for this session.
    pub lang: Option<String>,
    /// `?free=1`: free travel (all planets selectable on the map).
    pub free: bool,
    /// `?map=1`: skip the title, open the journey map.
    pub map: bool,
    /// `?ending=1`: jump to the ending scene.
    pub ending: bool,
    /// `?touch=1|0`: force the on-screen touch controls on/off.
    pub touch: Option<bool>,
    /// `?quality=low|medium|high`.
    pub quality: Option<String>,
    /// `?speed=2`: game-time multiplier (test mode defaults to 1.5).
    pub speed: Option<f32>,
    /// `?fresh=1`: ignore and do not write the save (tests set this implicitly).
    pub fresh: bool,
    /// Every raw parameter, for chapter-specific flags (`?b612_skip=1`).
    pub raw: HashMap<String, String>,
}

impl LaunchParams {
    pub fn from_query(q: &str) -> Self {
        let mut raw = HashMap::new();
        for pair in q.split('&').filter(|s| !s.is_empty()) {
            let (k, v) = pair.split_once('=').unwrap_or((pair, "1"));
            raw.insert(k.to_ascii_lowercase(), v.to_string());
        }
        let flag = |k: &str| raw.get(k).map(|v| v != "0" && v != "false").unwrap_or(false);
        let test = flag("test");
        LaunchParams {
            chapter: raw.get("chapter").cloned().filter(|s| !s.is_empty()),
            test,
            lang: raw.get("lang").cloned(),
            free: flag("free"),
            map: flag("map"),
            ending: flag("ending"),
            touch: raw.get("touch").map(|v| v != "0"),
            quality: raw.get("quality").cloned(),
            speed: raw.get("speed").and_then(|v| v.parse().ok()),
            fresh: flag("fresh") || test,
            raw,
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.raw.get(key).map(|s| s.as_str())
    }
}
