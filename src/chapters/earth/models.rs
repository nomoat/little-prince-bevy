//! Procedural fallbacks for every Earth asset key (assets/chapters/earth.ron).
//! Convention: metres, +Y up, front faces +Z, origin at the foot / base centre.
//! Named parts (`PartName`) are animated by the chapter's actor systems; a glTF
//! replacement without those parts simply stays static.

use crate::core::models::Part;
use crate::core::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub fn register(app: &mut App) {
    app.register_model("earth.snake", snake)
        .register_model("earth.snake_rock", snake_rock)
        .register_model("earth.flower", little_flower)
        .register_model("earth.fox", fox)
        .register_model("earth.apple_tree", apple_tree)
        .register_model("earth.rose_bed", rose_bed)
        .register_model("earth.garden_wall", garden_wall)
        .register_model("earth.hedge", hedge)
        .register_model("earth.gate", gate)
        .register_model("earth.arch", arch)
        .register_model("earth.fountain", fountain)
        .register_model("earth.urn", urn)
        .register_model("earth.topiary", topiary)
        .register_model("earth.bridge", bridge)
        .register_model("earth.cottage", cottage)
        .register_model("earth.signpost", signpost)
        .register_model("earth.cairn", cairn)
        .register_model("earth.platform", platform)
        .register_model("earth.cabin", cabin)
        .register_model("earth.bench", bench)
        .register_model("earth.signal", signal)
        .register_model("earth.notice", notice_board)
        .register_model("earth.pole", telegraph_pole)
        .register_model("earth.rail", rail)
        .register_model("earth.buffer", buffer_stop)
        .register_model("earth.loco", loco)
        .register_model("earth.carriage", carriage)
        .register_model("earth.plane", plane)
        .register_model("earth.crate", crate_box)
        .register_model("earth.wrench", wrench)
        .register_model("earth.bedroll", bedroll)
        .register_model("earth.pilot", pilot)
        .register_model("earth.well", well)
        .register_model("earth.stone_seat", stone_seat)
        .register_model("earth.column", column)
        .register_model("earth.column_broken", column_broken)
        .register_model("earth.block", block)
        .register_model("earth.slab", slab)
        .register_model("earth.doorway", doorway)
        .register_model("earth.ruin_wall", ruin_wall)
        .register_model("earth.step", step_block)
        .register_model("earth.landing", landing)
        .register_model("earth.stardust", stardust)
        .register_model("earth.star", star)
        .register_model("earth.footprint", footprint)
        .register_model("earth.cactus", cactus)
        .register_model("earth.cactus_s", cactus_small)
        .register_model("earth.peak", peak)
        .register_model("earth.boulder", boulder)
        .register_model("earth.wheat", wheat)
        .register_model("earth.bush", desert_bush)
        .register_model("earth.crop", crop)
        .register_model("earth.waterfall", waterfall)
        .register_model("earth.dune_grass", dune_grass);
}

// ------------------------------------------------------------ palette ----
const SAND: u32 = 0xe2c08d;
const STONE: u32 = 0xb9ab95;
const STONE_DARK: u32 = 0x8f826f;
const SANDSTONE: u32 = 0xd2b088;
const WOOD: u32 = 0x8a5f3c;
const WOOD_DARK: u32 = 0x5e4129;
const LEAF: u32 = 0x5f9a4e;
const LEAF_DARK: u32 = 0x4a7d3e;
const ROSE: u32 = 0xd8434e;
const IRON: u32 = 0x3a3a44;

fn torus_ring(m: &mut MeshBuilder, c: Vec3, ring_r: f32, tube_r: f32, segs: u32, color: Color) {
    for i in 0..segs {
        let a0 = i as f32 / segs as f32 * TAU;
        let a1 = (i + 1) as f32 / segs as f32 * TAU;
        let p0 = c + Vec3::new(a0.cos() * ring_r, 0.0, a0.sin() * ring_r);
        let p1 = c + Vec3::new(a1.cos() * ring_r, 0.0, a1.sin() * ring_r);
        m.tube(p0, p1, tube_r, tube_r, 6, color);
        m.sphere(p1, tube_r, 2, 6, color);
    }
}

// ------------------------------------------------------------- snake ----

/// Coiled desert snake (~1 m long). Parts: body coil + "neck" (pivot at the
/// coil top; straight up = raised, tilted forward = resting).
fn snake(p: &mut ModelParts) {
    let body = hex(0xd9b45a);
    let band = hex(0x8a6a2a);
    let mut m = MeshBuilder::new();
    torus_ring(&mut m, Vec3::Y * 0.04, 0.17, 0.042, 12, body);
    torus_ring(&mut m, Vec3::Y * 0.1, 0.12, 0.038, 10, band);
    torus_ring(&mut m, Vec3::Y * 0.15, 0.075, 0.034, 8, body);
    // tail tip trailing out
    m.tube(Vec3::new(0.17, 0.03, 0.0), Vec3::new(0.32, 0.015, -0.12), 0.03, 0.006, 5, body);
    p.add(&m);
    let mut n = MeshBuilder::new();
    let pts = [Vec3::ZERO, Vec3::new(0.0, 0.11, 0.025), Vec3::new(0.0, 0.22, 0.0), Vec3::new(0.0, 0.3, 0.05)];
    for (i, w) in pts.windows(2).enumerate() {
        let c = if i % 2 == 0 { body } else { band };
        n.tube(w[0], w[1], 0.034 - i as f32 * 0.003, 0.031 - i as f32 * 0.003, 7, c);
        n.sphere(w[1], 0.031 - i as f32 * 0.003, 2, 7, c);
    }
    n.ellipsoid(Vec3::new(0.0, 0.315, 0.09), Vec3::new(0.045, 0.03, 0.07), 3, 8, body);
    n.sphere(Vec3::new(0.03, 0.33, 0.12), 0.009, 2, 4, hex(0x1b1b22));
    n.sphere(Vec3::new(-0.03, 0.33, 0.12), 0.009, 2, 4, hex(0x1b1b22));
    n.tube(Vec3::new(0.0, 0.305, 0.155), Vec3::new(0.0, 0.3, 0.19), 0.004, 0.002, 3, hex(0xb0303a));
    p.add_part(Some("neck"), &n, Transform::from_xyz(0.0, 0.16, 0.0), PartMat::Vertex);
}

/// The snake's rock: top 0.30 m above the sand.
fn snake_rock(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::Y * 0.12, Vec3::new(0.62, 0.2, 0.55), 11, hex(0xa8957a));
    m.with(Transform::from_xyz(0.0, 0.26, 0.0), |m| {
        m.frustum(Vec3::ZERO, 0.5, 0.44, 0.04, 9, hex(0xb7a386));
    });
    p.add(&m);
}

/// A small three-petal desert flower (~0.3 m).
fn little_flower(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::new(0.0, 0.26, 0.02), 0.012, 0.009, 5, hex(0x6f8f45));
    m.ellipsoid(Vec3::new(0.05, 0.08, 0.0), Vec3::new(0.05, 0.012, 0.02), 2, 5, hex(0x7ea24f));
    m.ellipsoid(Vec3::new(-0.04, 0.12, 0.01), Vec3::new(0.045, 0.012, 0.02), 2, 5, hex(0x7ea24f));
    for i in 0..3 {
        let a = i as f32 / 3.0 * TAU;
        m.with(Transform::from_xyz(0.0, 0.27, 0.02).with_rotation(Quat::from_rotation_y(a) * Quat::from_rotation_x(0.5)), |m| {
            m.ellipsoid(Vec3::new(0.0, 0.0, 0.045), Vec3::new(0.03, 0.01, 0.05), 2, 6, hex(0xf3dfe6));
        });
    }
    m.sphere(Vec3::new(0.0, 0.28, 0.02), 0.018, 2, 5, hex(0xf0c75a));
    p.add(&m);
    let mut sand = MeshBuilder::new();
    sand.frustum(Vec3::ZERO, 0.18, 0.12, 0.03, 8, hex(0xd2ae78));
    p.add(&sand);
}

