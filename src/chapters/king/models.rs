//! Procedural fallbacks for the king chapter's asset keys (assets/chapters/king.ron).

use crate::core::prelude::*;

pub fn register(app: &mut App) {
    app.register_model("king.king", king)
        .register_model("king.throne", throne)
        .register_model("king.candelabrum", candelabrum)
        .register_model("king.door", door)
        .register_model("king.column", column)
        .register_model("king.banner", banner)
        .register_model("king.tower", tower)
        .register_model("king.cushion", cushion);
}

const ROBE: u32 = 0x7a2f5c;
const ROBE_DARK: u32 = 0x5e2347;
const ERMINE: u32 = 0xf4f1ea;
const GOLD: u32 = 0xe0b84a;
const SKIN: u32 = 0xf1d3b8;

/// The seated king (model faces +Z; origin at the floor under his seat).
/// Named parts (pivot): king_body (hips), king_head (neck), king_arm_r
/// (right shoulder, holds the scepter), king_arm_l (left shoulder), king_book
/// (the calendar on his left thigh).
fn king(p: &mut ModelParts) {
    let hip = Vec3::new(0.0, 0.5, 0.0);
    // the robe spilling over the seat and onto the floor (static)
    let mut r = MeshBuilder::new();
    r.lathe(
        Vec3::new(0.0, 0.0, 0.12),
        &[(0.0, 0.01), (0.72, 0.01), (0.66, 0.06), (0.42, 0.24), (0.3, 0.46), (0.0, 0.5)],
        14,
        hex(ROBE),
    );
    r.lathe(
        Vec3::new(0.0, 0.0, 0.12),
        &[(0.73, 0.0), (0.76, 0.03), (0.72, 0.07), (0.66, 0.065)],
        14,
        hex(ERMINE),
    );
    for i in 0..10 {
        let a = i as f32 / 10.0 * std::f32::consts::TAU;
        r.sphere(Vec3::new(a.cos() * 0.74, 0.05, 0.12 + a.sin() * 0.74), 0.012, 2, 4, hex(0x222222));
    }
    // thighs and knees under the robe
    r.ellipsoid(Vec3::new(0.12, 0.5, 0.2), Vec3::new(0.11, 0.09, 0.26), 3, 8, hex(ROBE_DARK));
    r.ellipsoid(Vec3::new(-0.12, 0.5, 0.2), Vec3::new(0.11, 0.09, 0.26), 3, 8, hex(ROBE_DARK));
    r.cuboid(Vec3::new(0.12, 0.05, 0.48), Vec3::new(0.1, 0.08, 0.18), hex(0x3b2a22));
    r.cuboid(Vec3::new(-0.12, 0.05, 0.48), Vec3::new(0.1, 0.08, 0.18), hex(0x3b2a22));
    p.add(&r);
    // torso: robe + ermine collar
    let mut b = MeshBuilder::new();
    b.lathe(
        Vec3::ZERO,
        &[(0.0, -0.02), (0.25, -0.02), (0.26, 0.15), (0.22, 0.42), (0.14, 0.55), (0.0, 0.56)],
        12,
        hex(ROBE),
    );
    b.lathe(
        Vec3::ZERO,
        &[(0.15, 0.5), (0.27, 0.46), (0.3, 0.4), (0.24, 0.36), (0.13, 0.48)],
        12,
        hex(ERMINE),
    );
    b.frustum(Vec3::new(0.0, 0.0, 0.0), 0.265, 0.265, 0.06, 12, hex(GOLD));
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        b.sphere(Vec3::new(a.cos() * 0.27, 0.41, a.sin() * 0.27), 0.014, 2, 4, hex(0x222222));
    }
    // ermine front band
    b.cuboid(Vec3::new(0.0, 0.25, 0.235), Vec3::new(0.09, 0.45, 0.03), hex(ERMINE));
    p.add_named("king_body", &b, Transform::from_translation(hip));
    // head: face, big white beard, crown
    let mut h = MeshBuilder::new();
    h.sphere(Vec3::new(0.0, 0.12, 0.0), 0.13, 5, 9, hex(SKIN));
    h.sphere(Vec3::new(0.0, 0.11, 0.13), 0.028, 2, 5, hex(0xe8b9a0));
    h.sphere(Vec3::new(0.045, 0.15, 0.115), 0.014, 2, 4, hex(0x2b2b3a));
    h.sphere(Vec3::new(-0.045, 0.15, 0.115), 0.014, 2, 4, hex(0x2b2b3a));
    h.ellipsoid(Vec3::new(0.0, 0.2, 0.1), Vec3::new(0.09, 0.02, 0.03), 2, 6, hex(0xeeeeee));
    h.lathe(
        Vec3::new(0.0, -0.18, 0.06),
        &[(0.0, 0.0), (0.06, 0.06), (0.12, 0.18), (0.13, 0.26), (0.09, 0.3), (0.0, 0.28)],
        9,
        hex(0xf2f2f0),
    );
    h.ellipsoid(Vec3::new(0.0, 0.04, 0.12), Vec3::new(0.08, 0.025, 0.03), 2, 6, hex(0xf2f2f0));
    h.ellipsoid(Vec3::new(0.0, 0.16, -0.03), Vec3::new(0.135, 0.09, 0.13), 3, 8, hex(0xf2f2f0));
    h.frustum(Vec3::new(0.0, 0.22, 0.0), 0.11, 0.12, 0.08, 10, hex(GOLD));
    for i in 0..6 {
        let a = i as f32 / 6.0 * std::f32::consts::TAU;
        let base = Vec3::new(a.cos() * 0.11, 0.3, a.sin() * 0.11);
        h.tube(base, base + Vec3::new(a.cos() * 0.02, 0.08, a.sin() * 0.02), 0.025, 0.0, 4, hex(GOLD));
        h.sphere(base + Vec3::new(a.cos() * 0.02, 0.09, a.sin() * 0.02), 0.012, 2, 4, hex(0xd04060));
    }
    h.ellipsoid(Vec3::new(0.0, 0.27, 0.0), Vec3::new(0.1, 0.04, 0.1), 2, 8, hex(0x9a2050));
    p.add_named("king_head", &h, Transform::from_xyz(0.0, 1.06, 0.0));
    // right arm (his right = -X) with the scepter, pivot at the shoulder
    let mut ar = MeshBuilder::new();
    ar.tube(Vec3::ZERO, Vec3::new(0.0, -0.26, 0.0), 0.06, 0.055, 7, hex(ROBE));
    ar.tube(Vec3::new(0.0, -0.26, 0.0), Vec3::new(0.0, -0.3, 0.24), 0.055, 0.05, 7, hex(ROBE));
    ar.frustum(Vec3::new(0.0, -0.3, 0.22), 0.055, 0.06, 0.04, 7, hex(ERMINE));
    ar.sphere(Vec3::new(0.0, -0.3, 0.29), 0.04, 2, 6, hex(SKIN));
    ar.tube(Vec3::new(0.0, -0.62, 0.3), Vec3::new(0.0, 0.18, 0.3), 0.016, 0.016, 6, hex(GOLD));
    ar.sphere(Vec3::new(0.0, 0.22, 0.3), 0.05, 3, 7, hex(GOLD));
    ar.tube(Vec3::new(0.0, 0.26, 0.3), Vec3::new(0.0, 0.33, 0.3), 0.012, 0.012, 4, hex(GOLD));
    ar.cuboid(Vec3::new(0.0, 0.31, 0.3), Vec3::new(0.06, 0.012, 0.012), hex(GOLD));
    p.add_named("king_arm_r", &ar, Transform::from_xyz(-0.25, 0.98, 0.0));
    // left arm, pivot at the shoulder, hand toward the book
    let mut al = MeshBuilder::new();
    al.tube(Vec3::ZERO, Vec3::new(0.0, -0.26, 0.0), 0.06, 0.055, 7, hex(ROBE));
    al.tube(Vec3::new(0.0, -0.26, 0.0), Vec3::new(0.0, -0.3, 0.24), 0.055, 0.05, 7, hex(ROBE));
    al.frustum(Vec3::new(0.0, -0.3, 0.22), 0.055, 0.06, 0.04, 7, hex(ERMINE));
    al.sphere(Vec3::new(0.0, -0.3, 0.29), 0.04, 2, 6, hex(SKIN));
    al.tube(Vec3::new(0.0, -0.3, 0.3), Vec3::new(0.0, -0.31, 0.37), 0.01, 0.008, 4, hex(SKIN));
    p.add_named("king_arm_l", &al, Transform::from_xyz(0.25, 0.98, 0.0));
    // the calendar (an almanac) on his left thigh
    let mut k = MeshBuilder::new();
    k.cuboid(Vec3::ZERO, Vec3::new(0.2, 0.04, 0.26), hex(0x6a2a2a));
    k.cuboid(Vec3::new(0.0, 0.022, 0.0), Vec3::new(0.18, 0.012, 0.24), hex(0xf1e8cf));
    k.cuboid(Vec3::new(0.0, 0.03, -0.02), Vec3::new(0.005, 0.004, 0.22), hex(0x9a8a70));
    p.add_named(
        "king_book",
        &k,
        Transform::from_xyz(0.13, 0.6, 0.24).with_rotation(Quat::from_rotation_x(-0.25)),
    );
}

