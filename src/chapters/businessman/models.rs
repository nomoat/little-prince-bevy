//! Procedural fallbacks for every businessman asset key (all replaceable via
//! `assets/chapters/businessman.ron`). Units: metres, models face +Z, origin
//! on the floor.

use crate::core::prelude::*;
use std::f32::consts::{PI, TAU};

const BRASS: u32 = 0xc9a24a;
const BRASS_DARK: u32 = 0x8a6a2c;
const WOOD: u32 = 0x7a5236;
const WOOD_DARK: u32 = 0x4e3322;
const IRON: u32 = 0x3a3842;
const STONE: u32 = 0x8b8279;
const PLASTER: u32 = 0xd9ccb2;
const PAPER: u32 = 0xeee2c4;
const INK: u32 = 0x2a2a3a;

pub fn register(app: &mut App) {
    app.register_model("businessman.observatory", observatory)
        .register_model("businessman.dome", dome)
        .register_model("businessman.door_leaf", door_leaf)
        .register_model("businessman.telescope", telescope)
        .register_model("businessman.rug", rug)
        .register_model("businessman.desk", desk)
        .register_model("businessman.chair", chair)
        .register_model("businessman.merchant", merchant)
        .register_model("businessman.bookshelf", bookshelf)
        .register_model("businessman.cabinet", cabinet)
        .register_model("businessman.low_cabinet", low_cabinet)
        .register_model("businessman.side_desk", side_desk)
        .register_model("businessman.table", small_table)
        .register_model("businessman.stool", stool)
        .register_model("businessman.armillary", armillary)
        .register_model("businessman.chart_north", chart_north)
        .register_model("businessman.chart_moon", chart_moon)
        .register_model("businessman.chart_constellation", chart_constellation)
        .register_model("businessman.chart_zodiac", chart_zodiac)
        .register_model("businessman.chart_orbits", chart_orbits)
        .register_model("businessman.lantern", lantern)
        .register_model("businessman.oil_lamp", oil_lamp)
        .register_model("businessman.candles", candles)
        .register_model("businessman.sextant", sextant)
        .register_model("businessman.scrolls", scrolls)
        .register_model("businessman.boxes", boxes)
        .register_model("businessman.float_rock", float_rock)
        .register_model("businessman.b612_view", b612_view);
}

/// A circle of tube segments (rings of armillary spheres, dome beams).
pub fn ring(m: &mut MeshBuilder, center: Vec3, radius: f32, rot: Quat, thick: f32, segs: u32, color: Color) {
    for i in 0..segs {
        let a0 = i as f32 / segs as f32 * TAU;
        let a1 = (i + 1) as f32 / segs as f32 * TAU;
        let p0 = center + rot * Vec3::new(a0.cos() * radius, 0.0, a0.sin() * radius);
        let p1 = center + rot * Vec3::new(a1.cos() * radius, 0.0, a1.sin() * radius);
        m.tube(p0, p1, thick, thick, 4, color);
    }
}

/// Plinth, wooden floor, the round wall with the south doorway, lintel,
/// door frame, cornice. (The glass dome is a separate key.)
fn observatory(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    // stone plinth: the floor is 0.36 m above the outside ground
    m.frustum(Vec3::ZERO, 3.8, 3.62, 0.36, 40, hex(STONE));
    m.frustum(Vec3::Y * 0.355, 3.26, 3.26, 0.012, 40, hex(0x8a6446));
    // floor boards (radial planks)
    for i in 0..20 {
        let a = i as f32 / 20.0 * TAU;
        let d = Vec3::new(a.sin(), 0.0, a.cos());
        m.tube(Vec3::Y * 0.37 + d * 0.3, Vec3::Y * 0.37 + d * 3.2, 0.006, 0.006, 3, hex(0x5e4330));
    }
    // doorstep
    m.cuboid(Vec3::new(0.0, 0.09, -3.92), Vec3::new(1.9, 0.18, 0.5), hex(0x9a9086));
    // walls: 24 segments of 15°, the two facing south leave the doorway
    let r = 3.375;
    let wall_h = 2.55;
    for i in 0..24 {
        let a = (i as f32 * 15.0 + 7.5).to_radians();
        let pos = Vec3::new(a.sin() * r, 0.0, a.cos() * r);
        let door = (a.to_degrees() - 180.0).abs() < 10.0;
        let t = Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(a));
        let shade = if i % 2 == 0 { hex(PLASTER) } else { scale_color(hex(PLASTER), 0.96) };
        m.with(t, |m| {
            if door {
                m.cuboid(Vec3::new(0.0, 0.36 + 2.25 + 0.15, 0.0), Vec3::new(0.9, 0.3, 0.25), shade);
            } else {
                m.cuboid(Vec3::new(0.0, 0.36 + wall_h * 0.5, 0.0), Vec3::new(0.9, wall_h, 0.25), shade);
                // wainscot inside
                m.cuboid(Vec3::new(0.0, 0.36 + 0.45, -0.135), Vec3::new(0.9, 0.9, 0.02), hex(WOOD));
                // stone base band outside
                m.cuboid(Vec3::new(0.0, 0.36 + 0.2, 0.135), Vec3::new(0.9, 0.4, 0.02), hex(0xb7ab98));
            }
            // cornice
            m.cuboid(Vec3::new(0.0, 0.36 + wall_h + 0.06, 0.05), Vec3::new(0.93, 0.12, 0.42), hex(0xb9aa8c));
        });
        // little round windows on the east/west/north
        if !door && i % 4 == 1 {
            let wp = Vec3::new(a.sin() * (r + 0.13), 1.95, a.cos() * (r + 0.13));
            m.with(Transform::from_translation(wp).with_rotation(Quat::from_rotation_y(a) * Quat::from_rotation_x(PI / 2.0)), |m| {
                m.frustum(Vec3::ZERO, 0.24, 0.24, 0.03, 14, hex(BRASS_DARK));
                m.frustum(Vec3::Y * 0.02, 0.19, 0.19, 0.02, 14, hex(0x1d2440));
            });
        }
    }
    // door frame (brass-capped posts)
    for x in [-0.9f32, 0.9] {
        m.cuboid(Vec3::new(x, 0.36 + 1.15, -3.36), Vec3::new(0.12, 2.3, 0.32), hex(WOOD_DARK));
    }
    m.cuboid(Vec3::new(0.0, 0.36 + 2.3, -3.36), Vec3::new(1.92, 0.12, 0.34), hex(WOOD_DARK));
    // a brass plate above the door
    m.cuboid(Vec3::new(0.0, 0.36 + 2.42, -3.55), Vec3::new(0.6, 0.12, 0.02), hex(BRASS));
    p.add(&m);
}

