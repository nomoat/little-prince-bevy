//! The Earth cast: snake (DesertSnakeBehaviour), fox (DesertFoxWander) and the
//! aviator (sleep → sit up → stand; crate seat; walk). Each animates the named
//! parts of its procedural model; glTF replacements without parts stay static.

use super::models::FOX_BODY_Y;
use crate::core::prelude::*;
use crate::core::story::StoryRunner;

fn wrap(a: f32) -> f32 {
    let mut d = a.rem_euclid(std::f32::consts::TAU);
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    d
}

fn approach(cur: f32, target: f32, rate: f32) -> f32 {
    if (target - cur).abs() <= rate { target } else { cur + rate * (target - cur).signum() }
}

fn approach_angle(cur: f32, target: f32, rate: f32) -> f32 {
    let d = wrap(target - cur);
    if d.abs() <= rate { target } else { cur + rate * d.signum() }
}

// =================================================================== SNAKE

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnakeState {
    Coiled,
    Raising,
    Raised,
    Settling,
    Leaving,
    Gone,
}

#[derive(Component, Debug)]
pub struct Snake {
    pub state: SnakeState,
    /// 0 = resting coiled, 1 = raised.
    pub raise: f32,
    pub look: f32,
    pub look_pitch: f32,
    /// Body yaw (radians).
    pub yaw: f32,
    pub crawled: f32,
    pub fade: f32,
    pub t: f32,
}

impl Snake {
    pub fn new(state: SnakeState, yaw_deg: f32) -> Self {
        Snake { raise: if state == SnakeState::Raised { 1.0 } else { 0.0 }, state, look: 0.0, look_pitch: 0.0, yaw: yaw_deg.to_radians(), crawled: 0.0, fade: 1.0, t: 0.0 }
    }
}

pub fn snake_update(
    time: Res<Time>,
    mut snakes: Query<(Entity, &mut Snake, &mut Transform, &mut Visibility)>,
    mut parts: Query<(&PartName, &ChildOf, &mut Transform), Without<Snake>>,
    player: Query<&Transform, (With<Player>, Without<Snake>, Without<PartName>)>,
) {
    let dt = time.delta_secs();
    let pp = player.single().ok().map(|t| t.translation);
    for (e, mut s, mut tf, mut vis) in &mut snakes {
        s.t += dt;
        match s.state {
            SnakeState::Coiled => s.raise = approach(s.raise, 0.0, dt / 0.8),
            SnakeState::Raising => {
                s.raise = approach(s.raise, 1.0, dt / 0.8);
                if s.raise >= 1.0 {
                    s.state = SnakeState::Raised;
                }
            }
            SnakeState::Raised => s.raise = approach(s.raise, 1.0, dt / 0.8),
            SnakeState::Settling => {
                s.raise = approach(s.raise, 0.0, dt / 1.2);
                if s.raise <= 0.0 {
                    s.state = SnakeState::Coiled;
                }
            }
            SnakeState::Leaving => {
                // crawl on toward the far dune at 2× speed; fade (sink) after 1.4 m
                s.raise = approach(s.raise, 0.25, dt / 0.5);
                let step = 0.7 * dt;
                s.crawled += step;
                let wig = (s.t * 9.0).sin() * 0.25;
                let f = Vec3::new(s.yaw.sin(), 0.0, s.yaw.cos());
                tf.translation += f * step;
                tf.rotation = Quat::from_rotation_y(s.yaw + wig);
                if s.crawled > 1.4 {
                    s.fade = (s.fade - dt / 0.5).max(0.0);
                    tf.scale = Vec3::splat(s.fade.max(0.001));
                    if s.fade <= 0.0 {
                        s.state = SnakeState::Gone;
                        *vis = Visibility::Hidden;
                    }
                }
            }
            SnakeState::Gone => {}
        }
        // head-look toward the player within 4 m (coiled ≤25°/10°, raised ≤75°/30°)
        let (max_yaw, max_pitch) = (25.0f32.to_radians() + s.raise * 50.0f32.to_radians(), 10.0f32.to_radians() + s.raise * 20.0f32.to_radians());
        let (mut ty, mut tp) = (0.0, 0.0);
        if let Some(p) = pp {
            let d = p - tf.translation;
            if d.xz().length() < 4.0 && s.state != SnakeState::Leaving {
                let body_yaw = tf.rotation.to_euler(EulerRot::YXZ).0;
                ty = wrap(d.x.atan2(d.z) - body_yaw).clamp(-max_yaw, max_yaw);
                tp = ((d.y + 0.9 - 0.45) / d.xz().length().max(0.3)).atan().clamp(-max_pitch, max_pitch);
            }
        }
        s.look = approach_angle(s.look, ty, dt * 2.5);
        s.look_pitch = approach(s.look_pitch, tp, dt * 1.5);
        let tilt = 70f32.to_radians() * (1.0 - s.raise) - s.look_pitch * s.raise;
        let breathe = (s.t * 1.7).sin() * 0.04 * s.raise;
        for (name, parent, mut ptf) in &mut parts {
            if parent.parent() == e && name.0 == "neck" {
                ptf.translation = Vec3::new(0.0, 0.16, 0.0);
                ptf.rotation = Quat::from_rotation_y(s.look) * Quat::from_rotation_x(tilt + breathe);
            }
        }
    }
}

