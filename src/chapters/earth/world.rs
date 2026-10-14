//! The two Earth scenes, laid out from the original's anchor positions
//! (docs/original/earth.md §1.1 / §2.1) scaled by 0.5 onto a domed planet:
//! Valley (snake → flower → summit → rose garden → fox → station) and
//! Dunes (train → plane camp → big dune → well → sacred way → doorway).

use super::actors::{Fox, Pilot, PilotPose, Snake, SnakeState};
use super::state::{EarthState, Leg, Scene, Sign, ValleyPart};
use crate::core::chapter::ChapterCx;
use crate::core::meshkit::MeshBuilder;
use crate::core::models::SharedMaterials;
use crate::core::prelude::*;
use crate::core::story::Anchors;

/// Original layout scale (original metres → ours).
pub const K: f32 = 0.5;

// ------------------------------------------------------------- valley ----
pub const V_SPAWN: Vec2 = Vec2::new(-15.5, -32.5);
pub const V_SNAKE: Vec2 = Vec2::new(-17.1, -31.9);
pub const V_FLOWER: Vec2 = Vec2::new(-16.2, -21.3);
pub const V_SUMMIT: Vec2 = Vec2::new(-16.1, -9.3);
pub const V_GARDEN: Vec2 = Vec2::new(-5.35, 4.8);
pub const V_ROSEBED: Vec2 = Vec2::new(-5.4, 6.6);
pub const V_FOUNTAIN: Vec2 = Vec2::new(-2.4, 4.6);
pub const V_FOX: Vec2 = Vec2::new(6.5, 15.6);
pub const V_TRUNK: Vec2 = Vec2::new(6.9, 17.4);
pub const V_BRIDGE: Vec2 = Vec2::new(3.3, 11.2);
pub const V_BOARD: Vec2 = Vec2::new(10.2, 31.9);
pub const V_RAIL_Z: f32 = 33.3;
/// Garden walls: x ∈ [GX0, GX1], z ∈ [GZ0, GZ1].
const GX0: f32 = -9.0;
const GX1: f32 = 2.0;
const GZ0: f32 = 1.0;
const GZ1: f32 = 10.0;
/// The stream runs along x + z = STREAM_C.
const STREAM_C: f32 = 14.5;

/// The walked route (for paths, scatter avoidance and guide cairns).
const V_ROUTE: &[Vec2] = &[
    Vec2::new(-15.5, -32.0),
    Vec2::new(-16.3, -21.5),
    Vec2::new(-16.1, -9.3),
    Vec2::new(-12.6, -2.0),
    Vec2::new(-10.4, 4.6),
    Vec2::new(-5.4, 4.6),
    Vec2::new(-2.0, 8.6),
    Vec2::new(2.6, 7.0),
    Vec2::new(3.3, 11.2),
    Vec2::new(6.0, 14.0),
    Vec2::new(7.2, 22.0),
    Vec2::new(9.2, 29.5),
];

// -------------------------------------------------------------- dunes ----
pub const D_STEPOFF: Vec2 = Vec2::new(-30.1, 4.2);
pub const D_RAIL_Z: f32 = 6.0;
pub const D_TRAIN_STOP_X: f32 = -17.5;
pub const D_PLANE: Vec2 = Vec2::new(-23.5, -4.0);
pub const D_BEDROLL: Vec2 = Vec2::new(-25.9, -4.3);
pub const D_PILOT_YAW: f32 = 334.0;
pub const D_CRATE: Vec2 = Vec2::new(-21.0, -0.7);
pub const D_DUNE_TOP: Vec2 = Vec2::new(-11.2, 10.7);
pub const D_WELL: Vec2 = Vec2::new(2.0, 4.2);
pub const D_WAY_START: Vec2 = Vec2::new(12.8, 1.4);
pub const D_DOOR: Vec2 = Vec2::new(22.7, -4.3);
/// Sacred-way axis (yaw 120°).
pub const WAY_YAW: f32 = 120.0;
pub const D_LANDING_TOP: f32 = 0.75;
pub const D_CREST: Vec2 = Vec2::new(30.25, -8.75);
/// Evening star direction (game azimuth / elevation, degrees).
pub const STAR_AZ: f32 = 120.4;
pub const STAR_EL: f32 = 26.7;

pub fn way_dir() -> Vec2 {
    let a = WAY_YAW.to_radians();
    Vec2::new(a.sin(), a.cos())
}
pub fn way_normal() -> Vec2 {
    let d = way_dir();
    Vec2::new(-d.y, d.x)
}
pub fn d_farewell() -> Vec2 {
    D_DOOR - way_dir() * 4.0
}
pub fn d_snake_spot() -> Vec2 {
    D_DOOR - way_dir() * 1.05
}
pub fn yaw_dir(yaw_deg: f32) -> Vec2 {
    let a = yaw_deg.to_radians();
    Vec2::new(a.sin(), a.cos())
}
pub fn yaw_to(from: Vec2, to: Vec2) -> f32 {
    let d = to - from;
    d.x.atan2(d.y).to_degrees()
}

/// Resolves `Flatten { height: None }` to explicit heights (sampled with the
/// mods before it). Works around core `Terrain::new`, which indexes
/// `flat_heights` before filling it when a planet has 2+ auto-height flattens.
fn resolve_flats(p: &mut PlanetParams) {
    for i in 0..p.mods.len() {
        if let TerrainMod::Flatten { center, height: None, .. } = p.mods[i] {
            let prior = PlanetParams { mods: p.mods[..i].to_vec(), ..p.clone() };
            let h = Terrain::new(prior).height(center.x, center.y);
            if let TerrainMod::Flatten { height, .. } = &mut p.mods[i] {
                *height = Some(h);
            }
        }
    }
}

fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

fn route_dist(route: &[Vec2], p: Vec2) -> f32 {
    route.windows(2).map(|w| seg_dist(p, w[0], w[1])).fold(f32::MAX, f32::min)
}