/// The glass dome: hemisphere glass, iron ribs, ring beam and a small cap.
fn dome(p: &mut ModelParts) {
    let base = Vec3::Y * 2.97;
    let rad = 3.5;
    let mut g = MeshBuilder::new();
    let rings = 9;
    let segs = 32;
    let pt = |i: u32, j: u32| {
        let th = i as f32 / rings as f32 * (PI / 2.0) * 0.96;
        let ph = j as f32 / segs as f32 * TAU;
        base + Vec3::new(th.cos() * ph.sin() * rad, th.sin() * rad * 0.82, th.cos() * ph.cos() * rad)
    };
    for i in 0..rings {
        for j in 0..segs {
            let tint = if (i + j) % 2 == 0 { hex(0x9fb8d8) } else { hex(0x8aa6cc) };
            g.quad(pt(i, j), pt(i, j + 1), pt(i + 1, j + 1), pt(i + 1, j), tint);
        }
    }
    let glass = StandardMaterial {
        base_color: Color::srgba(0.75, 0.85, 1.0, 0.13),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        perceptual_roughness: 0.05,
        reflectance: 0.7,
        double_sided: true,
        ..default()
    };
    p.add_part(Some("glass"), &g, Transform::IDENTITY, PartMat::Custom(glass));
    let mut f = MeshBuilder::new();
    for j in 0..12 {
        for i in 0..rings {
            f.tube(pt(i, j * segs / 12), pt(i + 1, j * segs / 12), 0.035, 0.035, 4, hex(IRON));
        }
    }
    for i in [0u32, 3, 6] {
        for j in 0..segs {
            f.tube(pt(i, j), pt(i, j + 1), 0.03, 0.03, 4, hex(if i == 0 { BRASS_DARK } else { IRON }));
        }
    }
    let top = pt(rings, 0);
    f.frustum(top - Vec3::Y * 0.05, 0.42, 0.28, 0.16, 16, hex(BRASS_DARK));
    f.sphere(top + Vec3::Y * 0.2, 0.08, 4, 8, hex(BRASS));
    p.add(&f);
}

/// One leaf of the double door: from the hinge (origin) along +X, 0.85 wide.
fn door_leaf(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.425, 1.125, 0.0), Vec3::new(0.84, 2.25, 0.06), hex(0x5b3a25));
    for (y, h) in [(0.62f32, 0.8f32), (1.62, 0.95)] {
        for z in [-0.035f32, 0.035] {
            m.cuboid(Vec3::new(0.43, y, z), Vec3::new(0.6, h, 0.012), hex(0x6e4a30));
        }
    }
    // iron straps and a brass ring handle near the seam
    for y in [0.35f32, 1.95] {
        m.cuboid(Vec3::new(0.3, y, 0.0), Vec3::new(0.56, 0.05, 0.075), hex(IRON));
    }
    for z in [-0.06f32, 0.06] {
        m.with(Transform::from_xyz(0.74, 1.05, z).with_rotation(Quat::from_rotation_x(PI / 2.0)), |m| {
            m.frustum(Vec3::ZERO, 0.05, 0.05, 0.02, 10, hex(BRASS));
        });
    }
    p.add(&m);
}

/// Brass telescope on a wooden tripod. Eyepiece at (0, 1.085, -0.451), tube
/// elevated 30° toward +Z (the original's TelescopeLook numbers).
fn telescope(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let apex = Vec3::new(0.0, 1.22, 0.02);
    for i in 0..3 {
        let a = (i as f32 * 120.0 + 60.0).to_radians();
        let foot = Vec3::new(a.sin() * 0.5, 0.0, a.cos() * 0.5);
        m.tube(foot, apex, 0.03, 0.022, 5, hex(WOOD));
        m.sphere(foot, 0.035, 3, 6, hex(BRASS_DARK));
    }
    m.frustum(apex - Vec3::Y * 0.06, 0.07, 0.06, 0.1, 10, hex(BRASS_DARK));
    let eye = Vec3::new(0.0, 1.085, -0.451);
    let dir = Vec3::new(0.0, 0.5, 0.866);
    let front = eye + dir * 1.35;
    m.tube(eye + dir * 0.08, front, 0.055, 0.085, 14, hex(BRASS));
    for k in [0.25f32, 0.7, 1.2] {
        let c = eye + dir * k;
        m.tube(c, c + dir * 0.04, 0.075 + k * 0.012, 0.075 + k * 0.012, 14, hex(BRASS_DARK));
    }
    m.tube(eye - dir * 0.04, eye + dir * 0.1, 0.022, 0.03, 8, hex(IRON));
    m.tube(front, front + dir * 0.03, 0.095, 0.095, 14, hex(BRASS_DARK));
    // fork and finder scope
    m.tube(apex, eye + dir * 0.55, 0.025, 0.025, 6, hex(IRON));
    let finder = eye + dir * 0.4 + Vec3::new(0.11, 0.07, -0.02);
    m.tube(finder, finder + dir * 0.35, 0.018, 0.022, 8, hex(BRASS));
    p.add(&m);
    let mut lens = MeshBuilder::new();
    lens.tube(front + dir * 0.025, front + dir * 0.03, 0.08, 0.08, 14, hex(0x9fc4ff));
    p.add_part(Some("lens"), &lens, Transform::IDENTITY, PartMat::Emissive(hex(0x7aa6ff), 0.8));
}

