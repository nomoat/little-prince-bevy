//! Third-person orbit camera (ThirdPersonCamera in the original): pivot at the
//! child's eye height, orbit yaw/pitch, smoothed zoom and follow, pulled in by
//! obstacles, kept above the ground. Scripted "shots" (BeginShot/CutTo/EndShot)
//! override it for story beats.

use super::input::InputFrame;
use super::physics::ColliderWorld;
use super::planet::Terrain;
use super::player::{Player, smooth_damp};
use bevy::prelude::*;

#[derive(Resource, Clone, Debug)]
pub struct CameraTuning {
    pub pivot_height: f32,
    pub default_distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub default_pitch: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub zoom_smooth_time: f32,
    pub follow_smooth_xz: f32,
    pub follow_smooth_y: f32,
    pub max_follow_lag: f32,
    pub collision_radius: f32,
    pub collision_recover_time: f32,
    pub ground_clearance: f32,
    pub fov: f32,
}

impl Default for CameraTuning {
    fn default() -> Self {
        CameraTuning {
            pivot_height: 1.05,
            default_distance: 4.6,
            min_distance: 1.6,
            max_distance: 9.0,
            default_pitch: 14.0,
            min_pitch: -20.0,
            max_pitch: 70.0,
            zoom_smooth_time: 0.12,
            follow_smooth_xz: 0.06,
            follow_smooth_y: 0.16,
            max_follow_lag: 1.2,
            collision_radius: 0.22,
            collision_recover_time: 0.35,
            ground_clearance: 0.35,
            fov: 50.0,
        }
    }
}

/// A scripted camera shot.
#[derive(Clone, Debug)]
pub struct Shot {
    pub position: Vec3,
    pub look_at: Vec3,
    /// Blend-in seconds (0 = cut).
    pub blend: f32,
    pub fov: Option<f32>,
}

/// Orbit state (yaw 0 = camera looking toward +Z).
#[derive(Resource, Clone, Debug)]
pub struct CameraRig {
    pub yaw: f32,
    pub pitch: f32,
    pub target_distance: f32,
    pub distance: f32,
    zoom_vel: f32,
    recover_vel: f32,
    pub pivot: Vec3,
    pivot_vel: Vec3,
    initialised: bool,
    /// Active scripted shot (None = follow the player).
    pub shot: Option<Shot>,
    shot_t: f32,
    shot_from: (Vec3, Vec3),
    /// Returning from a shot: blend factor 1→0.
    release_t: f32,
    pub last_pos: Vec3,
    pub last_look: Vec3,
    /// Orbit target override (title screen etc.): look at this point instead of the player.
    pub focus_override: Option<Vec3>,
}

impl Default for CameraRig {
    fn default() -> Self {
        CameraRig {
            yaw: 0.0,
            pitch: 14f32.to_radians(),
            target_distance: 4.6,
            distance: 4.6,
            zoom_vel: 0.0,
            recover_vel: 0.0,
            pivot: Vec3::ZERO,
            pivot_vel: Vec3::ZERO,
            initialised: false,
            shot: None,
            shot_t: 0.0,
            shot_from: (Vec3::ZERO, Vec3::ZERO),
            release_t: 0.0,
            last_pos: Vec3::new(0.0, 3.0, -6.0),
            last_look: Vec3::ZERO,
            focus_override: None,
        }
    }
}

impl CameraRig {
    /// Puts the camera behind a facing yaw (radians) and snaps next frame.
    pub fn snap_behind(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.initialised = false;
    }
    pub fn begin_shot(&mut self, shot: Shot) {
        self.shot_from = (self.last_pos, self.last_look);
        self.shot_t = if shot.blend <= 0.0 { 1.0 } else { 0.0 };
        self.shot = Some(shot);
    }
    pub fn end_shot(&mut self, blend: f32) {
        if self.shot.is_some() {
            self.shot_from = (self.last_pos, self.last_look);
            self.shot = None;
            self.release_t = if blend > 0.0 { 1.0 } else { 0.0 };
            self.shot_t = blend.max(0.01);
        }
    }
    pub fn reset(&mut self, tuning: &CameraTuning) {
        *self = CameraRig { pitch: tuning.default_pitch.to_radians(), target_distance: tuning.default_distance, distance: tuning.default_distance, ..default() };
    }
}

