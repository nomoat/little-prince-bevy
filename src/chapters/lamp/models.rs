//! Procedural fallbacks for the lamplighter chapter's asset keys
//! (assets/chapters/lamp.ron). Models face +Z, origin on the ground.

use crate::core::prelude::*;
use std::f32::consts::TAU;

const IRON: u32 = 0x2e3036;
const BRASS: u32 = 0xc39a46;

pub fn register(app: &mut App) {
    app.register_model("lamp.street_lamp", street_lamp)
        .register_model("lamp.lamplighter", lamplighter)
        .register_model("lamp.seat", seat)
        .register_model("lamp.tide_pool", tide_pool)
        .register_model("lamp.stone", flat_stone)
        .register_model("lamp.boulder", boulder)
        .register_model("lamp.cloud", cloud);
}

/// The street lamp: lantern centre 2.92 m above the foot (the wick target).
/// Named parts: "glass" (4 panes, emissive), "flame", "halo".
fn street_lamp(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.3, 0.26, 0.12, 8, hex(0x5a5a62));
    m.frustum(Vec3::Y * 0.12, 0.17, 0.12, 0.35, 8, hex(IRON));
    m.tube(Vec3::Y * 0.45, Vec3::Y * 2.62, 0.055, 0.045, 8, hex(IRON));
    for y in [0.55f32, 1.6, 2.5] {
        m.frustum(Vec3::Y * y, 0.075, 0.075, 0.05, 8, hex(IRON));
    }
    // ladder-rest crossbar
    m.tube(Vec3::new(-0.32, 2.45, 0.0), Vec3::new(0.32, 2.45, 0.0), 0.022, 0.022, 5, hex(IRON));
    for x in [-0.32f32, 0.32] {
        m.sphere(Vec3::new(x, 2.45, 0.0), 0.035, 3, 6, hex(BRASS));
    }
    // lantern frame: base cup, 4 corner bars, roof, finial
    let c = Vec3::Y * 2.92;
    m.frustum(c - Vec3::Y * 0.32, 0.09, 0.2, 0.1, 4, hex(IRON));
    for i in 0..4 {
        let a = (i as f32 * 90.0 + 45.0).to_radians();
        let o = Vec3::new(a.sin(), 0.0, a.cos()) * 0.2;
        m.tube(c + o - Vec3::Y * 0.22, c + o * 1.15 + Vec3::Y * 0.22, 0.014, 0.014, 4, hex(IRON));
    }
    m.frustum(c + Vec3::Y * 0.22, 0.3, 0.05, 0.22, 4, hex(IRON));
    m.frustum(c + Vec3::Y * 0.44, 0.05, 0.0, 0.12, 6, hex(BRASS));
    m.sphere(c + Vec3::Y * 0.58, 0.04, 3, 6, hex(BRASS));
    p.add(&m);
    let mut g = MeshBuilder::new();
    g.frustum(c - Vec3::Y * 0.22, 0.17, 0.21, 0.44, 4, hex(0xfff2d0));
    p.add_part(Some("glass"), &g, Transform::IDENTITY, PartMat::Custom(StandardMaterial {
        base_color: Color::srgba(1.0, 0.92, 0.75, 0.35),
        emissive: LinearRgba::BLACK,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        ..default()
    }));
    let mut f = MeshBuilder::new();
    f.ellipsoid(Vec3::Y * 0.05, Vec3::new(0.04, 0.09, 0.04), 3, 7, hex(0xffffff));
    p.add_part(Some("flame"), &f, Transform::from_translation(c - Vec3::Y * 0.1), PartMat::Emissive(hex(0xffa040), 14.0));
    let mut h = MeshBuilder::new();
    h.sphere(Vec3::ZERO, 0.42, 6, 12, hex(0xffffff));
    let halo = StandardMaterial {
        base_color: Color::srgba(1.0, 0.72, 0.38, 0.08),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        ..default()
    };
    p.add_part(Some("halo"), &h, Transform::from_translation(c), PartMat::Custom(halo));
    if let Some(last) = p.parts.last_mut() {
        last.cast_shadows = false;
    }
}

