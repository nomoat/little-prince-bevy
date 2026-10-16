//! Browser audio uses complete Web Audio buffers instead of CPAL's main-thread
//! sample-refill callbacks. Rendering stalls cannot starve a running music loop.
use super::{
    audio::{AudioCmd, AudioState},
    manifest::Manifest,
    meshkit::Rng,
    save::SaveData,
    synth,
};
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/web/audio.js")]
extern "C" {
    #[wasm_bindgen(js_name = playTrack)]
    fn play_track(bus: &str, key: &str, url: &str, volume: f32, bytes: &[u8]);
    #[wasm_bindgen(js_name = playEffect)]
    fn play_effect(url: &str, volume: f32, bytes: &[u8]);
    #[wasm_bindgen(js_name = setVolumes)]
    fn set_volumes(master: f32, music: f32, sfx: f32);
}

fn source(
    key: &str,
    manifest: &Manifest,
    state: &mut AudioState,
) -> Option<(String, f32, Vec<u8>)> {
    let entry = manifest.get(key);
    let volume = entry.map(|e| e.volume).unwrap_or(1.0);
    if let Some(e) = entry {
        let path = if !e.variants.is_empty() {
            let index =
                state.rng.get_or_insert(Rng::new(99)).next_u32() as usize % e.variants.len();
            Some(&e.variants[index])
        } else {
            e.path.as_ref()
        };
        if let Some(path) = path {
            return Some((format!("assets/{path}"), volume, Vec::new()));
        }
    }
    let id = entry
        .and_then(|e| e.procedural.as_deref())
        .unwrap_or(key.trim_start_matches("synth."));
    if !state.web_synths.contains_key(id) {
        if let Some(samples) = synth::render(id) {
            state
                .web_synths
                .insert(id.to_string(), synth::to_wav(&samples));
        } else {
            if state.warned.insert(key.to_string()) {
                warn!("audio '{key}': no file and no synth '{id}'");
            }
            return None;
        }
    }
    Some((format!("synth:{id}"), volume, state.web_synths[id].clone()))
}

pub(crate) fn handle_audio(
    mut cmds: MessageReader<AudioCmd>,
    manifest: Res<Manifest>,
    mut state: ResMut<AudioState>,
) {
    for cmd in cmds.read() {
        match cmd {
            AudioCmd::Sfx(key, multiplier) => {
                if let Some((url, volume, bytes)) = source(key, &manifest, &mut state) {
                    play_effect(&url, volume * multiplier, &bytes);
                }
            }
            AudioCmd::Music(key) | AudioCmd::Ambience(key) => {
                let music = matches!(cmd, AudioCmd::Music(_));
                let current = if music {
                    &mut state.music
                } else {
                    &mut state.ambience
                };
                if current == key {
                    continue;
                }
                *current = key.clone();
                let bus = if music { "music" } else { "ambience" };
                if let Some(key) = key {
                    if let Some((url, volume, bytes)) = source(key, &manifest, &mut state) {
                        play_track(bus, key, &url, volume, &bytes);
                    }
                } else {
                    play_track(bus, "", "", 0.0, &[]);
                }
            }
        }
    }
}

pub(crate) fn update_tracks(save: Res<SaveData>, mut last: Local<Option<[f32; 3]>>) {
    let s = &save.settings;
    let volumes = [s.master, s.music, s.sfx];
    if *last != Some(volumes) {
        set_volumes(s.master, s.music, s.sfx);
        *last = Some(volumes);
    }
}