// ===================================================================== FOX

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoxState {
    Wander,
    Alert,
    Approach,
    SitDown,
    Seated,
    StandUp,
    WatchWheat,
}

#[derive(Component, Debug)]
pub struct Fox {
    pub state: FoxState,
    pub center: Vec2,
    pub yaw: f32,
    pub hop: Option<Vec2>,
    pub idle: f32,
    pub stops: u32,
    pub sniff: f32,
    pub sit: f32,
    pub phase: f32,
    pub moving: f32,
    pub alert_out: f32,
    pub hold: f32,
    pub head_look: f32,
    pub rng: Rng,
    /// Where it sits (2.5 m from the prince) during the talk.
    pub sit_at: Option<Vec2>,
    pub wheat_yaw: f32,
}

impl Fox {
    pub fn new(center: Vec2, yaw_deg: f32) -> Self {
        Fox {
            state: FoxState::Wander,
            center,
            yaw: yaw_deg.to_radians(),
            hop: None,
            idle: 1.5,
            stops: 0,
            sniff: 0.0,
            sit: 0.0,
            phase: 0.0,
            moving: 0.0,
            alert_out: 0.0,
            hold: 0.0,
            head_look: 0.0,
            rng: Rng::new(4242),
            sit_at: None,
            wheat_yaw: yaw_deg.to_radians(),
        }
    }
}

const FOX_WALK: f32 = 0.349;
const FOX_TURN: f32 = 90.0;
const FOX_RADIUS: f32 = 2.5;

/// Starts the talk: approach and sit 2.5 m from the prince.
pub fn fox_begin_talk(world: &mut World) {
    let pp = {
        let mut q = world.query_filtered::<&Transform, With<Player>>();
        q.iter(world).next().map(|t| t.translation.xz())
    };
    let mut q = world.query::<(&mut Fox, &Transform)>();
    if let (Some(pp), Some((mut f, tf))) = (pp, q.iter_mut(world).next()) {
        let from = tf.translation.xz();
        let dir = (from - pp).normalize_or(Vec2::X);
        f.sit_at = Some(pp + dir * 2.5);
        f.state = FoxState::Approach;
    }
}

pub fn fox_watch_wheat(world: &mut World) {
    let mut q = world.query::<&mut Fox>();
    for mut f in q.iter_mut(world) {
        f.state = FoxState::StandUp;
    }
}