// --------------------------------------------------------------- fox ----

pub const FOX_BODY_Y: f32 = 0.32;

/// The fox (~0.75 m long). Parts: body (pivot at the rear hips), head (neck
/// joint), tail (root), four legs (hip/shoulder pivots).
fn fox(p: &mut ModelParts) {
    let orange = hex(0xd9792e);
    let white = hex(0xf4ece0);
    let dark = hex(0x3a2a22);
    let mut b = MeshBuilder::new();
    b.ellipsoid(Vec3::new(0.0, 0.0, 0.2), Vec3::new(0.12, 0.12, 0.27), 4, 9, orange);
    b.ellipsoid(Vec3::new(0.0, -0.04, 0.32), Vec3::new(0.085, 0.08, 0.12), 3, 7, white);
    p.add_part(Some("body"), &b, Transform::from_xyz(0.0, FOX_BODY_Y, -0.2), PartMat::Vertex);
    let mut h = MeshBuilder::new();
    h.sphere(Vec3::new(0.0, 0.05, 0.05), 0.1, 4, 8, orange);
    h.tube(Vec3::new(0.0, 0.03, 0.1), Vec3::new(0.0, 0.01, 0.24), 0.06, 0.02, 7, orange);
    h.ellipsoid(Vec3::new(0.0, 0.0, 0.14), Vec3::new(0.065, 0.045, 0.07), 3, 7, white);
    h.sphere(Vec3::new(0.0, 0.012, 0.245), 0.018, 2, 5, dark);
    for s in [-1.0f32, 1.0] {
        h.with(Transform::from_xyz(0.055 * s, 0.13, 0.03).with_rotation(Quat::from_rotation_z(-0.25 * s)), |m| {
            m.frustum(Vec3::ZERO, 0.045, 0.0, 0.11, 4, orange);
            m.frustum(Vec3::Y * 0.07, 0.018, 0.0, 0.045, 4, dark);
        });
        h.sphere(Vec3::new(0.04 * s, 0.08, 0.13), 0.013, 2, 4, dark);
    }
    p.add_part(Some("head"), &h, Transform::from_xyz(0.0, 0.42, 0.24), PartMat::Vertex);
    let mut t = MeshBuilder::new();
    t.ellipsoid(Vec3::new(0.0, 0.0, -0.2), Vec3::new(0.075, 0.075, 0.22), 3, 7, orange);
    t.ellipsoid(Vec3::new(0.0, 0.0, -0.39), Vec3::new(0.05, 0.05, 0.07), 2, 6, white);
    p.add_part(Some("tail"), &t, Transform::from_xyz(0.0, 0.36, -0.25).with_rotation(Quat::from_rotation_x(0.5)), PartMat::Vertex);
    let mut l = MeshBuilder::new();
    l.tube(Vec3::ZERO, Vec3::new(0.0, -0.28, 0.0), 0.028, 0.018, 5, orange);
    l.tube(Vec3::new(0.0, -0.2, 0.0), Vec3::new(0.0, -0.3, 0.012), 0.02, 0.018, 5, dark);
    for (name, x, z) in [("leg_fl", 0.065, 0.13), ("leg_fr", -0.065, 0.13), ("leg_bl", 0.075, -0.17), ("leg_br", -0.075, -0.17)] {
        p.add_part(Some(name), &l, Transform::from_xyz(x, 0.3, z), PartMat::Vertex);
    }
}

fn apple_tree(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.32, 0.0), (0.22, 0.3), (0.18, 1.6), (0.14, 2.2)], 8, hex(0x6e4c30));
    m.tube(Vec3::new(0.0, 1.7, 0.0), Vec3::new(0.7, 2.5, 0.2), 0.1, 0.06, 6, hex(0x6e4c30));
    m.tube(Vec3::new(0.0, 1.8, 0.0), Vec3::new(-0.6, 2.6, -0.3), 0.1, 0.06, 6, hex(0x6e4c30));
    let blobs = [(0.0, 3.0, 0.0, 1.4), (0.9, 2.7, 0.3, 1.0), (-0.9, 2.8, -0.2, 1.05), (0.2, 2.6, -0.9, 0.95), (-0.2, 2.6, 0.9, 0.95), (0.3, 3.7, 0.1, 0.9)];
    for (i, (x, y, z, r)) in blobs.iter().enumerate() {
        m.sphere(Vec3::new(*x, *y, *z), *r, 4, 8, if i % 2 == 0 { hex(LEAF) } else { hex(0x6eaa58) });
    }
    p.add(&m);
    let mut a = MeshBuilder::new();
    let mut rng = Rng::new(17);
    for _ in 0..18 {
        let (bx, by, bz, r) = blobs[(rng.f() * blobs.len() as f32) as usize % blobs.len()];
        let th = rng.range(0.0, TAU);
        let ph = rng.range(0.3, 2.4);
        let c = Vec3::new(bx, by, bz) + Vec3::new(ph.sin() * th.cos(), ph.cos(), ph.sin() * th.sin()) * (r * 0.97);
        a.sphere(c, 0.09, 2, 6, hex(0xd23a2e));
    }
    p.add(&a);
}

// ------------------------------------------------------- rose garden ----

fn rose_dots(m: &mut MeshBuilder, center: Vec3, half: Vec2, n: u32, seed: u64) {
    let mut rng = Rng::new(seed);
    for _ in 0..n {
        let c = center + Vec3::new(rng.range(-half.x, half.x), rng.range(-0.02, 0.06), rng.range(-half.y, half.y));
        let col = if rng.f() < 0.8 { hex(ROSE) } else { hex(0xf08da0) };
        m.sphere(c, rng.range(0.045, 0.07), 2, 5, col);
    }
}

/// A rose bed, 1.8 × 0.9 m.
fn rose_bed(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.09, Vec3::new(1.9, 0.18, 1.0), hex(STONE));
    m.cuboid(Vec3::Y * 0.17, Vec3::new(1.75, 0.04, 0.85), hex(0x5d4632));
    let mut rng = Rng::new(5);
    for i in 0..7 {
        for j in 0..3 {
            let c = Vec3::new(-0.72 + i as f32 * 0.24 + rng.range(-0.03, 0.03), 0.38, -0.26 + j as f32 * 0.26);
            m.sphere(c, 0.15, 3, 6, if (i + j) % 2 == 0 { hex(LEAF) } else { hex(LEAF_DARK) });
        }
    }
    rose_dots(&mut m, Vec3::Y * 0.5, Vec2::new(0.78, 0.36), 34, 9);
    p.add(&m);
}

/// Stone garden wall, 2 m long (along X), roses on top.
fn garden_wall(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.45, Vec3::new(2.02, 0.9, 0.4), hex(STONE));
    m.cuboid(Vec3::Y * 0.93, Vec3::new(2.08, 0.07, 0.46), hex(STONE_DARK));
    for i in 0..4 {
        m.sphere(Vec3::new(-0.75 + i as f32 * 0.5, 1.08, 0.0), 0.2, 3, 6, hex(LEAF));
    }
    rose_dots(&mut m, Vec3::Y * 1.13, Vec2::new(0.9, 0.12), 12, 21);
    p.add(&m);
}

/// Clipped hedge, 2 m long (along X).
fn hedge(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.5, Vec3::new(2.02, 1.0, 0.6), hex(LEAF_DARK));
    m.cuboid(Vec3::Y * 1.02, Vec3::new(1.9, 0.08, 0.5), hex(LEAF));
    rose_dots(&mut m, Vec3::new(0.0, 0.7, 0.31), Vec2::new(0.9, 0.0), 8, 33);
    p.add(&m);
}

