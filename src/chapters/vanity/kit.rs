//! Small shared toolkit for the interior chapters (vanity, bar, geo): box-built
//! buildings whose visual mesh and colliders come from ONE layout table,
//! swinging doors (the original's VanityDoor / BarDoor), sitting on chairs
//! (ChairSeat / BarSeat), named-part NPC rigs posed from code, a prince pose
//! override applied on top of the core animation (shrug, head down, clap
//! arms), flickering lamps and camera-shot helpers with FOV.
//!
//! Lives in `chapters/vanity/` because chapter authors only own their own
//! directories; `bar` and `geo` import it from here.

use crate::core::camera::CameraRig;
use crate::core::player::PrinceRig;
use crate::core::prelude::*;
use std::collections::HashMap;

// ------------------------------------------------------------ buildings ----

/// Collision behaviour of a layout box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Col {
    /// Visual only.
    None,
    /// Blocks the player and the camera; its top can be walked on.
    Solid,
    /// Blocks the player, top not walkable (furniture), camera passes.
    Furniture,
    /// Blocks the player only (thin rails, posts); camera passes.
    NoCam,
}

/// One box of a building layout (local coordinates, centre + full size).
#[derive(Clone, Copy, Debug)]
pub struct Bx {
    pub c: Vec3,
    pub s: Vec3,
    pub color: Color,
    pub col: Col,
}

pub fn bx(c: Vec3, s: Vec3, color: Color) -> Bx {
    Bx { c, s, color, col: Col::Solid }
}

impl Bx {
    pub fn vis(mut self) -> Self {
        self.col = Col::None;
        self
    }
    pub fn furniture(mut self) -> Self {
        self.col = Col::Furniture;
        self
    }
    pub fn nocam(mut self) -> Self {
        self.col = Col::NoCam;
        self
    }
    /// Collider only (invisible).
    pub fn hidden(self) -> Self {
        Bx { color: Color::NONE, ..self }
    }
}

/// Box from min/max corners.
pub fn bmm(min: Vec3, max: Vec3, color: Color) -> Bx {
    bx((min + max) * 0.5, max - min, color)
}

/// Adds the visible boxes of a layout to a mesh.
pub fn mesh_boxes(m: &mut MeshBuilder, boxes: &[Bx]) {
    for b in boxes {
        if b.color.alpha() > 0.0 {
            m.cuboid(b.c, b.s, b.color);
        }
    }
}

/// Spawns the colliders of a layout placed at `origin` (no rotation).
pub fn spawn_colliders(cx: &mut ChapterCx, origin: Vec3, boxes: &[Bx]) {
    for b in boxes {
        let c = match b.col {
            Col::None => continue,
            Col::Solid => Collider::cuboid(b.s),
            Col::Furniture => Collider::cuboid(b.s).not_walkable().no_camera(),
            Col::NoCam => Collider::cuboid(b.s).no_camera(),
        };
        let pos = origin + b.c - Vec3::Y * b.s.y * 0.5;
        cx.spawn((Transform::from_translation(pos), c));
    }
}

/// A pitched roof slab between two eave lines rising to a ridge (along Z).
pub fn roof_slab(m: &mut MeshBuilder, eave_x: f32, eave_y: f32, ridge_y: f32, z0: f32, z1: f32, thick: f32, color: Color) {
    let run = eave_x.abs();
    let rise = ridge_y - eave_y;
    let len = (run * run + rise * rise).sqrt();
    let ang = rise.atan2(run);
    let side = eave_x.signum();
    let mid = Vec3::new(eave_x * 0.5, (eave_y + ridge_y) * 0.5, (z0 + z1) * 0.5);
    let rot = Quat::from_rotation_z(ang * -side);
    m.with(Transform::from_translation(mid).with_rotation(rot), |m| {
        m.cuboid(Vec3::ZERO, Vec3::new(len + 0.25, thick, (z1 - z0).abs()), color);
    });
}

/// Triangular gable (both faces) in the plane z = `z`.
pub fn gable(m: &mut MeshBuilder, half_w: f32, y0: f32, y1: f32, z: f32, color: Color) {
    let a = Vec3::new(-half_w, y0, z);
    let b = Vec3::new(half_w, y0, z);
    let c = Vec3::new(0.0, y1, z);
    m.tri(a, b, c, color);
    m.tri(b, a, c, color);
}

/// Interactable builder extension: the facing half-angle (original maxFacingAngle).
pub trait MaxAngle {
    fn max_angle(self, deg: f32) -> Self;
}
impl MaxAngle for Interactable {
    fn max_angle(mut self, deg: f32) -> Self {
        self.max_facing_angle = deg;
        self
    }
}

// --------------------------------------------------------------- doors ----

