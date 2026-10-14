//! Small cinematic helpers shared by the b612 and king chapters (chapter code,
//! not core): camera shots with their own field of view (the original's
//! B612CameraSpec / KingStory shots carry a FOV), smoothed in and out so a
//! cut between two framings never pops back to the default lens.

use crate::core::camera::{CameraRig, CameraTuning, MainCamera, Shot};
use crate::core::player::smooth_damp;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct Director {
    /// Wanted FOV while a chapter shot is active (None = the game camera's FOV).
    pub want: Option<f32>,
    pub cur: f32,
    smooth: f32,
    vel: f32,
}

impl Default for Director {
    fn default() -> Self {
        Director {
            want: None,
            cur: 50.0,
            smooth: 0.5,
            vel: 0.0,
        }
    }
}

/// Resets the lens state (call from `build`).
pub fn reset(world: &mut World) {
    let fov = world.resource::<CameraTuning>().fov;
    world.insert_resource(Director { cur: fov, ..default() });
}

/// Begins a shot: camera position, look-at point, blend seconds (0 = cut), FOV.
pub fn shot(world: &mut World, pos: Vec3, look: Vec3, blend: f32, fov: f32) {
    // A glide that would swing the camera round (or through) the subject
    // becomes a cut.
    let last = world.resource::<CameraRig>().last_pos;
    let (a, b) = (last - look, pos - look);
    let blend = if blend > 0.0 && a.length() > 0.1 && b.length() > 0.1 && a.angle_between(b) > 100f32.to_radians() {
        0.0
    } else {
        blend
    };
    world.resource_mut::<CameraRig>().begin_shot(Shot {
        position: pos,
        look_at: look,
        blend,
        fov: None,
    });
    let mut d = world.resource_mut::<Director>();
    d.want = Some(fov);
    d.smooth = (blend * 0.45).max(0.03);
    if blend <= 0.0 {
        d.cur = fov;
        d.vel = 0.0;
    }
}

/// Ends the shot, blending back to the player camera over `blend` seconds.
pub fn release(world: &mut World, blend: f32) {
    world.resource_mut::<CameraRig>().end_shot(blend);
    let mut d = world.resource_mut::<Director>();
    d.want = None;
    d.smooth = (blend * 0.45).max(0.05);
}

/// Applies the smoothed FOV after the core camera ran (no-op when idle).
pub fn apply_fov(time: Res<Time<Real>>, tuning: Res<CameraTuning>, d: Option<ResMut<Director>>, mut cam: Query<&mut Projection, With<MainCamera>>) {
    let Some(mut d) = d else { return };
    let goal = d.want.unwrap_or(tuning.fov);
    if d.want.is_none() && (d.cur - tuning.fov).abs() < 0.02 {
        d.cur = tuning.fov;
        d.vel = 0.0;
        return;
    }
    let dt = time.delta_secs().clamp(1e-4, 0.1);
    let (cur, smooth) = (d.cur, d.smooth);
    let mut vel = d.vel;
    d.cur = smooth_damp(cur, goal, &mut vel, smooth, 400.0, dt);
    d.vel = vel;
    if let Ok(mut p) = cam.single_mut() {
        if let Projection::Perspective(pp) = &mut *p {
            pp.fov = d.cur.to_radians();
        }
    }
}

/// Point in an actor's local frame (x = the actor's right, y up, z = in front)
/// → world, for an actor at `origin` facing `yaw_deg` (0 = +Z).
pub fn local_to_world(origin: Vec3, yaw_deg: f32, local: Vec3) -> Vec3 {
    origin + Quat::from_rotation_y(yaw_deg.to_radians()) * Vec3::new(-local.x, local.y, local.z)
}