/// The star rug under the telescope.
fn rug(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 1.35, 1.35, 0.012, 40, hex(0xa8873e));
    m.frustum(Vec3::Y * 0.004, 1.25, 1.25, 0.012, 40, hex(0x1d2a52));
    let mut rng = Rng::new(328);
    for k in 0..26 {
        let a = rng.range(0.0, TAU);
        let r = if k < 8 { 0.95 + rng.range(0.0, 0.18) } else { rng.range(0.15, 1.05) };
        let c = Vec3::new(a.sin() * r, 0.019, a.cos() * r);
        let s = rng.range(0.025, 0.06);
        star(&mut m, c, s, hex(0xf2d27a));
    }
    p.add(&m);
}

/// A flat 4-point star lying on XZ at `c`.
fn star(m: &mut MeshBuilder, c: Vec3, s: f32, col: Color) {
    for i in 0..4 {
        let a = i as f32 * PI / 2.0;
        let tip = c + Vec3::new(a.sin(), 0.0, a.cos()) * s * 2.2;
        let l = c + Vec3::new((a + PI / 2.0).sin(), 0.0, (a + PI / 2.0).cos()) * s * 0.5;
        let r = c + Vec3::new((a - PI / 2.0).sin(), 0.0, (a - PI / 2.0).cos()) * s * 0.5;
        m.tri(c, l, tip, col);
        m.tri(c, tip, r, col);
    }
}

/// Writing desk (front = -Z toward the door, the merchant sits at +Z).
/// Named parts: "drawer" (slides +Z), "slip" (the paper slip).
fn desk(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.76, 0.0), Vec3::new(1.5, 0.06, 0.78), hex(WOOD));
    m.cuboid(Vec3::new(0.0, 0.79, 0.0), Vec3::new(1.36, 0.004, 0.64), hex(0x2f5a3a)); // leather inlay
    for (x, z) in [(-0.68f32, -0.33f32), (0.68, -0.33), (-0.68, 0.33), (0.68, 0.33)] {
        m.cuboid(Vec3::new(x, 0.37, z), Vec3::new(0.08, 0.74, 0.08), hex(WOOD_DARK));
    }
    // modesty panel facing the door, pedestal with drawers on the left
    m.cuboid(Vec3::new(0.0, 0.5, -0.36), Vec3::new(1.3, 0.46, 0.03), hex(WOOD_DARK));
    m.cuboid(Vec3::new(-0.45, 0.37, 0.0), Vec3::new(0.46, 0.74, 0.7), hex(WOOD));
    for y in [0.18f32, 0.42] {
        m.cuboid(Vec3::new(-0.45, y, 0.355), Vec3::new(0.4, 0.18, 0.01), hex(0x8a5e3c));
        m.sphere(Vec3::new(-0.45, y, 0.37), 0.018, 3, 6, hex(BRASS));
    }
    // the open ledger, ink pot, quill, paper stacks, an abacus
    let lx = -0.04;
    m.with(Transform::from_xyz(lx, 0.8, 0.17).with_rotation(Quat::from_rotation_y(0.06)), |m| {
        m.cuboid(Vec3::new(-0.16, 0.012, 0.0), Vec3::new(0.32, 0.024, 0.42), hex(PAPER));
        m.cuboid(Vec3::new(0.16, 0.012, 0.0), Vec3::new(0.32, 0.024, 0.42), hex(0xf2e8cf));
        m.cuboid(Vec3::new(0.0, 0.004, 0.0), Vec3::new(0.68, 0.008, 0.45), hex(0x6a2a24));
        for k in 0..9 {
            let z = -0.16 + k as f32 * 0.04;
            m.cuboid(Vec3::new(-0.16, 0.025, z), Vec3::new(0.24, 0.002, 0.006), hex(0x6d6a78));
            m.cuboid(Vec3::new(0.15, 0.025, z), Vec3::new(0.2 - (k % 3) as f32 * 0.04, 0.002, 0.006), hex(0x6d6a78));
        }
    });
    m.frustum(Vec3::new(0.45, 0.79, 0.12), 0.045, 0.035, 0.07, 10, hex(INK));
    m.tube(Vec3::new(0.45, 0.85, 0.12), Vec3::new(0.38, 1.05, 0.2), 0.006, 0.002, 4, hex(0xe8e4dc));
    for k in 0..6 {
        m.cuboid(Vec3::new(0.52, 0.8 + k as f32 * 0.012, -0.18), Vec3::new(0.24, 0.01, 0.3), scale_color(hex(PAPER), 0.92 + 0.02 * k as f32));
    }
    m.cuboid(Vec3::new(-0.5, 0.82, -0.16), Vec3::new(0.34, 0.04, 0.2), hex(WOOD_DARK));
    for r in 0..4 {
        for b in 0..6 {
            m.sphere(Vec3::new(-0.63 + b as f32 * 0.05, 0.85, -0.22 + r as f32 * 0.04), 0.014, 2, 5, hex(if b < 2 { 0xb04030 } else { 0xd8c8a0 }));
        }
    }
    p.add(&m);
    let mut d = MeshBuilder::new();
    d.cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.12, 0.6), hex(0x8a5e3c));
    d.cuboid(Vec3::new(0.0, 0.0, 0.3), Vec3::new(0.52, 0.14, 0.02), hex(0x6e4a30));
    d.sphere(Vec3::new(0.0, 0.0, 0.32), 0.02, 3, 6, hex(BRASS));
    p.add_named("drawer", &d, Transform::from_xyz(0.3, 0.66, 0.0));
    let mut s = MeshBuilder::new();
    s.cuboid(Vec3::ZERO, Vec3::new(0.14, 0.004, 0.08), hex(0xfff6e0));
    s.cuboid(Vec3::new(0.0, 0.003, 0.0), Vec3::new(0.1, 0.002, 0.012), hex(INK));
    p.add_named("slip", &s, Transform::from_xyz(0.3, 0.805, 0.22));
}

