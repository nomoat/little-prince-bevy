//! Small helpers shared by the businessman and lamplighter chapters
//! (both written by the same chapter agent): scripted camera shots with a
//! field of view, slow dolly pushes, the original's line pacing, procedural
//! glow textures behind manifest keys, and restoring the camera tuning when
//! the chapter ends (the core does not reset `CameraTuning` between chapters).

use crate::core::camera::CameraTuning;
use crate::core::i18n::StringTable;
use crate::core::manifest::Manifest;
use crate::core::prelude::*;
use crate::core::state::{ActiveChapter, AppState};
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Camera shot with a field of view (the original's B612CameraSpec: name,
/// position, look-at point, fov). `blend` 0 = cut.
pub fn shot_fov(pos: Vec3, look: Vec3, fov: f32, blend: f32) -> Cmd {
    run(move |w: &mut World| {
        w.resource_mut::<Dolly>().0 = None;
        w.resource_mut::<CameraRig>().begin_shot(Shot { position: pos, look_at: look, blend, fov: Some(fov) });
    })
}

/// A slow camera move from the current shot to another one over `secs`
/// (MoveShot in the original: landing push, two-shot push-in).
pub fn dolly_to(pos: Vec3, look: Vec3, fov: f32, secs: f32) -> Cmd {
    run(move |w: &mut World| start_dolly(w, pos, look, fov, secs))
}

pub fn start_dolly(w: &mut World, pos: Vec3, look: Vec3, fov: f32, secs: f32) {
    let (from, look_from, fov_from) = {
        let rig = w.resource::<CameraRig>();
        match &rig.shot {
            Some(s) => (s.position, s.look_at, s.fov.unwrap_or(fov)),
            None => (rig.last_pos, rig.last_look, fov),
        }
    };
    if w.resource::<CameraRig>().shot.is_none() {
        w.resource_mut::<CameraRig>().begin_shot(Shot { position: from, look_at: look_from, blend: 0.0, fov: Some(fov_from) });
    }
    w.resource_mut::<Dolly>().0 = Some(DollyAnim { from, to: pos, look_from, look_to: look, fov_from, fov_to: fov, t: 0.0, dur: secs.max(0.05) });
}

#[derive(Clone, Debug)]
pub struct DollyAnim {
    pub from: Vec3,
    pub to: Vec3,
    pub look_from: Vec3,
    pub look_to: Vec3,
    pub fov_from: f32,
    pub fov_to: f32,
    pub t: f32,
    pub dur: f32,
}

#[derive(Resource, Default)]
pub struct Dolly(pub Option<DollyAnim>);

/// Advances the dolly (registered once; harmless when idle).
pub fn update_dolly(time: Res<Time>, mut dolly: ResMut<Dolly>, mut rig: ResMut<CameraRig>) {
    let Some(d) = dolly.0.as_mut() else { return };
    let Some(shot) = rig.shot.as_mut() else {
        dolly.0 = None;
        return;
    };
    d.t = (d.t + time.delta_secs() / d.dur).min(1.0);
    let k = ease_in_out(d.t);
    shot.position = d.from.lerp(d.to, k);
    shot.look_at = d.look_from.lerp(d.look_to, k);
    shot.fov = Some(d.fov_from + (d.fov_to - d.fov_from) * k);
    if d.t >= 1.0 {
        dolly.0 = None;
    }
}

pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The original's line pacing: a line stays `0.18 s × chars + 1.2 s`
/// (at least 2.6 s); a click advances sooner. Returns a self-advancing line.
pub fn talk(table: StringTable, who: Speaker, key: &'static str) -> Cmd {
    let chars = table.iter().find(|e| e.0 == key).map(|e| e.1.chars().count()).unwrap_or(12) as f32;
    let total = (0.18 * chars + 1.2).max(2.6);
    // the HUD counts the hold after the typewriter reveal (0.04 s per CJK glyph)
    let hold = (total - 0.04 * chars).max(1.2);
    say_for(who, key, hold)
}

/// Camera tuning saved when one of our chapters changes it; restored as
/// soon as no chapter of ours is running.
#[derive(Resource, Default)]
pub struct SavedCamTuning(pub Option<CameraTuning>);

pub fn save_cam_tuning(w: &mut World) {
    if w.resource::<SavedCamTuning>().0.is_none() {
        let t = w.resource::<CameraTuning>().clone();
        w.resource_mut::<SavedCamTuning>().0 = Some(t);
    }
}

pub fn restore_cam_tuning(
    mut saved: ResMut<SavedCamTuning>,
    mut tuning: ResMut<CameraTuning>,
    active: Option<Res<ActiveChapter>>,
    state: Res<State<AppState>>,
) {
    if saved.0.is_none() {
        return;
    }
    let ours = *state.get() == AppState::Chapter
        && active.and_then(|a| a.id.clone()).map(|id| id == "businessman" || id == "lamp").unwrap_or(false);
    if !ours {
        if let Some(t) = saved.0.take() {
            *tuning = t;
        }
    }
}

/// A square RGBA texture: either the file a manifest key points at, or a
/// procedural glow ("glow": soft radial falloff, "flare": four-point star,
/// "dot": small hard disc) generated in code like the original's
/// TelescopeSky dot/flare textures.
pub fn texture(w: &mut World, key: &str, procedural: &str) -> Handle<Image> {
    let path = w.resource::<Manifest>().path(key).map(|s| s.to_string());
    if let Some(p) = path {
        return w.resource::<AssetServer>().load(p);
    }
    let n = 128u32;
    let mut data = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let u = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let r = (u * u + v * v).sqrt();
            let a = match procedural {
                "flare" => {
                    let arm = |a: f32, b: f32| (1.0 - a.abs()).max(0.0).powf(3.0) * (-(b * b) * 900.0).exp();
                    let core = (-(r * r) * 60.0).exp();
                    (arm(u, v) + arm(v, u) + core).min(1.0)
                }
                "dot" => 1.0 - crate::core::planet::smoothstep(0.55, 0.8, r),
                _ => {
                    let g = (1.0 - r).max(0.0);
                    (g * g * 0.85 + (-(r * r) * 40.0).exp() * 0.6).min(1.0)
                }
            };
            let a8 = (a.clamp(0.0, 1.0) * 255.0) as u8;
            data.extend([255, 255, 255, a8]);
        }
    }
    let img = Image::new(
        Extent3d { width: n, height: n, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    w.resource_mut::<Assets<Image>>().add(img)
}

/// A camera-facing quad mesh (unit size, in XY, facing +Z).
pub fn quad_mesh() -> Mesh {
    Mesh::from(Rectangle::new(1.0, 1.0))
}

/// Rotation that makes a +Z-facing quad at `pos` face the point `eye`.
pub fn face_toward(pos: Vec3, eye: Vec3) -> Quat {
    let d = (eye - pos).normalize_or(Vec3::Z);
    Transform::from_translation(pos).looking_to(-d, Vec3::Y).rotation
}

/// Direction from a game azimuth (0 = +Z, 90 = +X) and elevation, degrees.
pub fn dir_from(az_deg: f32, el_deg: f32) -> Vec3 {
    let (a, e) = (az_deg.to_radians(), el_deg.to_radians());
    Vec3::new(a.sin() * e.cos(), e.sin(), a.cos() * e.cos())
}

/// Yaw (deg, 0 = +Z, 90 = +X) of a direction.
pub fn yaw_of(d: Vec3) -> f32 {
    d.x.atan2(d.z).to_degrees()
}

/// Rotates a local XZ offset by a yaw (deg) — same convention as entity yaws.
pub fn rot_y(yaw_deg: f32, v: Vec3) -> Vec3 {
    Quat::from_rotation_y(yaw_deg.to_radians()) * v
}

/// Hides the prince when a close camera (a scripted shot, a tight room)
/// would be inside his head — the original camera's hide/show with hysteresis.
pub fn hide_prince_near_camera(
    cam: Query<&GlobalTransform, With<crate::core::camera::MainCamera>>,
    player: Query<&Transform, With<Player>>,
    mut vis: Query<&mut Visibility, With<crate::core::player::PlayerVisual>>,
) {
    let (Ok(c), Ok(p)) = (cam.single(), player.single()) else { return };
    let head = p.translation + Vec3::Y * 1.0;
    let d = c.translation().distance(head);
    for mut v in &mut vis {
        let hidden = *v == Visibility::Hidden;
        if !hidden && d < 0.5 {
            *v = Visibility::Hidden;
        } else if hidden && d > 0.65 {
            *v = Visibility::Inherited;
        }
    }
}