/// High-backed throne (front = +Z).
fn throne(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let red = hex(0xa0283c);
    let gold = hex(GOLD);
    m.cuboid(Vec3::new(0.0, 0.22, 0.0), Vec3::new(0.86, 0.44, 0.72), hex(0x6b3a2a));
    m.cuboid(Vec3::new(0.0, 0.47, 0.03), Vec3::new(0.76, 0.08, 0.64), red);
    m.cuboid(Vec3::new(0.0, 1.15, -0.36), Vec3::new(0.86, 1.5, 0.12), hex(0x6b3a2a));
    m.cuboid(Vec3::new(0.0, 1.12, -0.29), Vec3::new(0.66, 1.25, 0.04), red);
    for s in [-1.0f32, 1.0] {
        m.cuboid(Vec3::new(s * 0.4, 0.68, 0.0), Vec3::new(0.08, 0.06, 0.66), gold);
        m.cuboid(Vec3::new(s * 0.4, 0.58, 0.3), Vec3::new(0.07, 0.2, 0.07), gold);
        m.sphere(Vec3::new(s * 0.43, 1.95, -0.36), 0.07, 3, 7, gold);
    }
    m.lathe(
        Vec3::new(0.0, 1.9, -0.36),
        &[(0.0, 0.0), (0.2, 0.05), (0.12, 0.18), (0.05, 0.26), (0.0, 0.36)],
        8,
        gold,
    );
    m.sphere(Vec3::new(0.0, 1.6, -0.27), 0.07, 3, 7, hex(0xf0d060));
    p.add(&m);
}

