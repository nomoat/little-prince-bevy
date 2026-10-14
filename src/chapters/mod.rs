//! The eight planets, in journey order. Each lives in its own directory and
//! implements [`crate::core::chapter::Chapter`]. Chapter authors edit only
//! their own directory plus their manifest `assets/chapters/<id>.ron`.

pub mod b612;
pub mod bar;
pub mod businessman;
pub mod earth;
pub mod geo;
pub mod king;
pub mod lamp;
pub mod vanity;

use crate::core::chapter::{Chapter, ChapterRegistry};
use crate::core::i18n::I18n;
use bevy::prelude::*;
use std::sync::Arc;

/// All chapters in journey order (B612 → king → vanity → bar → businessman →
/// lamplighter → geographer → earth).
pub fn all() -> Vec<Arc<dyn Chapter>> {
    vec![
        Arc::new(b612::B612),
        Arc::new(king::King),
        Arc::new(vanity::Vanity),
        Arc::new(bar::Bar),
        Arc::new(businessman::Businessman),
        Arc::new(lamp::Lamp),
        Arc::new(geo::Geo),
        Arc::new(earth::Earth),
    ]
}

/// Registers every chapter: string tables, app hooks and the registry.
pub fn register(app: &mut App) {
    let chapters = all();
    for c in &chapters {
        app.world_mut().resource_mut::<I18n>().add_table(c.strings());
        c.setup_app(app);
    }
    app.insert_resource(ChapterRegistry(chapters));
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    /// Every string key has both a Chinese and an English text, and every
    /// `"<chapter>.xxx"` literal in a chapter's source exists in a table.
    #[test]
    fn strings_complete() {
        let mut keys = HashSet::new();
        let mut bad = Vec::new();
        let mut tables: Vec<crate::core::i18n::StringTable> = vec![crate::core::i18n::CORE_STRINGS, crate::core::ui::hud::SPEAKER_STRINGS];
        for c in super::all() {
            tables.push(c.strings());
        }
        for t in tables {
            for (k, zh, en) in t.iter() {
                keys.insert(k.to_string());
                if (zh.trim().is_empty() || en.trim().is_empty()) && *k != "core.narrator" {
                    bad.push(format!("empty text for {k}"));
                }
            }
        }
        for c in super::all() {
            let id = c.info().id;
            let dir = format!("{}/src/chapters/{id}", env!("CARGO_MANIFEST_DIR"));
            for f in std::fs::read_dir(&dir).unwrap() {
                let src = std::fs::read_to_string(f.unwrap().path()).unwrap();
                for lit in src.split('"').skip(1).step_by(2) {
                    let is_key = lit.starts_with(&format!("{id}.")) && lit.len() > id.len() + 1 && lit.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '.' || ch == '_');
                    if !is_key {
                        continue;
                    }
                    let rest = &lit[id.len() + 1..];
                    let head = rest.split('.').next().unwrap_or("");
                    let short_id = {
                        let letters = rest.trim_end_matches(|ch: char| ch.is_ascii_digit());
                        letters.len() <= 2 && letters.len() < rest.len() && !rest.contains('.')
                    };
                    let looks_text = short_id || (rest.contains('.') && ["l", "st", "ev", "verb", "target", "tg", "t", "mono", "teach", "line", "say", "hint", "card"].contains(&head));
                    if is_key && looks_text && !keys.contains(lit) && !lit.contains("..") {
                        bad.push(format!("{id}: key \"{lit}\" used but not in a string table"));
                    }
                }
            }
        }
        bad.sort();
        bad.dedup();
        for b in &bad {
            println!("{b}");
        }
        assert!(bad.is_empty(), "{} string problems", bad.len());
    }
}
