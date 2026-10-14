//! The prince: character controller + procedural low-poly model + animation.
//! Numbers follow the original PlayerController's documented tuning: a
//! child's light jog by default (2.4 m/s), Shift for the fast run (3.5 m/s),
//! slow scripted walks (~1 m/s), slopes slow you from 20° and stop you past
//! 35°, an optional small jump, turning in place for sharp turns.

use super::camera::CameraRig;
use super::input::InputFrame;
use super::manifest::Manifest;
use super::meshkit::{MeshBuilder, hex};
use super::models::SharedMaterials;
use super::physics::{ColliderWorld, STEP_HEIGHT, ground_height};
use super::planet::Terrain;
use bevy::prelude::*;

#[derive(Resource, Clone, Debug)]
pub struct PlayerTuning {
    pub walk_speed: f32,
    pub jog_speed: f32,
    pub run_speed: f32,
    pub run_enabled: bool,
    pub acceleration: f32,
    pub deceleration: f32,
    pub turn_smooth_time: f32,
    pub max_turn_speed: f32,
    pub full_speed_angle: f32,
    pub turn_in_place_angle: f32,
    pub gravity: f32,
    pub slope_slow_start: f32,
    pub slope_limit: f32,
    pub slope_slow_at_limit: f32,
    pub jump_enabled: bool,
    pub jump_height: f32,
    pub jump_cooldown: f32,
    pub radius: f32,
    pub height: f32,
}

impl Default for PlayerTuning {
    fn default() -> Self {
        PlayerTuning {
            walk_speed: 1.0,
            jog_speed: 2.4,
            run_speed: 3.5,
            run_enabled: true,
            acceleration: 9.0,
            deceleration: 12.0,
            turn_smooth_time: 0.07,
            max_turn_speed: 720.0,
            full_speed_angle: 35.0,
            turn_in_place_angle: 110.0,
            gravity: 20.0,
            slope_slow_start: 20.0,
            slope_limit: 35.0,
            slope_slow_at_limit: 0.55,
            jump_enabled: true,
            jump_height: 0.55,
            jump_cooldown: 0.35,
            radius: 0.24,
            height: 1.2,
        }
    }
}

/// How the player is driven (PlayerController.DriveMode).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Drive {
    /// Normal input.
    #[default]
    Player,
    /// Walking to a point by script (story `WalkTo`, autopilot).
    WalkTo { target: Vec2, speed: f32, stop_radius: f32 },
    /// Frozen: chapter code moves the transform (sitting, carried by birds …).
    External,
}

/// A pose overlay for the procedural rig (chapters set it via [`Player::pose`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pose {
    #[default]
    None,
    Sit,
    Kneel,
    Wave,
    Reach,
    Clap,
    LookUp,
    Carry,
}

#[derive(Component, Debug)]
pub struct Player {
    pub drive: Drive,
    pub pose: Pose,
    pub velocity: Vec3,
    /// Facing yaw (radians; 0 = +Z).
    pub yaw: f32,
    pub planar_speed: f32,
    pub grounded: bool,
    pub air_time: f32,
    last_jump: f32,
    speed: f32,
    yaw_vel: f32,
    /// Stride phase for animation / footsteps.
    pub stride: f32,
    /// Set when a footstep lands (for audio).
    pub footstep: bool,
    pub face_target: Option<f32>,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            drive: Drive::Player,
            pose: Pose::None,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            planar_speed: 0.0,
            grounded: true,
            air_time: 0.0,
            last_jump: -10.0,
            speed: 0.0,
            yaw_vel: 0.0,
            stride: 0.0,
            footstep: false,
            face_target: None,
        }
    }
}

impl Player {
    pub fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos())
    }
    pub fn is_arrived(&self) -> bool {
        !matches!(self.drive, Drive::WalkTo { .. })
    }
}

/// Parts of the procedural prince for animation.
#[derive(Component)]
pub struct PrinceRig {
    pub body: Entity,
    pub head: Entity,
    pub leg_l: Entity,
    pub leg_r: Entity,
    pub arm_l: Entity,
    pub arm_r: Entity,
    pub scarf: Vec<Entity>,
}

