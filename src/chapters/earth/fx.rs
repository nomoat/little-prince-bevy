//! Small chapter mechanics: the prince's head/upper-body look (star, flower,
//! crying, the glance back at the garden), guide signs, footprints in the
//! sand, the stardust trail, the dunes' dusk ramp + evening star, the train
//! arrival slide and the ending camera.

use super::state::*;
use super::world::{self, D_DOOR, STAR_AZ, STAR_EL};
use crate::core::camera::{CameraRig, MainCamera, Shot};
use crate::core::player::PrinceRig;
use crate::core::prelude::*;

// --------------------------------------------------------- prince look ----

/// Extra head pitch / head yaw / upper-body yaw for the procedural prince (degrees).
#[derive(Resource, Default, Debug)]
pub struct PrinceLook {
    pub pitch: f32,
    pub head_yaw: f32,
    pub body_yaw: f32,
    /// Degrees per second toward the targets.
    pub rate: f32,
    cur: Vec3,
    /// Seconds left in a glance (then everything returns to 0).
    pub glance: f32,
}

pub fn look(world: &mut World, pitch: f32, head_yaw: f32, body_yaw: f32, rate: f32) {
    let mut l = world.resource_mut::<PrinceLook>();
    l.pitch = pitch;
    l.head_yaw = head_yaw;
    l.body_yaw = body_yaw;
    l.rate = rate;
    l.glance = 0.0;
}

/// Turns head + upper body toward `at` for `secs` (head ≤ 60°, total ≤ 95°, feet still).
pub fn glance(world: &mut World, at: Vec3, secs: f32) {
    let (pos, yaw) = {
        let mut q = world.query::<(&Transform, &Player)>();
        match q.iter(world).next() {
            Some((t, p)) => (t.translation, p.yaw),
            None => return,
        }
    };
    let d = at - pos;
    let mut rel = (d.x.atan2(d.z) - yaw).to_degrees().rem_euclid(360.0);
    if rel > 180.0 {
        rel -= 360.0;
    }
    let total = rel.clamp(-95.0, 95.0);
    let head = total.clamp(-60.0, 60.0);
    let mut l = world.resource_mut::<PrinceLook>();
    l.head_yaw = head;
    l.body_yaw = total - head;
    l.pitch = 0.0;
    l.rate = 110.0;
    l.glance = secs;
}

pub fn prince_look(time: Res<Time>, mut look: ResMut<PrinceLook>, q: Query<&PrinceRig, With<Player>>, mut tfs: Query<&mut Transform>) {
    let dt = time.delta_secs();
    if look.glance > 0.0 {
        look.glance -= dt;
        if look.glance <= 0.0 {
            look.head_yaw = 0.0;
            look.body_yaw = 0.0;
            look.rate = 110.0;
        }
    }
    let target = Vec3::new(look.pitch, look.head_yaw, look.body_yaw);
    let rate = look.rate.max(10.0) * dt;
    let d = target - look.cur;
    look.cur = if d.length() <= rate { target } else { look.cur + d.normalize() * rate };
    if look.cur.length() < 0.01 && target.length() < 0.01 {
        return;
    }
    let Ok(rig) = q.single() else { return };
    let c = look.cur;
    if let Ok(mut t) = tfs.get_mut(rig.head) {
        t.rotation = Quat::from_rotation_y(c.y.to_radians()) * Quat::from_rotation_x(-c.x.to_radians());
    }
    if let Ok(mut t) = tfs.get_mut(rig.body) {
        t.rotation = Quat::from_rotation_y(c.z.to_radians());
    }
}

// --------------------------------------------------------- guide signs ----

/// The goal marker lights for 4 s when a leg starts and near each of its signs (6 m).
pub fn guide_signs(mut st: ResMut<EarthState>, mut hud: ResMut<Hud>, player: Query<&Transform, With<Player>>) {
    if st.scene != Scene::Valley {
        return;
    }
    if st.leg != st.leg_shown {
        st.leg_shown = st.leg;
        if st.leg != Leg::None {
            hud.flash_marker(4.0);
        }
    }
    let Ok(p) = player.single() else { return };
    let pp = p.translation.xz();
    let leg = st.leg;
    for s in st.signs.iter_mut() {
        if s.leg == leg && !s.fired && s.pos.distance(pp) < 6.0 {
            s.fired = true;
            hud.flash_marker(4.0);
        }
    }
}

// ---------------------------------------------------------- footprints ----

fn place_print(st: &mut EarthState, terrain: &Terrain, tfs: &mut Query<(&mut Transform, &mut Visibility), (Without<Player>, Without<super::actors::Pilot>)>, at: Vec2, yaw: f32, left: bool) {
    if st.footprints.is_empty() {
        return;
    }
    let side = Vec2::new(yaw.cos(), -yaw.sin()) * if left { 0.06 } else { -0.06 };
    let p = at + side;
    let n = terrain.normal(p.x, p.y);
    let e = st.footprints[st.fp_next % st.footprints.len()];
    st.fp_next += 1;
    if let Ok((mut t, mut v)) = tfs.get_mut(e) {
        t.translation = terrain.ground(p.x, p.y) + n * 0.012;
        t.rotation = Quat::from_rotation_arc(Vec3::Y, n) * Quat::from_rotation_y(yaw);
        *v = Visibility::Visible;
    }
}