/// Iron double gate standing open (opening 2 m wide, along X).
fn gate(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for s in [-1.0f32, 1.0] {
        m.cuboid(Vec3::new(1.2 * s, 0.75, 0.0), Vec3::new(0.4, 1.5, 0.4), hex(STONE));
        m.lathe(Vec3::new(1.2 * s, 1.5, 0.0), &[(0.12, 0.0), (0.16, 0.08), (0.0, 0.3)], 6, hex(STONE_DARK));
        // open leaf swung inward (+Z), hinged at the pillar
        let hinge = Vec3::new(1.0 * s, 0.0, 0.0);
        m.with(Transform::from_translation(hinge).with_rotation(Quat::from_rotation_y(-1.25 * s)), |m| {
            for k in 0..6 {
                let x = -s * (0.08 + k as f32 * 0.17);
                m.cuboid(Vec3::new(x, 0.65, 0.0), Vec3::new(0.03, 1.2, 0.03), hex(IRON));
                m.sphere(Vec3::new(x, 1.28, 0.0), 0.035, 2, 4, hex(IRON));
            }
            m.cuboid(Vec3::new(-s * 0.5, 0.15, 0.0), Vec3::new(0.96, 0.04, 0.04), hex(IRON));
            m.cuboid(Vec3::new(-s * 0.5, 1.1, 0.0), Vec3::new(0.96, 0.04, 0.04), hex(IRON));
        });
    }
    p.add(&m);
}

/// Rose arch over a 2 m opening (along X).
fn arch(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for s in [-1.0f32, 1.0] {
        m.cuboid(Vec3::new(1.1 * s, 1.0, 0.0), Vec3::new(0.18, 2.0, 0.18), hex(0xe9e4d8));
    }
    let n = 9;
    for i in 0..n {
        let a0 = i as f32 / n as f32 * PI;
        let a1 = (i + 1) as f32 / n as f32 * PI;
        let p0 = Vec3::new(a0.cos() * 1.1, 2.0 + a0.sin() * 0.55, 0.0);
        let p1 = Vec3::new(a1.cos() * 1.1, 2.0 + a1.sin() * 0.55, 0.0);
        m.tube(p0, p1, 0.07, 0.07, 5, hex(0xe9e4d8));
        m.sphere(p0, 0.22, 3, 6, hex(LEAF));
    }
    rose_dots(&mut m, Vec3::new(0.0, 2.35, 0.0), Vec2::new(1.0, 0.18), 18, 41);
    for s in [-1.0f32, 1.0] {
        rose_dots(&mut m, Vec3::new(1.1 * s, 1.3, 0.0), Vec2::new(0.15, 0.15), 6, 43 + s as u64);
    }
    p.add(&m);
}

fn fountain(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.0, 0.0), (1.0, 0.0), (1.05, 0.42), (0.92, 0.48), (0.86, 0.3), (0.0, 0.3)], 18, hex(STONE));
    m.lathe(Vec3::ZERO, &[(0.16, 0.3), (0.12, 1.0), (0.45, 1.12), (0.42, 1.22), (0.0, 1.16)], 12, hex(0xc7bba8));
    m.lathe(Vec3::Y * 1.2, &[(0.08, 0.0), (0.05, 0.25), (0.1, 0.32), (0.0, 0.36)], 8, hex(0xc7bba8));
    p.add(&m);
    let mut w = MeshBuilder::new();
    w.frustum(Vec3::Y * 0.36, 0.88, 0.88, 0.02, 18, hex(0x6fa8c8));
    w.frustum(Vec3::Y * 1.14, 0.4, 0.4, 0.02, 12, hex(0x6fa8c8));
    w.tube(Vec3::Y * 1.5, Vec3::Y * 1.2, 0.03, 0.06, 6, hex(0xbfe2f2));
    p.add_part(Some("water"), &w, Transform::IDENTITY, PartMat::Glass(Color::srgba(0.45, 0.72, 0.88, 0.7)));
}

fn urn(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.0, 0.0), (0.2, 0.0), (0.12, 0.12), (0.26, 0.45), (0.22, 0.6), (0.27, 0.66), (0.0, 0.6)], 10, hex(0xd8d0c0));
    p.add(&m);
    let mut f = MeshBuilder::new();
    f.sphere(Vec3::Y * 0.7, 0.24, 3, 7, hex(LEAF));
    rose_dots(&mut f, Vec3::Y * 0.84, Vec2::new(0.14, 0.14), 7, 51);
    p.add(&f);
}

fn topiary(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.24, 0.3, 0.4, 8, hex(0xb5654a));
    m.tube(Vec3::Y * 0.4, Vec3::Y * 0.8, 0.04, 0.04, 5, hex(WOOD));
    m.sphere(Vec3::Y * 1.05, 0.38, 4, 8, hex(LEAF_DARK));
    m.sphere(Vec3::Y * 1.55, 0.22, 3, 7, hex(LEAF));
    p.add(&m);
}

/// Wooden footbridge, deck 3.4 m long along Z, 1.3 m wide, deck top at 0.32 m.
fn bridge(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for i in 0..13 {
        let z = -1.62 + i as f32 * 0.27;
        let arch = 0.32 - (z / 1.7).powi(2) * 0.12;
        m.cuboid(Vec3::new(0.0, arch - 0.04, z), Vec3::new(1.3, 0.07, 0.24), if i % 2 == 0 { hex(WOOD) } else { hex(0x9a6c45) });
    }
    for s in [-1.0f32, 1.0] {
        m.cuboid(Vec3::new(0.62 * s, 0.12, 0.0), Vec3::new(0.1, 0.2, 3.5), hex(WOOD_DARK));
        for k in 0..4 {
            let z = -1.6 + k as f32 * 1.07;
            m.cuboid(Vec3::new(0.62 * s, 0.55, z), Vec3::new(0.08, 0.8, 0.08), hex(WOOD_DARK));
        }
        m.cuboid(Vec3::new(0.62 * s, 0.92, 0.0), Vec3::new(0.07, 0.07, 3.3), hex(WOOD));
    }
    p.add(&m);
}

fn cottage(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 1.1, Vec3::new(3.2, 2.2, 2.8), hex(0xcfc2a8));
    m.cuboid(Vec3::Y * 0.15, Vec3::new(3.3, 0.3, 2.9), hex(STONE_DARK));
    // pitched roof (ridge along X)
    let (w, d, h0, h1) = (1.8f32, 1.65f32, 2.2f32, 3.5f32);
    let roof = hex(0x9c4a32);
    m.quad(Vec3::new(-w, h0, d), Vec3::new(w, h0, d), Vec3::new(w, h1, 0.0), Vec3::new(-w, h1, 0.0), roof);
    m.quad(Vec3::new(w, h0, -d), Vec3::new(-w, h0, -d), Vec3::new(-w, h1, 0.0), Vec3::new(w, h1, 0.0), scale_color(roof, 0.85));
    m.tri(Vec3::new(1.6, 2.2, 1.4), Vec3::new(1.6, 2.2, -1.4), Vec3::new(1.6, 3.4, 0.0), hex(0xcfc2a8));
    m.tri(Vec3::new(-1.6, 2.2, -1.4), Vec3::new(-1.6, 2.2, 1.4), Vec3::new(-1.6, 3.4, 0.0), hex(0xcfc2a8));
    m.cuboid(Vec3::new(1.0, 3.4, -0.5), Vec3::new(0.35, 0.9, 0.35), hex(STONE_DARK));
    m.cuboid(Vec3::new(0.0, 0.85, 1.41), Vec3::new(0.7, 1.5, 0.05), hex(WOOD_DARK));
    for x in [-1.0f32, 1.0] {
        m.cuboid(Vec3::new(x, 1.45, 1.41), Vec3::new(0.55, 0.5, 0.05), hex(0x5f7d96));
        m.cuboid(Vec3::new(x, 1.15, 1.45), Vec3::new(0.65, 0.07, 0.1), hex(WOOD));
    }
    p.add(&m);
}