/// A swinging door leaf pivot (hinge). The leaf model and its collider are
/// children. `open` 0..1 eases toward `target` over `dur` seconds.
#[derive(Component, Debug, Clone)]
pub struct Door {
    pub id: &'static str,
    pub base_yaw: f32,
    /// +1 / -1: rotation sign that swings the leaf inward.
    pub sign: f32,
    pub max_deg: f32,
    /// Leaf length (m), for the "not through the player" sweep test.
    pub len: f32,
    pub open: f32,
    pub target: f32,
    pub dur: f32,
    /// World direction the leaf swings into (inward).
    pub inward: Vec2,
    pub blocked: bool,
}

impl Door {
    pub fn new(id: &'static str, base_yaw_deg: f32, sign: f32, max_deg: f32, len: f32, dur: f32, inward: Vec2) -> Self {
        Door { id, base_yaw: base_yaw_deg.to_radians(), sign, max_deg, len, open: 0.0, target: 0.0, dur, inward, blocked: false }
    }
    pub fn angle_deg(&self) -> f32 {
        ease(self.open) * self.max_deg
    }
    pub fn is_open(&self) -> bool {
        self.target > 0.5
    }
}

pub fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    ease((x - a) / (b - a))
}

/// Spawns a door hinge at `hinge` (world), the leaf model `key` offset by
/// `leaf_offset` (local, the leaf's centre), and a box collider of `size`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_door(cx: &mut ChapterCx, door: Door, hinge: Vec3, key: &str, leaf_offset: Vec3, size: Vec3) -> Entity {
    let pivot = cx.empty(hinge, door.base_yaw.to_degrees());
    cx.world.entity_mut(pivot).insert(door);
    let leaf = cx.model(key, leaf_offset, 0.0, 1.0);
    cx.world.entity_mut(pivot).add_child(leaf);
    let col = cx.spawn((Transform::from_translation(Vec3::new(leaf_offset.x, 0.0, leaf_offset.z)), Collider::cuboid(size).no_camera()));
    cx.world.entity_mut(pivot).add_child(col);
    pivot
}

/// Animates every door; a leaf never swings through the player.
pub fn animate_doors(time: Res<Time>, mut q: Query<(&mut Door, &mut Transform, &GlobalTransform), Without<Player>>, player: Query<&Transform, With<Player>>) {
    let dt = time.delta_secs();
    let pp = player.single().map(|t| t.translation).ok();
    for (mut d, mut tf, gt) in &mut q {
        if (d.open - d.target).abs() < 1e-4 {
            d.blocked = false;
        } else {
            let dir = (d.target - d.open).signum();
            // leaf sweep test: the player stands inside the quarter-disc the leaf sweeps
            let mut blocked = false;
            if let Some(p) = pp {
                let h = gt.translation();
                let rel = Vec2::new(p.x - h.x, p.z - h.z);
                if rel.length() < d.len + 0.25 && rel.dot(d.inward) > -0.15 && d.open > 0.03 {
                    blocked = true;
                }
            }
            d.blocked = blocked;
            if !blocked {
                d.open = (d.open + dir * dt / d.dur.max(0.05)).clamp(0.0, 1.0);
                if (dir > 0.0 && d.open > d.target) || (dir < 0.0 && d.open < d.target) {
                    d.open = d.target;
                }
            }
        }
        tf.rotation = Quat::from_rotation_y(d.base_yaw + d.sign * d.angle_deg().to_radians());
    }
}

// ------------------------------------------------------------- seating ----

/// Seats the prince: places him on `seat` (world, seat height), faces `yaw_deg`, sit pose.
pub fn sit_player(world: &mut World, seat: Vec3, yaw_deg: f32) {
    let mut q = world.query::<(&mut Player, &mut Transform)>();
    if let Some((mut pl, mut tf)) = q.iter_mut(world).next() {
        tf.translation = seat;
        pl.yaw = yaw_deg.to_radians();
        pl.face_target = None;
        pl.velocity = Vec3::ZERO;
        pl.pose = Pose::Sit;
        pl.drive = Drive::External;
    }
}

/// Stands the prince up at `pos` (ground) facing `yaw_deg`.
pub fn stand_player(world: &mut World, pos: Vec3, yaw_deg: f32) {
    let mut q = world.query::<(&mut Player, &mut Transform)>();
    if let Some((mut pl, mut tf)) = q.iter_mut(world).next() {
        tf.translation = pos;
        pl.yaw = yaw_deg.to_radians();
        pl.face_target = None;
        pl.velocity = Vec3::ZERO;
        pl.pose = Pose::None;
        pl.drive = Drive::Player;
        pl.grounded = false;
    }
}

pub fn set_drive(world: &mut World, drive: Drive) {
    let mut q = world.query::<&mut Player>();
    if let Some(mut pl) = q.iter_mut(world).next() {
        pl.drive = drive;
    }
}