/// The visual child of the player (rotated by yaw).
#[derive(Component)]
pub struct PlayerVisual;

/// Spawns the player at a ground position facing `yaw_deg` (0 = +Z).
pub fn spawn_player(world: &mut World, pos: Vec3, yaw_deg: f32) -> Entity {
    let yaw = yaw_deg.to_radians();
    let shared = world.resource::<SharedMaterials>().clone();
    let gltf_path = world.resource::<Manifest>().get("model.prince").and_then(|e| e.path.clone());
    let player = world
        .spawn((
            Name::new("Player"),
            Player { yaw, ..default() },
            Transform::from_translation(pos),
            Visibility::default(),
            super::state::ChapterScoped::default(),
        ))
        .id();
    let visual = world
        .spawn((PlayerVisual, Transform::from_rotation(Quat::from_rotation_y(yaw)), Visibility::default()))
        .id();
    world.entity_mut(player).add_child(visual);
    // a soft moonlit fill that keeps the prince readable on night planets
    // (CharacterFillLight in the original); its strength follows the night
    let fill = world
        .spawn((PrinceFill, PointLight { color: Color::srgb(0.78, 0.84, 1.0), intensity: 0.0, range: 3.2, shadow_maps_enabled: false, ..default() }, Transform::from_xyz(0.0, 1.9, 0.0)))
        .id();
    world.entity_mut(player).add_child(fill);
    if gltf_path.is_some() {
        // A replacement model from the manifest (glTF). Animation: see docs/ASSETS.md.
        let child = world.spawn((Transform::default(), Visibility::default(), super::models::PendingModel("model.prince".into()))).id();
        world.entity_mut(visual).add_child(child);
        return player;
    }
    let rig = build_prince(world, visual, &shared);
    world.entity_mut(player).insert(rig);
    player
}

fn part(world: &mut World, parent: Entity, mb: &MeshBuilder, t: Transform, mat: &Handle<StandardMaterial>) -> Entity {
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mb.build());
    let e = world.spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), t)).id();
    world.entity_mut(parent).add_child(e);
    e
}

