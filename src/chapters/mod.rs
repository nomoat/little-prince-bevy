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