/// Way sign: post + two arrow boards pointing +Z.
fn signpost(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.75, Vec3::new(0.09, 1.5, 0.09), hex(WOOD_DARK));
    for (y, l, c) in [(1.3f32, 0.55f32, 0xb98a55u32), (1.05, 0.45, 0xa67a4a)] {
        m.cuboid(Vec3::new(0.0, y, 0.05 + l * 0.5), Vec3::new(0.05, 0.18, l), hex(c));
        m.with(Transform::from_xyz(0.0, y, 0.05 + l), |m| {
            m.tri(Vec3::new(0.026, 0.13, 0.0), Vec3::new(0.026, -0.13, 0.0), Vec3::new(0.026, 0.0, 0.16), hex(c));
            m.tri(Vec3::new(-0.026, -0.13, 0.0), Vec3::new(-0.026, 0.13, 0.0), Vec3::new(-0.026, 0.0, 0.16), hex(c));
            m.quad(Vec3::new(0.026, 0.13, 0.0), Vec3::new(0.026, 0.0, 0.16), Vec3::new(-0.026, 0.0, 0.16), Vec3::new(-0.026, 0.13, 0.0), hex(c));
            m.quad(Vec3::new(-0.026, -0.13, 0.0), Vec3::new(-0.026, 0.0, 0.16), Vec3::new(0.026, 0.0, 0.16), Vec3::new(0.026, -0.13, 0.0), hex(c));
        });
        // painted ink marks on the boards
        for k in 0..3 {
            m.cuboid(Vec3::new(0.03, y, 0.15 + k as f32 * l * 0.25), Vec3::new(0.005, 0.07, l * 0.15), hex(0x3a2c20));
        }
    }
    p.add(&m);
}

fn cairn(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let mut y = 0.0;
    for (i, r) in [0.3f32, 0.24, 0.19, 0.14, 0.1].iter().enumerate() {
        let h = r * 0.55;
        m.rock(Vec3::new(((i * 7) % 3) as f32 * 0.02, y + h * 0.5, 0.0), Vec3::new(*r, h * 0.6, r * 0.9), i as u32 * 3 + 1, hex(0xa69780));
        y += h * 0.95;
    }
    p.add(&m);
}

// ----------------------------------------------------------- station ----

/// Station platform, 14 × 1.8 m (length along X), top at 0.3 m.
fn platform(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.15, Vec3::new(14.0, 0.3, 1.8), hex(0xb6aa98));
    m.cuboid(Vec3::new(0.0, 0.305, 0.82), Vec3::new(14.0, 0.02, 0.12), hex(0xe8d26a));
    for i in 0..14 {
        m.cuboid(Vec3::new(-6.5 + i as f32, 0.302, 0.0), Vec3::new(0.02, 0.01, 1.7), hex(0x9a8f80));
    }
    p.add(&m);
}

fn cabin(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 1.2, Vec3::new(3.2, 2.4, 2.4), hex(0xc98f5a));
    for i in 0..8 {
        m.cuboid(Vec3::new(0.0, 0.2 + i as f32 * 0.3, 1.21), Vec3::new(3.2, 0.03, 0.02), hex(0x9c6a3e));
    }
    m.cuboid(Vec3::new(0.0, 2.5, 0.0), Vec3::new(3.8, 0.12, 3.0), hex(0x6a4a3a));
    m.quad(Vec3::new(-1.9, 2.56, 1.5), Vec3::new(1.9, 2.56, 1.5), Vec3::new(1.9, 3.3, 0.0), Vec3::new(-1.9, 3.3, 0.0), hex(0x7d3b2c));
    m.quad(Vec3::new(1.9, 2.56, -1.5), Vec3::new(-1.9, 2.56, -1.5), Vec3::new(-1.9, 3.3, 0.0), Vec3::new(1.9, 3.3, 0.0), hex(0x6a3226));
    m.cuboid(Vec3::new(-0.7, 0.95, 1.22), Vec3::new(0.75, 1.7, 0.04), hex(WOOD_DARK));
    m.cuboid(Vec3::new(0.75, 1.4, 1.22), Vec3::new(0.9, 0.7, 0.04), hex(0x5f7d96));
    m.cuboid(Vec3::new(0.0, 2.25, 1.26), Vec3::new(1.6, 0.32, 0.05), hex(0xf2e6c8));
    m.cuboid(Vec3::new(0.0, 2.25, 1.29), Vec3::new(1.1, 0.08, 0.01), hex(0x3a2c20));
    p.add(&m);
    let mut lamp = MeshBuilder::new();
    lamp.sphere(Vec3::new(0.75, 2.05, 1.32), 0.09, 3, 6, hex(0xffe2a0));
    p.add_part(Some("lamp"), &lamp, Transform::IDENTITY, PartMat::Emissive(hex(0xffc864), 2.5));
}

fn bench(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for x in [-0.6f32, 0.6] {
        m.cuboid(Vec3::new(x, 0.2, 0.0), Vec3::new(0.08, 0.4, 0.4), hex(IRON));
        m.cuboid(Vec3::new(x, 0.6, -0.2), Vec3::new(0.06, 0.5, 0.06), hex(IRON));
    }
    for k in 0..3 {
        m.cuboid(Vec3::new(0.0, 0.42, -0.13 + k as f32 * 0.13), Vec3::new(1.5, 0.04, 0.11), hex(0x9a6c45));
    }
    for k in 0..2 {
        m.cuboid(Vec3::new(0.0, 0.62 + k as f32 * 0.17, -0.22), Vec3::new(1.5, 0.12, 0.03), hex(0x9a6c45));
    }
    p.add(&m);
}

fn signal(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.08, 0.05, 3.2, 6, hex(0x4a4a52));
    m.cuboid(Vec3::new(0.35, 2.9, 0.06), Vec3::new(0.8, 0.14, 0.04), hex(0xc8332e));
    m.cuboid(Vec3::new(0.55, 2.9, 0.08), Vec3::new(0.12, 0.15, 0.01), hex(0xf2f2f2));
    m.cuboid(Vec3::new(0.0, 2.55, 0.08), Vec3::new(0.22, 0.32, 0.12), hex(0x222228));
    p.add(&m);
    let mut l = MeshBuilder::new();
    l.sphere(Vec3::new(0.0, 2.6, 0.16), 0.06, 2, 6, hex(0x9be06a));
    p.add_part(Some("light"), &l, Transform::IDENTITY, PartMat::Emissive(hex(0x7bd84a), 3.0));
}

fn notice_board(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for x in [-0.6f32, 0.6] {
        m.cuboid(Vec3::new(x, 0.8, 0.0), Vec3::new(0.08, 1.6, 0.08), hex(WOOD_DARK));
    }
    m.cuboid(Vec3::new(0.0, 1.2, 0.0), Vec3::new(1.3, 0.8, 0.05), hex(0x7a5536));
    for (x, y, c) in [(-0.35f32, 1.3f32, 0xf2ead2u32), (0.1, 1.15, 0xe9dfc0), (0.4, 1.35, 0xf4e8d8)] {
        m.cuboid(Vec3::new(x, y, 0.03), Vec3::new(0.3, 0.38, 0.01), hex(c));
    }
    m.quad(Vec3::new(-0.75, 1.62, 0.15), Vec3::new(0.75, 1.62, 0.15), Vec3::new(0.75, 1.75, -0.1), Vec3::new(-0.75, 1.75, -0.1), hex(0x6a3226));
    p.add(&m);
}

fn telegraph_pole(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.1, 0.07, 4.2, 6, hex(0x6e5236));
    m.cuboid(Vec3::Y * 3.9, Vec3::new(1.1, 0.08, 0.08), hex(0x6e5236));
    for x in [-0.45f32, -0.15, 0.15, 0.45] {
        m.frustum(Vec3::new(x, 3.94, 0.0), 0.03, 0.025, 0.1, 5, hex(0xdfe6ea));
    }
    p.add(&m);
}