/// The procedural little prince (~1.2 m): green coat, golden hair, long scarf.
fn build_prince(world: &mut World, visual: Entity, shared: &SharedMaterials) -> PrinceRig {
    let m = &shared.vertex;
    let coat = hex(0x5f8f5a);
    let coat_dark = hex(0x4c7549);
    let skin = hex(0xf4d6bd);
    let hair = hex(0xf2c14e);
    let boot = hex(0x6b4b3a);
    let trouser = hex(0x7fa36a);
    let scarf_c = hex(0xf0c23a);

    let root = world.spawn((Transform::default(), Visibility::default())).id();
    world.entity_mut(visual).add_child(root);

    // Body: long coat (flared lathe) with a belt.
    let mut b = MeshBuilder::new();
    b.lathe(Vec3::ZERO, &[(0.0, 0.0), (0.26, 0.0), (0.22, 0.22), (0.17, 0.42), (0.15, 0.5), (0.0, 0.52)], 10, coat);
    b.frustum(Vec3::Y * 0.27, 0.205, 0.195, 0.05, 10, coat_dark);
    // collar
    b.frustum(Vec3::Y * 0.47, 0.17, 0.12, 0.08, 10, coat_dark);
    let body = part(world, root, &b, Transform::from_xyz(0.0, 0.42, 0.0), m);

    // Head with hair tufts.
    let mut h = MeshBuilder::new();
    h.sphere(Vec3::ZERO, 0.15, 5, 9, skin);
    h.ellipsoid(Vec3::new(0.0, 0.05, -0.015), Vec3::new(0.16, 0.12, 0.155), 3, 9, hair);
    for i in 0..7 {
        let a = i as f32 / 7.0 * std::f32::consts::TAU;
        let base = Vec3::new(a.cos() * 0.1, 0.1, a.sin() * 0.1 - 0.01);
        let tip = base + Vec3::new(a.cos() * 0.09, 0.07 + (i % 3) as f32 * 0.02, a.sin() * 0.09);
        h.tube(base, tip, 0.045, 0.0, 4, hair);
    }
    // eyes
    h.sphere(Vec3::new(0.055, 0.0, 0.135), 0.018, 2, 4, hex(0x2b2b3a));
    h.sphere(Vec3::new(-0.055, 0.0, 0.135), 0.018, 2, 4, hex(0x2b2b3a));
    let head = part(world, body, &h, Transform::from_xyz(0.0, 0.66, 0.0), m);

    // Legs (pivot at hip).
    let mut l = MeshBuilder::new();
    l.frustum(Vec3::new(0.0, -0.42, 0.0), 0.055, 0.06, 0.42, 6, trouser);
    l.cuboid(Vec3::new(0.0, -0.4, 0.035), Vec3::new(0.1, 0.08, 0.17), boot);
    let leg_l = part(world, root, &l, Transform::from_xyz(0.085, 0.44, 0.0), m);
    let leg_r = part(world, root, &l, Transform::from_xyz(-0.085, 0.44, 0.0), m);

    // Arms (pivot at shoulder).
    let mut a = MeshBuilder::new();
    a.frustum(Vec3::new(0.0, -0.36, 0.0), 0.04, 0.055, 0.36, 6, coat);
    a.sphere(Vec3::new(0.0, -0.38, 0.0), 0.04, 2, 5, skin);
    let arm_l = part(world, body, &a, Transform::from_xyz(0.19, 0.44, 0.0), m);
    let arm_r = part(world, body, &a, Transform::from_xyz(-0.19, 0.44, 0.0), m);

    // Scarf: a wrap at the neck + trailing segments (animated in the wind).
    let mut w = MeshBuilder::new();
    w.frustum(Vec3::Y * 0.5, 0.13, 0.12, 0.07, 10, scarf_c);
    part(world, body, &w, Transform::IDENTITY, m);
    let mut scarf = Vec::new();
    let mut parent = body;
    let mut seg = MeshBuilder::new();
    seg.cuboid(Vec3::new(0.0, 0.0, -0.09), Vec3::new(0.085, 0.02, 0.18), scarf_c);
    for i in 0..4 {
        let t = if i == 0 { Transform::from_xyz(0.06, 0.52, -0.1) } else { Transform::from_xyz(0.0, 0.0, -0.18) };
        let e = part(world, parent, &seg, t, m);
        scarf.push(e);
        parent = e;
    }
    PrinceRig { body, head, leg_l, leg_r, arm_l, arm_r, scarf }
}

fn smooth_damp_angle(cur: f32, target: f32, vel: &mut f32, smooth: f32, max_speed: f32, dt: f32) -> f32 {
    let mut delta = (target - cur) % std::f32::consts::TAU;
    if delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    if delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    let t = cur + delta;
    smooth_damp(cur, t, vel, smooth, max_speed, dt)
}

/// Unity-style SmoothDamp.
pub fn smooth_damp(cur: f32, target: f32, vel: &mut f32, smooth: f32, max_speed: f32, dt: f32) -> f32 {
    let smooth = smooth.max(1e-4);
    let omega = 2.0 / smooth;
    let x = omega * dt;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let max_change = max_speed * smooth;
    let change = (cur - target).clamp(-max_change, max_change);
    let t2 = cur - change;
    let temp = (*vel + omega * change) * dt;
    *vel = (*vel - omega * temp) * exp;
    let mut out = t2 + (change + temp) * exp;
    if (target - cur > 0.0) == (out > target) {
        out = target;
        *vel = (out - target) / dt.max(1e-5);
    }
    out
}

