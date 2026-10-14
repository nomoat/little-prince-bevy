//! Procedural fallbacks for every B-612 asset key (see assets/chapters/b612.ron).
//! Each one can be replaced by a glTF file through the manifest.

use crate::core::prelude::*;

pub fn register(app: &mut App) {
    app.register_model("b612.rose", rose)
        .register_model("b612.cloche", cloche)
        .register_model("b612.boulder", boulder)
        .register_model("b612.screen", wind_screen)
        .register_model("b612.can", can)
        .register_model("b612.shed", shed)
        .register_model("b612.volcano", volcano)
        .register_model("b612.crater", crater)
        .register_model("b612.sprout", sprout)
        .register_model("b612.pad", sleeping_pad)
        .register_model("b612.stone", ring_stone)
        .register_model("b612.slab", slab)
        .register_model("b612.butterfly", butterfly)
        .register_model("b612.bird", bird)
        .register_model("b612.rake", rake)
        .register_model("b612.cloud", cloud);
}

/// The rose (~0.55 m): "rose_stem" (stem, leaves, four thorns) and
/// "rose_head" (the bloom, pivot at the top of the stem) for acting.
fn rose(p: &mut ModelParts) {
    let stem_c = hex(0x4f8a45);
    let leaf_c = hex(0x5d9c4e);
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::new(0.0, 0.25, 0.01), 0.016, 0.014, 6, stem_c);
    m.tube(Vec3::new(0.0, 0.25, 0.01), Vec3::new(0.0, 0.46, 0.0), 0.014, 0.012, 6, stem_c);
    for (y, a, s) in [(0.14f32, 0.3f32, 1.0f32), (0.27, 3.4, 0.85)] {
        m.with(
            Transform::from_xyz(0.0, y, 0.0).with_rotation(Quat::from_rotation_y(a) * Quat::from_rotation_z(1.0)),
            |m| {
                m.ellipsoid(Vec3::new(0.0, 0.07 * s, 0.0), Vec3::new(0.035 * s, 0.075 * s, 0.012), 2, 6, leaf_c);
            },
        );
    }
    // her four thorns
    for i in 0..4 {
        let a = i as f32 * 1.7;
        let y = 0.08 + i as f32 * 0.075;
        let base = Vec3::new(a.cos() * 0.012, y, a.sin() * 0.012);
        m.tube(base, base + Vec3::new(a.cos() * 0.035, 0.018, a.sin() * 0.035), 0.007, 0.0, 3, hex(0x3d6a37));
    }
    p.add_named("rose_stem", &m, Transform::IDENTITY);
    let mut h = MeshBuilder::new();
    h.lathe(Vec3::ZERO, &[(0.0, -0.02), (0.035, 0.0), (0.03, 0.03), (0.0, 0.035)], 6, hex(0x4f8a45));
    // sepals
    for i in 0..5 {
        let a = i as f32 / 5.0 * std::f32::consts::TAU;
        h.tube(
            Vec3::new(0.0, 0.01, 0.0),
            Vec3::new(a.cos() * 0.06, -0.015, a.sin() * 0.06),
            0.012,
            0.0,
            3,
            hex(0x5d9c4e),
        );
    }
    // cup of petals + inner bud
    h.lathe(
        Vec3::new(0.0, 0.015, 0.0),
        &[(0.0, 0.0), (0.06, 0.02), (0.095, 0.07), (0.1, 0.11), (0.085, 0.13)],
        9,
        hex(0xc93a46),
    );
    for i in 0..5 {
        let a = i as f32 / 5.0 * std::f32::consts::TAU + 0.3;
        h.with(
            Transform::from_xyz(a.cos() * 0.06, 0.08, a.sin() * 0.06).with_rotation(Quat::from_rotation_y(-a) * Quat::from_rotation_z(-0.5)),
            |m| {
                m.ellipsoid(Vec3::ZERO, Vec3::new(0.02, 0.06, 0.05), 2, 6, hex(0xd8434e));
            },
        );
    }
    h.sphere(Vec3::new(0.0, 0.11, 0.0), 0.06, 4, 8, hex(0xe0525c));
    h.with(Transform::from_xyz(0.0, 0.15, 0.0).with_rotation(Quat::from_rotation_y(0.6)), |m| {
        m.ellipsoid(Vec3::ZERO, Vec3::new(0.045, 0.03, 0.03), 2, 6, hex(0xb83140));
    });
    p.add_named("rose_head", &h, Transform::from_xyz(0.0, 0.46, 0.0));
}