/// DesertFootprints: prints ≥ 0.22 m apart for the prince, ≥ 0.30 m for the aviator (sand only).
pub fn footprints(
    mut st: ResMut<EarthState>,
    terrain: Option<Res<Terrain>>,
    player: Query<(&Transform, &Player)>,
    pilot: Query<(&Transform, &super::actors::Pilot), Without<Player>>,
    mut tfs: Query<(&mut Transform, &mut Visibility), (Without<Player>, Without<super::actors::Pilot>)>,
) {
    if st.scene != Scene::Dunes {
        return;
    }
    let Some(t) = terrain.as_deref() else { return };
    if let Ok((ptf, pl)) = player.single() {
        let p = ptf.translation;
        let on_sand = pl.grounded && (p.y - t.height(p.x, p.z)).abs() < 0.06 && pl.drive != Drive::External;
        let far = st.last_print.map(|l| l.xz().distance(p.xz()) >= 0.22).unwrap_or(true);
        if on_sand && far && pl.planar_speed > 0.1 {
            st.last_print = Some(p);
            st.print_left = !st.print_left;
            let left = st.print_left;
            place_print(&mut st, t, &mut tfs, p.xz(), pl.yaw, left);
        }
    }
    if let Ok((ptf, pi)) = pilot.single() {
        let p = ptf.translation;
        let far = st.last_pilot_print.map(|l| l.xz().distance(p.xz()) >= 0.30).unwrap_or(true);
        if pi.moved && far {
            st.last_pilot_print = Some(p);
            st.pilot_left = !st.pilot_left;
            let left = st.pilot_left;
            place_print(&mut st, t, &mut tfs, p.xz(), pi.yaw, left);
        }
    }
}

// ------------------------------------------------------ stardust trail ----

pub fn stardust_trail(time: Res<Time>, st: Res<EarthState>, terrain: Option<Res<Terrain>>, player: Query<&Transform, With<Player>>, mut dots: Query<(&TrailDot, &mut Transform, &mut Visibility), Without<Player>>) {
    let (Some(target), Ok(p), Some(t)) = (st.trail, player.single(), terrain.as_deref()) else {
        for (_, _, mut v) in &mut dots {
            *v = Visibility::Hidden;
        }
        return;
    };
    let from = p.translation.xz();
    let d = target.xz() - from;
    let dist = d.length();
    let dir = d.normalize_or_zero();
    let off = (time.elapsed_secs() * 0.8).rem_euclid(0.9);
    for (dot, mut tf, mut v) in &mut dots {
        let s = 1.0 + dot.0 as f32 * 0.9 + off;
        if s > dist - 0.8 {
            *v = Visibility::Hidden;
            continue;
        }
        let q = from + dir * s;
        let bob = (time.elapsed_secs() * 3.0 + dot.0 as f32).sin() * 0.04;
        tf.translation = t.ground(q.x, q.y) + Vec3::Y * (0.14 + bob);
        let fade = ((dist - 0.8 - s) / 1.5).clamp(0.2, 1.0) * (s / 1.6).min(1.0);
        tf.scale = Vec3::splat(0.6 + fade * 0.7);
        *v = Visibility::Visible;
    }
}

// ------------------------------------------------- dusk + evening star ----

/// Sun elevation for a twilight share (0 = afternoon light, 1 = night).
pub fn share_elevation(share: f32) -> f32 {
    32.0 - 40.0 * share
}