/// The merchant's high-backed chair (seat faces +Z).
fn chair(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.45, 0.0), Vec3::new(0.52, 0.07, 0.5), hex(WOOD_DARK));
    m.cuboid(Vec3::new(0.0, 0.49, 0.0), Vec3::new(0.46, 0.04, 0.44), hex(0x6b2730));
    for (x, z) in [(-0.22f32, -0.21f32), (0.22, -0.21), (-0.22, 0.21), (0.22, 0.21)] {
        m.cuboid(Vec3::new(x, 0.22, z), Vec3::new(0.05, 0.44, 0.05), hex(WOOD_DARK));
    }
    m.cuboid(Vec3::new(0.0, 0.95, -0.24), Vec3::new(0.52, 0.9, 0.06), hex(WOOD_DARK));
    m.cuboid(Vec3::new(0.0, 0.92, -0.205), Vec3::new(0.42, 0.7, 0.02), hex(0x6b2730));
    m.sphere(Vec3::new(-0.24, 1.42, -0.24), 0.04, 3, 6, hex(BRASS));
    m.sphere(Vec3::new(0.24, 1.42, -0.24), 0.04, 3, 6, hex(BRASS));
    p.add(&m);
}

/// The businessman, seated, facing +Z. Named parts: "head" (pivot at the
/// neck), "arm" (writing arm, pivot at the right shoulder).
fn merchant(p: &mut ModelParts) {
    let suit = hex(0x34384c);
    let vest = hex(0x7a3030);
    let skin = hex(0xf0cfb2);
    let mut b = MeshBuilder::new();
    // seated legs
    for x in [-0.1f32, 0.1] {
        b.tube(Vec3::new(x, 0.52, -0.02), Vec3::new(x, 0.52, 0.36), 0.075, 0.065, 8, suit);
        b.tube(Vec3::new(x, 0.52, 0.36), Vec3::new(x, 0.06, 0.4), 0.06, 0.05, 8, suit);
        b.cuboid(Vec3::new(x, 0.04, 0.45), Vec3::new(0.09, 0.07, 0.2), hex(0x1d1a18));
    }
    // torso: round belly, waistcoat, coat, collar
    b.lathe(Vec3::new(0.0, 0.48, 0.0), &[(0.0, 0.0), (0.21, 0.02), (0.25, 0.15), (0.24, 0.29), (0.2, 0.4), (0.12, 0.47), (0.0, 0.49)], 12, suit);
    b.ellipsoid(Vec3::new(0.0, 0.68, 0.06), Vec3::new(0.18, 0.17, 0.17), 4, 10, vest);
    b.cuboid(Vec3::new(0.0, 0.9, 0.1), Vec3::new(0.08, 0.08, 0.04), hex(0xf4f0e6));
    for k in 0..3 {
        b.sphere(Vec3::new(0.0, 0.6 + k as f32 * 0.07, 0.225), 0.012, 2, 5, hex(BRASS));
    }
    // left arm resting on the desk
    b.tube(Vec3::new(0.22, 0.9, 0.0), Vec3::new(0.26, 0.78, 0.22), 0.055, 0.05, 6, suit);
    b.tube(Vec3::new(0.26, 0.78, 0.22), Vec3::new(0.16, 0.8, 0.42), 0.05, 0.045, 6, suit);
    b.sphere(Vec3::new(0.14, 0.8, 0.46), 0.045, 3, 6, skin);
    p.add_named("body", &b, Transform::IDENTITY);
    // head: pivot at the neck; bald crown, grey fringe, spectacles
    let mut h = MeshBuilder::new();
    h.tube(Vec3::ZERO, Vec3::Y * 0.08, 0.05, 0.05, 6, skin);
    h.sphere(Vec3::Y * 0.18, 0.13, 6, 10, skin);
    h.ellipsoid(Vec3::new(0.0, 0.15, -0.03), Vec3::new(0.138, 0.07, 0.11), 3, 10, hex(0xb7b2a8)); // grey fringe
    h.sphere(Vec3::new(0.0, 0.17, 0.13), 0.03, 3, 6, hex(0xe8b89a)); // nose
    h.ellipsoid(Vec3::new(0.0, 0.12, 0.12), Vec3::new(0.07, 0.018, 0.02), 2, 6, hex(0x9a948a)); // moustache
    for x in [-0.05f32, 0.05] {
        h.with(Transform::from_xyz(x, 0.2, 0.12).with_rotation(Quat::from_rotation_x(PI / 2.0)), |m| {
            m.frustum(Vec3::ZERO, 0.035, 0.035, 0.008, 10, hex(BRASS));
        });
    }
    h.tube(Vec3::new(-0.015, 0.2, 0.125), Vec3::new(0.015, 0.2, 0.125), 0.004, 0.004, 3, hex(BRASS));
    p.add_named("head", &h, Transform::from_xyz(0.0, 0.93, 0.02));
    // writing arm: pivot at the right shoulder, hand on the ledger
    let mut a = MeshBuilder::new();
    let elbow = Vec3::new(0.02, -0.16, 0.18);
    let hand = Vec3::new(0.14, -0.11, 0.42);
    a.tube(Vec3::ZERO, elbow, 0.055, 0.05, 6, suit);
    a.tube(elbow, hand, 0.05, 0.045, 6, suit);
    a.sphere(hand, 0.045, 3, 6, skin);
    a.tube(hand + Vec3::new(0.0, -0.04, 0.03), hand + Vec3::new(-0.03, 0.12, -0.02), 0.006, 0.004, 4, hex(0xd8c070));
    p.add_named("arm", &a, Transform::from_xyz(-0.22, 0.9, 0.0));
}