pub fn fox_update(
    time: Res<Time>,
    mut runner: ResMut<StoryRunner>,
    terrain: Option<Res<Terrain>>,
    mut foxes: Query<(Entity, &mut Fox, &mut Transform)>,
    mut parts: Query<(&PartName, &ChildOf, &mut Transform), Without<Fox>>,
    player: Query<&Transform, (With<Player>, Without<Fox>, Without<PartName>)>,
) {
    let dt = time.delta_secs();
    let Ok(ptf) = player.single() else { return };
    let pp = ptf.translation.xz();
    for (e, mut f, mut tf) in &mut foxes {
        let pos = tf.translation.xz();
        let to_p = pp - pos;
        let dist_p = to_p.length();
        let mut want_move: Option<(Vec2, f32)> = None;
        let mut face: Option<f32> = None;
        match f.state {
            FoxState::Wander => {
                if dist_p < 6.0 {
                    f.state = FoxState::Alert;
                    f.alert_out = 0.0;
                } else if f.sniff > 0.0 {
                    f.sniff -= dt;
                    if f.sniff <= 0.0 {
                        f.idle = f.rng.range(0.5, 1.5);
                    }
                } else if let Some(h) = f.hop {
                    want_move = Some((h, FOX_WALK));
                    if pos.distance(h) < 0.08 {
                        f.hop = None;
                        f.stops += 1;
                        if f.stops % 3 == 0 {
                            f.sniff = 1.2;
                        } else {
                            f.idle = f.rng.range(2.0, 5.0);
                        }
                    }
                } else {
                    f.idle -= dt;
                    if f.idle <= 0.0 {
                        // pick a hop of 1–3 m inside the 2.5 m wander circle, away from the trunk
                        for _ in 0..12 {
                            let a = f.rng.range(0.0, std::f32::consts::TAU);
                            let d = f.rng.range(1.0, 3.0);
                            let c = pos + Vec2::new(a.sin(), a.cos()) * d;
                            if c.distance(f.center) <= FOX_RADIUS && c.distance(super::world::V_TRUNK) > 0.7 + 0.35 {
                                f.hop = Some(c);
                                break;
                            }
                        }
                        f.idle = 1.0;
                    }
                }
            }
            FoxState::Alert => {
                face = Some(to_p.x.atan2(to_p.y));
                if dist_p > 8.0 {
                    f.alert_out += dt;
                    if f.alert_out > 3.0 {
                        f.state = FoxState::Wander;
                    }
                } else {
                    f.alert_out = 0.0;
                }
            }
            FoxState::Approach => {
                let t = f.sit_at.unwrap_or(pos);
                want_move = Some((t, 0.75));
                if pos.distance(t) < 0.1 {
                    f.state = FoxState::SitDown;
                }
            }
            FoxState::SitDown => {
                face = Some(to_p.x.atan2(to_p.y));
                f.sit = approach(f.sit, 1.0, dt / 0.6);
                if f.sit >= 1.0 {
                    f.state = FoxState::Seated;
                    runner.set_flag("fox_seated");
                }
            }
            FoxState::Seated => face = Some(to_p.x.atan2(to_p.y)),
            FoxState::StandUp => {
                f.sit = approach(f.sit, 0.0, dt / 0.6);
                if f.sit <= 0.0 {
                    f.state = FoxState::WatchWheat;
                }
            }
            FoxState::WatchWheat => face = Some(f.wheat_yaw),
        }
        // locomotion: turn first (90°/s), then walk; hold still if the prince is < 0.9 m ahead
        f.moving = 0.0;
        if let Some((t, speed)) = want_move {
            let d = t - pos;
            if d.length() > 0.02 {
                let ty = d.x.atan2(d.y);
                f.yaw = approach_angle(f.yaw, ty, FOX_TURN.to_radians() * dt * if f.state == FoxState::Approach { 2.0 } else { 1.0 });
                let fwd = Vec2::new(f.yaw.sin(), f.yaw.cos());
                let blocked = dist_p < 0.9 && fwd.dot(to_p.normalize_or_zero()) > 0.5;
                if blocked {
                    f.hold += dt;
                    if f.hold > 1.5 && f.state == FoxState::Wander {
                        f.hop = None;
                        f.hold = 0.0;
                    }
                } else if wrap(ty - f.yaw).abs() < 0.35 {
                    f.hold = 0.0;
                    let step = (speed * dt).min(d.length());
                    let np = pos + fwd * step;
                    tf.translation.x = np.x;
                    tf.translation.z = np.y;
                    f.moving = speed;
                }
            }
        } else if let Some(fy) = face {
            f.yaw = approach_angle(f.yaw, fy, FOX_TURN.to_radians() * dt);
        }
        if let Some(t) = terrain.as_deref() {
            tf.translation.y = t.height(tf.translation.x, tf.translation.z);
        }
        tf.rotation = Quat::from_rotation_y(f.yaw);
        f.phase += dt * (4.0 + f.moving * 14.0);
        let head_target = if f.state == FoxState::Alert || f.state == FoxState::Seated { wrap(to_p.x.atan2(to_p.y) - f.yaw).clamp(-0.8, 0.8) } else { 0.0 };
        f.head_look = approach_angle(f.head_look, head_target, dt * 2.5);
        let sniff = if f.sniff > 0.0 { 1.0 } else { 0.0 };
        pose_fox(e, &f, sniff, &mut parts);
    }
}