/// A 2 m rail segment along Z (gauge 1.0 m).
fn rail(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.03, Vec3::new(1.9, 0.06, 2.02), hex(0x9a8d7c));
    for k in 0..3 {
        m.cuboid(Vec3::new(0.0, 0.09, -0.66 + k as f32 * 0.66), Vec3::new(1.5, 0.08, 0.22), hex(0x6a4c34));
    }
    for x in [-0.5f32, 0.5] {
        m.cuboid(Vec3::new(x, 0.17, 0.0), Vec3::new(0.07, 0.09, 2.02), hex(0x5c5c66));
    }
    p.add(&m);
}

fn buffer_stop(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for x in [-0.5f32, 0.5] {
        m.tube(Vec3::new(x, 0.1, -0.8), Vec3::new(x, 0.9, 0.0), 0.06, 0.06, 5, hex(0x4a4a52));
        m.cuboid(Vec3::new(x, 0.5, 0.0), Vec3::new(0.08, 0.9, 0.08), hex(0x4a4a52));
    }
    m.cuboid(Vec3::new(0.0, 0.85, 0.05), Vec3::new(1.6, 0.3, 0.2), hex(0xc8332e));
    for k in 0..3 {
        m.cuboid(Vec3::new(-0.5 + k as f32 * 0.5, 0.85, 0.16), Vec3::new(0.22, 0.3, 0.01), hex(0xf2f2f2));
    }
    p.add(&m);
}

fn wheel(m: &mut MeshBuilder, c: Vec3, r: f32, color: Color) {
    m.tube(c - Vec3::X * 0.05, c + Vec3::X * 0.05, r, r, 12, color);
    m.tube(c - Vec3::X * 0.07, c + Vec3::X * 0.07, r * 0.25, r * 0.25, 6, hex(0x2a2a30));
}

/// Steam locomotive, 6.3 m long, front toward +Z.
fn loco(p: &mut ModelParts) {
    let green = hex(0x2f5a46);
    let black = hex(0x26262c);
    let brass = hex(0xd8b04a);
    let red = hex(0xb23a2e);
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.85, 0.0), Vec3::new(2.0, 0.25, 6.3), black);
    m.tube(Vec3::new(0.0, 1.55, -1.6), Vec3::new(0.0, 1.55, 2.7), 0.72, 0.72, 14, green);
    m.tube(Vec3::new(0.0, 1.55, 2.7), Vec3::new(0.0, 1.55, 2.82), 0.72, 0.6, 14, black);
    for z in [-0.6f32, 0.6, 1.8] {
        m.tube(Vec3::new(0.0, 1.55, z), Vec3::new(0.0, 1.55, z + 0.08), 0.75, 0.75, 14, brass);
    }
    m.frustum(Vec3::new(0.0, 2.15, 2.1), 0.2, 0.26, 0.75, 10, black);
    m.frustum(Vec3::new(0.0, 2.88, 2.1), 0.3, 0.3, 0.12, 10, black);
    m.lathe(Vec3::new(0.0, 2.2, 0.6), &[(0.3, 0.0), (0.28, 0.25), (0.15, 0.4), (0.0, 0.42)], 10, brass);
    // cab
    m.cuboid(Vec3::new(0.0, 1.9, -2.3), Vec3::new(2.1, 2.0, 1.7), green);
    m.cuboid(Vec3::new(0.0, 2.95, -2.3), Vec3::new(2.4, 0.12, 2.0), black);
    for x in [-1.06f32, 1.06] {
        m.cuboid(Vec3::new(x, 2.25, -2.3), Vec3::new(0.02, 0.6, 0.9), hex(0x1d2a33));
    }
    // cowcatcher + lamp
    m.with(Transform::from_xyz(0.0, 0.55, 3.25).with_rotation(Quat::from_rotation_x(0.6)), |m| {
        m.cuboid(Vec3::ZERO, Vec3::new(1.8, 0.08, 0.7), red);
    });
    m.cuboid(Vec3::new(0.0, 0.95, 3.15), Vec3::new(2.1, 0.22, 0.12), red);
    for (z, r) in [(1.9f32, 0.45f32), (0.6, 0.62), (-0.7, 0.62), (-2.4, 0.45)] {
        for x in [-0.88f32, 0.88] {
            wheel(&mut m, Vec3::new(x, r, z), r, red);
        }
    }
    m.cuboid(Vec3::new(0.94, 0.62, -0.05), Vec3::new(0.05, 0.08, 1.4), hex(0x9a9aa2));
    m.cuboid(Vec3::new(-0.94, 0.62, -0.05), Vec3::new(0.05, 0.08, 1.4), hex(0x9a9aa2));
    p.add(&m);
    let mut l = MeshBuilder::new();
    l.tube(Vec3::new(0.0, 1.55, 2.82), Vec3::new(0.0, 1.55, 2.9), 0.18, 0.18, 8, hex(0xfff0c0));
    p.add_part(Some("lamp"), &l, Transform::IDENTITY, PartMat::Emissive(hex(0xffd890), 2.0));
}

/// Passenger carriage, 6.5 m long along Z.
fn carriage(p: &mut ModelParts) {
    let body = hex(0x7a2e2a);
    let cream = hex(0xeadfc4);
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.8, 0.0), Vec3::new(1.9, 0.2, 6.3), hex(0x26262c));
    m.cuboid(Vec3::new(0.0, 1.55, 0.0), Vec3::new(2.15, 1.3, 6.3), body);
    m.cuboid(Vec3::new(0.0, 2.45, 0.0), Vec3::new(2.15, 0.5, 6.3), cream);
    m.cuboid(Vec3::new(0.0, 2.75, 0.0), Vec3::new(2.3, 0.1, 6.5), hex(0x4a4a52));
    m.cuboid(Vec3::new(0.0, 2.88, 0.0), Vec3::new(1.6, 0.16, 6.3), hex(0x4a4a52));
    for i in 0..5 {
        let z = -2.4 + i as f32 * 1.2;
        for x in [-1.08f32, 1.08] {
            m.cuboid(Vec3::new(x, 2.4, z), Vec3::new(0.02, 0.45, 0.8), hex(0x2a3c4a));
        }
    }
    for x in [-1.08f32, 1.08] {
        m.cuboid(Vec3::new(x, 1.6, 3.0), Vec3::new(0.02, 1.6, 0.6), hex(0x5e2420));
        m.cuboid(Vec3::new(x, 1.6, -3.0), Vec3::new(0.02, 1.6, 0.6), hex(0x5e2420));
    }
    for z in [-2.2f32, 2.2] {
        m.cuboid(Vec3::new(0.0, 0.5, z), Vec3::new(1.6, 0.25, 1.4), hex(0x26262c));
        for zz in [z - 0.5, z + 0.5] {
            for x in [-0.88f32, 0.88] {
                wheel(&mut m, Vec3::new(x, 0.42, zz), 0.42, hex(0x3a3a40));
            }
        }
    }
    p.add(&m);
}

// --------------------------------------------------------------- dunes ----