fn bookshelf(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 1.0, -0.15), Vec3::new(1.0, 2.0, 0.04), hex(WOOD_DARK));
    for x in [-0.49f32, 0.49] {
        m.cuboid(Vec3::new(x, 1.0, 0.0), Vec3::new(0.04, 2.0, 0.34), hex(WOOD));
    }
    let mut rng = Rng::new(51);
    let cols = [0x7a2c2c, 0x2c4a7a, 0x3d6a3a, 0x8a6a2a, 0x5a3a6a, 0x9a8a6a, 0x2a2a3a];
    for s in 0..5 {
        let y = 0.04 + s as f32 * 0.46;
        m.cuboid(Vec3::new(0.0, y, 0.0), Vec3::new(0.96, 0.03, 0.32), hex(WOOD));
        if s == 4 {
            continue;
        }
        let mut x = -0.45;
        while x < 0.42 {
            let w = rng.range(0.035, 0.07);
            let h = rng.range(0.24, 0.38);
            let c = cols[(rng.next_u32() % cols.len() as u32) as usize];
            if rng.f() > 0.12 {
                m.cuboid(Vec3::new(x + w * 0.5, y + 0.015 + h * 0.5, 0.0), Vec3::new(w * 0.92, h, 0.24), hex(c));
            }
            x += w;
        }
    }
    m.cuboid(Vec3::new(0.0, 2.02, 0.0), Vec3::new(1.04, 0.05, 0.36), hex(WOOD_DARK));
    p.add(&m);
}

fn drawers(m: &mut MeshBuilder, w: f32, h: f32, d: f32, rows: u32, cols: u32) {
    m.cuboid(Vec3::new(0.0, h * 0.5, 0.0), Vec3::new(w, h, d), hex(WOOD));
    let (cw, ch) = (w / cols as f32, (h - 0.08) / rows as f32);
    for r in 0..rows {
        for c in 0..cols {
            let x = -w * 0.5 + cw * (c as f32 + 0.5);
            let y = 0.06 + ch * (r as f32 + 0.5);
            m.cuboid(Vec3::new(x, y, d * 0.5 + 0.005), Vec3::new(cw * 0.86, ch * 0.8, 0.01), hex(0x8a5e3c));
            m.cuboid(Vec3::new(x, y + ch * 0.18, d * 0.5 + 0.012), Vec3::new(cw * 0.3, 0.035, 0.008), hex(PAPER));
            m.sphere(Vec3::new(x, y - 0.02, d * 0.5 + 0.02), 0.013, 2, 5, hex(BRASS));
        }
    }
    m.cuboid(Vec3::new(0.0, h + 0.02, 0.0), Vec3::new(w + 0.04, 0.04, d + 0.04), hex(WOOD_DARK));
}

/// Archive cabinet with many little labelled drawers.
fn cabinet(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    drawers(&mut m, 0.9, 1.5, 0.42, 6, 4);
    for k in 0..3 {
        m.with(Transform::from_xyz(-0.25 + k as f32 * 0.22, 1.56, 0.0).with_rotation(Quat::from_rotation_z(PI / 2.0)), |m| {
            m.tube(Vec3::new(0.0, -0.12, 0.0), Vec3::new(0.0, 0.12, 0.0), 0.04, 0.04, 8, hex(PAPER));
        });
    }
    p.add(&m);
}

fn low_cabinet(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    drawers(&mut m, 1.0, 0.8, 0.42, 2, 3);
    p.add(&m);
}

/// Side desk with an open star atlas.
fn side_desk(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.74, 0.0), Vec3::new(1.1, 0.05, 0.55), hex(WOOD));
    for (x, z) in [(-0.5f32, -0.23f32), (0.5, -0.23), (-0.5, 0.23), (0.5, 0.23)] {
        m.cuboid(Vec3::new(x, 0.37, z), Vec3::new(0.06, 0.72, 0.06), hex(WOOD_DARK));
    }
    m.cuboid(Vec3::new(-0.15, 0.775, 0.02), Vec3::new(0.5, 0.02, 0.34), hex(0x1d2a52));
    let mut rng = Rng::new(7);
    for _ in 0..18 {
        m.cuboid(Vec3::new(-0.15 + rng.range(-0.22, 0.22), 0.787, 0.02 + rng.range(-0.15, 0.15)), Vec3::splat(0.012), hex(0xf2d27a));
    }
    m.cuboid(Vec3::new(0.32, 0.78, -0.05), Vec3::new(0.18, 0.03, 0.26), hex(0x6a2a24));
    p.add(&m);
}