fn pose_fox(e: Entity, f: &Fox, sniff: f32, parts: &mut Query<(&PartName, &ChildOf, &mut Transform), Without<Fox>>) {
    let s = f.sit;
    let body_pos = Vec3::new(0.0, FOX_BODY_Y - 0.2 * s, -0.2);
    let body_rot = Quat::from_rotation_x(-35f32.to_radians() * s);
    let body = Transform::from_translation(body_pos).with_rotation(body_rot);
    let swing = if f.moving > 0.0 { f.phase.sin() * 0.55 } else { 0.0 };
    for (name, parent, mut tf) in parts.iter_mut() {
        if parent.parent() != e {
            continue;
        }
        let t = match name.0.as_str() {
            "body" => body,
            "head" => {
                let p = body.transform_point(Vec3::new(0.0, 0.1, 0.44));
                Transform::from_translation(p).with_rotation(body_rot * Quat::from_rotation_x(35f32.to_radians() * s * 0.85) * Quat::from_rotation_y(f.head_look) * Quat::from_rotation_x(0.6 * sniff))
            }
            "tail" => {
                let p = body.transform_point(Vec3::new(0.0, 0.04, -0.05));
                let wag = (f.phase * 0.6).sin() * 0.25;
                let stand = body_rot * Quat::from_rotation_y(wag) * Quat::from_rotation_x(-0.35);
                let sit = Quat::from_rotation_y(1.1) * Quat::from_rotation_x(0.1);
                Transform::from_translation(p).with_rotation(stand.slerp(sit, s))
            }
            "leg_fl" | "leg_fr" => {
                let x = if name.0 == "leg_fl" { 0.065 } else { -0.065 };
                let p = body.transform_point(Vec3::new(x, -0.02, 0.33));
                let sw = if name.0 == "leg_fl" { swing } else { -swing };
                Transform::from_translation(p).with_rotation(Quat::from_rotation_x(sw * (1.0 - s)))
            }
            "leg_bl" | "leg_br" => {
                let x = if name.0 == "leg_bl" { 0.075 } else { -0.075 };
                let p = body.transform_point(Vec3::new(x, -0.02, 0.03));
                let sw = if name.0 == "leg_bl" { -swing } else { swing };
                Transform::from_translation(p).with_rotation(Quat::from_rotation_x(sw * (1.0 - s) - 75f32.to_radians() * s))
            }
            _ => continue,
        };
        *tf = t;
    }
}

// =================================================================== PILOT

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PilotPose {
    Sleep,
    SitUp,
    Stand,
    SitCrate,
    Walk,
}

#[derive(Clone, Copy, Debug, Default)]
struct PoseParams {
    hip_y: f32,
    torso: f32,
    thigh: f32,
    knee: f32,
    arm: f32,
    head: f32,
}

