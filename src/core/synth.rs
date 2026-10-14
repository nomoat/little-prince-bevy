//! Procedural audio: every sound key without a file falls back to one of
//! these generators (rendered once to an in-memory WAV). Keeps the game
//! fully playable with zero audio files, and keeps each sound replaceable.

use super::meshkit::Rng;
use std::f32::consts::TAU;

pub const RATE: u32 = 22050;

/// Renders a generator by id. Unknown ids return None.
pub fn render(id: &str) -> Option<Vec<f32>> {
    let s = match id {
        "chime" => chime(&[880.0, 1320.0, 1760.0], 1.6),
        "chime_low" => chime(&[440.0, 660.0, 880.0], 1.8),
        "bell" => bell(523.25, 2.5),
        "click" => click(),
        "tick" => tick(),
        "step" => step(),
        "step_soft" => step_soft(),
        "whoosh" => whoosh(0.9),
        "pop" => pop(),
        "clap" => clap(),
        "sparkle" => arpeggio(&[1046.5, 1318.5, 1568.0, 2093.0], 0.07, 1.2),
        "depart" => arpeggio(&[523.25, 659.25, 783.99, 1046.5, 1318.5], 0.12, 2.2),
        "found" => arpeggio(&[783.99, 1046.5, 1318.5], 0.09, 1.4),
        "banner" => arpeggio(&[659.25, 783.99, 987.77], 0.11, 1.6),
        "water" => water(1.2),
        "creak" => creak(),
        "wind" => wind(12.0),
        "night" => crickets(10.0),
        "pad_title" => pad(&[[57, 60, 64, 69], [53, 57, 60, 65], [55, 59, 62, 67], [52, 55, 60, 64]], 5.0, 0.6, 1),
        "pad_home" => pad(&[[60, 64, 67, 72], [57, 60, 64, 69], [53, 57, 60, 65], [55, 59, 62, 67]], 5.0, 0.55, 2),
        "pad_wonder" => pad(&[[62, 66, 69, 74], [59, 62, 66, 71], [55, 59, 62, 67], [57, 61, 64, 69]], 5.0, 0.5, 3),
        "pad_melancholy" => pad(&[[57, 60, 64, 69], [55, 58, 62, 67], [53, 57, 60, 65], [52, 56, 59, 64]], 6.0, 0.4, 4),
        "pad_playful" => pad(&[[60, 64, 67, 72], [62, 65, 69, 74], [64, 67, 71, 76], [65, 69, 72, 77]], 3.5, 0.75, 5),
        "pad_night" => pad(&[[50, 57, 62, 65], [48, 55, 60, 64], [46, 53, 58, 62], [45, 52, 57, 60]], 6.0, 0.35, 6),
        "pad_desert" => pad(&[[52, 59, 64, 67], [50, 57, 62, 66], [48, 55, 60, 64], [47, 54, 59, 62]], 6.0, 0.45, 7),
        "pad_ending" => pad(&[[60, 64, 67, 71], [57, 60, 64, 67], [53, 57, 60, 64], [55, 59, 62, 65]], 6.5, 0.5, 8),
        _ => return None,
    };
    Some(s)
}

/// 16-bit mono WAV bytes.
pub fn to_wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&RATE.to_le_bytes());
    v.extend_from_slice(&(RATE * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let i = (s.clamp(-1.0, 1.0) * 32000.0) as i16;
        v.extend_from_slice(&i.to_le_bytes());
    }
    v
}

fn n(secs: f32) -> usize {
    (secs * RATE as f32) as usize
}

fn midi(m: i32) -> f32 {
    440.0 * 2f32.powf((m - 69) as f32 / 12.0)
}

fn chime(freqs: &[f32], dur: f32) -> Vec<f32> {
    let len = n(dur);
    (0..len)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let env = (t * 200.0).min(1.0) * (-t * 3.0).exp();
            freqs.iter().enumerate().map(|(k, f)| (TAU * f * t).sin() / (k as f32 + 1.5)).sum::<f32>() * env * 0.45
        })
        .collect()
}