fn small_table(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::Y * 0.7, 0.42, 0.42, 0.04, 20, hex(WOOD));
    m.tube(Vec3::Y * 0.05, Vec3::Y * 0.7, 0.04, 0.035, 8, hex(WOOD_DARK));
    m.frustum(Vec3::ZERO, 0.24, 0.1, 0.06, 12, hex(WOOD_DARK));
    p.add(&m);
}

fn stool(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::Y * 0.46, 0.17, 0.17, 0.05, 14, hex(0x6b2730));
    for i in 0..3 {
        let a = (i as f32 * 120.0).to_radians();
        m.tube(Vec3::new(a.sin() * 0.18, 0.0, a.cos() * 0.18), Vec3::new(a.sin() * 0.1, 0.46, a.cos() * 0.1), 0.02, 0.02, 5, hex(WOOD_DARK));
    }
    p.add(&m);
}

/// Armillary sphere on a stand (height ~0.55, sits on a table top).
fn armillary(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.1, 0.07, 0.04, 12, hex(BRASS_DARK));
    m.tube(Vec3::Y * 0.04, Vec3::Y * 0.22, 0.015, 0.012, 6, hex(BRASS_DARK));
    let c = Vec3::Y * 0.4;
    ring(&mut m, c, 0.17, Quat::IDENTITY, 0.007, 20, hex(BRASS));
    ring(&mut m, c, 0.17, Quat::from_rotation_x(PI / 2.0), 0.007, 20, hex(BRASS));
    ring(&mut m, c, 0.17, Quat::from_rotation_z(PI / 2.0), 0.007, 20, hex(BRASS));
    ring(&mut m, c, 0.15, Quat::from_rotation_x(0.41), 0.006, 20, hex(BRASS_DARK));
    m.tube(c - Vec3::new(0.0, 0.2, 0.0), c + Vec3::new(0.0, 0.2, 0.0), 0.005, 0.005, 4, hex(IRON));
    m.sphere(c, 0.045, 4, 8, hex(0x3a6aa8));
    p.add(&m);
}

fn chart_base(m: &mut MeshBuilder) {
    m.cuboid(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.9, 0.7, 0.02), hex(WOOD_DARK));
    m.cuboid(Vec3::new(0.0, 0.0, 0.012), Vec3::new(0.82, 0.62, 0.01), hex(0xe7d7b0));
}

fn dot(m: &mut MeshBuilder, x: f32, y: f32, s: f32, c: u32) {
    m.cuboid(Vec3::new(x, y, 0.02), Vec3::new(s, s, 0.004), hex(c));
}

fn circle(m: &mut MeshBuilder, x: f32, y: f32, r: f32, c: u32) {
    ring(m, Vec3::new(x, y, 0.02), r, Quat::from_rotation_x(PI / 2.0), 0.0035, 28, hex(c));
}

/// Wall charts (hang on the wall, face +Z, centred at the origin).
fn chart_north(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    chart_base(&mut m);
    for (i, r) in [0.08f32, 0.16, 0.24].iter().enumerate() {
        circle(&mut m, 0.0, 0.0, *r, if i == 2 { 0x8a5a2a } else { 0x6d6a78 });
    }
    let mut rng = Rng::new(3);
    for _ in 0..30 {
        dot(&mut m, rng.range(-0.36, 0.36), rng.range(-0.27, 0.27), rng.range(0.008, 0.02), INK);
    }
    dot(&mut m, 0.0, 0.0, 0.03, 0xb03a2a);
    p.add(&m);
}
fn chart_moon(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    chart_base(&mut m);
    for k in 0..6 {
        let x = -0.32 + k as f32 * 0.128;
        m.with(Transform::from_xyz(x, 0.08, 0.02).with_rotation(Quat::from_rotation_x(PI / 2.0)), |m| {
            m.frustum(Vec3::ZERO, 0.05, 0.05, 0.004, 14, hex(0x2a2a3a));
        });
        m.with(Transform::from_xyz(x + (k as f32 - 2.5) * 0.012, 0.08, 0.026).with_rotation(Quat::from_rotation_x(PI / 2.0)), |m| {
            m.frustum(Vec3::ZERO, 0.045, 0.045, 0.004, 14, hex(0xeae2c8));
        });
    }
    m.cuboid(Vec3::new(0.0, -0.12, 0.02), Vec3::new(0.7, 0.006, 0.004), hex(INK));
    p.add(&m);
}
fn chart_constellation(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    chart_base(&mut m);
    m.cuboid(Vec3::new(0.0, 0.0, 0.016), Vec3::new(0.78, 0.58, 0.004), hex(0x1d2a52));
    let pts = [(-0.3f32, 0.15f32), (-0.15, 0.2), (0.0, 0.1), (0.12, 0.18), (0.25, 0.05), (0.1, -0.12), (-0.1, -0.18)];
    for w in pts.windows(2) {
        m.tube(Vec3::new(w[0].0, w[0].1, 0.022), Vec3::new(w[1].0, w[1].1, 0.022), 0.002, 0.002, 3, hex(0x8aa0d0));
    }
    for (x, y) in pts {
        dot(&mut m, x, y, 0.022, 0xf2d27a);
    }
    p.add(&m);
}
fn chart_zodiac(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    chart_base(&mut m);
    circle(&mut m, 0.0, 0.0, 0.26, 0x6d6a78);
    circle(&mut m, 0.0, 0.0, 0.19, 0x6d6a78);
    for k in 0..12 {
        let a = k as f32 / 12.0 * TAU;
        m.tube(Vec3::new(a.cos() * 0.19, a.sin() * 0.19, 0.02), Vec3::new(a.cos() * 0.26, a.sin() * 0.26, 0.02), 0.002, 0.002, 3, hex(0x6d6a78));
        let b = a + TAU / 24.0;
        dot(&mut m, b.cos() * 0.225, b.sin() * 0.225, 0.02, [0xb03a2a, 0x2c4a7a, 0x3d6a3a][k % 3]);
    }
    dot(&mut m, 0.0, 0.0, 0.05, 0xd8a43a);
    p.add(&m);
}
fn chart_orbits(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    chart_base(&mut m);
    dot(&mut m, 0.0, 0.0, 0.05, 0xd8a43a);
    for (i, r) in [0.07f32, 0.12, 0.18, 0.25].iter().enumerate() {
        circle(&mut m, 0.0, 0.0, *r, 0x6d6a78);
        let a = i as f32 * 1.9;
        dot(&mut m, a.cos() * r, a.sin() * r, 0.022, [0x8a8a8a, 0xc9a24a, 0x3a6aa8, 0xb03a2a][i]);
    }
    p.add(&m);
}

