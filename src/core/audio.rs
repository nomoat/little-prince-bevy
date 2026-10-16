//! Manifest-driven audio: music (one track, cross-faded), an ambience bed and
//! one-shot effects. A key resolves to a file (`path`/`variants`) or to a
//! synthesized fallback (`procedural: "pad_home"`, see synth.rs).
//!
//! From any system: `audio.write(AudioCmd::sfx("sfx.chime"))`, or story
//! commands `Cmd::Music("music.b612")` / `Cmd::Sfx(..)`.

#[cfg(not(target_arch = "wasm32"))]
use super::manifest::Manifest;
use super::meshkit::Rng;
use super::player::Player;
#[cfg(not(target_arch = "wasm32"))]
use super::save::SaveData;
#[cfg(not(target_arch = "wasm32"))]
use super::synth;
#[cfg(not(target_arch = "wasm32"))]
use bevy::audio::{AudioSinkPlayback, PlaybackMode, Volume};
use bevy::prelude::*;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
pub(crate) use super::web_audio::{handle_audio, update_tracks};

#[derive(Message, Clone, Debug)]
pub enum AudioCmd {
    /// One-shot effect by key, with a volume multiplier.
    Sfx(String, f32),
    /// Switch the music (None = fade out).
    Music(Option<String>),
    /// Switch the ambience bed (None = fade out).
    Ambience(Option<String>),
}