/// The glass cloche: 0.46 m across, 0.65 m tall at scale 1 (as the original).
fn cloche(p: &mut ModelParts) {
    let mut g = MeshBuilder::new();
    let mut prof = Vec::new();
    for i in 0..=8 {
        let t = i as f32 / 8.0;
        let a = t * std::f32::consts::FRAC_PI_2;
        prof.push((0.23 * a.cos(), 0.02 + 0.55 * a.sin()));
    }
    prof.push((0.0, 0.575));
    g.lathe(Vec3::ZERO, &[(0.235, 0.0), (0.235, 0.02)], 16, hex(0xffffff));
    g.lathe(Vec3::ZERO, &prof, 16, hex(0xffffff));
    p.add_part(
        Some("cloche_glass"),
        &g,
        Transform::IDENTITY,
        PartMat::Glass(Color::srgba(0.82, 0.93, 1.0, 0.28)),
    );
    let mut k = MeshBuilder::new();
    k.sphere(Vec3::new(0.0, 0.61, 0.0), 0.04, 3, 8, hex(0xdfeaf2));
    k.frustum(Vec3::new(0.0, 0.565, 0.0), 0.02, 0.016, 0.04, 8, hex(0xdfeaf2));
    p.add_part(None, &k, Transform::IDENTITY, PartMat::Glass(Color::srgba(0.85, 0.95, 1.0, 0.55)));
}

/// Big flat-topped boulder that holds the cloche (top at 0.55 m).
fn boulder(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(
        Vec3::ZERO,
        &[(0.0, -0.1), (0.5, -0.1), (0.56, 0.12), (0.5, 0.42), (0.42, 0.53), (0.3, 0.55), (0.0, 0.555)],
        9,
        hex(0x9a8f86),
    );
    m.rock(Vec3::new(0.35, 0.05, 0.3), Vec3::new(0.22, 0.16, 0.2), 4, hex(0x8a8078));
    m.ellipsoid(Vec3::new(-0.2, 0.47, -0.25), Vec3::new(0.18, 0.06, 0.14), 2, 6, hex(0x7f9a58));
    p.add(&m);
}

/// Small three-panel wind screen (~0.75 m).
fn wind_screen(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let wood = hex(0x9a6b45);
    let cloth = hex(0xe8dcc0);
    for (i, a) in [-0.5f32, 0.0, 0.5].iter().enumerate() {
        let x = (i as f32 - 1.0) * 0.27;
        m.with(Transform::from_xyz(x, 0.0, a.abs() * 0.12).with_rotation(Quat::from_rotation_y(*a)), |m| {
            m.cuboid(Vec3::new(-0.125, 0.38, 0.0), Vec3::new(0.025, 0.76, 0.03), wood);
            m.cuboid(Vec3::new(0.125, 0.38, 0.0), Vec3::new(0.025, 0.76, 0.03), wood);
            m.cuboid(Vec3::new(0.0, 0.74, 0.0), Vec3::new(0.27, 0.03, 0.03), wood);
            m.cuboid(Vec3::new(0.0, 0.1, 0.0), Vec3::new(0.27, 0.03, 0.03), wood);
            m.cuboid(Vec3::new(0.0, 0.42, 0.0), Vec3::new(0.23, 0.6, 0.012), cloth);
        });
    }
    p.add(&m);
}