/// Wall lantern on a bracket (bracket at the origin against the wall,
/// lantern stands out along +Z). Named part "flame".
fn lantern(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.0, 0.02), Vec3::new(0.08, 0.16, 0.04), hex(IRON));
    m.tube(Vec3::new(0.0, 0.0, 0.03), Vec3::new(0.0, 0.08, 0.22), 0.012, 0.012, 5, hex(IRON));
    let c = Vec3::new(0.0, -0.06, 0.22);
    m.frustum(c - Vec3::Y * 0.12, 0.07, 0.07, 0.02, 6, hex(IRON));
    m.frustum(c + Vec3::Y * 0.1, 0.09, 0.0, 0.08, 6, hex(IRON));
    for i in 0..6 {
        let a = (i as f32 * 60.0).to_radians();
        let o = Vec3::new(a.sin() * 0.07, 0.0, a.cos() * 0.07);
        m.tube(c + o - Vec3::Y * 0.1, c + o + Vec3::Y * 0.1, 0.006, 0.006, 3, hex(IRON));
    }
    p.add(&m);
    let mut g = MeshBuilder::new();
    g.frustum(c - Vec3::Y * 0.1, 0.066, 0.066, 0.2, 6, hex(0xffe0a0));
    p.add_part(Some("glass"), &g, Transform::IDENTITY, PartMat::Glass(Color::srgba(1.0, 0.85, 0.55, 0.35)));
    let mut f = MeshBuilder::new();
    f.ellipsoid(Vec3::ZERO, Vec3::new(0.018, 0.04, 0.018), 3, 6, hex(0xffffff));
    p.add_part(Some("flame"), &f, Transform::from_translation(c), PartMat::Emissive(hex(0xffb050), 8.0));
}

fn oil_lamp(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.06, 0.05, 0.02, 10, hex(BRASS_DARK));
    m.lathe(Vec3::Y * 0.02, &[(0.0, 0.0), (0.06, 0.0), (0.07, 0.04), (0.04, 0.08), (0.02, 0.09)], 10, hex(BRASS));
    p.add(&m);
    let mut g = MeshBuilder::new();
    g.lathe(Vec3::Y * 0.1, &[(0.025, 0.0), (0.045, 0.05), (0.035, 0.13), (0.03, 0.14)], 10, hex(0xffe8c0));
    p.add_part(Some("glass"), &g, Transform::IDENTITY, PartMat::Glass(Color::srgba(1.0, 0.92, 0.75, 0.3)));
    let mut f = MeshBuilder::new();
    f.ellipsoid(Vec3::ZERO, Vec3::new(0.012, 0.03, 0.012), 3, 6, hex(0xffffff));
    p.add_part(Some("flame"), &f, Transform::from_xyz(0.0, 0.15, 0.0), PartMat::Emissive(hex(0xffb050), 9.0));
}

fn candles(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.08, 0.07, 0.015, 12, hex(BRASS_DARK));
    let spots = [(0.0f32, 0.0f32, 0.14f32), (0.04, 0.03, 0.1), (-0.035, 0.03, 0.08)];
    for (x, z, h) in spots {
        m.frustum(Vec3::new(x, 0.015, z), 0.016, 0.015, h, 8, hex(0xf2ead6));
    }
    p.add(&m);
    let mut f = MeshBuilder::new();
    for (x, z, h) in spots {
        f.ellipsoid(Vec3::new(x, 0.03 + h, z), Vec3::new(0.008, 0.02, 0.008), 2, 5, hex(0xffffff));
    }
    p.add_part(Some("flame"), &f, Transform::IDENTITY, PartMat::Emissive(hex(0xffb050), 9.0));
}

fn sextant(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let rot = Quat::from_rotation_x(PI / 2.0);
    for k in 0..7 {
        let a0 = -0.5 + k as f32 * 0.17;
        let a1 = a0 + 0.17;
        let q = |a: f32| Vec3::new(a.sin() * 0.16, 0.02, a.cos() * 0.16);
        m.tube(q(a0), q(a1), 0.006, 0.006, 4, hex(BRASS));
    }
    let _ = rot;
    m.tube(Vec3::new(0.0, 0.02, 0.0), Vec3::new((-0.5f32).sin() * 0.16, 0.02, (-0.5f32).cos() * 0.16), 0.005, 0.005, 4, hex(BRASS));
    m.tube(Vec3::new(0.0, 0.02, 0.0), Vec3::new(0.69f32.sin() * 0.16, 0.02, 0.69f32.cos() * 0.16), 0.005, 0.005, 4, hex(BRASS));
    m.tube(Vec3::new(0.0, 0.02, 0.0), Vec3::new(0.05, 0.02, 0.15), 0.006, 0.006, 4, hex(BRASS_DARK));
    m.tube(Vec3::new(0.02, 0.03, 0.05), Vec3::new(0.09, 0.03, 0.08), 0.012, 0.012, 6, hex(IRON));
    p.add(&m);
}