fn bell(f: f32, dur: f32) -> Vec<f32> {
    let len = n(dur);
    let partials = [(1.0, 1.0), (2.01, 0.5), (2.76, 0.35), (5.4, 0.2), (8.9, 0.1)];
    (0..len)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            partials.iter().map(|(m, a)| (TAU * f * m * t).sin() * a * (-t * (1.5 + m * 0.6)).exp()).sum::<f32>() * 0.35 * (t * 300.0).min(1.0)
        })
        .collect()
}

fn click() -> Vec<f32> {
    let mut r = Rng::new(3);
    (0..n(0.05)).map(|i| {
        let t = i as f32 / RATE as f32;
        ((TAU * 1800.0 * t).sin() * 0.5 + (r.f() - 0.5) * 0.4) * (-t * 90.0).exp() * 0.6
    }).collect()
}

fn tick() -> Vec<f32> {
    (0..n(0.04)).map(|i| {
        let t = i as f32 / RATE as f32;
        (TAU * 2600.0 * t).sin() * (-t * 120.0).exp() * 0.4
    }).collect()
}

fn lowpass(x: &mut [f32], a: f32) {
    let mut y = 0.0;
    for s in x.iter_mut() {
        y += a * (*s - y);
        *s = y;
    }
}

fn step() -> Vec<f32> {
    let mut r = Rng::new(11);
    let mut v: Vec<f32> = (0..n(0.12)).map(|i| {
        let t = i as f32 / RATE as f32;
        (r.f() * 2.0 - 1.0) * (-t * 40.0).exp()
    }).collect();
    lowpass(&mut v, 0.12);
    v.iter_mut().for_each(|s| *s *= 1.6);
    v
}

fn step_soft() -> Vec<f32> {
    let mut v = step();
    lowpass(&mut v, 0.3);
    v.iter_mut().for_each(|s| *s *= 0.6);
    v
}

fn whoosh(dur: f32) -> Vec<f32> {
    let mut r = Rng::new(5);
    let len = n(dur);
    let mut v: Vec<f32> = (0..len).map(|i| {
        let t = i as f32 / len as f32;
        (r.f() * 2.0 - 1.0) * (t * std::f32::consts::PI).sin().powf(1.5)
    }).collect();
    lowpass(&mut v, 0.06);
    v.iter_mut().for_each(|s| *s *= 2.2);
    v
}

fn pop() -> Vec<f32> {
    (0..n(0.12)).map(|i| {
        let t = i as f32 / RATE as f32;
        let f = 900.0 - t * 4000.0;
        (TAU * f.max(200.0) * t).sin() * (-t * 35.0).exp() * 0.6
    }).collect()
}

fn clap() -> Vec<f32> {
    let mut r = Rng::new(17);
    let mut v: Vec<f32> = (0..n(0.18)).map(|i| {
        let t = i as f32 / RATE as f32;
        let bursts = [0.0, 0.012, 0.024];
        let env: f32 = bursts.iter().map(|b| if t >= *b { (-(t - b) * 60.0).exp() } else { 0.0 }).sum();
        (r.f() * 2.0 - 1.0) * env * 0.5
    }).collect();
    lowpass(&mut v, 0.5);
    v
}

fn arpeggio(freqs: &[f32], gap: f32, dur: f32) -> Vec<f32> {
    let len = n(dur);
    let mut v = vec![0.0f32; len];
    for (k, f) in freqs.iter().enumerate() {
        let start = n(gap * k as f32);
        for i in start..len {
            let t = (i - start) as f32 / RATE as f32;
            v[i] += ((TAU * f * t).sin() + 0.3 * (TAU * f * 2.0 * t).sin()) * (-t * 3.5).exp() * (t * 300.0).min(1.0) * 0.25;
        }
    }
    v
}

fn water(dur: f32) -> Vec<f32> {
    let mut r = Rng::new(23);
    let len = n(dur);
    let mut v: Vec<f32> = (0..len).map(|i| {
        let t = i as f32 / len as f32;
        let bub = if r.f() > 0.9995 { 1.0 } else { 0.0 };
        (r.f() * 2.0 - 1.0) * 0.4 * (1.0 - t) + bub
    }).collect();
    lowpass(&mut v, 0.25);
    v
}