/// Places `n` copies of a model where `ok(p)` holds (uniform over a disc).
fn scatter_where(cx: &mut ChapterCx, key: &str, n: u32, seed: u64, scale: (f32, f32), radius: f32, ok: impl Fn(Vec2) -> bool) -> Vec<Entity> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::new();
    let mut tries = 0;
    while (out.len() as u32) < n && tries < n * 40 {
        tries += 1;
        let a = rng.range(0.0, std::f32::consts::TAU);
        let r = rng.f().sqrt() * radius;
        let p = Vec2::new(a.sin() * r, a.cos() * r);
        if !ok(p) {
            continue;
        }
        let s = rng.range(scale.0, scale.1);
        let g = cx.ground(p.x, p.y);
        out.push(cx.model(key, g, rng.range(0.0, 360.0), s));
    }
    out
}

/// A wall run of 2 m segments from `a` to `b` with one box collider.
fn wall_run(cx: &mut ChapterCx, key: &str, a: Vec2, b: Vec2, height: f32, thick: f32) {
    let d = b - a;
    let len = d.length();
    let n = (len / 2.0).ceil().max(1.0) as i32;
    let yaw = d.x.atan2(d.y).to_degrees() - 90.0; // model length runs along local X
    for i in 0..n {
        let t = (i as f32 + 0.5) / n as f32;
        let p = a + d * t;
        let g = cx.ground(p.x, p.y);
        let e = cx.model(key, g - Vec3::Y * 0.05, yaw, 1.0);
        if let Some(mut tf) = cx.world.get_mut::<Transform>(e) {
            tf.scale = Vec3::new(len / n as f32 / 2.0 + 0.02, 1.0, 1.0);
        }
    }
    let c = a + d * 0.5;
    let g = cx.ground(c.x, c.y);
    let e = cx.empty(g - Vec3::Y * 0.2, yaw);
    cx.collider(e, Collider::cuboid(Vec3::new(len, height + 0.2, thick)));
}

fn box_collider(cx: &mut ChapterCx, at: Vec3, yaw: f32, size: Vec3) -> Entity {
    let e = cx.empty(at, yaw);
    cx.collider(e, Collider::cuboid(size));
    e
}

/// Spawns a terrain-following ribbon (water) along a polyline.
fn ribbon(cx: &mut ChapterCx, pts: &[Vec2], width: f32, lift: f32, color: Color) {
    let Some(t) = cx.terrain().cloned() else { return };
    let mut m = MeshBuilder::new();
    let mut samples = Vec::new();
    for w in pts.windows(2) {
        let n = (w[0].distance(w[1]) / 0.8).ceil().max(1.0) as i32;
        for i in 0..n {
            samples.push(w[0].lerp(w[1], i as f32 / n as f32));
        }
    }
    samples.push(*pts.last().unwrap());
    for i in 0..samples.len() - 1 {
        let (a, b) = (samples[i], samples[i + 1]);
        let dir = (b - a).normalize_or_zero();
        let n = Vec2::new(-dir.y, dir.x) * width * 0.5;
        let y = |p: Vec2| t.height(p.x, p.y) + lift;
        let (a0, a1, b0, b1) = (a - n, a + n, b - n, b + n);
        m.quad(Vec3::new(a0.x, y(a), a0.y), Vec3::new(a1.x, y(a), a1.y), Vec3::new(b1.x, y(b), b1.y), Vec3::new(b0.x, y(b), b0.y), color);
        m.quad(Vec3::new(a0.x, y(a), a0.y), Vec3::new(b0.x, y(b), b0.y), Vec3::new(b1.x, y(b), b1.y), Vec3::new(a1.x, y(a), a1.y), color);
    }
    let mesh = cx.world.resource_mut::<Assets<Mesh>>().add(m.build());
    let mat = cx.world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.75, 0.9, 0.78),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        reflectance: 0.5,
        ..default()
    });
    cx.spawn((Mesh3d(mesh), MeshMaterial3d(mat), Transform::default(), bevy::light::NotShadowCaster));
}

/// Rail segments along +X at `z`, following the ground (pitched per segment).
fn rail_line(cx: &mut ChapterCx, z: f32, x0: f32, x1: f32, max_r: f32) {
    let mut x = x0;
    while x < x1 {
        let (a, b) = (cx.ground(x, z), cx.ground(x + 2.0, z));
        let mid = (a + b) * 0.5;
        if Vec2::new(mid.x, mid.z).length() < max_r {
            let pitch = ((b.y - a.y) / 2.0).atan();
            let e = cx.model("model.earth.rail", mid - Vec3::Y * 0.04, 90.0, 1.0);
            if let Some(mut tf) = cx.world.get_mut::<Transform>(e) {
                tf.rotation = Quat::from_rotation_y(90f32.to_radians()) * Quat::from_rotation_x(-pitch);
            }
        }
        x += 2.0;
    }
}

/// Train: locomotive at `front_x` heading +X, two carriages behind. Returns the entities.
pub fn spawn_train(cx: &mut ChapterCx, front_x: f32, z: f32, y: f32) -> Vec<Entity> {
    let mut v = Vec::new();
    let loco_c = front_x - 3.15;
    let c1 = loco_c - 3.15 - 3.25 - 0.1;
    let c2 = c1 - 6.6;
    v.push(cx.model("model.earth.loco", Vec3::new(loco_c, y, z), 90.0, 1.0));
    v.push(cx.model("model.earth.carriage", Vec3::new(c1, y, z), 90.0, 1.0));
    v.push(cx.model("model.earth.carriage", Vec3::new(c2, y, z), 90.0, 1.0));
    for e in &v {
        cx.world.entity_mut(*e).insert(super::state::TrainCar);
    }
    // the train blocks the player (one box per car)
    for (cxp, len) in [(loco_c, 6.3), (c1, 6.4), (c2, 6.4)] {
        let e = box_collider(cx, Vec3::new(cxp, y, z), 90.0, Vec3::new(2.2, 3.0, len));
        cx.world.entity_mut(e).insert(super::state::TrainCar);
    }
    v
}

fn signpost(cx: &mut ChapterCx, at: Vec2, toward: Vec2) {
    let g = cx.ground(at.x, at.y);
    let e = cx.model("model.earth.signpost", g, yaw_to(at, toward), 1.0);
    cx.collider(e, Collider::cylinder(0.12, 1.5).no_camera());
}

// ===================================================================== VALLEY