impl AudioCmd {
    pub fn sfx(key: &str) -> Self {
        AudioCmd::Sfx(key.to_string(), 1.0)
    }
    pub fn music(key: &str) -> Self {
        AudioCmd::Music(Some(key.to_string()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bus {
    Music,
    Ambience,
    Sfx,
}

/// A playing looped track (music/ambience) with a fader.
#[derive(Component)]
pub struct Track {
    pub bus: Bus,
    pub key: String,
    pub base: f32,
    pub fade: f32,
    pub target: f32,
    pub fade_speed: f32,
}

#[derive(Resource, Default)]
pub struct AudioState {
    pub music: Option<String>,
    pub ambience: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    synth_cache: HashMap<String, Handle<AudioSource>>,
    pub(super) rng: Option<Rng>,
    #[cfg(target_arch = "wasm32")]
    pub(super) web_synths: HashMap<String, Vec<u8>>,
    pub(super) warned: std::collections::HashSet<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve(
    key: &str,
    manifest: &Manifest,
    state: &mut AudioState,
    server: &AssetServer,
    sources: &mut Assets<AudioSource>,
) -> Option<(Handle<AudioSource>, f32)> {
    let entry = manifest.get(key);
    let vol = entry.map(|e| e.volume).unwrap_or(1.0);
    if let Some(e) = entry {
        if !e.variants.is_empty() {
            let rng = state.rng.get_or_insert(Rng::new(99));
            let i = (rng.next_u32() as usize) % e.variants.len();
            return Some((server.load(e.variants[i].clone()), vol));
        }
        if let Some(p) = &e.path {
            return Some((server.load(p.clone()), vol));
        }
    }
    let gen_id = entry.and_then(|e| e.procedural.clone()).unwrap_or_else(|| key.trim_start_matches("synth.").to_string());
    if let Some(h) = state.synth_cache.get(&gen_id) {
        return Some((h.clone(), vol));
    }
    match synth::render(&gen_id) {
        Some(samples) => {
            let bytes: Arc<[u8]> = synth::to_wav(&samples).into();
            let h = sources.add(AudioSource { bytes });
            state.synth_cache.insert(gen_id, h.clone());
            Some((h, vol))
        }
        None => {
            if state.warned.insert(key.to_string()) {
                warn!("audio '{key}': no file and no synth '{gen_id}'");
            }
            None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn bus_volume(save: &SaveData, bus: Bus) -> f32 {
    let s = &save.settings;
    s.master
        * match bus {
            Bus::Music => s.music,
            Bus::Ambience => s.sfx * 0.8,
            Bus::Sfx => s.sfx,
        }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_audio(
    mut commands: Commands,
    mut cmds: MessageReader<AudioCmd>,
    manifest: Res<Manifest>,
    mut state: ResMut<AudioState>,
    server: Res<AssetServer>,
    mut sources: ResMut<Assets<AudioSource>>,
    save: Res<SaveData>,
    mut tracks: Query<(Entity, &mut Track)>,
) {
    for cmd in cmds.read() {
        match cmd {
            AudioCmd::Sfx(key, v) => {
                if let Some((h, vol)) = resolve(key, &manifest, &mut state, &server, &mut sources) {
                    let volume = vol * v * bus_volume(&save, Bus::Sfx);
                    if volume > 0.001 {
                        commands.spawn((AudioPlayer::new(h), PlaybackSettings { mode: PlaybackMode::Despawn, volume: Volume::Linear(volume), ..default() }));
                    }
                }
            }
            AudioCmd::Music(key) | AudioCmd::Ambience(key) => {
                let bus = if matches!(cmd, AudioCmd::Music(_)) { Bus::Music } else { Bus::Ambience };
                let current = if bus == Bus::Music { &mut state.music } else { &mut state.ambience };
                if current.as_ref() == key.as_ref() {
                    continue;
                }
                *current = key.clone();
                for (_, mut t) in &mut tracks {
                    if t.bus == bus {
                        t.target = 0.0;
                        t.fade_speed = 0.6;
                    }
                }
                if let Some(k) = key {
                    if let Some((h, vol)) = resolve(k, &manifest, &mut state, &server, &mut sources) {
                        commands.spawn((
                            AudioPlayer::new(h),
                            PlaybackSettings { mode: PlaybackMode::Loop, volume: Volume::Linear(0.0001), ..default() },
                            Track { bus, key: k.clone(), base: vol, fade: 0.0, target: 1.0, fade_speed: 0.5 },
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn update_tracks(
    mut commands: Commands,
    time: Res<Time<Real>>,
    save: Res<SaveData>,
    mut q: Query<(Entity, &mut Track, Option<&mut AudioSink>)>,
) {
    let dt = time.delta_secs();
    for (e, mut t, sink) in &mut q {
        let step = t.fade_speed * dt;
        t.fade = if t.fade < t.target { (t.fade + step).min(t.target) } else { (t.fade - step).max(t.target) };
        if let Some(mut s) = sink {
            let v = (t.base * t.fade * bus_volume(&save, t.bus)).max(0.0);
            s.set_volume(Volume::Linear(v));
        }
        if t.target == 0.0 && t.fade <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

/// Per-surface footstep zones (boxes: centre, half extents, sfx key), set by
/// chapters with [`super::chapter::ChapterCx::step_zone`]. Outside every zone
/// the default `sfx.step` plays.
#[derive(Resource, Default, Clone)]
pub struct StepZones(pub Vec<(Vec3, Vec3, String)>);

/// Footsteps from the player's stride.
pub(crate) fn footsteps(q: Query<(&Player, &Transform)>, zones: Option<Res<StepZones>>, mut out: MessageWriter<AudioCmd>) {
    for (p, tf) in &q {
        if p.footstep && p.grounded {
            let v = (p.planar_speed / 3.5).clamp(0.25, 0.8);
            let pos = tf.translation;
            let key = zones
                .as_ref()
                .and_then(|z| z.0.iter().find(|(c, h, _)| (pos - *c).abs().cmple(*h).all()).map(|z| z.2.clone()))
                .unwrap_or_else(|| "sfx.step".into());
            out.write(AudioCmd::Sfx(key, v));
        }
    }
}

/// Small UI sounds so every player action is heard: a soft click when an
/// interaction fires, a tick when a dialogue line is advanced by hand, and a
/// low chime when the objective changes.
pub(crate) fn feedback(
    mut used: MessageReader<super::interact::Interacted>,
    input: Res<super::input::InputFrame>,
    hud: Res<super::ui::hud::Hud>,
    mut last_obj: Local<Option<(Option<&'static str>, Option<&'static str>)>>,
    mut out: MessageWriter<AudioCmd>,
) {
    if used.read().count() > 0 {
        out.write(AudioCmd::Sfx("sfx.select".into(), 0.35));
    }
    if input.advance && hud.dialogue.is_some() {
        out.write(AudioCmd::Sfx("sfx.tick".into(), 0.22));
    }
    let obj = hud.objective.as_ref().map(|o| (o.0, o.1));
    if obj != *last_obj {
        if let Some((_, Some(_))) = obj {
            if last_obj.is_some() {
                out.write(AudioCmd::Sfx("sfx.chime_low".into(), 0.3));
            }
        }
        *last_obj = obj;
    }
}