/// The character controller (runs on virtual time; sub-steps large frames).
#[allow(clippy::too_many_arguments)]
pub(crate) fn player_controller(
    time: Res<Time>,
    input: Res<InputFrame>,
    tuning: Res<PlayerTuning>,
    rig: Res<CameraRig>,
    terrain: Option<Res<Terrain>>,
    cols: Res<ColliderWorld>,
    mut q: Query<(&mut Player, &mut Transform), Without<PlayerVisual>>,
    mut vis: Query<&mut Transform, With<PlayerVisual>>,
) {
    let Ok((mut pl, mut tf)) = q.single_mut() else { return };
    let total = time.delta_secs().min(0.25);
    let steps = (total / (1.0 / 30.0)).ceil().max(1.0) as i32;
    let dt = total / steps as f32;
    let terrain = terrain.as_deref();
    pl.footstep = false;
    for _ in 0..steps {
        step(&mut pl, &mut tf, dt, &input, &tuning, rig.yaw, terrain, &cols, time.elapsed_secs());
    }
    if let Ok(mut v) = vis.single_mut() {
        v.rotation = Quat::from_rotation_y(pl.yaw);
    }
}

#[allow(clippy::too_many_arguments)]
fn step(pl: &mut Player, tf: &mut Transform, dt: f32, input: &InputFrame, tu: &PlayerTuning, cam_yaw: f32, terrain: Option<&Terrain>, cols: &ColliderWorld, now: f32) {
    let mut pos = tf.translation;
    // ---- desired direction & speed
    let (mut dir, mut want_speed) = (Vec2::ZERO, 0.0);
    match pl.drive.clone() {
        Drive::Player => {
            if let Some(w) = input.move_world {
                if w.length() > 0.05 {
                    dir = w.normalize();
                    want_speed = if input.run && tu.run_enabled { tu.run_speed } else { tu.jog_speed } * w.length().min(1.0);
                }
            } else if input.move_axis.length() > 0.05 {
                // camera-relative: forward is the camera's look direction.
                let f = Vec2::new(cam_yaw.sin(), cam_yaw.cos());
                let r = Vec2::new(-f.y, f.x);
                let v = f * input.move_axis.y + r * input.move_axis.x;
                dir = v.normalize_or_zero();
                let mag = input.move_axis.length().min(1.0);
                want_speed = if input.run && tu.run_enabled { tu.run_speed } else { tu.jog_speed } * mag;
            }
        }
        Drive::WalkTo { target, speed, stop_radius } => {
            let d = target - Vec2::new(pos.x, pos.z);
            let dist = d.length();
            if dist <= stop_radius {
                pl.drive = Drive::Player;
            } else {
                dir = d / dist;
                want_speed = speed * (dist / 0.6).clamp(0.35, 1.0);
            }
        }
        Drive::External => {
            pl.speed = 0.0;
            pl.planar_speed = 0.0;
            pl.velocity = Vec3::ZERO;
            if let Some(fy) = pl.face_target {
                pl.yaw = smooth_damp_angle(pl.yaw, fy, &mut pl.yaw_vel, tu.turn_smooth_time * 2.0, tu.max_turn_speed.to_radians(), dt);
            }
            return;
        }
    }
    // ---- turning (turn in place when the error is large)
    let mut facing_err = 0.0f32;
    if dir != Vec2::ZERO {
        let target_yaw = dir.x.atan2(dir.y);
        let mut d = (target_yaw - pl.yaw).rem_euclid(std::f32::consts::TAU);
        if d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        }
        facing_err = d.abs().to_degrees();
        pl.yaw = smooth_damp_angle(pl.yaw, target_yaw, &mut pl.yaw_vel, tu.turn_smooth_time, tu.max_turn_speed.to_radians(), dt);
        pl.face_target = None;
    } else if let Some(fy) = pl.face_target {
        pl.yaw = smooth_damp_angle(pl.yaw, fy, &mut pl.yaw_vel, tu.turn_smooth_time * 2.0, tu.max_turn_speed.to_radians(), dt);
    }
    if facing_err > tu.full_speed_angle {
        let k = 1.0 - ((facing_err - tu.full_speed_angle) / (tu.turn_in_place_angle - tu.full_speed_angle)).clamp(0.0, 1.0);
        want_speed *= k;
    }
    // ---- slope slow-down
    if let Some(t) = terrain {
        let slope = t.slope_deg(pos.x, pos.z);
        if slope > tu.slope_slow_start && dir != Vec2::ZERO {
            // only when walking uphill
            let n = t.normal(pos.x, pos.z);
            let uphill = -Vec2::new(n.x, n.z).normalize_or_zero();
            if uphill.dot(dir) > 0.0 {
                let k = ((slope - tu.slope_slow_start) / (tu.slope_limit - tu.slope_slow_start)).clamp(0.0, 1.0);
                want_speed *= 1.0 - k * (1.0 - tu.slope_slow_at_limit);
                if slope > tu.slope_limit + 3.0 {
                    want_speed = 0.0;
                }
            }
        }
    }
    let rate = if want_speed > pl.speed { tu.acceleration } else { tu.deceleration };
    pl.speed += (want_speed - pl.speed).clamp(-rate * dt, rate * dt);
    let fwd = Vec2::new(pl.yaw.sin(), pl.yaw.cos());
    let mut planar = fwd * pl.speed;
    // boundary push (B612PlanetBoundary)
    if let Some(t) = terrain {
        planar += t.boundary_push(pos);
    }
    pos.x += planar.x * dt;
    pos.z += planar.y * dt;
    // ---- vertical
    let ground = ground_height(terrain, cols, pos, tu.radius);
    if pl.grounded && input.jump && tu.jump_enabled && now - pl.last_jump > tu.jump_cooldown && pl.drive == Drive::Player {
        pl.velocity.y = (2.0 * tu.gravity * tu.jump_height).sqrt();
        pl.grounded = false;
        pl.last_jump = now;
    }
    if pl.grounded {
        // snap down slopes / steps
        if pos.y - ground < STEP_HEIGHT + 0.05 {
            pos.y = ground;
            pl.velocity.y = 0.0;
        } else {
            pl.grounded = false;
        }
    }
    if !pl.grounded {
        pl.velocity.y -= tu.gravity * dt;
        pos.y += pl.velocity.y * dt;
        pl.air_time += dt;
        let g = ground_height(terrain, cols, pos, tu.radius);
        if pos.y <= g && pl.velocity.y <= 0.0 {
            pos.y = g;
            pl.velocity.y = 0.0;
            pl.grounded = true;
            pl.air_time = 0.0;
            pl.footstep = true;
        }
    }
    // ---- collisions
    pos = cols.resolve(pos, tu.radius, tu.height);
    if let Some(t) = terrain {
        pos = t.clamp_inside(pos);
        // never sink below the terrain
        let h = t.height(pos.x, pos.z);
        if pos.y < h {
            pos.y = h;
        }
    }
    let moved = Vec2::new(pos.x - tf.translation.x, pos.z - tf.translation.z);
    pl.planar_speed = moved.length() / dt.max(1e-4);
    pl.velocity.x = moved.x / dt.max(1e-4);
    pl.velocity.z = moved.y / dt.max(1e-4);
    // stride
    if pl.grounded && pl.planar_speed > 0.2 {
        let before = pl.stride;
        pl.stride += dt * (2.2 + pl.planar_speed * 1.25);
        if (before / std::f32::consts::PI).floor() != (pl.stride / std::f32::consts::PI).floor() {
            pl.footstep = true;
        }
    }
    tf.translation = pos;
}