/// The lamplighter, standing, facing +Z. Named parts: "body", "head"
/// (neck pivot), "arm" + "tip" (right arm and lighting wand, shoulder pivot,
/// modelled in the RAISED pose: the wand tip reaches (0.06, 2.92, 0.96)),
/// "arm_l" (left arm, shoulder pivot).
fn lamplighter(p: &mut ModelParts) {
    let coat = hex(0x2d3f66);
    let trim = hex(0xc49a46);
    let skin = hex(0xefc9a8);
    let mut b = MeshBuilder::new();
    for x in [-0.1f32, 0.1] {
        b.tube(Vec3::new(x, 0.05, 0.0), Vec3::new(x, 0.75, 0.0), 0.065, 0.075, 7, hex(0x3a3434));
        b.cuboid(Vec3::new(x, 0.04, 0.05), Vec3::new(0.1, 0.08, 0.24), hex(0x1d1a18));
    }
    // long coat with brass buttons and a belt
    b.lathe(Vec3::Y * 0.55, &[(0.0, 0.0), (0.27, 0.0), (0.22, 0.45), (0.2, 0.75), (0.15, 0.85), (0.0, 0.88)], 10, coat);
    b.frustum(Vec3::Y * 0.95, 0.215, 0.215, 0.06, 10, hex(0x3a2a20));
    for k in 0..4 {
        b.sphere(Vec3::new(0.0, 0.75 + k as f32 * 0.14, 0.215), 0.016, 2, 5, trim);
    }
    b.cuboid(Vec3::new(0.0, 1.38, 0.05), Vec3::new(0.3, 0.06, 0.2), trim); // collar
    p.add_named("body", &b, Transform::IDENTITY);
    let mut h = MeshBuilder::new();
    h.tube(Vec3::ZERO, Vec3::Y * 0.08, 0.05, 0.05, 6, skin);
    h.sphere(Vec3::Y * 0.18, 0.12, 6, 10, skin);
    h.sphere(Vec3::new(0.0, 0.17, 0.12), 0.028, 3, 6, hex(0xe0a888));
    h.ellipsoid(Vec3::new(0.0, 0.11, 0.11), Vec3::new(0.08, 0.025, 0.03), 2, 6, hex(0x5a4030)); // moustache
    // peaked cap
    h.frustum(Vec3::Y * 0.24, 0.13, 0.125, 0.09, 12, coat);
    h.frustum(Vec3::Y * 0.33, 0.125, 0.0, 0.03, 12, coat);
    h.cuboid(Vec3::new(0.0, 0.245, 0.13), Vec3::new(0.18, 0.015, 0.1), hex(0x1d1d26));
    h.cuboid(Vec3::new(0.0, 0.27, 0.124), Vec3::new(0.05, 0.03, 0.01), trim);
    p.add_named("head", &h, Transform::from_xyz(0.0, 1.42, 0.0));
    // right arm + wand (raised pose)
    let pivot = Vec3::new(-0.2, 1.32, 0.05);
    let tip = Vec3::new(0.06, 2.92, 0.96) - pivot;
    let dir = tip.normalize();
    let hand = dir * 0.42;
    let mut a = MeshBuilder::new();
    a.tube(Vec3::ZERO, hand, 0.06, 0.05, 6, coat);
    a.sphere(hand, 0.045, 3, 6, skin);
    a.tube(hand - dir * 0.55, tip, 0.016, 0.012, 5, hex(0x7a5236));
    a.tube(tip - dir * 0.12, tip - dir * 0.02, 0.02, 0.02, 5, hex(BRASS));
    // the snuffer hook
    a.tube(tip - dir * 0.05, tip - dir * 0.05 + Vec3::new(0.0, -0.06, 0.06), 0.008, 0.008, 4, hex(BRASS));
    p.add_named("arm", &a, Transform::from_translation(pivot));
    let mut t = MeshBuilder::new();
    t.sphere(tip, 0.022, 3, 6, hex(0xffffff));
    p.add_part(Some("tip"), &t, Transform::from_translation(pivot), PartMat::Emissive(hex(0xff9a40), 6.0));
    // left arm hanging
    let mut l = MeshBuilder::new();
    l.tube(Vec3::ZERO, Vec3::new(0.05, -0.5, 0.06), 0.06, 0.05, 6, coat);
    l.sphere(Vec3::new(0.05, -0.54, 0.07), 0.045, 3, 6, skin);
    p.add_named("arm_l", &l, Transform::from_xyz(0.22, 1.32, 0.03));
}

/// The sunset seat: a rock ledge with a flat top 0.42 m above its foot.
fn seat(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::new(0.0, 0.18, 0.0), Vec3::new(0.62, 0.3, 0.48), 21, hex(0x7d7468));
    m.frustum(Vec3::Y * 0.3, 0.42, 0.38, 0.12, 9, hex(0x8a8174));
    m.rock(Vec3::new(-0.45, 0.25, -0.35), Vec3::new(0.35, 0.42, 0.3), 22, hex(0x6f665c));
    p.add(&m);
}

/// A tide pool: a shallow round water surface with a boulder rim.
fn tide_pool(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for i in 0..9 {
        let a = i as f32 / 9.0 * TAU;
        let r = 0.85 + (i % 3) as f32 * 0.08;
        m.rock(Vec3::new(a.sin() * r, 0.08, a.cos() * r), Vec3::new(0.28, 0.2, 0.24), 30 + i, hex(0x6f6a62));
    }
    m.frustum(Vec3::Y * -0.08, 0.8, 0.82, 0.1, 18, hex(0x3a4a48));
    p.add(&m);
    let mut w = MeshBuilder::new();
    w.frustum(Vec3::Y * 0.02, 0.8, 0.8, 0.005, 18, hex(0x3d6f86));
    p.add_part(Some("water"), &w, Transform::IDENTITY, PartMat::Custom(StandardMaterial {
        base_color: Color::srgba(0.35, 0.55, 0.7, 0.75),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.05,
        reflectance: 0.8,
        ..default()
    }));
}

/// A flat stepping stone of the path.
fn flat_stone(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::Y * -0.06, 0.34, 0.3, 0.1, 7, hex(0x9a9286));
    p.add(&m);
}

fn boulder(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::new(0.0, 0.25, 0.0), Vec3::new(0.6, 0.45, 0.5), 41, hex(0x6c645c));
    p.add(&m);
}

/// A low cloud puff for the cloud sea at the horizon (colour set at runtime).
fn cloud(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for (x, y, s) in [(0.0f32, 0.0f32, 1.0f32), (1.1, -0.1, 0.8), (-1.2, -0.15, 0.75), (0.5, 0.35, 0.7), (-0.6, 0.3, 0.6)] {
        m.ellipsoid(Vec3::new(x, y, 0.0), Vec3::new(1.2 * s, 0.45 * s, 0.7 * s), 4, 9, hex(0xffffff));
    }
    // lit by the low sun, but not fogged away at the horizon
    let mat = StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 1.0, fog_enabled: false, ..default() };
    p.add_part(Some("cloud"), &m, Transform::IDENTITY, PartMat::Custom(mat));
}