pub fn build_valley(cx: &mut ChapterCx) {
    // Station corridor height: sample the base dome first.
    let mut p = PlanetParams::sized(40.0);
    p.ground = hex(0xe3c08b);
    p.ground2 = hex(0xd5ae78);
    p.dry = hex(0xc8a273);
    p.rock = hex(0xa07b58);
    p.noise_amp = 0.28;
    p.noise_wavelength = 9.0;
    p.seed = 81;
    p.underside_depth = 22.0;
    let base = Terrain::new(p.clone());
    let h_station = base.height(9.0, 31.5);
    let mut mods = vec![
        TerrainMod::Mound { center: Vec2::new(-14.0, -11.0), radius: 14.0, height: 3.6 },
        TerrainMod::Mound { center: Vec2::new(-28.0, -6.0), radius: 10.0, height: 3.2 },
        TerrainMod::Mound { center: Vec2::new(-24.0, 14.0), radius: 9.0, height: 2.4 },
        TerrainMod::Mound { center: Vec2::new(22.0, -18.0), radius: 10.0, height: 1.6 },
        TerrainMod::Mound { center: Vec2::new(-2.0, -26.0), radius: 8.0, height: 1.2 },
        TerrainMod::Flatten { center: Vec2::new(-16.0, -32.2), radius: 2.5, blend: 2.0, height: None },
        TerrainMod::Flatten { center: V_SUMMIT, radius: 1.4, blend: 1.6, height: None },
        TerrainMod::Flatten { center: Vec2::new(-3.5, 5.5), radius: 7.0, blend: 3.0, height: None },
        TerrainMod::Flatten { center: V_FOX, radius: 3.5, blend: 2.5, height: None },
    ];
    for i in 0..7 {
        mods.push(TerrainMod::Flatten { center: Vec2::new(-6.0 + i as f32 * 4.0, 32.4), radius: 2.6, blend: 2.5, height: Some(h_station) });
    }
    // stream channel along x + z = STREAM_C
    let s0 = Vec2::new(-12.5, STREAM_C + 12.5);
    let s1 = Vec2::new(30.0, STREAM_C - 30.0);
    let n = 34;
    for i in 0..=n {
        let c = s0.lerp(s1, i as f32 / n as f32);
        mods.push(TerrainMod::Mound { center: c, radius: 1.5, height: -0.22 });
    }
    // colours: meadow north, lawn, clearing, paths
    mods.push(TerrainMod::Paint { center: Vec2::new(4.0, 14.0), radius: 14.0, blend: 7.0, color: hex(0x9fb46a) });
    mods.push(TerrainMod::Paint { center: Vec2::new(-14.0, 18.0), radius: 9.0, blend: 6.0, color: hex(0xa7b56e) });
    mods.push(TerrainMod::Paint { center: Vec2::new(-3.5, 5.5), radius: 5.5, blend: 1.5, color: hex(0x87ad5a) });
    mods.push(TerrainMod::Paint { center: V_FOX, radius: 3.6, blend: 1.5, color: hex(0x9cc06a) });
    for i in 0..=n {
        let c = s0.lerp(s1, i as f32 / n as f32);
        mods.push(TerrainMod::Paint { center: c, radius: 1.3, blend: 0.8, color: hex(0x6f8f58) });
    }
    for w in V_ROUTE.windows(2) {
        let k = (w[0].distance(w[1]) / 1.4).ceil() as i32;
        for i in 0..k {
            let c = w[0].lerp(w[1], i as f32 / k as f32);
            let green = c.y > -3.0;
            mods.push(TerrainMod::Paint { center: c, radius: 0.7, blend: 0.6, color: if green { hex(0xc2ab7a) } else { hex(0xcfa76f) } });
        }
    }
    p.mods = mods;
    resolve_flats(&mut p);
    cx.planet(p);
    let mut pal = SkyPalette::default();
    pal.day_horizon = hex(0xe8dcc2);
    pal.day_zenith = hex(0x5a8fd6);
    pal.fog = 0.7;
    cx.sky(10.5, 0.0, None, pal);
    cx.ambience("amb.wind");

    let anchor = |cx: &mut ChapterCx, name: &str, p: Vec2, yaw: f32| {
        let g = cx.ground(p.x, p.y);
        cx.anchor(name, g, yaw);
    };

    // ---- V1: snake on its rock beside the landing spot
    let rock_p = cx.ground(V_SNAKE.x, V_SNAKE.y);
    cx.model("model.earth.snake_rock", rock_p, 30.0, 1.0);
    let post = cx.empty(rock_p, 0.0);
    cx.collider(post, Collider::cylinder(0.62 + 0.05, 1.6).no_camera().not_walkable());
    let snake_yaw = yaw_to(V_SNAKE, V_SPAWN + Vec2::new(0.0, 1.5));
    let snake = cx.model("model.earth.snake", rock_p + Vec3::Y * 0.3, snake_yaw, 1.0);
    cx.world.entity_mut(snake).insert(Snake::new(SnakeState::Coiled, snake_yaw));
    cx.interactable(snake, Interactable::new("snake", "earth.verb.snake").radius(2.2).focus(Vec3::Y * 0.35).prompt_height(0.5));
    anchor(cx, "snake", V_SNAKE, snake_yaw);

    // ---- V-a little flower, V-b summit echo
    let fp = cx.ground(V_FLOWER.x, V_FLOWER.y);
    let flower = cx.model("model.earth.flower", fp, 20.0, 1.0);
    cx.interactable(flower, Interactable::new("flower", "earth.verb.talk").radius(1.9).focus(Vec3::Y * 0.25).prompt_height(0.4).disabled());
    anchor(cx, "flower", V_FLOWER, 0.0);
    let sp = cx.ground(V_SUMMIT.x, V_SUMMIT.y);
    let echo = cx.empty(sp, 0.0);
    cx.interactable(echo, Interactable::new("echo", "earth.verb.call").radius(3.5).focus(Vec3::Y * 0.6).prompt_height(0.8).any_facing().disabled());
    cx.anchor("summit", sp, yaw_to(V_SUMMIT, V_GARDEN));

    // the echo mountains: rock spires west of the pass
    for (x, z, s, yaw) in [(-27.5f32, -12.0f32, 1.25f32, 10.0f32), (-25.5, -1.5, 1.0, 70.0), (-31.0, -20.0, 0.95, 140.0), (-23.0, 7.5, 0.75, 200.0), (-33.5, 3.0, 1.35, 260.0), (-21.0, -19.0, 0.6, 300.0), (-7.0, -16.5, 0.45, 40.0), (-31.0, 12.0, 0.9, 30.0)] {
        let g = cx.ground(x, z);
        let e = cx.model("model.earth.peak", g - Vec3::Y * 0.6, yaw, s);
        cx.collider(e, Collider::cylinder(3.6 * s, 10.0 * s));
    }

    // ---- V2 rose garden
    wall_run(cx, "model.earth.garden_wall", Vec2::new(GX0, GZ0), Vec2::new(GX1, GZ0), 1.0, 0.45);
    wall_run(cx, "model.earth.garden_wall", Vec2::new(GX0, GZ1), Vec2::new(GX1, GZ1), 1.0, 0.45);
    wall_run(cx, "model.earth.hedge", Vec2::new(GX0, GZ0), Vec2::new(GX0, 3.6), 1.0, 0.6);
    wall_run(cx, "model.earth.hedge", Vec2::new(GX0, 5.6), Vec2::new(GX0, GZ1), 1.0, 0.6);
    wall_run(cx, "model.earth.hedge", Vec2::new(GX1, GZ0), Vec2::new(GX1, 6.0), 1.0, 0.6);
    wall_run(cx, "model.earth.hedge", Vec2::new(GX1, 8.0), Vec2::new(GX1, GZ1), 1.0, 0.6);
    let gp = cx.ground(GX0, 4.6);
    cx.model("model.earth.gate", gp, 90.0, 1.0);
    let ap = cx.ground(GX1, 7.0);
    cx.model("model.earth.arch", ap, 90.0, 1.0);
    let fpos = cx.ground(V_FOUNTAIN.x, V_FOUNTAIN.y);
    let f = cx.model("model.earth.fountain", fpos, 0.0, 1.0);
    cx.collider(f, Collider::cylinder(1.05, 0.5));
    for (i, (x, z)) in [(V_ROSEBED.x, V_ROSEBED.y), (-5.4, 2.6), (-0.6, 8.3), (-0.6, 2.4), (-7.4, 8.7)].iter().enumerate() {
        let g = cx.ground(*x, *z);
        let bed = cx.model("model.earth.rose_bed", g, if i == 4 { 0.0 } else { 0.0 }, 1.0);
        cx.collider(bed, Collider::cuboid(Vec3::new(1.9, 0.55, 1.0)).no_camera());
        if i == 0 {
            cx.interactable(bed, Interactable::new("roses", "earth.verb.roses").radius(2.4).focus(Vec3::Y * 0.5).prompt_height(0.4).disabled());
        }
    }
    anchor(cx, "garden", V_GARDEN, 0.0);
    anchor(cx, "roses_look", V_ROSEBED, 0.0);
    for (x, z) in [(-8.3, 3.2), (-8.3, 6.0)] {
        let g = cx.ground(x, z);
        let e = cx.model("model.earth.urn", g, 0.0, 1.0);
        cx.collider(e, Collider::cylinder(0.27, 0.8).no_camera());
    }
    for (x, z) in [(-8.2, 9.2), (1.2, 1.8), (1.2, 9.2), (-8.2, 1.8)] {
        let g = cx.ground(x, z);
        let e = cx.model("model.earth.topiary", g, 0.0, 1.0);
        cx.collider(e, Collider::cylinder(0.3, 1.0).no_camera());
    }
    // fallen petals and roses on the lawn
    let mut rng = Rng::new(301);
    for _ in 0..18 {
        let x = rng.range(GX0 + 0.8, GX1 - 0.8);
        let z = rng.range(GZ0 + 0.8, GZ1 - 0.8);
        let g = cx.ground(x, z);
        cx.model("model.flower_red", g, rng.range(0.0, 360.0), rng.range(1.0, 1.5));
    }

    // ---- stream, waterfall, bridge
    ribbon(cx, &[s0, s1], 1.5, -0.02, hex(0x7fb6d6));
    let wf = cx.ground(-13.2, STREAM_C + 13.2);
    let wfe = cx.model("model.earth.waterfall", wf - Vec3::Y * 0.3, 135.0, 1.3);
    cx.collider(wfe, Collider::cylinder(2.2, 3.0));
    let bdir = Vec2::new(1.0, 1.0).normalize();
    let bank = V_BRIDGE - bdir * 1.6;
    let by = cx.ground(bank.x, bank.y).y.max(cx.ground(V_BRIDGE.x + bdir.x * 1.6, V_BRIDGE.y + bdir.y * 1.6).y);
    let bp = Vec3::new(V_BRIDGE.x, by - 0.12, V_BRIDGE.y);
    cx.model("model.earth.bridge", bp, 45.0, 1.0);
    box_collider(cx, bp - Vec3::Y * 0.4, 45.0, Vec3::new(1.2, 0.6, 3.4));
    for s in [-1.0f32, 1.0] {
        let rail = bp + Vec3::new(-bdir.y * 0.62 * s, 0.0, bdir.x * 0.62 * s);
        let e = cx.empty(rail, 45.0);
        cx.collider(e, Collider::cuboid(Vec3::new(0.1, 1.0, 3.4)).no_camera().not_walkable());
    }
    // water blocks except at the bridge
    let sdir = (s1 - s0).normalize();
    let cross = V_BRIDGE;
    for (a, b) in [(s0, cross - sdir * 1.0), (cross + sdir * 1.0, s1)] {
        let c = (a + b) * 0.5;
        let g = cx.ground(c.x, c.y);
        let e = cx.empty(g - Vec3::Y * 0.3, yaw_to(a, b));
        cx.collider(e, Collider::cuboid(Vec3::new(1.1, 1.0, a.distance(b))).no_camera().not_walkable());
    }

    // ---- V3 fox clearing: apple tree, wheat field, cottages, plots
    let tp = cx.ground(V_TRUNK.x, V_TRUNK.y);
    let tree = cx.model("model.earth.apple_tree", tp, 0.0, 1.0);
    cx.collider(tree, Collider::cylinder(0.35, 2.2));
    anchor(cx, "fox_tree", V_TRUNK, 0.0);
    let fox_p = cx.ground(V_FOX.x, V_FOX.y);
    let fox = cx.model("model.earth.fox", fox_p, 79.3, 1.0);
    cx.world.entity_mut(fox).insert(Fox::new(V_FOX, 79.3));
    cx.interactable(fox, Interactable::new("fox", "earth.verb.talk").radius(3.6).focus(Vec3::Y * 0.35).prompt_height(0.4).disabled());
    let mut x = 10.6;
    let mut k = 0;
    while x < 17.5 {
        let mut z = 10.6;
        while z < 20.5 {
            let j = ((k * 37) % 11) as f32 * 0.04;
            let g = cx.ground(x + j, z - j);
            if g.xz().length() < 38.0 {
                cx.model("model.earth.wheat", g, (k * 47 % 360) as f32, 1.0);
            }
            z += 0.85;
            k += 1;
        }
        x += 0.85;
    }
    for i in 0..6 {
        let z = 10.8 + i as f32 * 1.8;
        let g = cx.ground(9.9, z);
        cx.model("model.earth.fence", g, 90.0, 1.0);
    }
    for (x, z, yaw) in [(-4.5f32, 20.5f32, 160.0f32), (15.0, 5.0, 250.0)] {
        let g = cx.ground(x, z);
        let e = cx.model("model.earth.cottage", g - Vec3::Y * 0.1, yaw, 1.0);
        cx.collider(e, Collider::cuboid(Vec3::new(3.3, 2.4, 2.9)));
    }
    for (x, z, yaw) in [(-7.4f32, 17.5f32, 160.0f32), (-6.3, 17.1, 160.0), (-5.2, 16.7, 160.0), (17.5, 7.6, 250.0), (17.1, 8.7, 250.0)] {
        let g = cx.ground(x, z);
        cx.model("model.earth.crop", g, yaw, 1.0);
    }

    // ---- V4 station
    let hs = h_station;
    let plat = cx.model("model.earth.platform", Vec3::new(9.0, hs, 31.5), 0.0, 1.0);
    cx.collider(plat, Collider::cuboid(Vec3::new(14.0, 0.3, 1.8)).no_camera());
    let cab = cx.model("model.earth.cabin", Vec3::new(14.0, hs, 28.4), 0.0, 1.0);
    cx.collider(cab, Collider::cuboid(Vec3::new(3.2, 2.5, 2.4)));
    for x in [4.5f32, 12.5] {
        let e = cx.model("model.earth.bench", Vec3::new(x, hs + 0.3, 31.0), 0.0, 1.0);
        cx.collider(e, Collider::cuboid(Vec3::new(1.5, 0.45, 0.45)).no_camera());
    }
    for x in [2.6f32, 15.4] {
        let e = cx.model("model.lamp_post", Vec3::new(x, hs + 0.3, 30.9), 0.0, 1.0);
        cx.collider(e, Collider::cylinder(0.12, 2.3).no_camera());
    }
    let sig = cx.model("model.earth.signal", Vec3::new(18.0, hs, 32.0), 0.0, 1.0);
    cx.collider(sig, Collider::cylinder(0.12, 3.0).no_camera());
    let nb = cx.model("model.earth.notice", Vec3::new(7.4, hs + 0.3, 31.0), 0.0, 1.0);
    cx.collider(nb, Collider::cuboid(Vec3::new(1.4, 1.8, 0.2)).no_camera());
    let mut x = -24.0;
    while x < 28.0 {
        let g = cx.ground(x, 35.6);
        if g.xz().length() < 39.5 {
            let e = cx.model("model.earth.pole", g, 0.0, 1.0);
            cx.collider(e, Collider::cylinder(0.12, 4.0).no_camera());
        }
        x += 8.0;
    }
    rail_line(cx, V_RAIL_Z, -30.0, 30.0, 43.0);
    spawn_train(cx, 17.0, V_RAIL_Z, hs + 0.2);
    let bp = Vec3::new(V_BOARD.x, hs + 0.3, V_BOARD.y);
    let board = cx.empty(bp, 0.0);
    cx.interactable(board, Interactable::new("train", "earth.verb.board").radius(3.2).priority(5).focus(Vec3::new(0.0, 1.0, 0.6)).prompt_height(0.6).any_facing().disabled());
    cx.anchor("board", bp, 0.0);
    cx.anchor("door", Vec3::new(V_BOARD.x, hs + 0.3, V_BOARD.y + 0.55), 0.0);

    // ---- way signs (guide marker lights near them) and cairns
    let signs = [
        (Vec2::new(-16.9, -26.0), Leg::Roses, V_SUMMIT),
        (Vec2::new(-14.6, -12.8), Leg::Roses, Vec2::new(-10.4, 4.6)),
        (Vec2::new(-10.9, 2.7), Leg::Roses, V_GARDEN),
        (Vec2::new(3.2, 5.5), Leg::Fox, V_BRIDGE),
        (Vec2::new(2.0, 9.5), Leg::Fox, V_FOX),
        (Vec2::new(7.9, 21.0), Leg::Train, V_BOARD),
        (Vec2::new(8.2, 28.6), Leg::Train, V_BOARD),
    ];
    for (p, leg, toward) in signs {
        signpost(cx, p, toward);
        cx.world.resource_mut::<EarthState>().signs.push(Sign { pos: p, leg, fired: false });
    }
    for (x, z) in [(-14.6f32, -29.0f32), (-17.6, -17.5), (-13.8, -5.4), (-11.8, 0.6), (4.8, 13.3), (8.3, 19.2), (8.7, 25.8)] {
        let g = cx.ground(x, z);
        cx.model("model.earth.cairn", g, x * 40.0, 1.0);
    }

    // ---- scatter: desert south, meadow north
    let route = V_ROUTE;
    let clear = move |p: Vec2| {
        route_dist(route, p) > 2.2
            && !(p.x > GX0 - 2.0 && p.x < GX1 + 2.0 && p.y > GZ0 - 2.0 && p.y < GZ1 + 2.0)
            && ((p.x + p.y - STREAM_C).abs() / std::f32::consts::SQRT_2) > 1.6
            && p.distance(V_FOX) > 4.5
            && !(p.x > 9.0 && p.x < 18.5 && p.y > 10.0 && p.y < 21.0)
            && !(p.y > 26.0 && p.x > -8.0 && p.x < 22.0)
            && p.distance(V_SPAWN) > 3.5
            && p.distance(Vec2::new(-4.5, 20.5)) > 3.5
            && p.distance(Vec2::new(15.0, 5.0)) > 3.5
    };
    let desert = move |p: Vec2| clear(p) && p.y < -6.0 && p.x + p.y < 8.0;
    let meadow = move |p: Vec2| clear(p) && p.y > 2.0 && p.x + p.y > -2.0;
    scatter_where(cx, "model.earth.cactus", 12, 11, (0.75, 1.25), 37.0, desert);
    scatter_where(cx, "model.earth.cactus_s", 10, 12, (0.8, 1.3), 37.0, desert);
    scatter_where(cx, "model.earth.boulder", 9, 13, (0.6, 1.4), 37.0, desert);
    scatter_where(cx, "model.earth.dune_grass", 40, 14, (0.8, 1.5), 38.0, desert);
    scatter_where(cx, "model.rock", 8, 15, (0.6, 1.2), 37.0, desert);
    let trees = scatter_where(cx, "model.tree_oak", 7, 21, (1.4, 2.0), 36.0, move |p| meadow(p) && p.distance(V_TRUNK) > 6.0);
    for e in trees {
        cx.collider(e, Collider::cylinder(0.3, 2.0));
    }
    scatter_where(cx, "model.earth.bush", 14, 22, (0.8, 1.3), 37.0, meadow);
    scatter_where(cx, "model.flower_yellow", 18, 23, (1.0, 1.6), 37.0, meadow);
    scatter_where(cx, "model.flower_purple", 14, 24, (1.0, 1.6), 37.0, meadow);
    scatter_where(cx, "model.grass_tuft", 70, 25, (0.8, 1.4), 38.0, meadow);

    // ---- landing spot (the prince fell here), facing up the valley (+Z)
    anchor(cx, "spawn", V_SPAWN, 0.0);
    let _ = K;
}