/// Procedural animation of the rig from speed / air / pose.
pub(crate) fn animate_prince(
    time: Res<Time>,
    q: Query<(&Player, &PrinceRig)>,
    mut tfs: Query<&mut Transform>,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    for (pl, rig) in &q {
        let speed = pl.planar_speed;
        let moving = (speed / 2.4).clamp(0.0, 1.4);
        let s = pl.stride;
        let swing = s.sin() * 0.75 * moving.min(1.0);
        let k = (dt * 12.0).min(1.0);
        let mut set = |e: Entity, rot: Quat, pos: Option<Vec3>| {
            if let Ok(mut tf) = tfs.get_mut(e) {
                tf.rotation = tf.rotation.slerp(rot, k);
                if let Some(p) = pos {
                    tf.translation = tf.translation.lerp(p, k);
                }
            }
        };
        let bob = if pl.grounded { (s * 2.0).sin().abs() * 0.035 * moving } else { 0.0 };
        let breathe = (t * 1.6).sin() * 0.008;
        let lean = moving * 0.12;
        let (mut leg_l, mut leg_r) = (Quat::from_rotation_x(swing), Quat::from_rotation_x(-swing));
        let (mut arm_l, mut arm_r) = (
            Quat::from_rotation_x(-swing * 0.8) * Quat::from_rotation_z(0.08),
            Quat::from_rotation_x(swing * 0.8) * Quat::from_rotation_z(-0.08),
        );
        let mut body_pos = Vec3::new(0.0, 0.42 + bob + breathe, 0.0);
        let mut body_rot = Quat::from_rotation_x(lean);
        let mut head_rot = Quat::IDENTITY;
        if !pl.grounded {
            leg_l = Quat::from_rotation_x(-0.5);
            leg_r = Quat::from_rotation_x(0.3);
            arm_l = Quat::from_rotation_z(0.9);
            arm_r = Quat::from_rotation_z(-0.9);
        }
        match pl.pose {
            Pose::None | Pose::Carry => {}
            Pose::Sit => {
                leg_l = Quat::from_rotation_x(-1.45);
                leg_r = Quat::from_rotation_x(-1.45);
                body_pos.y = 0.42 - 0.02;
                body_rot = Quat::from_rotation_x(-0.05);
                arm_l = Quat::from_rotation_x(-0.5);
                arm_r = Quat::from_rotation_x(-0.5);
            }
            Pose::Kneel => {
                leg_l = Quat::from_rotation_x(-1.3);
                leg_r = Quat::from_rotation_x(0.2);
                body_pos.y = 0.2;
                body_rot = Quat::from_rotation_x(0.25);
                arm_l = Quat::from_rotation_x(-0.9);
                arm_r = Quat::from_rotation_x(-0.9);
            }
            Pose::Wave => {
                arm_r = Quat::from_rotation_z(-2.6 + (t * 7.0).sin() * 0.35);
            }
            Pose::Reach => {
                arm_l = Quat::from_rotation_x(-1.3);
                arm_r = Quat::from_rotation_x(-1.3);
            }
            Pose::Clap => {
                let c = ((t * 9.0).sin() * 0.5 + 0.5) * 0.35;
                arm_l = Quat::from_rotation_x(-1.2) * Quat::from_rotation_z(-0.5 + c);
                arm_r = Quat::from_rotation_x(-1.2) * Quat::from_rotation_z(0.5 - c);
            }
            Pose::LookUp => {
                head_rot = Quat::from_rotation_x(-0.6);
            }
        }
        if pl.pose == Pose::Carry {
            arm_l = Quat::from_rotation_x(-1.1);
            arm_r = Quat::from_rotation_x(-1.1);
        }
        set(rig.leg_l, leg_l, None);
        set(rig.leg_r, leg_r, None);
        set(rig.arm_l, arm_l, None);
        set(rig.arm_r, arm_r, None);
        set(rig.body, body_rot, Some(body_pos));
        set(rig.head, head_rot, None);
        // scarf flutter
        for (i, e) in rig.scarf.iter().enumerate() {
            // at rest the tail hangs down his back and stirs a little; it lifts and
            // streams behind him as he walks / runs
            let fl = (t * (2.2 + moving * 4.8) + i as f32 * 0.9).sin() * (0.08 + moving * 0.3);
            let (rest, fly) = if i == 0 { (-0.55, 0.45) } else { (-0.22, -0.06) };
            let lift = rest + (fly - rest) * moving.clamp(0.0, 1.0) + (t * 1.3 + i as f32).sin() * 0.04;
            set(*e, Quat::from_rotation_x(lift) * Quat::from_rotation_y(fl), None);
        }
    }
}

/// The prince's night fill light (see `spawn_player`).
#[derive(Component)]
pub struct PrinceFill;

pub(crate) fn prince_fill(sky: Res<super::sky::Sky>, mut q: Query<&mut PointLight, With<PrinceFill>>) {
    for mut l in &mut q {
        l.intensity = 5500.0 * sky.night.clamp(0.0, 1.0);
    }
}