/// Watering can (~0.3 m); spout along +Z. "can_body" can be tilted to pour.
fn can(p: &mut ModelParts) {
    let tin = hex(0x6f9fb0);
    let dark = hex(0x4f7c8c);
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.11, 0.1, 0.22, 12, tin);
    m.frustum(Vec3::new(0.0, 0.22, 0.0), 0.1, 0.06, 0.03, 12, dark);
    m.tube(Vec3::new(0.0, 0.05, 0.09), Vec3::new(0.0, 0.27, 0.3), 0.022, 0.015, 6, tin);
    m.with(Transform::from_xyz(0.0, 0.27, 0.3).with_rotation(Quat::from_rotation_x(0.9)), |m| {
        m.frustum(Vec3::ZERO, 0.02, 0.045, 0.04, 8, dark);
    });
    // handle arch
    let pts: Vec<Vec3> = (0..=6)
        .map(|i| {
            let a = i as f32 / 6.0 * std::f32::consts::PI;
            Vec3::new(0.0, 0.24 + a.sin() * 0.1, -0.08 + (1.0 - a.cos()) * 0.07)
        })
        .collect();
    for w in pts.windows(2) {
        m.tube(w[0], w[1], 0.012, 0.012, 5, dark);
    }
    p.add_named("can_body", &m, Transform::IDENTITY);
}

/// Open-front tool shed (front = +Z): 2.4 m wide, 1.8 m deep, ~2.2 m tall.
fn shed(p: &mut ModelParts) {
    let plank = hex(0xa77b52);
    let plank2 = hex(0x93693f);
    let roof = hex(0x7d4a3a);
    let mut m = MeshBuilder::new();
    // floor boards
    m.cuboid(Vec3::new(0.0, 0.04, 0.0), Vec3::new(2.4, 0.08, 1.8), plank2);
    // back wall (planks)
    for i in 0..8 {
        let x = -1.05 + i as f32 * 0.3;
        m.cuboid(
            Vec3::new(x, 1.0, -0.86),
            Vec3::new(0.28, 2.0 + (i % 2) as f32 * 0.04, 0.07),
            if i % 2 == 0 { plank } else { plank2 },
        );
    }
    // side walls
    for s in [-1.0f32, 1.0] {
        for i in 0..6 {
            let z = -0.75 + i as f32 * 0.3;
            let h = 2.0 - (z + 0.9) * 0.18;
            m.cuboid(
                Vec3::new(s * 1.16, h * 0.5, z),
                Vec3::new(0.07, h, 0.28),
                if i % 2 == 0 { plank2 } else { plank },
            );
        }
        m.cuboid(Vec3::new(s * 1.16, 0.95, 0.88), Vec3::new(0.12, 1.9, 0.12), plank2);
    }
    // slanted roof
    m.with(Transform::from_xyz(0.0, 2.02, 0.0).with_rotation(Quat::from_rotation_x(0.18)), |m| {
        m.cuboid(Vec3::ZERO, Vec3::new(2.7, 0.08, 2.3), roof);
        for i in 0..6 {
            m.cuboid(Vec3::new(-1.1 + i as f32 * 0.44, 0.06, 0.0), Vec3::new(0.05, 0.04, 2.3), hex(0x6a3e31));
        }
    });
    // shelf with pots, a hook with a coil of rope
    m.cuboid(Vec3::new(0.3, 1.25, -0.7), Vec3::new(1.4, 0.05, 0.3), plank2);
    m.frustum(Vec3::new(0.0, 1.28, -0.7), 0.08, 0.1, 0.14, 8, hex(0xb5643c));
    m.frustum(Vec3::new(0.35, 1.28, -0.7), 0.06, 0.08, 0.1, 8, hex(0xb5643c));
    m.frustum(Vec3::new(0.7, 1.28, -0.7), 0.07, 0.07, 0.18, 8, hex(0xd8c79a));
    m.lathe(
        Vec3::new(-0.75, 1.3, -0.78),
        &[(0.12, -0.03), (0.15, 0.0), (0.12, 0.03), (0.09, 0.0), (0.12, -0.03)],
        10,
        hex(0xcdb98a),
    );
    // crate
    m.cuboid(Vec3::new(-0.7, 0.28, -0.45), Vec3::new(0.5, 0.4, 0.45), hex(0xb08a5a));
    p.add(&m);
}