fn scrolls(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for (i, (x, z, a)) in [(0.0f32, 0.0f32, 0.2f32), (0.06, 0.07, -0.3), (-0.05, 0.08, 1.2), (0.02, 0.03, 0.9)].iter().enumerate() {
        let y = 0.035 + if i == 3 { 0.06 } else { 0.0 };
        let d = Vec3::new(a.sin(), 0.0, a.cos()) * 0.17;
        m.tube(Vec3::new(*x, y, *z) - d, Vec3::new(*x, y, *z) + d, 0.035, 0.035, 8, scale_color(hex(PAPER), 0.92 + 0.03 * i as f32));
        m.tube(Vec3::new(*x, y, *z) - d * 0.2, Vec3::new(*x, y, *z) + d * 0.2, 0.037, 0.037, 8, hex(0x8a2a2a));
    }
    p.add(&m);
}

fn boxes(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.18, 0.0), Vec3::new(0.5, 0.36, 0.4), hex(WOOD));
    m.cuboid(Vec3::new(0.0, 0.36, 0.0), Vec3::new(0.52, 0.03, 0.42), hex(WOOD_DARK));
    m.cuboid(Vec3::new(0.05, 0.5, 0.02), Vec3::new(0.34, 0.24, 0.3), hex(0x8a6446));
    m.cuboid(Vec3::new(0.05, 0.63, 0.02), Vec3::new(0.36, 0.02, 0.32), hex(WOOD_DARK));
    m.cuboid(Vec3::new(0.05, 0.5, 0.175), Vec3::new(0.1, 0.06, 0.01), hex(BRASS));
    p.add(&m);
}

/// A floating rock (fallback for the Kenney rock).
fn float_rock(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::ZERO, Vec3::new(0.7, 0.5, 0.6), 11, hex(0x6f6a72));
    m.rock(Vec3::new(0.3, 0.25, 0.1), Vec3::new(0.35, 0.3, 0.3), 12, hex(0x7f7a80));
    p.add(&m);
}

/// B-612 as seen through the telescope: a small round world with three
/// volcanoes, a rose and a baobab sprout. Radius 1, centred at the origin,
/// faces +Z. Unlit (it is a picture in the sky), no fog.
fn b612_view(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let rings = 14;
    let segs = 24;
    for i in 0..rings {
        for j in 0..segs {
            let pt = |i: u32, j: u32| {
                let th = i as f32 / rings as f32 * PI;
                let ph = j as f32 / segs as f32 * TAU;
                Vec3::new(th.sin() * ph.sin(), th.cos(), th.sin() * ph.cos())
            };
            let a = pt(i, j);
            let lightk = (a.dot(Vec3::new(-0.4, 0.5, 0.75).normalize()) * 0.5 + 0.5).powf(0.8);
            let base = if a.y > 0.55 { hex(0x8fae5a) } else if (i * 7 + j * 3) % 11 == 0 { hex(0x8c9a5c) } else { hex(0x7f9e52) };
            let c = mix_color(scale_color(base, 0.35), base, lightk);
            m.quad(a, pt(i + 1, j), pt(i + 1, j + 1), pt(i, j + 1), c);
        }
    }
    // three volcanoes on the upper rim, a rose, a sprout
    for (k, (ax, ay)) in [(-0.55f32, 0.7f32), (-0.2, 0.92), (0.45, 0.78)].iter().enumerate() {
        let n = Vec3::new(*ax, *ay, 0.35).normalize();
        let base = n * 0.98;
        let rot = Quat::from_rotation_arc(Vec3::Y, n);
        m.with(Transform::from_translation(base).with_rotation(rot), |m| {
            m.frustum(Vec3::ZERO, 0.17, 0.06, 0.16 + k as f32 * 0.02, 8, hex(0x6f625a));
            m.frustum(Vec3::Y * 0.16, 0.05, 0.0, 0.02, 8, hex(if k == 2 { 0x5a5048 } else { 0xff8a3d }));
        });
    }
    let rn = Vec3::new(0.1, 0.75, 0.66).normalize();
    m.with(Transform::from_translation(rn * 0.98).with_rotation(Quat::from_rotation_arc(Vec3::Y, rn)), |m| {
        m.tube(Vec3::ZERO, Vec3::Y * 0.16, 0.012, 0.01, 4, hex(0x3f7a3a));
        m.sphere(Vec3::Y * 0.19, 0.045, 3, 6, hex(0xe0404c));
    });
    let sn = Vec3::new(0.62, 0.42, 0.66).normalize();
    m.with(Transform::from_translation(sn * 0.98).with_rotation(Quat::from_rotation_arc(Vec3::Y, sn)), |m| {
        m.tube(Vec3::ZERO, Vec3::Y * 0.08, 0.012, 0.008, 4, hex(0x6c8f3e));
        m.sphere(Vec3::Y * 0.09, 0.03, 2, 5, hex(0x8fbe5e));
    });
    let mat = StandardMaterial { base_color: Color::WHITE, unlit: true, fog_enabled: false, ..default() };
    p.add_part(Some("planet"), &m, Transform::IDENTITY, PartMat::Custom(mat));
}