pub fn face_yaw(world: &mut World, yaw_deg: f32) {
    let mut q = world.query::<&mut Player>();
    if let Some(mut pl) = q.iter_mut(world).next() {
        pl.face_target = Some(yaw_deg.to_radians());
    }
}

pub fn player_pos(world: &mut World) -> Option<Vec3> {
    let mut q = world.query_filtered::<&Transform, With<Player>>();
    q.iter(world).next().map(|t| t.translation)
}

pub fn player_seated(world: &mut World) -> bool {
    let mut q = world.query::<&Player>();
    q.iter(world).next().map(|p| p.pose == Pose::Sit).unwrap_or(false)
}

// --------------------------------------------------------------- shots ----

/// Begins a camera shot with an explicit FOV (story `shot` has none).
pub fn shot_fov(world: &mut World, pos: Vec3, look: Vec3, blend: f32, fov: f32) {
    world.resource_mut::<CameraRig>().begin_shot(Shot { position: pos, look_at: look, blend, fov: Some(fov) });
}

/// Script command: camera shot with FOV.
pub fn shot_cmd(pos: Vec3, look: Vec3, blend: f32, fov: f32) -> Cmd {
    run(move |w: &mut World| shot_fov(w, pos, look, blend, fov))
}

/// Script command: cut to `a`, then push toward `b` over `secs`.
pub fn push_cmd(a: (Vec3, Vec3, f32), b: (Vec3, Vec3, f32), secs: f32) -> Cmd {
    run(move |w: &mut World| {
        shot_fov(w, a.0, a.1, 0.0, a.2);
        // the cut lands this frame; the push blends from it
        let mut rig = w.resource_mut::<CameraRig>();
        rig.last_pos = a.0;
        rig.last_look = a.1;
        rig.begin_shot(Shot { position: b.0, look_at: b.1, blend: secs, fov: Some(b.2) });
    })
}

/// Ends a shot, first turning the follow orbit to `yaw_deg` / `pitch_deg`.
pub fn end_shot_to(world: &mut World, yaw_deg: Option<f32>, pitch_deg: Option<f32>, blend: f32) {
    let mut rig = world.resource_mut::<CameraRig>();
    if let Some(y) = yaw_deg {
        rig.yaw = y.to_radians();
    }
    if let Some(p) = pitch_deg {
        rig.pitch = p.to_radians();
    }
    rig.end_shot(blend);
}

/// Yaw (deg) looking from `from` to `to`.
pub fn yaw_to(from: Vec3, to: Vec3) -> f32 {
    let d = to - from;
    d.x.atan2(d.z).to_degrees()
}

// ----------------------------------------------------------- NPC rigs ----

/// A procedural NPC whose named parts are posed from code. Parts are found
/// lazily among the model's children (a glTF replacement simply has none).
#[derive(Component, Default, Debug)]
pub struct Rig {
    pub parts: HashMap<String, (Entity, Transform)>,
}

impl Rig {
    pub fn get(&self, name: &str) -> Option<(Entity, Transform)> {
        self.parts.get(name).copied()
    }
}

pub fn bind_rigs(mut rigs: Query<(&mut Rig, &Children)>, parts: Query<(&PartName, &Transform), Without<Rig>>) {
    for (mut rig, children) in &mut rigs {
        if !rig.parts.is_empty() {
            continue;
        }
        for c in children.iter() {
            if let Ok((n, t)) = parts.get(c) {
                rig.parts.insert(n.0.clone(), (c, *t));
            }
        }
    }
}

/// Transforms of rig parts (never the player).
pub type PartTfs<'w, 's> = Query<'w, 's, &'static mut Transform, (Without<Rig>, Without<Player>)>;

/// Applies a rotation (relative to the part's rest) and a translation offset.
pub fn pose_part(tfs: &mut PartTfs, rig: &Rig, name: &str, rot: Quat, offset: Vec3) {
    if let Some((e, base)) = rig.get(name) {
        if let Ok(mut t) = tfs.get_mut(e) {
            t.rotation = base.rotation * rot;
            t.translation = base.translation + offset;
        }
    }
}

/// Places a prop part at an absolute local transform.
pub fn place_part(tfs: &mut PartTfs, rig: &Rig, name: &str, tf: Transform) {
    if let Some((e, _)) = rig.get(name) {
        if let Ok(mut t) = tfs.get_mut(e) {
            *t = tf;
        }
    }
}