/// The player at the valley landing spot.
pub fn spawn_valley_player(cx: &mut ChapterCx) {
    let p = cx.ground(V_SPAWN.x, V_SPAWN.y);
    cx.player(p, 0.0);
}

/// Marks everything spawned so far as part of the valley scene (despawned on the switch).
pub fn mark_valley(world: &mut World) {
    let mut q = world.query_filtered::<Entity, (With<ChapterScoped>, Without<Player>)>();
    let es: Vec<Entity> = q.iter(world).collect();
    for e in es {
        world.entity_mut(e).insert(ValleyPart);
    }
}

// ====================================================================== DUNES

pub fn build_dunes(cx: &mut ChapterCx) {
    let mut p = PlanetParams::sized(40.0);
    p.ground = hex(0xe8c793);
    p.ground2 = hex(0xdbb47c);
    p.dry = hex(0xd2a86f);
    p.rock = hex(0xa98560);
    p.noise_amp = 0.32;
    p.noise_wavelength = 10.0;
    p.seed = 113;
    p.underside_depth = 22.0;
    let mut mods = vec![
        TerrainMod::Mound { center: D_DUNE_TOP, radius: 9.5, height: 4.0 },
        TerrainMod::Mound { center: Vec2::new(-2.0, 18.5), radius: 8.0, height: 2.6 },
        TerrainMod::Mound { center: Vec2::new(12.0, 16.0), radius: 9.0, height: 3.0 },
        TerrainMod::Mound { center: Vec2::new(26.0, 11.0), radius: 7.0, height: 2.2 },
        TerrainMod::Mound { center: Vec2::new(-31.0, -16.0), radius: 8.0, height: 2.5 },
        TerrainMod::Mound { center: Vec2::new(-14.0, -21.0), radius: 9.0, height: 3.2 },
        TerrainMod::Mound { center: Vec2::new(2.0, -18.0), radius: 8.0, height: 2.4 },
        TerrainMod::Mound { center: Vec2::new(16.0, -17.0), radius: 7.0, height: 2.0 },
        TerrainMod::Mound { center: D_CREST + Vec2::new(1.5, -1.0), radius: 6.5, height: 2.6 },
        TerrainMod::Mound { center: Vec2::new(-34.0, 15.0), radius: 7.0, height: 2.0 },
        TerrainMod::Mound { center: Vec2::new(19.0, 27.0), radius: 8.0, height: 2.5 },
        TerrainMod::Flatten { center: Vec2::new(-23.5, -2.5), radius: 5.5, blend: 3.0, height: None },
        TerrainMod::Flatten { center: D_WELL, radius: 2.6, blend: 2.0, height: None },
    ];
    let mut probe = PlanetParams { mods: mods.clone(), ..p.clone() };
    resolve_flats(&mut probe);
    mods = probe.mods;
    let base = Terrain::new(PlanetParams { mods: mods.clone(), ..p.clone() });
    let h_rail = base.height(-30.0, D_RAIL_Z);
    let h_way = base.height(D_WAY_START.x, D_WAY_START.y) - 0.2;
    let a = way_dir();
    for i in 0..9 {
        let c = D_WAY_START + a * (i as f32 * 2.0);
        mods.push(TerrainMod::Flatten { center: c, radius: 2.8, blend: 2.5, height: Some(h_way) });
    }
    for i in 0..9 {
        mods.push(TerrainMod::Flatten { center: Vec2::new(-46.0 + i as f32 * 4.0, D_RAIL_Z), radius: 2.4, blend: 2.0, height: Some(h_rail) });
    }
    // sand tones: darker troughs, paler crests, the stone path
    mods.push(TerrainMod::Paint { center: D_DUNE_TOP, radius: 5.0, blend: 5.0, color: hex(0xf0d6a4) });
    mods.push(TerrainMod::Paint { center: Vec2::new(12.0, 16.0), radius: 4.0, blend: 5.0, color: hex(0xf0d6a4) });
    mods.push(TerrainMod::Paint { center: D_WELL, radius: 2.4, blend: 1.5, color: hex(0xcfa978) });
    p.mods = mods;
    resolve_flats(&mut p);
    cx.planet(p);
    let mut pal = SkyPalette::default();
    pal.day_horizon = hex(0xf0dcbc);
    pal.day_zenith = hex(0x5c8ed0);
    pal.golden_horizon = hex(0xf7c88a);
    pal.dusk_horizon = hex(0xee8a5a);
    pal.fog = 0.6;
    let morning = Sky::default().hour_for_elevation(22.0, false);
    cx.sky(morning, 0.0, None, pal);
    cx.ambience("amb.wind");

    let g = |cx: &ChapterCx, p: Vec2| cx.ground(p.x, p.y);

    // ---- railway end + arriving train (stands at the stop; the arrival animates it)
    rail_line(cx, D_RAIL_Z, -48.0, -15.0, 43.0);
    let bs = Vec3::new(-14.6, h_rail, D_RAIL_Z);
    let b = cx.model("model.earth.buffer", bs, 270.0, 1.0);
    cx.collider(b, Collider::cuboid(Vec3::new(1.7, 1.1, 0.4)).no_camera());
    spawn_train(cx, D_TRAIN_STOP_X, D_RAIL_Z, h_rail + 0.2);
    cx.anchor("stepoff", g(cx, D_STEPOFF), 150.0);

    // ---- plane camp
    let pp = g(cx, D_PLANE);
    cx.model("model.earth.plane", pp, 40.0, 1.0);
    let fwd = yaw_dir(40.0);
    let e = box_collider(cx, pp, 40.0, Vec3::new(1.4, 1.9, 6.2));
    let _ = e;
    let wing = pp + Vec3::new(fwd.x, 0.0, fwd.y) * 0.9;
    box_collider(cx, wing, 40.0, Vec3::new(9.4, 0.9, 1.5));
    let bed = g(cx, D_BEDROLL);
    cx.model("model.earth.bedroll", bed, D_PILOT_YAW, 1.0);
    let pilot = cx.model("model.earth.pilot", bed, D_PILOT_YAW, 1.0);
    cx.world.entity_mut(pilot).insert(Pilot::new(PilotPose::Sleep));
    cx.interactable(pilot, Interactable::new("pilot_wake", "earth.verb.wake").radius(2.6).focus(Vec3::Y * 0.35).prompt_height(0.4).any_facing().disabled());
    cx.anchor("pilot", bed, D_PILOT_YAW);
    let crate_yaw = 20.0;
    let cp = g(cx, D_CRATE);
    let c = cx.model("model.earth.crate", cp, crate_yaw, 1.0);
    cx.collider(c, Collider::cuboid(Vec3::new(0.7, 0.45, 0.5)).no_camera());
    cx.anchor("crate", cp, crate_yaw);
    let c2p = g(cx, D_CRATE + Vec2::new(0.9, -0.7));
    let c2 = cx.model("model.earth.crate", c2p, 70.0, 1.0);
    cx.collider(c2, Collider::cuboid(Vec3::new(0.7, 0.45, 0.5)).no_camera());
    cx.model("model.earth.wrench", c2p + Vec3::Y * 0.455, 30.0, 1.0);
    for (x, z, yaw) in [(-21.2f32, -6.4f32, 10.0f32), (-21.9, -7.0, 55.0), (-26.6, -1.2, 80.0)] {
        let pp = cx.ground(x, z);
        let e = cx.model("model.earth.crate", pp, yaw, 1.0);
        cx.collider(e, Collider::cuboid(Vec3::new(0.7, 0.45, 0.5)).no_camera());
    }

    // ---- the well with two stone seats
    let wp = g(cx, D_WELL);
    let well = cx.model("model.earth.well", wp, 25.0, 1.0);
    cx.collider(well, Collider::cylinder(0.95, 0.75));
    cx.interactable(well, Interactable::new("well", "earth.verb.winch").radius(3.0).focus(Vec3::Y * 0.8).prompt_height(0.7).any_facing().disabled());
    cx.world.entity_mut(well).insert(super::state::WellRig);
    cx.anchor("well", wp, 0.0);
    for o in [Vec2::new(-1.5, 1.4), Vec2::new(1.6, -1.3)] {
        let sp = g(cx, D_WELL + o);
        let e = cx.model("model.earth.stone_seat", sp, 0.0, 1.0);
        cx.collider(e, Collider::cylinder(0.32, 0.4).no_camera());
    }

    // ---- the sacred way: slabs, columns, steps, landing, doorway, ruined wall
    let n = way_normal();
    let wy = WAY_YAW;
    let l = D_DOOR.distance(D_WAY_START);
    let mut s = 0.0;
    while s < l - 2.6 {
        let c = D_WAY_START + a * s;
        let sp = g(cx, c);
        cx.model("model.earth.slab", sp - Vec3::Y * 0.03, wy + if (s as i32) % 2 == 0 { 2.0 } else { -3.0 }, 1.0);
        s += 1.0;
    }
    let mut k = 0;
    let mut s = 0.6;
    while s < l - 6.0 {
        for side in [-1.0f32, 1.0] {
            let c = D_WAY_START + a * s + n * (2.3 * side);
            let cp = g(cx, c);
            let key = if (k + (side > 0.0) as i32) % 3 == 1 { "model.earth.column_broken" } else { "model.earth.column" };
            let e = cx.model(key, cp - Vec3::Y * 0.1, wy, 1.0);
            cx.collider(e, Collider::cylinder(0.33, 3.0));
        }
        k += 1;
        s += 2.4;
    }
    for (o, side) in [(Vec2::new(-1.0, 0.0), 3.2f32), (Vec2::new(3.0, 0.0), -3.4)] {
        let c = D_WAY_START + a * (o.x + 2.0) + n * side;
        let bp = g(cx, c);
        cx.model("model.earth.block", bp, 30.0 * side, 1.0);
    }
    let door = g(cx, D_DOOR);
    let base_y = door.y.min(g(cx, D_DOOR - a * 3.0).y);
    let landing_c = D_DOOR - a * 0.8;
    let lp = Vec3::new(landing_c.x, base_y, landing_c.y);
    cx.model("model.earth.landing", lp, wy, 1.0);
    box_collider(cx, lp, wy, Vec3::new(4.0, D_LANDING_TOP, 2.4));
    for (dist, h) in [(0.225f32, 0.5f32), (0.675, 0.25)] {
        for c in [D_DOOR - a * (2.0 + dist), D_DOOR + a * (0.4 + dist)] {
            let sp = Vec3::new(c.x, base_y, c.y);
            let e = cx.model("model.earth.step", sp, wy, 1.0);
            if let Some(mut tf) = cx.world.get_mut::<Transform>(e) {
                tf.scale = Vec3::new(1.0, h / 0.25, 1.0);
            }
            box_collider(cx, sp, wy, Vec3::new(3.0, h, 0.45));
        }
    }
    let dp = Vec3::new(D_DOOR.x, base_y + D_LANDING_TOP, D_DOOR.y);
    cx.model("model.earth.doorway", dp, wy, 1.0);
    for side in [-1.0f32, 1.0] {
        let pc = D_DOOR + n * (1.1 * side);
        box_collider(cx, Vec3::new(pc.x, dp.y, pc.y), wy, Vec3::new(0.6, 2.5, 0.7));
    }
    for side in [-1.0f32, 1.0] {
        for (i, off) in [2.4f32, 4.4, 6.3].iter().enumerate() {
            let c = D_DOOR + n * (off * side) + a * (i as f32 * 0.15);
            let wp = g(cx, c);
            let y = if *off < 3.0 { base_y } else { wp.y };
            cx.model("model.earth.ruin_wall", Vec3::new(c.x, y - 0.1, c.y), wy, 1.0);
            box_collider(cx, Vec3::new(c.x, y - 0.1, c.y), wy, Vec3::new(2.0, 2.3, 0.6));
        }
    }
    cx.anchor("door", dp, wy);
    let ss = d_snake_spot();
    cx.anchor("snake_spot", Vec3::new(ss.x, base_y + D_LANDING_TOP, ss.y), wy);
    let fw = d_farewell();
    cx.anchor("farewell", g(cx, fw), wy);
    cx.anchor("gate", dp, wy);
    let snake = cx.model("model.earth.snake", dp, wy + 180.0, 1.0);
    cx.world.entity_mut(snake).insert(Snake::new(SnakeState::Raised, wy + 180.0));
    cx.interactable(snake, Interactable::new("snake2", "earth.verb.snake").radius(2.2).focus(Vec3::Y * 0.75).prompt_height(0.3).any_facing().disabled());

    // ---- evening star over the last crest (fades in with the dusk)
    let (az, el) = (STAR_AZ.to_radians(), STAR_EL.to_radians());
    let dir = Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos());
    let star = cx.model("model.earth.star", dp + dir * 150.0, 0.0, 0.001);
    cx.world.entity_mut(star).insert(super::state::EveningStar);

    // ---- decor: dry grass tufts, a few rocks
    let keep = move |p: Vec2| {
        p.distance(D_WELL) > 3.0
            && p.distance(D_PLANE) > 7.0
            && seg_dist(p, D_WAY_START - a * 2.0, D_DOOR + a * 4.0) > 3.6
            && (p.y - D_RAIL_Z).abs() > 2.5
            && seg_dist(p, D_CRATE, D_WELL) > 1.5
    };
    scatter_where(cx, "model.earth.dune_grass", 70, 71, (0.8, 1.6), 38.0, keep);
    scatter_where(cx, "model.earth.boulder", 6, 72, (0.5, 1.0), 38.0, keep);
    scatter_where(cx, "model.earth.bush", 8, 73, (0.6, 1.0), 38.0, keep);

    // ---- footprint pool, stardust trail dots
    let mut pool = Vec::new();
    for _ in 0..super::state::FOOTPRINTS {
        let e = cx.model("model.earth.footprint", Vec3::new(0.0, -50.0, 0.0), 0.0, 1.0);
        cx.world.entity_mut(e).insert(Visibility::Hidden);
        pool.push(e);
    }
    let mut dots = Vec::new();
    for i in 0..16 {
        let e = cx.model("model.earth.stardust", Vec3::new(0.0, -50.0, 0.0), 0.0, 1.0);
        cx.world.entity_mut(e).insert((Visibility::Hidden, super::state::TrailDot(i)));
        dots.push(e);
    }
    let mut st = cx.world.resource_mut::<EarthState>();
    st.scene = Scene::Dunes;
    st.footprints = pool;
    st.fp_next = 0;
    st.last_print = None;
    st.leg = Leg::None;
}