/// Active volcano: knee-high cone (foot ≈ 2.48× height), glowing vent.
fn volcano(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(
        Vec3::ZERO,
        &[
            (0.0, -0.1),
            (1.12, -0.1),
            (1.0, 0.12),
            (0.72, 0.42),
            (0.42, 0.78),
            (0.3, 0.9),
            (0.2, 0.84),
            (0.12, 0.74),
            (0.0, 0.72),
        ],
        11,
        hex(0x7f6f66),
    );
    m.lathe(Vec3::new(0.0, 0.0, 0.0), &[(0.43, 0.77), (0.33, 0.885), (0.305, 0.9)], 11, hex(0x5f514a));
    for i in 0..5 {
        let a = i as f32 * 1.3;
        m.rock(Vec3::new(a.cos() * 1.0, 0.05, a.sin() * 1.0), Vec3::new(0.16, 0.1, 0.13), i, hex(0x8a7c72));
    }
    p.add(&m);
    let mut g = MeshBuilder::new();
    g.frustum(Vec3::Y * 0.72, 0.17, 0.05, 0.06, 9, hex(0xff8a3d));
    p.add_part(Some("vent_glow"), &g, Transform::IDENTITY, PartMat::Emissive(hex(0xff7a30), 3.0));
}

/// Dormant crater ring.
fn crater(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(
        Vec3::ZERO,
        &[(0.0, 0.08), (0.55, 0.06), (0.62, 0.32), (0.78, 0.38), (1.0, 0.2), (1.15, -0.1)],
        12,
        hex(0x8b7d72),
    );
    m.ellipsoid(Vec3::new(0.0, 0.08, 0.0), Vec3::new(0.5, 0.02, 0.5), 2, 10, hex(0x6b5f58));
    p.add(&m);
}

fn sprout(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::Y * 0.25, 0.03, 0.02, 5, hex(0x6c8f3e));
    for i in 0..3 {
        let a = i as f32 * 2.1;
        m.with(
            Transform::from_xyz(0.0, 0.24, 0.0).with_rotation(Quat::from_rotation_y(a) * Quat::from_rotation_z(0.8)),
            |m| {
                m.ellipsoid(Vec3::new(0.0, 0.07, 0.0), Vec3::new(0.05, 0.08, 0.02), 2, 5, hex(0x7fae4e));
            },
        );
    }
    p.add(&m);
}

/// Sleeping pad + pillow + folded blanket.
fn sleeping_pad(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.ellipsoid(Vec3::new(0.0, 0.05, 0.0), Vec3::new(0.42, 0.06, 0.8), 3, 10, hex(0xcfb98e));
    m.ellipsoid(Vec3::new(0.0, 0.1, -0.62), Vec3::new(0.24, 0.07, 0.13), 3, 8, hex(0xf1ead8));
    m.ellipsoid(Vec3::new(0.05, 0.1, 0.35), Vec3::new(0.36, 0.05, 0.3), 3, 8, hex(0x6f88b8));
    p.add(&m);
}

fn ring_stone(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::new(0.0, 0.08, 0.0), Vec3::new(0.22, 0.16, 0.18), 3, hex(0x9a9188));
    p.add(&m);
}

/// Flat fetch slab at the spring.
fn slab(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(
        Vec3::ZERO,
        &[(0.0, -0.05), (0.42, -0.05), (0.45, 0.04), (0.4, 0.09), (0.0, 0.1)],
        7,
        hex(0xa9a197),
    );
    p.add(&m);
}

/// Butterfly: body + two named wings (flapped by code).
fn butterfly(p: &mut ModelParts) {
    let mut b = MeshBuilder::new();
    b.ellipsoid(Vec3::ZERO, Vec3::new(0.008, 0.008, 0.035), 2, 4, hex(0x3a2f2a));
    p.add(&b);
    for (name, s) in [("bfly_wing_l", 1.0f32), ("bfly_wing_r", -1.0)] {
        let mut w = MeshBuilder::new();
        w.tri(Vec3::ZERO, Vec3::new(s * 0.06, 0.0, 0.035), Vec3::new(s * 0.055, 0.0, -0.03), hex(0xf2c14e));
        w.tri(Vec3::ZERO, Vec3::new(s * 0.055, 0.0, -0.03), Vec3::new(s * 0.06, 0.0, 0.035), hex(0xe9a53a));
        p.add_part(Some(name), &w, Transform::IDENTITY, PartMat::Vertex);
    }
}