fn creak() -> Vec<f32> {
    (0..n(0.6)).map(|i| {
        let t = i as f32 / RATE as f32;
        let f = 180.0 + (t * 9.0).sin() * 40.0;
        let saw = ((f * t).fract() * 2.0 - 1.0) * 0.25;
        saw * (t * 20.0).min(1.0) * (1.0 - t / 0.6)
    }).collect()
}

/// Looping wind bed (seamless: cross-faded ends).
fn wind(dur: f32) -> Vec<f32> {
    let mut r = Rng::new(31);
    let len = n(dur);
    let mut v: Vec<f32> = (0..len).map(|_| r.f() * 2.0 - 1.0).collect();
    lowpass(&mut v, 0.02);
    for (i, s) in v.iter_mut().enumerate() {
        let t = i as f32 / RATE as f32;
        *s *= 3.0 * (0.6 + 0.4 * (TAU * t / dur * 2.0).sin());
    }
    loopify(&mut v, n(1.0));
    v
}

fn crickets(dur: f32) -> Vec<f32> {
    let len = n(dur);
    let mut v = wind(dur);
    v.iter_mut().for_each(|s| *s *= 0.4);
    let mut r = Rng::new(41);
    let mut i = 0;
    while i < len {
        let f = r.range(4200.0, 5200.0);
        let chirps = 3 + (r.f() * 3.0) as usize;
        for c in 0..chirps {
            let st = i + c * n(0.06);
            for k in 0..n(0.035) {
                if st + k < len {
                    let t = k as f32 / RATE as f32;
                    v[st + k] += (TAU * f * t).sin() * (t * std::f32::consts::PI / 0.035).sin() * 0.06;
                }
            }
        }
        i += n(r.range(0.5, 1.4));
    }
    v
}

fn loopify(v: &mut [f32], fade: usize) {
    let len = v.len();
    if fade * 2 >= len {
        return;
    }
    for i in 0..fade {
        let k = i as f32 / fade as f32;
        let tail = v[len - fade + i];
        v[i] = v[i] * k + tail * (1.0 - k);
    }
}

/// Soft music loop: a chord progression of warm pads plus a sparse bell
/// arpeggio. `bar` seconds per chord; `motion` 0..1 arpeggio density.
fn pad(chords: &[[i32; 4]], bar: f32, motion: f32, seed: u64) -> Vec<f32> {
    let total = bar * chords.len() as f32;
    let len = n(total);
    let mut v = vec![0.0f32; len];
    let mut r = Rng::new(seed);
    for (ci, ch) in chords.iter().enumerate() {
        let start = n(bar * ci as f32);
        let end = n(bar * (ci as f32 + 1.0) + 1.2).min(len + n(1.2));
        for (k, m) in ch.iter().enumerate() {
            let f = midi(*m - 12);
            let det = 1.0 + (k as f32 - 1.5) * 0.0015;
            for i in start..end {
                let idx = i % len;
                let t = (i - start) as f32 / RATE as f32;
                let env = (t / 1.2).min(1.0) * (1.0 - ((t - bar).max(0.0) / 1.2)).clamp(0.0, 1.0);
                let s = (TAU * f * det * t).sin() * 0.6 + (TAU * f * 2.0 * t).sin() * 0.15 + (TAU * f * 0.5 * t).sin() * 0.2;
                v[idx] += s * env * 0.07;
            }
        }
        // bell arpeggio
        let notes = (bar * 2.0 * motion).round() as usize;
        for a in 0..notes {
            let m = ch[(r.next_u32() as usize) % 4] + 12;
            let f = midi(m);
            let st = start + n(bar / notes.max(1) as f32 * a as f32 + r.range(0.0, 0.08));
            for i in 0..n(1.6) {
                let idx = (st + i) % len;
                let t = i as f32 / RATE as f32;
                v[idx] += ((TAU * f * t).sin() + 0.25 * (TAU * f * 3.0 * t).sin()) * (-t * 2.6).exp() * (t * 400.0).min(1.0) * 0.05;
            }
        }
    }
    // gentle soft-clip
    for s in v.iter_mut() {
        *s = (*s * 1.4).tanh() * 0.8;
    }
    v
}
