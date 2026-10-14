//! 小王子：星辰之旅 — a code-driven Bevy rebuild of the original's design.
//! `core` is the engine (planet, controller, camera, story, UI, audio, save,
//! journey flow, test harness); `chapters` holds the eight planets.

pub mod chapters;
pub mod core;

use bevy::prelude::*;

/// Builds and runs the game.
pub fn run() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    let mut app = App::new();
    core::build_app(&mut app);
    chapters::register(&mut app);
    app.run();
}