/// The one game camera.
#[derive(Component)]
pub struct MainCamera;

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_camera(
    time: Res<Time<Real>>,
    input: Res<InputFrame>,
    tuning: Res<CameraTuning>,
    mut rig: ResMut<CameraRig>,
    terrain: Option<Res<Terrain>>,
    cols: Res<ColliderWorld>,
    player: Query<(&Transform, &Player), Without<MainCamera>>,
    mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    let Ok((mut ctf, mut proj)) = cam.single_mut() else { return };
    let dt = time.delta_secs().clamp(1e-4, 0.1);
    let tu = &*tuning;
    let target = match (rig.focus_override, player.single()) {
        (Some(p), _) => Some(p),
        (None, Ok((ptf, _))) => Some(ptf.translation),
        _ => None,
    };
    // ---- orbit input
    rig.yaw -= input.look.x.to_radians();
    rig.pitch = (rig.pitch + input.look.y.to_radians()).clamp(tu.min_pitch.to_radians(), tu.max_pitch.to_radians());
    rig.target_distance = (rig.target_distance + input.zoom).clamp(tu.min_distance, tu.max_distance);
    let (mut desired_pos, mut look) = (rig.last_pos, rig.last_look);
    if let Some(tp) = target {
        let pivot_goal = tp + Vec3::Y * tu.pivot_height;
        if !rig.initialised {
            rig.pivot = pivot_goal;
            rig.pivot_vel = Vec3::ZERO;
            rig.distance = rig.target_distance;
            rig.initialised = true;
        }
        // follow smoothing (separate XZ / Y), with max lag
        let mut pv = rig.pivot_vel;
        let p = rig.pivot;
        let nx = smooth_damp(p.x, pivot_goal.x, &mut pv.x, tu.follow_smooth_xz, 100.0, dt);
        let nz = smooth_damp(p.z, pivot_goal.z, &mut pv.z, tu.follow_smooth_xz, 100.0, dt);
        let ny = smooth_damp(p.y, pivot_goal.y, &mut pv.y, tu.follow_smooth_y, 100.0, dt);
        rig.pivot_vel = pv;
        let mut np = Vec3::new(nx, ny, nz);
        let lag = np - pivot_goal;
        if lag.length() > tu.max_follow_lag {
            np = pivot_goal + lag.normalize() * tu.max_follow_lag;
        }
        rig.pivot = np;
        // zoom smoothing
        let mut zv = rig.zoom_vel;
        let want = rig.target_distance;
        let zoom = smooth_damp(rig.distance, want, &mut zv, tu.zoom_smooth_time, 100.0, dt);
        rig.zoom_vel = zv;
        // direction from pivot to camera
        let fwd = Vec3::new(rig.yaw.sin() * rig.pitch.cos(), -rig.pitch.sin(), rig.yaw.cos() * rig.pitch.cos());
        let back = -fwd;
        // obstacle pull-in
        let mut dist = zoom;
        if let Some(hit) = cols.ray_camera(rig.pivot, back, zoom, tu.collision_radius) {
            dist = dist.min(hit.max(tu.min_distance * 0.5));
        }
        // terrain clearance: march and lift
        let mut pos = rig.pivot + back * dist;
        if let Some(t) = terrain.as_deref() {
            let g = t.height(pos.x, pos.z) + tu.ground_clearance;
            if pos.y < g {
                pos.y = g;
            }
        }
        // recover smoothly when the obstacle goes away
        if dist < rig.distance {
            rig.distance = dist;
            rig.recover_vel = 0.0;
        } else {
            let mut rv = rig.recover_vel;
            rig.distance = smooth_damp(rig.distance, dist, &mut rv, tu.collision_recover_time, 100.0, dt);
            rig.recover_vel = rv;
            pos = rig.pivot + back * rig.distance;
            if let Some(t) = terrain.as_deref() {
                pos.y = pos.y.max(t.height(pos.x, pos.z) + tu.ground_clearance);
            }
        }
        desired_pos = pos;
        look = rig.pivot;
    }
    // ---- shots
    let mut fov = tu.fov;
    if let Some(shot) = rig.shot.clone() {
        rig.shot_t = (rig.shot_t + dt / shot.blend.max(0.01)).min(1.0);
        let k = ease(rig.shot_t);
        look = rig.shot_from.1.lerp(shot.look_at, k);
        desired_pos = orbit_blend(rig.shot_from, (shot.position, shot.look_at), k, look);
        if let Some(f) = shot.fov {
            fov = tu.fov + (f - tu.fov) * k;
        }
    } else if rig.release_t > 0.0 {
        rig.release_t = (rig.release_t - dt / rig.shot_t.max(0.01)).max(0.0);
        let k = ease(rig.release_t);
        let to = (desired_pos, look);
        look = look.lerp(rig.shot_from.1, k);
        desired_pos = orbit_blend(to, rig.shot_from, k, look);
    }
    rig.last_pos = desired_pos;
    rig.last_look = look;
    *ctf = Transform::from_translation(desired_pos).looking_at(look, Vec3::Y);
    if let Projection::Perspective(p) = &mut *proj {
        p.fov = fov.to_radians();
    }
}

/// Blends two camera placements (position, look-at) by orbiting around the
/// blended look point (yaw / pitch / distance), so a blend from the front of the
/// prince to behind him swings around him instead of passing through his head.
fn orbit_blend(a: (Vec3, Vec3), b: (Vec3, Vec3), k: f32, look: Vec3) -> Vec3 {
    let (oa, ob) = (a.0 - a.1, b.0 - b.1);
    let (da, db) = (oa.length(), ob.length());
    if da < 1e-3 || db < 1e-3 {
        return a.0.lerp(b.0, k);
    }
    let sph = |o: Vec3, d: f32| (o.x.atan2(o.z), (o.y / d).clamp(-1.0, 1.0).asin());
    let (ya, pa) = sph(oa, da);
    let (yb, pb) = sph(ob, db);
    let mut dy = (yb - ya).rem_euclid(std::f32::consts::TAU);
    if dy > std::f32::consts::PI {
        dy -= std::f32::consts::TAU;
    }
    let (y, p, d) = (ya + dy * k, pa + (pb - pa) * k, da + (db - da) * k);
    look + Vec3::new(y.sin() * p.cos(), p.sin(), y.cos() * p.cos()) * d
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