/// The aviator's crashed monoplane (fuselage along +Z, nose at +Z).
fn plane(p: &mut ModelParts) {
    let cream = hex(0xe9dcc0);
    let red = hex(0xb84a3a);
    let metal = hex(0x8c8c94);
    let mut m = MeshBuilder::new();
    m.with(Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(0.1) * Quat::from_rotation_x(0.06)), |m| {
        m.tube(Vec3::new(0.0, 1.0, -3.6), Vec3::new(0.0, 1.0, 1.4), 0.25, 0.68, 12, cream);
        m.tube(Vec3::new(0.0, 1.0, 1.4), Vec3::new(0.0, 1.0, 2.4), 0.68, 0.5, 12, red);
        m.tube(Vec3::new(0.0, 1.0, 2.4), Vec3::new(0.0, 1.0, 2.7), 0.5, 0.15, 12, metal);
        // propeller (one blade bent)
        m.cuboid(Vec3::new(0.0, 1.55, 2.75), Vec3::new(0.12, 1.0, 0.05), hex(0x6e5236));
        m.with(Transform::from_xyz(0.0, 0.62, 2.75).with_rotation(Quat::from_rotation_x(0.6)), |m| {
            m.cuboid(Vec3::new(0.0, -0.3, 0.0), Vec3::new(0.12, 0.7, 0.05), hex(0x6e5236));
        });
        // wings
        m.cuboid(Vec3::new(0.0, 0.62, 0.9), Vec3::new(9.4, 0.12, 1.5), cream);
        m.cuboid(Vec3::new(4.2, 0.63, 0.9), Vec3::new(1.0, 0.13, 1.52), red);
        m.cuboid(Vec3::new(-4.2, 0.63, 0.9), Vec3::new(1.0, 0.13, 1.52), red);
        // cockpit
        m.tube(Vec3::new(0.0, 1.62, -0.2), Vec3::new(0.0, 1.66, 0.4), 0.38, 0.38, 10, hex(0x4a3a30));
        m.with(Transform::from_xyz(0.0, 1.62, 0.55).with_rotation(Quat::from_rotation_x(-0.5)), |m| {
            m.cuboid(Vec3::ZERO, Vec3::new(0.7, 0.35, 0.03), hex(0x9fc4d4));
        });
        // tail
        m.cuboid(Vec3::new(0.0, 1.15, -3.4), Vec3::new(2.6, 0.08, 0.8), cream);
        m.with(Transform::from_xyz(0.0, 1.5, -3.45), |m| {
            m.tri(Vec3::new(0.0, -0.3, 0.4), Vec3::new(0.0, -0.3, -0.45), Vec3::new(0.0, 0.7, -0.45), red);
            m.tri(Vec3::new(0.0, -0.3, -0.45), Vec3::new(0.0, -0.3, 0.4), Vec3::new(0.0, 0.7, -0.45), red);
        });
        // landing gear (one wheel buried)
        m.tube(Vec3::new(0.9, 0.62, 1.2), Vec3::new(1.0, 0.25, 1.3), 0.05, 0.05, 5, metal);
        m.tube(Vec3::new(0.95, 0.25, 1.3), Vec3::new(1.1, 0.25, 1.3), 0.25, 0.25, 10, hex(0x2a2a30));
    });
    // a wing panel lying in the sand, tools
    m.with(Transform::from_xyz(-3.4, 0.08, -2.0).with_rotation(Quat::from_rotation_y(0.7)), |m| {
        m.cuboid(Vec3::ZERO, Vec3::new(1.6, 0.08, 0.9), cream);
    });
    p.add(&m);
}

fn crate_box(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.225, Vec3::new(0.7, 0.45, 0.5), hex(0xb08454));
    for y in [0.04f32, 0.41] {
        m.cuboid(Vec3::new(0.0, y, 0.0), Vec3::new(0.72, 0.06, 0.52), hex(0x8a6540));
    }
    for x in [-0.33f32, 0.33] {
        m.cuboid(Vec3::new(x, 0.225, 0.0), Vec3::new(0.06, 0.45, 0.52), hex(0x8a6540));
    }
    m.cuboid(Vec3::new(0.0, 0.25, 0.255), Vec3::new(0.3, 0.12, 0.01), hex(0x3a2c20));
    p.add(&m);
}

fn wrench(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.012, 0.0), Vec3::new(0.035, 0.02, 0.26), hex(0x9a9aa4));
    for z in [-0.14f32, 0.14] {
        m.frustum(Vec3::new(0.0, 0.0, z), 0.04, 0.04, 0.025, 6, hex(0x8a8a94));
    }
    p.add(&m);
}

fn bedroll(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.05, 0.0), Vec3::new(0.85, 0.1, 2.0), hex(0x7b7a52));
    m.cuboid(Vec3::new(0.0, 0.08, 0.4), Vec3::new(0.86, 0.1, 1.1), hex(0x5f6a4a));
    m.ellipsoid(Vec3::new(0.0, 0.14, -0.82), Vec3::new(0.3, 0.09, 0.18), 3, 8, hex(0x6a4a34));
    p.add(&m);
}

/// The aviator (~1.75 m). Parts: torso (pivot at hips, includes nothing below),
/// head, arm_l/arm_r (shoulder pivots), thigh_l/thigh_r (hip), shin_l/shin_r (knee).
/// Part base transforms are recomputed every frame by the pilot pose system.
fn pilot(p: &mut ModelParts) {
    let jacket = hex(0x7a5236);
    let shirt = hex(0xe8e0cc);
    let trouser = hex(0xa79a72);
    let skin = hex(0xe9c3a0);
    let boot = hex(0x3e2c22);
    let mut t = MeshBuilder::new();
    t.lathe(Vec3::ZERO, &[(0.0, 0.0), (0.2, 0.0), (0.22, 0.25), (0.25, 0.5), (0.2, 0.6), (0.0, 0.62)], 10, jacket);
    t.frustum(Vec3::Y * 0.5, 0.12, 0.1, 0.12, 8, shirt);
    t.ellipsoid(Vec3::new(0.06, 0.56, 0.1), Vec3::new(0.09, 0.05, 0.06), 2, 6, hex(0xf2f2f2));
    t.cuboid(Vec3::new(0.0, 0.03, 0.0), Vec3::new(0.42, 0.06, 0.32), hex(0x4a3428));
    p.add_part(Some("torso"), &t, Transform::IDENTITY, PartMat::Vertex);
    let mut h = MeshBuilder::new();
    h.tube(Vec3::ZERO, Vec3::Y * 0.08, 0.05, 0.05, 6, skin);
    h.sphere(Vec3::Y * 0.17, 0.115, 4, 9, skin);
    h.ellipsoid(Vec3::new(0.0, 0.21, -0.01), Vec3::new(0.125, 0.09, 0.13), 3, 9, hex(0x5a3e2c));
    for s in [-1.0f32, 1.0] {
        h.ellipsoid(Vec3::new(0.11 * s, 0.17, 0.0), Vec3::new(0.03, 0.06, 0.05), 2, 5, hex(0x5a3e2c));
        h.sphere(Vec3::new(0.045 * s, 0.25, 0.1), 0.032, 2, 6, hex(0x8aa6b0));
        h.sphere(Vec3::new(0.04 * s, 0.17, 0.105), 0.011, 2, 4, hex(0x2b2b3a));
    }
    h.cuboid(Vec3::new(0.0, 0.25, 0.1), Vec3::new(0.16, 0.025, 0.02), hex(0x3a2a20));
    h.ellipsoid(Vec3::new(0.0, 0.13, 0.11), Vec3::new(0.025, 0.02, 0.025), 2, 4, hex(0xd9a888));
    p.add_part(Some("head"), &h, Transform::IDENTITY, PartMat::Vertex);
    let mut a = MeshBuilder::new();
    a.tube(Vec3::ZERO, Vec3::new(0.0, -0.55, 0.0), 0.065, 0.05, 6, jacket);
    a.sphere(Vec3::new(0.0, -0.6, 0.0), 0.05, 2, 6, skin);
    p.add_part(Some("arm_l"), &a, Transform::IDENTITY, PartMat::Vertex);
    p.add_part(Some("arm_r"), &a, Transform::IDENTITY, PartMat::Vertex);
    let mut th = MeshBuilder::new();
    th.tube(Vec3::ZERO, Vec3::new(0.0, -0.45, 0.0), 0.085, 0.07, 7, trouser);
    p.add_part(Some("thigh_l"), &th, Transform::IDENTITY, PartMat::Vertex);
    p.add_part(Some("thigh_r"), &th, Transform::IDENTITY, PartMat::Vertex);
    let mut sh = MeshBuilder::new();
    sh.tube(Vec3::ZERO, Vec3::new(0.0, -0.33, 0.0), 0.07, 0.06, 7, trouser);
    sh.tube(Vec3::new(0.0, -0.28, 0.0), Vec3::new(0.0, -0.42, 0.0), 0.075, 0.075, 7, boot);
    sh.cuboid(Vec3::new(0.0, -0.43, 0.05), Vec3::new(0.11, 0.07, 0.24), boot);
    p.add_part(Some("shin_l"), &sh, Transform::IDENTITY, PartMat::Vertex);
    p.add_part(Some("shin_r"), &sh, Transform::IDENTITY, PartMat::Vertex);
}