/// Ground point helper for systems.
pub fn ground_of(world: &World, p: Vec2) -> Vec3 {
    world.get_resource::<Terrain>().map(|t| t.ground(p.x, p.y)).unwrap_or(Vec3::new(p.x, 0.0, p.y))
}

/// Switch from the valley to the dunes (called while the railway page covers the screen).
pub fn switch_to_dunes(world: &mut World) {
    let mut q = world.query_filtered::<Entity, With<ValleyPart>>();
    let es: Vec<Entity> = q.iter(world).collect();
    for e in es {
        if let Ok(em) = world.get_entity_mut(e) {
            em.despawn();
        }
    }
    world.insert_resource(Anchors::default());
    let test = world.resource::<crate::core::params::LaunchParams>().test;
    {
        let mut cx = ChapterCx { world, id: "earth", test };
        build_dunes(&mut cx);
    }
    // park the prince inside the arriving carriage (hidden) until he steps off
    let p = world.resource::<Anchors>().get("stepoff").map(|a| a.0).unwrap_or(Vec3::ZERO);
    let mut q = world.query::<(&mut Player, &mut Transform, &mut Visibility)>();
    if let Some((mut pl, mut tf, mut vis)) = q.iter_mut(world).next() {
        tf.translation = p + Vec3::Z * 1.8;
        pl.drive = Drive::External;
        pl.velocity = Vec3::ZERO;
        *vis = Visibility::Hidden;
    }
    world.write_message(AudioCmd::music("music.earth_dunes"));
    let _ = world.get_resource::<SharedMaterials>();
}