/// Three-armed candelabrum ~1.7 m; part "flames" glows.
fn candelabrum(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let g = hex(0xc9a04a);
    m.frustum(Vec3::ZERO, 0.2, 0.06, 0.12, 8, g);
    m.frustum(Vec3::Y * 0.12, 0.03, 0.025, 1.3, 8, g);
    m.sphere(Vec3::Y * 0.7, 0.05, 3, 7, g);
    m.tube(Vec3::Y * 1.38, Vec3::new(0.24, 1.45, 0.0), 0.018, 0.018, 5, g);
    m.tube(Vec3::Y * 1.38, Vec3::new(-0.24, 1.45, 0.0), 0.018, 0.018, 5, g);
    let mut f = MeshBuilder::new();
    for x in [-0.24f32, 0.0, 0.24] {
        let y = if x == 0.0 { 1.42 } else { 1.45 };
        m.frustum(Vec3::new(x, y, 0.0), 0.05, 0.04, 0.03, 8, g);
        m.frustum(Vec3::new(x, y + 0.03, 0.0), 0.022, 0.022, 0.14, 7, hex(0xf6f0e0));
        f.ellipsoid(Vec3::new(x, y + 0.21, 0.0), Vec3::new(0.018, 0.04, 0.018), 2, 6, hex(0xffd070));
    }
    p.add(&m);
    p.add_part(Some("flames"), &f, Transform::IDENTITY, PartMat::Emissive(hex(0xffb040), 6.0));
}