pub fn dusk_ramp(time: Res<Time>, mut st: ResMut<EarthState>, mut sky: ResMut<Sky>, player: Query<&Transform, With<Player>>, mut star: Query<&mut Transform, (With<EveningStar>, Without<Player>)>) {
    if st.scene != Scene::Dunes {
        return;
    }
    let dt = time.delta_secs();
    let share = match st.dusk {
        Dusk::Off => return,
        Dusk::Fixed(s) => s,
        Dusk::Corridor => {
            let Ok(p) = player.single() else { return };
            let (a, b) = (world::D_WELL, D_DOOR);
            let ab = b - a;
            let t = ((p.translation.xz() - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            let target = 0.12 + 0.48 * t;
            // never go back toward the afternoon
            st.share.max(target)
        }
        Dusk::Ease { from, to, t, secs } => {
            let nt = (t + dt / secs).min(1.0);
            st.dusk = if nt >= 1.0 { Dusk::Fixed(to) } else { Dusk::Ease { from, to, t: nt, secs } };
            let k = nt * nt * (3.0 - 2.0 * nt);
            from + (to - from) * k
        }
    };
    st.share = share;
    let h = sky.hour_for_elevation(share_elevation(share), true);
    sky.hour = h;
    let k = ((share - 0.12) / (0.5 - 0.12)).clamp(0.0, 1.0);
    for mut t in &mut star {
        t.scale = Vec3::splat(0.001 + k * 5.0);
    }
}

/// Stars always face the camera.
pub fn star_billboard(cam: Query<&GlobalTransform, With<MainCamera>>, mut q: Query<&mut Transform, Or<(With<EveningStar>, With<HisStar>)>>) {
    let Ok(c) = cam.single() else { return };
    for mut t in &mut q {
        let d = c.translation() - t.translation;
        if d.length_squared() > 1e-4 {
            t.rotation = Transform::IDENTITY.looking_to(-d, Vec3::Y).rotation;
        }
    }
}

/// Shows his star 40 m above the prince, a little ahead of him.
pub fn show_his_star(world: &mut World) {
    let (pos, yaw) = {
        let mut q = world.query::<(&Transform, &Player)>();
        match q.iter(world).next() {
            Some((t, p)) => (t.translation, p.yaw),
            None => return,
        }
    };
    let fwd = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let at = pos + fwd * 16.0 + Vec3::Y * 40.0;
    let mut q = world.query_filtered::<&mut Transform, With<HisStar>>();
    for mut t in q.iter_mut(world) {
        t.translation = at;
        t.scale = Vec3::splat(2.6);
    }
}

pub fn hide_his_star(world: &mut World) {
    let mut q = world.query_filtered::<&mut Transform, With<HisStar>>();
    for mut t in q.iter_mut(world) {
        t.scale = Vec3::splat(0.001);
    }
}

pub fn his_star_pos(world: &mut World) -> Option<Vec3> {
    let mut q = world.query_filtered::<&Transform, With<HisStar>>();
    q.iter(world).next().map(|t| t.translation)
}

/// Direction of the evening star (unit).
pub fn star_dir() -> Vec3 {
    let (az, el) = (STAR_AZ.to_radians(), STAR_EL.to_radians());
    Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos())
}

// ------------------------------------------------------- train arrival ----

pub fn start_train_arrival(world: &mut World) {
    let mut q = world.query_filtered::<(Entity, &Transform), With<TrainCar>>();
    let cars: Vec<(Entity, f32)> = q.iter(world).map(|(e, t)| (e, t.translation.x)).collect();
    for (e, x) in cars {
        world.entity_mut(e).insert(RestX(x));
    }
    world.resource_mut::<EarthState>().train_anim = Some(TrainAnim { t: 0.0, secs: 2.0, dist: 9.0 });
}

pub fn train_arrival(time: Res<Time>, mut st: ResMut<EarthState>, mut runner: ResMut<StoryRunner>, mut q: Query<(&RestX, &mut Transform), With<TrainCar>>) {
    let Some(mut a) = st.train_anim else { return };
    a.t = (a.t + time.delta_secs()).min(a.secs);
    let k = a.t / a.secs;
    let ease = 1.0 - (1.0 - k).powi(3);
    for (r, mut t) in &mut q {
        t.translation.x = r.0 - a.dist * (1.0 - ease);
    }
    if a.t >= a.secs {
        st.train_anim = None;
        runner.set_flag("train_stopped");
    } else {
        st.train_anim = Some(a);
    }
}

// --------------------------------------------------------- ending walk ----

/// Ending camera: raised shot over the pilot's shoulder; 5 m before the
/// doorway it sinks to a low shot from the pilot and tilts up the sacred way
/// toward the evening star; 3 m past the doorway the end card starts.
pub fn ending_walk(mut st: ResMut<EarthState>, mut runner: ResMut<StoryRunner>, mut rig: ResMut<CameraRig>, player: Query<&Transform, With<Player>>, anchors: Res<crate::core::story::Anchors>) {
    if st.end_walk == 0 || st.end_walk >= 3 {
        return;
    }
    let Ok(p) = player.single() else { return };
    let a = world::way_dir();
    let along = (p.translation.xz() - D_DOOR).dot(a);
    let Some((fw, _)) = anchors.get("farewell") else { return };
    let back = Vec3::new(-a.x, 0.0, -a.y);
    let right = Vec3::new(-(world::WAY_YAW.to_radians()).cos(), 0.0, (world::WAY_YAW.to_radians()).sin());
    if st.end_walk == 1 && along > -5.0 {
        st.end_walk = 2;
        let door = anchors.get("door").map(|d| d.0).unwrap_or(fw);
        let from = fw + back * 3.2 + right * 0.9 + Vec3::Y * 0.9;
        let look = door + star_dir() * 22.0;
        rig.begin_shot(Shot { position: from, look_at: look, blend: 4.0, fov: Some(50.0) });
    }
    if st.end_walk == 2 && along >= 3.0 {
        st.end_walk = 3;
        runner.set_flag("past_door");
    }
}