/// Old stone well with a windlass. Part "bucket" (moved by the well rig).
fn well(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let mut rng = Rng::new(3);
    for ring in 0..4 {
        let y = ring as f32 * 0.18;
        for i in 0..14 {
            let a = (i as f32 + (ring % 2) as f32 * 0.5) / 14.0 * TAU;
            let c = Vec3::new(a.cos() * 0.78, y + 0.09, a.sin() * 0.78);
            let col = scale_color(hex(0xa89a84), rng.range(0.85, 1.08));
            m.with(Transform::from_translation(c).with_rotation(Quat::from_rotation_y(-a)), |m| {
                m.cuboid(Vec3::ZERO, Vec3::new(0.24, 0.17, 0.36), col);
            });
        }
    }
    m.frustum(Vec3::Y * 0.45, 0.62, 0.62, 0.01, 14, hex(0x1d2630));
    for s in [-1.0f32, 1.0] {
        m.cuboid(Vec3::new(0.95 * s, 0.95, 0.0), Vec3::new(0.14, 1.9, 0.14), hex(WOOD_DARK));
    }
    m.tube(Vec3::new(-1.0, 1.6, 0.0), Vec3::new(1.0, 1.6, 0.0), 0.08, 0.08, 8, hex(WOOD));
    m.tube(Vec3::new(-0.5, 1.6, 0.0), Vec3::new(0.3, 1.6, 0.0), 0.12, 0.12, 8, hex(0x9a8a6a));
    m.cuboid(Vec3::new(1.12, 1.6, 0.12), Vec3::new(0.05, 0.05, 0.3), hex(IRON));
    m.cuboid(Vec3::new(1.15, 1.6, 0.28), Vec3::new(0.18, 0.04, 0.04), hex(IRON));
    // roof
    m.quad(Vec3::new(-1.1, 1.9, 0.75), Vec3::new(1.1, 1.9, 0.75), Vec3::new(1.1, 2.35, 0.0), Vec3::new(-1.1, 2.35, 0.0), hex(0x8a5a3a));
    m.quad(Vec3::new(1.1, 1.9, -0.75), Vec3::new(-1.1, 1.9, -0.75), Vec3::new(-1.1, 2.35, 0.0), Vec3::new(1.1, 2.35, 0.0), hex(0x7a4c30));
    p.add(&m);
    let mut b = MeshBuilder::new();
    b.lathe(Vec3::ZERO, &[(0.0, 0.0), (0.13, 0.0), (0.17, 0.26), (0.0, 0.24)], 10, hex(0x8a6a48));
    b.frustum(Vec3::Y * 0.22, 0.155, 0.155, 0.01, 10, hex(0x5f9ac0));
    b.tube(Vec3::new(-0.16, 0.26, 0.0), Vec3::new(0.0, 0.42, 0.0), 0.01, 0.01, 4, hex(IRON));
    b.tube(Vec3::new(0.16, 0.26, 0.0), Vec3::new(0.0, 0.42, 0.0), 0.01, 0.01, 4, hex(IRON));
    p.add_part(Some("bucket"), &b, Transform::from_xyz(0.0, -0.6, 0.0), PartMat::Vertex);
    let mut r = MeshBuilder::new();
    r.tube(Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0), 0.012, 0.012, 4, hex(0xc8b48a));
    p.add_part(Some("rope"), &r, Transform::from_xyz(0.0, 1.6, 0.0), PartMat::Vertex);
}

fn stone_seat(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::Y * 0.2, Vec3::new(0.38, 0.22, 0.3), 5, hex(0xb3a58c));
    m.frustum(Vec3::Y * 0.36, 0.3, 0.28, 0.06, 8, hex(0xbfb198));
    p.add(&m);
}

fn column(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.12, Vec3::new(0.7, 0.24, 0.7), hex(SANDSTONE));
    m.lathe(Vec3::Y * 0.24, &[(0.26, 0.0), (0.22, 0.1), (0.2, 2.6), (0.24, 2.7)], 12, hex(0xdcc09a));
    m.cuboid(Vec3::Y * 3.05, Vec3::new(0.66, 0.2, 0.66), hex(SANDSTONE));
    p.add(&m);
}

fn column_broken(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.12, Vec3::new(0.7, 0.24, 0.7), hex(SANDSTONE));
    m.lathe(Vec3::Y * 0.24, &[(0.26, 0.0), (0.22, 0.1), (0.2, 1.4), (0.12, 1.55)], 12, hex(0xdcc09a));
    m.with(Transform::from_xyz(0.9, 0.2, 0.3).with_rotation(Quat::from_rotation_z(FRAC_PI_2 - 0.05)), |m| {
        m.frustum(Vec3::ZERO, 0.2, 0.2, 0.9, 10, hex(0xd5b991));
    });
    p.add(&m);
}

fn block(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::Y * 0.3, Vec3::new(0.55, 0.32, 0.45), 9, hex(SANDSTONE));
    p.add(&m);
}

/// Stone path slab, 1.2 × 0.9 m.
fn slab(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.03, Vec3::new(1.2, 0.08, 0.9), hex(0xcbb795));
    m.cuboid(Vec3::new(0.0, 0.072, 0.0), Vec3::new(1.05, 0.01, 0.75), hex(0xd6c3a2));
    p.add(&m);
}

/// The doorway: two pillars and a lintel; opening 1.6 m wide (along X), 2.4 m high.
fn doorway(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for s in [-1.0f32, 1.0] {
        m.cuboid(Vec3::new(1.1 * s, 1.25, 0.0), Vec3::new(0.6, 2.5, 0.7), hex(SANDSTONE));
        m.cuboid(Vec3::new(1.1 * s, 0.1, 0.0), Vec3::new(0.75, 0.2, 0.85), hex(0xc4a47e));
        for k in 0..3 {
            m.cuboid(Vec3::new(1.1 * s, 0.6 + k as f32 * 0.7, 0.36), Vec3::new(0.5, 0.04, 0.01), hex(0xb08f6a));
        }
    }
    m.cuboid(Vec3::new(0.0, 2.72, 0.0), Vec3::new(3.0, 0.45, 0.8), hex(0xdcc09a));
    m.cuboid(Vec3::new(0.0, 3.0, 0.0), Vec3::new(2.6, 0.12, 0.65), hex(0xc4a47e));
    m.cuboid(Vec3::new(0.0, 2.72, 0.41), Vec3::new(1.2, 0.12, 0.01), hex(0xa7865f));
    p.add(&m);
}

/// Ruined sacred wall, 2 m long (along X), ~2.2 m high, ragged top.
fn ruin_wall(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let mut rng = Rng::new(8);
    for i in 0..4 {
        let x = -0.75 + i as f32 * 0.5;
        let h = rng.range(1.5, 2.3);
        m.cuboid(Vec3::new(x, h * 0.5, 0.0), Vec3::new(0.52, h, 0.6), scale_color(hex(SANDSTONE), rng.range(0.9, 1.05)));
        for k in 0..(h / 0.35) as i32 {
            m.cuboid(Vec3::new(x, 0.35 * k as f32 + 0.17, 0.305), Vec3::new(0.5, 0.02, 0.01), hex(0xb08f6a));
        }
    }
    p.add(&m);
}

/// One sacred-way step, 3 m wide (X) × 0.25 m × 0.45 m deep.
fn step_block(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.125, Vec3::new(3.0, 0.25, 0.45), hex(0xcdb08a));
    m.cuboid(Vec3::new(0.0, 0.252, 0.2), Vec3::new(3.0, 0.01, 0.04), hex(0xb7966e));
    p.add(&m);
}