/// One door leaf 1.4 x 2.4 m, hinge at local x = -0.7 (origin at the leaf centre bottom).
fn door(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let wood = hex(0x7a4a2c);
    for i in 0..5 {
        let x = -0.56 + i as f32 * 0.28;
        m.cuboid(
            Vec3::new(x, 1.2, 0.0),
            Vec3::new(0.27, 2.4, 0.08),
            if i % 2 == 0 { wood } else { hex(0x6c4026) },
        );
    }
    for y in [0.35f32, 1.2, 2.05] {
        m.cuboid(Vec3::new(0.0, y, 0.045), Vec3::new(1.4, 0.07, 0.02), hex(0x3a3a40));
        m.cuboid(Vec3::new(0.0, y, -0.045), Vec3::new(1.4, 0.07, 0.02), hex(0x3a3a40));
    }
    m.lathe(
        Vec3::new(0.48, 1.1, -0.07),
        &[(0.06, 0.0), (0.075, 0.012), (0.06, 0.024), (0.045, 0.012), (0.06, 0.0)],
        10,
        hex(0xd0a040),
    );
    m.lathe(
        Vec3::new(0.48, 1.1, 0.07),
        &[(0.06, 0.0), (0.075, 0.012), (0.06, 0.024), (0.045, 0.012), (0.06, 0.0)],
        10,
        hex(0xd0a040),
    );
    p.add(&m);
}

fn column(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let st = hex(0xe8e0d6);
    m.cuboid(Vec3::new(0.0, 0.1, 0.0), Vec3::new(0.62, 0.2, 0.62), st);
    m.frustum(Vec3::Y * 0.2, 0.24, 0.21, 4.4, 12, st);
    m.cuboid(Vec3::new(0.0, 4.7, 0.0), Vec3::new(0.6, 0.2, 0.6), st);
    m.frustum(Vec3::Y * 4.45, 0.21, 0.29, 0.15, 12, hex(0xd8c070));
    p.add(&m);
}

/// Hanging banner (faces +Z): purple cloth with a golden star.
fn banner(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.05, 0.05), hex(0xc9a04a));
    m.quad(
        Vec3::new(-0.42, -0.02, 0.03),
        Vec3::new(0.42, -0.02, 0.03),
        Vec3::new(0.42, -1.6, 0.03),
        Vec3::new(-0.42, -1.6, 0.03),
        hex(0x5b2a7a),
    );
    m.tri(
        Vec3::new(-0.42, -1.6, 0.03),
        Vec3::new(0.42, -1.6, 0.03),
        Vec3::new(0.0, -1.9, 0.03),
        hex(0x5b2a7a),
    );
    let c = Vec3::new(0.0, -0.8, 0.04);
    for i in 0..5 {
        let a = i as f32 / 5.0 * std::f32::consts::TAU;
        let b = a + std::f32::consts::TAU / 10.0;
        let a2 = a - std::f32::consts::TAU / 10.0;
        let tip = c + Vec3::new(a.sin() * 0.24, a.cos() * 0.24, 0.0);
        let l = c + Vec3::new(a2.sin() * 0.1, a2.cos() * 0.1, 0.0);
        let r = c + Vec3::new(b.sin() * 0.1, b.cos() * 0.1, 0.0);
        m.tri(l, r, tip, hex(GOLD));
        m.tri(c, r, l, hex(GOLD));
    }
    p.add(&m);
}

/// Round tower with a pointed roof and a gold finial (~8 m).
fn tower(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 1.1, 1.0, 6.2, 14, hex(0xeee6dc));
    m.frustum(Vec3::Y * 6.2, 1.18, 1.18, 0.3, 14, hex(0xd8c070));
    m.frustum(Vec3::Y * 6.5, 1.25, 0.0, 2.2, 14, hex(0x4a5a9a));
    m.tube(Vec3::Y * 8.6, Vec3::Y * 9.2, 0.04, 0.02, 5, hex(GOLD));
    m.sphere(Vec3::Y * 9.25, 0.09, 3, 6, hex(GOLD));
    for i in 0..4 {
        let a = i as f32 / 4.0 * std::f32::consts::TAU + 0.4;
        m.cuboid(Vec3::new(a.cos() * 1.03, 4.4, a.sin() * 1.03), Vec3::new(0.3, 0.7, 0.3), hex(0x2a2440));
    }
    p.add(&m);
}

/// A small velvet footstool for the audience stance (decor).
fn cushion(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.ellipsoid(Vec3::new(0.0, 0.08, 0.0), Vec3::new(0.25, 0.08, 0.2), 3, 9, hex(0x8a2a4a));
    for (x, z) in [(-0.17f32, -0.12f32), (0.17, -0.12), (-0.17, 0.12), (0.17, 0.12)] {
        m.sphere(Vec3::new(x, 0.08, z), 0.02, 2, 4, hex(GOLD));
    }
    p.add(&m);
}
