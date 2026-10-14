//! Manifest-driven audio: music (one track, cross-faded), an ambience bed and
//! one-shot effects. A key resolves to a file (`path`/`variants`) or to a
//! synthesized fallback (`procedural: "pad_home"`, see synth.rs).
//!
//! From any system: `audio.write(AudioCmd::sfx("sfx.chime"))`, or story
//! commands `Cmd::Music("music.b612")` / `Cmd::Sfx(..)`.

use super::manifest::Manifest;
use super::meshkit::Rng;
use super::player::Player;
use super::save::SaveData;
use super::synth;
use bevy::audio::{AudioSinkPlayback, PlaybackMode, Volume};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

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
    synth_cache: HashMap<String, Handle<AudioSource>>,
    rng: Option<Rng>,
    warned: std::collections::HashSet<String>,
}

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

fn bus_volume(save: &SaveData, bus: Bus) -> f32 {
    let s = &save.settings;
    s.master
        * match bus {
            Bus::Music => s.music,
            Bus::Ambience => s.sfx * 0.8,
            Bus::Sfx => s.sfx,
        }
}

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

/// Footsteps from the player's stride.
pub(crate) fn footsteps(q: Query<&Player>, mut out: MessageWriter<AudioCmd>) {
    for p in &q {
        if p.footstep && p.grounded {
            let v = (p.planar_speed / 3.5).clamp(0.25, 0.8);
            out.write(AudioCmd::Sfx("sfx.step".into(), v));
        }
    }
}