impl PoseParams {
    fn of(p: PilotPose) -> Self {
        let d = |x: f32| x.to_radians();
        match p {
            PilotPose::Sleep => PoseParams { hip_y: 0.14, torso: d(-90.0), thigh: d(-90.0), knee: 0.0, arm: d(-8.0), head: d(-5.0) },
            PilotPose::SitUp => PoseParams { hip_y: 0.14, torso: d(-8.0), thigh: d(-88.0), knee: d(12.0), arm: d(-25.0), head: 0.0 },
            PilotPose::Stand | PilotPose::Walk => PoseParams { hip_y: 0.87, torso: 0.0, thigh: 0.0, knee: 0.0, arm: 0.0, head: 0.0 },
            PilotPose::SitCrate => PoseParams { hip_y: 0.5, torso: d(6.0), thigh: d(-86.0), knee: d(88.0), arm: d(-45.0), head: d(5.0) },
        }
    }
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        let l = |x: f32, y: f32| x + (y - x) * t;
        PoseParams { hip_y: l(a.hip_y, b.hip_y), torso: l(a.torso, b.torso), thigh: l(a.thigh, b.thigh), knee: l(a.knee, b.knee), arm: l(a.arm, b.arm), head: l(a.head, b.head) }
    }
}

#[derive(Component, Debug)]
pub struct Pilot {
    pub pose: PilotPose,
    prev: PilotPose,
    blend: f32,
    pub blend_secs: f32,
    /// Walk target + speed (pose Walk while moving, then Stand).
    pub walk: Option<(Vec2, f32)>,
    pub phase: f32,
    /// Head-look target (world).
    pub look_at: Option<Vec3>,
    head_yaw: f32,
    head_pitch: f32,
    pub yaw: f32,
    pub yaw_target: Option<f32>,
    pub moved: bool,
}

impl Pilot {
    pub fn new(pose: PilotPose) -> Self {
        Pilot { pose, prev: pose, blend: 1.0, blend_secs: 0.8, walk: None, phase: 0.0, look_at: None, head_yaw: 0.0, head_pitch: 0.0, yaw: super::world::D_PILOT_YAW.to_radians(), yaw_target: None, moved: false }
    }
    pub fn set_pose(&mut self, p: PilotPose, secs: f32) {
        if p != self.pose {
            self.prev = self.pose;
            self.pose = p;
            self.blend = if secs <= 0.0 { 1.0 } else { 0.0 };
            self.blend_secs = secs.max(0.01);
        }
    }
}