/// Migrating bird (goose-like), ~0.6 m wingspan; wings named for flapping.
fn bird(p: &mut ModelParts) {
    let mut b = MeshBuilder::new();
    b.ellipsoid(Vec3::ZERO, Vec3::new(0.07, 0.06, 0.2), 3, 6, hex(0xece6dc));
    b.tube(Vec3::new(0.0, 0.02, 0.16), Vec3::new(0.0, 0.07, 0.3), 0.025, 0.02, 5, hex(0x3d3a38));
    b.sphere(Vec3::new(0.0, 0.08, 0.31), 0.035, 3, 6, hex(0x3d3a38));
    b.tube(Vec3::new(0.0, 0.075, 0.34), Vec3::new(0.0, 0.07, 0.39), 0.012, 0.0, 4, hex(0xe0a040));
    b.tube(Vec3::new(0.0, 0.0, -0.18), Vec3::new(0.0, 0.01, -0.27), 0.04, 0.0, 4, hex(0xb8b0a4));
    p.add(&b);
    for (name, s) in [("bird_wing_l", 1.0f32), ("bird_wing_r", -1.0)] {
        let mut w = MeshBuilder::new();
        let c = hex(0xd9d1c4);
        w.tri(Vec3::new(0.0, 0.0, 0.08), Vec3::new(s * 0.34, 0.0, -0.02), Vec3::new(0.0, 0.0, -0.08), c);
        w.tri(Vec3::new(0.0, 0.0, 0.08), Vec3::new(0.0, 0.0, -0.08), Vec3::new(s * 0.34, 0.0, -0.02), c);
        w.tri(
            Vec3::new(s * 0.2, 0.0, 0.03),
            Vec3::new(s * 0.34, 0.0, -0.02),
            Vec3::new(s * 0.3, 0.0, -0.06),
            hex(0x5a5550),
        );
        w.tri(
            Vec3::new(s * 0.2, 0.0, 0.03),
            Vec3::new(s * 0.3, 0.0, -0.06),
            Vec3::new(s * 0.34, 0.0, -0.02),
            hex(0x5a5550),
        );
        p.add_part(Some(name), &w, Transform::IDENTITY, PartMat::Vertex);
    }
}

/// A rake leaning on the shed (decor).
fn rake(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::new(0.0, 1.3, 0.0), 0.02, 0.02, 5, hex(0x9a6b45));
    m.cuboid(Vec3::new(0.0, 0.02, 0.0), Vec3::new(0.32, 0.04, 0.04), hex(0x6c6c70));
    for i in 0..6 {
        m.cuboid(Vec3::new(-0.14 + i as f32 * 0.056, -0.04, 0.0), Vec3::new(0.012, 0.08, 0.012), hex(0x6c6c70));
    }
    p.add(&m);
}

/// A soft cloud bank (part of the cloud sea below the rim), ~8 m long.
fn cloud(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let c = hex(0xf6f3f0);
    for (x, y, z, r) in [
        (0.0f32, 0.0f32, 0.0f32, 2.2f32),
        (2.2, -0.3, 0.6, 1.7),
        (-2.3, -0.2, -0.4, 1.8),
        (0.8, 0.6, -0.9, 1.5),
        (-0.9, 0.5, 1.0, 1.4),
        (3.8, -0.6, -0.3, 1.2),
        (-3.9, -0.5, 0.5, 1.1),
    ] {
        m.ellipsoid(Vec3::new(x, y, z), Vec3::new(r * 1.2, r * 0.55, r), 4, 9, c);
    }
    // slightly self-lit so the banks stay soft and bright against any sky
    p.add_part(
        None,
        &m,
        Transform::IDENTITY,
        PartMat::Custom(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(0.42, 0.4, 0.4),
            perceptual_roughness: 1.0,
            reflectance: 0.1,
            ..default()
        }),
    );
    if let Some(last) = p.parts.last_mut() {
        last.cast_shadows = false;
    }
}