/// Head look toward a world point within range, clamped (VanityPerformerLook style).
pub fn head_look(npc_tf: &GlobalTransform, head_local: Vec3, target: Vec3, range: f32, max_yaw: f32, pitch: (f32, f32)) -> Quat {
    let head_w = npc_tf.transform_point(head_local);
    let d = target - head_w;
    if d.length() > range || d.length() < 0.05 {
        return Quat::IDENTITY;
    }
    let inv = npc_tf.affine().inverse();
    let l = inv.transform_vector3(d);
    let yaw = l.x.atan2(l.z).to_degrees().clamp(-max_yaw, max_yaw);
    let horiz = Vec2::new(l.x, l.z).length();
    let pit = (-l.y).atan2(horiz).to_degrees().clamp(-pitch.1, -pitch.0);
    Quat::from_rotation_y(yaw.to_radians()) * Quat::from_rotation_x(pit.to_radians())
}

/// Arm builder: a sleeve hanging down from the shoulder pivot (−Y) with a hand.
pub fn arm_mesh(len: f32, r0: f32, r1: f32, sleeve: Color, skin: Color) -> MeshBuilder {
    let mut a = MeshBuilder::new();
    a.frustum(Vec3::new(0.0, -len, 0.0), r1, r0, len, 7, sleeve);
    a.sphere(Vec3::new(0.0, -len - 0.04, 0.0), r1 * 0.95, 3, 6, skin);
    a
}

// ------------------------------------------------------ prince override ----

/// Pose overrides applied on top of the core prince animation (each frame,
/// after `animate_prince`). Chapters set fields; `None` = core animation.
#[derive(Resource, Default, Debug, Clone)]
pub struct PrinceOverride {
    pub arm_l: Option<Quat>,
    pub arm_r: Option<Quat>,
    pub head: Option<Quat>,
    pub body: Option<Quat>,
}

pub fn apply_prince_override(ov: Res<PrinceOverride>, q: Query<&PrinceRig>, mut tfs: Query<&mut Transform>) {
    let Ok(rig) = q.single() else { return };
    for (e, r) in [(rig.arm_l, ov.arm_l), (rig.arm_r, ov.arm_r), (rig.head, ov.head), (rig.body, ov.body)] {
        if let (Some(r), Ok(mut t)) = (r, tfs.get_mut(e)) {
            t.rotation = r;
        }
    }
}

/// Clap arm pose (shared by vanity's clap clip). `t` = seconds into the
/// 2.03 s clip; returns (left, right) shoulder rotations.
pub fn clap_arms(t: f32, contacts: &[f32]) -> (Quat, Quat) {
    let raise = smooth(0.0, 0.3, t) * (1.0 - smooth(1.65, 2.03, t));
    // hands apart between contacts, together on each contact
    let mut close = 0.0f32;
    for c in contacts {
        let d = (t - c).abs();
        close = close.max(1.0 - (d / 0.13).min(1.0));
    }
    let open = 0.55 - 0.45 * close;
    let l = Quat::from_rotation_x(-1.25 * raise) * Quat::from_rotation_z((-open) * raise + 0.08 * (1.0 - raise));
    let r = Quat::from_rotation_x(-1.25 * raise) * Quat::from_rotation_z(open * raise - 0.08 * (1.0 - raise));
    (l, r)
}

// ---------------------------------------------------------------- lamps ----

/// Flickering point light (warm lamps, candles, fireplace).
#[derive(Component, Debug, Clone)]
pub struct Flicker {
    pub base: f32,
    pub amount: f32,
    pub speed: f32,
    pub seed: f32,
    /// Extra multiplier set by chapter code (door opening brightens the bar).
    pub scale: f32,
}

impl Flicker {
    pub fn new(base: f32, amount: f32, speed: f32, seed: f32) -> Self {
        Flicker { base, amount, speed, seed, scale: 1.0 }
    }
}

pub fn flicker_lights(time: Res<Time>, mut q: Query<(&Flicker, &mut PointLight)>) {
    let t = time.elapsed_secs();
    for (f, mut l) in &mut q {
        let n = (t * f.speed + f.seed).sin() * 0.5 + (t * f.speed * 2.3 + f.seed * 1.7).sin() * 0.3 + (t * f.speed * 5.1 + f.seed * 3.1).sin() * 0.2;
        l.intensity = f.base * f.scale * (1.0 + n * f.amount);
    }
}

/// A flickering warm light, scoped to the chapter.
pub fn lamp(cx: &mut ChapterCx, pos: Vec3, color: Color, intensity: f32, range: f32, flicker: f32, seed: f32) -> Entity {
    let e = cx.point_light(pos, color, intensity, range);
    cx.world.entity_mut(e).insert(Flicker::new(intensity, flicker, 7.0, seed));
    e
}

/// Registers the kit systems for one chapter.
pub fn register(app: &mut App, id: &'static str) {
    app.init_resource::<PrinceOverride>();
    app.add_systems(Update, (bind_rigs, animate_doors, flicker_lights).in_set(CoreSet::Gameplay).run_if(in_chapter(id)));
    app.add_systems(Update, apply_prince_override.in_set(CoreSet::Presentation).after(crate::core::player::animate_prince).run_if(in_chapter(id)));
}