pub fn pilot_update(
    time: Res<Time>,
    mut runner: ResMut<StoryRunner>,
    terrain: Option<Res<Terrain>>,
    mut pilots: Query<(Entity, &mut Pilot, &mut Transform)>,
    mut parts: Query<(&PartName, &ChildOf, &mut Transform), Without<Pilot>>,
) {
    let dt = time.delta_secs();
    for (e, mut p, mut tf) in &mut pilots {
        p.blend = (p.blend + dt / p.blend_secs).min(1.0);
        p.moved = false;
        if let Some((t, speed)) = p.walk {
            let pos = tf.translation.xz();
            let d = t - pos;
            if d.length() < 0.05 {
                p.walk = None;
                p.set_pose(PilotPose::Stand, 0.3);
                runner.set_flag("pilot_arrived");
            } else {
                if p.pose != PilotPose::Walk {
                    p.set_pose(PilotPose::Walk, 0.25);
                }
                let ty = d.x.atan2(d.y);
                p.yaw = approach_angle(p.yaw, ty, 165f32.to_radians() * dt);
                let step = (speed * dt).min(d.length());
                let np = pos + d.normalize() * step;
                tf.translation.x = np.x;
                tf.translation.z = np.y;
                if let Some(tr) = terrain.as_deref() {
                    tf.translation.y = tr.height(np.x, np.y);
                }
                p.phase += dt * speed * 4.2;
                p.moved = true;
            }
        } else if let Some(ty) = p.yaw_target {
            p.yaw = approach_angle(p.yaw, ty, 165f32.to_radians() * dt);
        }
        tf.rotation = Quat::from_rotation_y(p.yaw);
        // head-look (pitch limits 20° up / 35° down)
        let (mut hy, mut hp) = (0.0, 0.0);
        if let Some(l) = p.look_at {
            if matches!(p.pose, PilotPose::Stand | PilotPose::SitCrate | PilotPose::SitUp | PilotPose::Walk) {
                let head = tf.translation + Vec3::Y * 1.6;
                let d = l - head;
                hy = wrap(d.x.atan2(d.z) - p.yaw).clamp(-1.1, 1.1);
                hp = (d.y / d.xz().length().max(0.2)).atan().clamp(-35f32.to_radians(), 20f32.to_radians());
            }
        }
        p.head_yaw = approach_angle(p.head_yaw, hy, dt * 2.0);
        p.head_pitch = approach(p.head_pitch, hp, dt * 1.5);
        let k = p.blend * p.blend * (3.0 - 2.0 * p.blend);
        let mut pp = PoseParams::lerp(PoseParams::of(p.prev), PoseParams::of(p.pose), k);
        let walking = p.pose == PilotPose::Walk && p.moved;
        let sw = if walking { p.phase.sin() } else { 0.0 };
        let breathe = (time.elapsed_secs() * 1.3).sin() * 0.015;
        pp.torso += breathe * if p.pose == PilotPose::Sleep { 0.0 } else { 1.0 };
        let torso = Transform::from_xyz(0.0, pp.hip_y, 0.0).with_rotation(Quat::from_rotation_x(pp.torso));
        for (name, parent, mut ptf) in &mut parts {
            if parent.parent() != e {
                continue;
            }
            let side = if name.0.ends_with("_l") { 1.0 } else { -1.0 };
            let t = match name.0.as_str() {
                "torso" => torso,
                "head" => {
                    let pos = torso.transform_point(Vec3::new(0.0, 0.6, 0.0));
                    Transform::from_translation(pos).with_rotation(torso.rotation * Quat::from_rotation_y(p.head_yaw) * Quat::from_rotation_x(pp.head - p.head_pitch))
                }
                "arm_l" | "arm_r" => {
                    let pos = torso.transform_point(Vec3::new(0.27 * side, 0.55, 0.0));
                    Transform::from_translation(pos).with_rotation(torso.rotation * Quat::from_rotation_x(pp.arm - sw * 0.35 * side) * Quat::from_rotation_z(0.08 * side))
                }
                "thigh_l" | "thigh_r" => Transform::from_xyz(0.11 * side, pp.hip_y, 0.0).with_rotation(Quat::from_rotation_x(pp.thigh + sw * 0.45 * side)),
                "shin_l" | "shin_r" => {
                    let thigh = Transform::from_xyz(0.11 * side, pp.hip_y, 0.0).with_rotation(Quat::from_rotation_x(pp.thigh + sw * 0.45 * side));
                    let pos = thigh.transform_point(Vec3::new(0.0, -0.45, 0.0));
                    let knee = pp.knee + if walking { (sw * side).max(0.0) * 0.6 } else { 0.0 };
                    Transform::from_translation(pos).with_rotation(thigh.rotation * Quat::from_rotation_x(knee))
                }
                _ => continue,
            };
            *ptf = t;
        }
    }
}

/// Pilot helpers for script code.
pub fn with_pilot(world: &mut World, f: impl FnOnce(&mut Pilot, &mut Transform)) {
    let mut q = world.query::<(&mut Pilot, &mut Transform)>();
    if let Some((mut p, mut t)) = q.iter_mut(world).next() {
        f(&mut p, &mut t);
    }
}

pub fn with_snake<F: FnMut(&mut Snake, &mut Transform)>(world: &mut World, raised_only: bool, mut f: F) {
    let mut q = world.query::<(&mut Snake, &mut Transform)>();
    for (mut s, mut t) in q.iter_mut(world) {
        if !raised_only || s.state != SnakeState::Gone {
            f(&mut s, &mut t);
        }
    }
}