/// Landing before the doorway: 4 × 0.75 × 2.4 m (deep along Z).
fn landing(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.375, Vec3::new(4.0, 0.75, 2.4), hex(0xc8aa82));
    for i in 0..4 {
        m.cuboid(Vec3::new(-1.5 + i as f32, 0.752, 0.0), Vec3::new(0.02, 0.01, 2.3), hex(0xae9068));
    }
    p.add(&m);
}

fn stardust(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.sphere(Vec3::ZERO, 0.05, 2, 6, hex(0xfff2c0));
    p.parts.push(Part { mesh: m.build(), transform: Transform::IDENTITY, mat: PartMat::Emissive(hex(0xffd870), 6.0), name: None, cast_shadows: false });
}

/// A golden four-pointed star with a soft halo (unlit, unaffected by fog), ~1 m.
fn star(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let gold = hex(0xffe08a);
    let pts = 8;
    for face in [1.0f32, -1.0] {
        for i in 0..pts {
            let a0 = i as f32 / pts as f32 * TAU;
            let a1 = (i + 1) as f32 / pts as f32 * TAU;
            let r0 = if i % 2 == 0 { 0.5 } else { 0.14 };
            let r1 = if (i + 1) % 2 == 0 { 0.5 } else { 0.14 };
            let v0 = Vec3::new(a0.sin() * r0, a0.cos() * r0, 0.0);
            let v1 = Vec3::new(a1.sin() * r1, a1.cos() * r1, 0.0);
            if face > 0.0 {
                m.tri(Vec3::Z * 0.02, v1, v0, gold);
            } else {
                m.tri(-Vec3::Z * 0.02, v0, v1, gold);
            }
        }
    }
    let mat = StandardMaterial { base_color: Color::WHITE, emissive: LinearRgba::rgb(4.0, 3.2, 1.4), unlit: true, fog_enabled: false, ..default() };
    p.parts.push(Part { mesh: m.build(), transform: Transform::IDENTITY, mat: PartMat::Custom(mat), name: Some("core".into()), cast_shadows: false });
    let mut h = MeshBuilder::new();
    h.sphere(Vec3::ZERO, 0.42, 4, 10, hex(0xfff0c8));
    let halo = StandardMaterial { base_color: Color::srgba(1.0, 0.92, 0.6, 0.22), unlit: true, fog_enabled: false, alpha_mode: AlphaMode::Add, ..default() };
    p.parts.push(Part { mesh: h.build(), transform: Transform::IDENTITY, mat: PartMat::Custom(halo), name: Some("halo".into()), cast_shadows: false });
}

/// A footprint in the sand: 0.10 × 0.20 m (the original's size).
fn footprint(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let c = hex(0xb8935f);
    let n = 10;
    for i in 0..n {
        let a0 = i as f32 / n as f32 * TAU;
        let a1 = (i + 1) as f32 / n as f32 * TAU;
        m.tri(Vec3::ZERO, Vec3::new(a1.cos() * 0.05, 0.0, a1.sin() * 0.1), Vec3::new(a0.cos() * 0.05, 0.0, a0.sin() * 0.1), c);
    }
    p.parts.push(Part { mesh: m.build(), transform: Transform::IDENTITY, mat: PartMat::Vertex, name: None, cast_shadows: false });
}

// ---------------------------------------------------- vegetation / rock ----

fn cactus(p: &mut ModelParts) {
    let g = hex(0x5f8f4a);
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.18, 0.0), (0.2, 0.2), (0.18, 1.6), (0.1, 1.78), (0.0, 1.82)], 8, g);
    m.tube(Vec3::new(0.15, 0.7, 0.0), Vec3::new(0.45, 0.75, 0.0), 0.1, 0.1, 6, g);
    m.tube(Vec3::new(0.45, 0.72, 0.0), Vec3::new(0.48, 1.25, 0.0), 0.1, 0.07, 6, g);
    m.tube(Vec3::new(-0.15, 0.95, 0.0), Vec3::new(-0.4, 1.0, 0.05), 0.09, 0.09, 6, g);
    m.tube(Vec3::new(-0.4, 0.98, 0.05), Vec3::new(-0.42, 1.35, 0.05), 0.09, 0.06, 6, g);
    p.add(&m);
}

fn cactus_small(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.2, 0.0), (0.24, 0.3), (0.18, 0.95), (0.0, 1.05)], 8, hex(0x6a9a52));
    p.add(&m);
}

/// A tall rock spire, ~10 m.
fn peak(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::Y * 3.0, Vec3::new(4.6, 3.4, 3.4), 21, hex(0xb07a52));
    m.rock(Vec3::new(0.4, 7.0, 0.2), Vec3::new(3.0, 3.2, 2.4), 22, hex(0xc08a5e));
    m.rock(Vec3::new(-0.3, 9.2, 0.0), Vec3::new(1.6, 1.4, 1.4), 23, hex(0xcf9c6c));
    p.add(&m);
}

fn boulder(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::Y * 0.35, Vec3::new(1.2, 0.6, 1.4), 31, hex(0xa98a6a));
    p.add(&m);
}

fn wheat(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let mut rng = Rng::new(61);
    for _ in 0..9 {
        let b = Vec3::new(rng.range(-0.25, 0.25), 0.0, rng.range(-0.22, 0.22));
        let t = b + Vec3::new(rng.range(-0.06, 0.06), rng.range(0.72, 0.9), rng.range(-0.06, 0.06));
        m.tube(b, t, 0.012, 0.008, 3, hex(0xc9a64a));
        m.ellipsoid(t + Vec3::Y * 0.06, Vec3::new(0.025, 0.08, 0.025), 2, 4, hex(0xe7c25f));
    }
    p.add(&m);
}

fn desert_bush(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.sphere(Vec3::Y * 0.3, 0.45, 3, 7, hex(0x8a9a5a));
    m.sphere(Vec3::new(0.35, 0.22, 0.1), 0.3, 3, 6, hex(0x7a8a4e));
    p.add(&m);
}

fn crop(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.05, Vec3::new(0.5, 0.1, 1.8), hex(0x6a4e36));
    for k in 0..5 {
        m.sphere(Vec3::new(0.0, 0.2, -0.7 + k as f32 * 0.35), 0.13, 2, 6, hex(0x6fae4e));
    }
    p.add(&m);
}

fn waterfall(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::new(-1.2, 1.5, 0.0), Vec3::new(1.2, 1.8, 1.4), 41, hex(0x9a8670));
    m.rock(Vec3::new(1.2, 1.5, 0.0), Vec3::new(1.2, 1.8, 1.4), 42, hex(0x9a8670));
    m.rock(Vec3::new(0.0, 2.6, -0.6), Vec3::new(1.4, 1.0, 1.0), 43, hex(0xa8947c));
    p.add(&m);
    let mut w = MeshBuilder::new();
    w.cuboid(Vec3::new(0.0, 1.4, 0.15), Vec3::new(0.8, 2.6, 0.12), hex(0xbfe2f2));
    w.frustum(Vec3::new(0.0, 0.02, 0.6), 0.9, 0.9, 0.04, 10, hex(0x8ec4dc));
    p.add_part(Some("water"), &w, Transform::IDENTITY, PartMat::Glass(Color::srgba(0.6, 0.82, 0.95, 0.7)));
}

fn dune_grass(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for i in 0..7 {
        let a = i as f32 * 0.9;
        let base = Vec3::new(a.cos() * 0.05, 0.0, a.sin() * 0.05);
        let tip = base + Vec3::new(a.cos() * 0.18, 0.35 + (i % 3) as f32 * 0.08, a.sin() * 0.18);
        m.tube(base, tip, 0.018, 0.0, 3, hex(0xb7a76a));
    }
    p.parts.push(Part { mesh: m.build(), transform: Transform::IDENTITY, mat: PartMat::Vertex, name: None, cast_shadows: false });
}

#[allow(dead_code)]
pub fn sand_color() -> Color {
    hex(SAND)
}
