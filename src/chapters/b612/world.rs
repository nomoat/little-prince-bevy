//! B-612 world: the domed planet and its sites (B612LayoutData in the
//! original): sleeping nook, tool shed, the rose on her knoll with the wind
//! screen and the cloche boulder, the spring with its fetch slabs, brook and
//! waterfall, three volcanoes, baobab sprouts, the path to the sunset chair.

use super::systems::*;
use crate::core::prelude::*;
use crate::core::sky::Sky;

// Site positions (x, z). Game compass: +Z = east (sunrise), -Z = west (sunset),
// -X = north (Sky::north_azimuth = 270).
pub const NOOK: Vec2 = Vec2::new(-2.0, -5.0);
pub const ROSE: Vec2 = Vec2::new(-1.0, 4.5);
pub const BOULDER: Vec2 = Vec2::new(0.05, 4.75);
pub const SHED: Vec2 = Vec2::new(-8.5, 3.0);
pub const POND: Vec2 = Vec2::new(9.0, 1.5);
pub const CHAIR: Vec2 = Vec2::new(1.0, -18.0);
pub const VOLC_A: Vec2 = Vec2::new(-12.0, -6.5);
pub const VOLC_B: Vec2 = Vec2::new(-14.5, -1.5);
pub const CRATER_C: Vec2 = Vec2::new(-9.5, -11.5);
pub const SPROUTS: [Vec2; 4] = [Vec2::new(7.0, -9.0), Vec2::new(9.0, -7.0), Vec2::new(6.0, -11.5), Vec2::new(8.6, -10.4)];
/// Fetch slabs around the spring: main slab + three more (game azimuths).
pub const SLAB_AZ: [f32; 4] = [292.0, 240.0, 12.0, 138.0];
/// Brook outflow azimuth from the spring toward the rim.
pub const BROOK_AZ: f32 = 80.0;
/// Rose faces the way the prince arrives from his nook.
pub const ROSE_YAW: f32 = 186.0;
pub const SHED_YAW: f32 = 109.0;

pub fn dir_az(az_deg: f32) -> Vec2 {
    let a = az_deg.to_radians();
    Vec2::new(a.sin(), a.cos())
}

/// Plain local → world for props (x = local +X).
pub fn xf(origin: Vec3, yaw_deg: f32, local: Vec3) -> Vec3 {
    origin + Quat::from_rotation_y(yaw_deg.to_radians()) * local
}

fn brook_points() -> Vec<Vec2> {
    let d = dir_az(BROOK_AZ);
    let start = POND + d * 1.55;
    vec![
        start,
        Vec2::new(12.6, 2.5),
        Vec2::new(14.6, 2.2),
        Vec2::new(16.8, 2.7),
        Vec2::new(19.0, 3.4),
        Vec2::new(21.2, 3.3),
        Vec2::new(23.0, 3.8),
        Vec2::new(24.4, 4.2),
    ]
}

pub fn build(cx: &mut ChapterCx) {
    super::director::reset(cx.world);
    // ---------------------------------------------------------- terrain
    let mut p = PlanetParams::default();
    p.walk_radius = 22.0;
    p.soft_radius = 21.9;
    p.hard_radius = 23.2;
    p.failsafe_radius = 24.0;
    p.lip_start = 23.0;
    p.hole_radius = 26.5;
    p.min_push = 0.25;
    p.max_push = 5.0;
    p.cap_radius = 55.0;
    p.noise_amp = 0.16;
    p.ground = hex(0x9db86a);
    p.ground2 = hex(0x86a557);
    let mut mods = vec![
        // nook: a dug pit, ramp out toward the east (smooth bowl)
        TerrainMod::Mound {
            center: NOOK,
            radius: 2.1,
            height: -0.42,
        },
        TerrainMod::Paint {
            center: NOOK,
            radius: 1.3,
            blend: 0.6,
            color: hex(0x9c8a66),
        },
        // the rose's knoll, visible over the crown from the nook
        TerrainMod::Mound {
            center: ROSE + Vec2::new(0.3, 0.2),
            radius: 3.6,
            height: 0.6,
        },
        TerrainMod::Paint {
            center: ROSE,
            radius: 0.9,
            blend: 0.7,
            color: hex(0x6f9450),
        },
        // shed and chair pads
        TerrainMod::Flatten {
            center: SHED,
            radius: 2.0,
            blend: 1.4,
            height: None,
        },
        TerrainMod::Paint {
            center: SHED,
            radius: 1.6,
            blend: 0.8,
            color: hex(0xb19a72),
        },
        TerrainMod::Flatten {
            center: CHAIR,
            radius: 1.3,
            blend: 1.2,
            height: None,
        },
        // spring: level pad + pool
        TerrainMod::Flatten {
            center: POND,
            radius: 3.0,
            blend: 1.6,
            height: None,
        },
        TerrainMod::Mound {
            center: POND,
            radius: 2.3,
            height: -0.62,
        },
        TerrainMod::Paint {
            center: POND,
            radius: 2.3,
            blend: 0.6,
            color: hex(0xb5a682),
        },
        // volcano feet (bare ground)
        TerrainMod::Paint {
            center: VOLC_A,
            radius: 1.5,
            blend: 1.0,
            color: hex(0x9a8a74),
        },
        TerrainMod::Paint {
            center: VOLC_B,
            radius: 1.5,
            blend: 1.0,
            color: hex(0x9a8a74),
        },
        TerrainMod::Paint {
            center: CRATER_C,
            radius: 1.4,
            blend: 1.0,
            color: hex(0x9a8a74),
        },
    ];
    // path from the nook to the chair (packed earth)
    let n = 12;
    for i in 1..n {
        let t = i as f32 / n as f32;
        let c = NOOK.lerp(CHAIR, t) + Vec2::new((t * 6.0).sin() * 0.8, 0.0);
        mods.push(TerrainMod::Paint {
            center: c,
            radius: 0.42,
            blend: 0.4,
            color: hex(0xb7a47a),
        });
    }
    // brook bed (wet sand)
    let bp = brook_points();
    for w in bp.windows(2) {
        for k in 0..3 {
            let c = w[0].lerp(w[1], k as f32 / 3.0);
            mods.push(TerrainMod::Paint {
                center: c,
                radius: 0.45,
                blend: 0.35,
                color: hex(0x8f8a6a),
            });
        }
    }
    p.mods = mods;
    cx.planet(resolve_flattens(p));
    let cap = Sky::default().hour_for_elevation(22.0, true);
    // Morning: the sun has just risen in the east. The free day creeps toward
    // the afternoon and rests above 22° — it never sets by itself (DayHold).
    cx.sky(8.3, 0.6, Some(cap), SkyPalette::default());
    cx.ambience("amb.wind");

    let g = |cx: &ChapterCx, v: Vec2| cx.ground(v.x, v.y);

    // ---------------------------------------------------------- nook
    let nook = g(cx, NOOK);
    cx.model("model.b612.pad", nook, 0.0, 1.0);
    for i in 0..12 {
        let az = i as f32 * 30.0 + 15.0;
        // leave the ramp toward the east (+Z, azimuth 0) open
        if az < 40.0 || az > 320.0 {
            continue;
        }
        let q = NOOK + dir_az(az) * 1.95;
        let s = 0.8 + (i % 3) as f32 * 0.18;
        let e = cx.model("model.b612.stone", g(cx, q), az * 2.3, s);
        cx.collider(e, Collider::cylinder(0.18 * s, 0.25).no_camera());
    }

    // ---------------------------------------------------------- shed + can
    let shed = g(cx, SHED);
    cx.model("model.b612.shed", shed, SHED_YAW, 1.0);
    for (c, size) in [
        (Vec3::new(0.0, 0.0, -0.88), Vec3::new(2.45, 2.0, 0.14)),
        (Vec3::new(-1.16, 0.0, 0.0), Vec3::new(0.14, 2.0, 1.85)),
        (Vec3::new(1.16, 0.0, 0.0), Vec3::new(0.14, 2.0, 1.85)),
    ] {
        let e = cx.empty(xf(shed, SHED_YAW, c), SHED_YAW);
        cx.collider(e, Collider::cuboid(size));
    }
    let floor = cx.empty(shed, SHED_YAW);
    cx.collider(floor, Collider::cuboid(Vec3::new(2.4, 0.08, 1.8)).no_camera());
    let rake = cx.model("model.b612.rake", xf(shed, SHED_YAW, Vec3::new(1.32, 0.0, 0.55)), SHED_YAW + 90.0, 1.0);
    if let Some(mut t) = cx.world.get_mut::<Transform>(rake) {
        t.rotation *= Quat::from_rotation_z(0.18);
    }
    let can_home = xf(shed, SHED_YAW, Vec3::new(0.45, 0.08, 0.15));
    let can = cx.model("model.b612.can", can_home, SHED_YAW + 200.0, 1.0);
    cx.interactable(
        can,
        Interactable::new("can", "b612.verb.pick_can")
            .radius(1.5)
            .focus(Vec3::Y * 0.15)
            .prompt_height(0.4),
    );
    let home = cx.empty(can_home, SHED_YAW);
    cx.interactable(
        home,
        Interactable::new("can_home", "b612.verb.put_back")
            .radius(1.3)
            .focus(Vec3::Y * 0.15)
            .prompt_height(0.5)
            .disabled(),
    );
    cx.anchor("can_home", can_home, 0.0);

    // ---------------------------------------------------------- the rose
    let rose_p = g(cx, ROSE);
    let rose = cx.model("model.b612.rose", rose_p, ROSE_YAW, 1.2);
    cx.collider(rose, Collider::cylinder(0.16, 0.6).no_camera());
    cx.interactable(
        rose,
        Interactable::new("rose_water", "b612.verb.water")
            .radius(1.6)
            .focus(Vec3::Y * 0.3)
            .prompt_height(0.5)
            .disabled(),
    );
    cx.anchor("rose", rose_p, ROSE_YAW);
    // wind screen north-north-west of her (compass NNW = game azimuth 247.5)
    let scr = ROSE + dir_az(247.5) * 0.72;
    let scr_p = g(cx, scr);
    let screen = cx.model("model.b612.screen", scr_p, 67.5, 0.7);
    cx.collider(screen, Collider::cuboid(Vec3::new(0.6, 0.55, 0.12)).no_camera());
    // soil around her stem (darkens when watered)
    let soil = spawn_soil(cx, rose_p);
    // the big flat-topped boulder with the glass cloche on it
    let boulder_p = g(cx, BOULDER);
    let boulder = cx.model("model.b612.boulder", boulder_p, 30.0, 1.0);
    cx.collider(boulder, Collider::cylinder(0.5, 0.55).no_camera());
    cx.interactable(
        boulder,
        Interactable::new("cloche_place", "b612.verb.put_glass")
            .radius(2.0)
            .hold(1.2)
            .focus(Vec3::Y * 0.6)
            .prompt_height(0.5)
            .any_facing()
            .disabled(),
    );
    let cloche_home = boulder_p + Vec3::Y * 0.555;
    let cloche = cx.model("model.b612.cloche", cloche_home, 0.0, 0.75);
    cx.interactable(
        cloche,
        Interactable::new("cloche", "b612.verb.lift_glass")
            .radius(1.7)
            .focus(Vec3::Y * 0.25)
            .prompt_height(0.4)
            .disabled(),
    );
    cx.anchor("cloche", cloche_home, 0.0);
    // where things happen around her
    let to_nook = dir_az(ROSE_YAW);
    let fw_stand = g(cx, ROSE + to_nook * 0.75);
    let care_stand = g(cx, ROSE + (to_nook + Vec2::new(0.25, 0.0)).normalize() * 0.72);
    let clo_stand = g(cx, ROSE.lerp(BOULDER, 0.5) + Vec2::new(0.05, -0.12));
    let spot_p = g(cx, ROSE + Vec2::new(-0.45, 0.62));
    let spot = cx.empty(spot_p, 0.0);
    cx.interactable(
        spot,
        Interactable::new("can_spot", "b612.verb.put_spot")
            .radius(1.4)
            .focus(Vec3::Y * 0.1)
            .prompt_height(0.5)
            .disabled(),
    );
    cx.anchor("can_spot", spot_p, 0.0);

    // ---------------------------------------------------------- the spring
    let pond_p = g(cx, POND);
    let edge_h = cx.ground(POND.x + 1.55, POND.y).y;
    let water_y = edge_h - 0.03;
    spawn_water(cx, POND, water_y);
    let mut slabs = Vec::new();
    for (i, az) in SLAB_AZ.iter().enumerate() {
        let d = dir_az(*az);
        let sp = g(cx, POND + d * 2.0);
        let e = cx.model("model.b612.slab", sp, *az, if i == 0 { 1.15 } else { 0.95 });
        cx.collider(e, Collider::cylinder(0.4, 0.1).no_camera());
        let id = format!("spring_{i}");
        let mut it = Interactable::new(&id, "b612.verb.fill")
            .radius(1.8)
            .focus(Vec3::Y * 0.1)
            .prompt_height(0.5)
            .disabled();
        it.max_facing_angle = 80.0;
        cx.interactable(e, it);
        let stand = g(cx, POND + d * 2.3);
        slabs.push((sp, stand, *az + 180.0));
    }
    // stone ring with an overflow notch toward the brook
    for i in 0..18 {
        let az = i as f32 * 20.0 + 5.0;
        let near_slab = SLAB_AZ.iter().any(|s| ang_diff(az, *s) < 16.0);
        if ang_diff(az, BROOK_AZ) < 14.0 || near_slab {
            continue;
        }
        let q = POND + dir_az(az) * 2.05;
        let s = 0.7 + (i % 4) as f32 * 0.12;
        let e = cx.model("model.b612.stone", g(cx, q), az * 1.7, s);
        cx.collider(e, Collider::cylinder(0.17 * s, 0.25).no_camera());
    }
    spawn_brook(cx, &brook_points());
    cx.anchor("spring", pond_p, 0.0);

    // ---------------------------------------------------------- volcanoes, sprouts
    let mut vents = Vec::new();
    for (c, h) in [(VOLC_A, 1.0f32), (VOLC_B, 0.85)] {
        let vp = g(cx, c);
        let e = cx.model("model.b612.volcano", vp, c.x * 20.0, h);
        cx.collider(e, Collider::cylinder(0.95 * h, 0.85 * h));
        vents.push(vp + Vec3::Y * 0.8 * h);
    }
    let cp = g(cx, CRATER_C);
    let e = cx.model("model.b612.crater", cp, 0.0, 1.0);
    cx.collider(e, Collider::cylinder(1.0, 0.34).no_camera());
    for (i, s) in SPROUTS.iter().enumerate() {
        cx.model("model.b612.sprout", g(cx, *s), i as f32 * 77.0, 0.8 + (i % 2) as f32 * 0.3);
    }

    // ---------------------------------------------------------- the chair
    let chair_p = g(cx, CHAIR);
    let chair = cx.model("model.b612.chair", chair_p, 180.0, 1.0);
    cx.collider(chair, Collider::cuboid(Vec3::new(0.5, 0.45, 0.5)).no_camera());
    cx.interactable(
        chair,
        Interactable::new("chair", "b612.verb.sit")
            .radius(1.8)
            .focus(Vec3::Y * 0.4)
            .prompt_height(0.6)
            .disabled(),
    );
    cx.anchor("chair", chair_p, 180.0);

    // ---------------------------------------------------------- dressing
    let avoid = [
        (NOOK, 2.6),
        (ROSE, 2.0),
        (SHED, 2.8),
        (POND, 3.0),
        (CHAIR, 2.0),
        (VOLC_A, 1.8),
        (VOLC_B, 1.8),
        (CRATER_C, 1.8),
        (Vec2::new(16.0, 2.6), 1.6),
        (Vec2::new(19.5, 3.3), 1.4),
    ];
    cx.scatter("model.rock", 9, 7.0, 20.5, 3, (0.45, 1.0), &avoid);
    cx.scatter("model.grass_tuft", 90, 1.5, 21.0, 5, (0.7, 1.3), &avoid);
    cx.scatter("model.flower", 14, 3.0, 19.0, 9, (0.8, 1.2), &avoid);
    cx.scatter("model.flower_yellow", 10, 4.0, 18.0, 13, (0.8, 1.1), &avoid);
    cx.scatter("model.bush", 5, 12.0, 20.0, 21, (0.6, 0.9), &avoid);

    // the cloud sea below the rim
    let mut rng = Rng::new(77);
    for i in 0..40 {
        let a = i as f32 / 40.0 * 360.0 + rng.range(-4.0, 4.0);
        let r = rng.range(42.0, 72.0);
        let d = dir_az(a);
        let y = rng.range(-21.0, -15.0);
        cx.model("model.b612.cloud", Vec3::new(d.x * r, y, d.y * r), a + 90.0, rng.range(1.6, 2.6));
    }

    // ---------------------------------------------------------- the prince, asleep
    let feet = nook + Vec3::new(0.0, 0.0, 0.5);
    let player = cx.player(feet, 0.0);

    let sites = Sites {
        rose: rose_p,
        boulder: boulder_p,
        cloche_home,
        cloche_stand: clo_stand,
        can_home,
        can_spot: spot_p,
        fw_stand,
        care_stand,
        slabs,
        pond: pond_p,
        water_y,
        chair: chair_p,
        nook,
        feet,
        vents,
        brook_end: {
            let b = brook_points();
            let l = b[b.len() - 1];
            cx.ground(l.x, l.y)
        },
    };
    // `?b612_real=1` keeps real durations in test mode (for screenshots)
    let test = cx.test && cx.world.resource::<crate::core::params::LaunchParams>().get("b612_real").is_none();
    cx.world.insert_resource(sites);
    super::systems::init_state(cx.world, test, player, rose, can, cloche, chair, soil);
    super::systems::spawn_life(cx);
}

/// Gives every `Flatten { height: None }` an explicit height (sampled the
/// same way the core does: all mounds, plus the flattens before it), so a
/// planet may use several level pads.
pub fn resolve_flattens(mut p: PlanetParams) -> PlanetParams {
    for i in 0..p.mods.len() {
        if let TerrainMod::Flatten { center, height: None, .. } = p.mods[i].clone() {
            let mut probe = p.clone();
            probe.mods = p
                .mods
                .iter()
                .enumerate()
                .filter(|(j, m)| !matches!(m, TerrainMod::Flatten { .. }) || *j < i)
                .map(|(_, m)| m.clone())
                .collect();
            let h = Terrain::new(probe).height(center.x, center.y);
            if let TerrainMod::Flatten { height, .. } = &mut p.mods[i] {
                *height = Some(h);
            }
        }
    }
    p
}

pub fn ang_diff(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

fn spawn_soil(cx: &mut ChapterCx, at: Vec3) -> Entity {
    let mut m = MeshBuilder::new();
    let terrain = cx.terrain().cloned();
    let segs = 14;
    let r = 0.34;
    for i in 0..segs {
        let a0 = i as f32 / segs as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / segs as f32 * std::f32::consts::TAU;
        let p = |a: f32, rr: f32| {
            let x = at.x + a.cos() * rr;
            let z = at.z + a.sin() * rr;
            let y = terrain.as_ref().map(|t| t.height(x, z)).unwrap_or(at.y) + 0.012;
            Vec3::new(x, y, z) - at
        };
        let c = Vec3::Y * 0.015;
        m.tri(c, p(a1, r), p(a0, r), Color::WHITE);
    }
    let mesh = cx.world.resource_mut::<Assets<Mesh>>().add(m.build());
    let mat = cx.world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: hex(0x8a6f50),
        perceptual_roughness: 0.95,
        ..default()
    });
    cx.spawn((Mesh3d(mesh), MeshMaterial3d(mat), Transform::from_translation(at), bevy::light::NotShadowCaster))
}

fn water_material(cx: &mut ChapterCx, c: Color, unlit: bool) -> Handle<StandardMaterial> {
    cx.world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: c,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.08,
        reflectance: 0.5,
        unlit,
        cull_mode: None,
        ..default()
    })
}

fn spawn_water(cx: &mut ChapterCx, c: Vec2, y: f32) {
    let mut m = MeshBuilder::new();
    let segs = 24;
    for i in 0..segs {
        let a0 = i as f32 / segs as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / segs as f32 * std::f32::consts::TAU;
        m.tri(
            Vec3::ZERO,
            Vec3::new(a1.cos() * 1.8, 0.0, a1.sin() * 1.8),
            Vec3::new(a0.cos() * 1.8, 0.0, a0.sin() * 1.8),
            Color::WHITE,
        );
    }
    let mesh = cx.world.resource_mut::<Assets<Mesh>>().add(m.build());
    let mat = water_material(cx, Color::srgba(0.42, 0.66, 0.78, 0.82), false);
    cx.spawn((
        Name::new("SpringWater"),
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_xyz(c.x, y, c.y),
        bevy::light::NotShadowCaster,
    ));
    // the sand-boil vent at the bottom (lighter disc)
    let mut v = MeshBuilder::new();
    v.ellipsoid(Vec3::ZERO, Vec3::new(0.35, 0.02, 0.35), 2, 10, hex(0xe0d4b0));
    let vm = cx.world.resource_mut::<Assets<Mesh>>().add(v.build());
    let sm = cx.world.resource::<crate::core::models::SharedMaterials>().vertex.clone();
    let bottom = cx.ground(c.x, c.y).y + 0.02;
    cx.spawn((Mesh3d(vm), MeshMaterial3d(sm), Transform::from_xyz(c.x + 0.2, bottom, c.y - 0.1)));
}

/// Step-pool brook to the rim and a waterfall off the edge.
fn spawn_brook(cx: &mut ChapterCx, pts: &[Vec2]) {
    let terrain = cx.terrain().cloned().expect("planet");
    let walk = terrain.params.walk_radius;
    // densify
    let mut line: Vec<Vec2> = Vec::new();
    for w in pts.windows(2) {
        let n = ((w[1] - w[0]).length() / 0.35).ceil().max(1.0) as usize;
        for k in 0..n {
            line.push(w[0].lerp(w[1], k as f32 / n as f32));
        }
    }
    line.push(*pts.last().unwrap());
    let mut m = MeshBuilder::new();
    let mut last_y = f32::MAX;
    let mut rib: Vec<(Vec3, Vec3)> = Vec::new();
    for (i, p) in line.iter().enumerate() {
        let d = if i + 1 < line.len() { line[i + 1] - *p } else { *p - line[i - 1] }.normalize_or_zero();
        let side = Vec2::new(-d.y, d.x);
        let r = p.length();
        let wdt = if r > walk + 0.5 { 0.3 } else { 0.24 + ((i as f32) * 0.9).sin().abs() * 0.08 };
        let y = (terrain.height(p.x, p.y) + 0.045).min(last_y);
        last_y = y;
        let a = Vec3::new(
            p.x + side.x * wdt,
            terrain.height(p.x + side.x * wdt, p.y + side.y * wdt).max(y - 0.05) + 0.0,
            p.y + side.y * wdt,
        );
        let b = Vec3::new(
            p.x - side.x * wdt,
            terrain.height(p.x - side.x * wdt, p.y - side.y * wdt).max(y - 0.05) + 0.0,
            p.y - side.y * wdt,
        );
        rib.push((Vec3::new(a.x, y.max(a.y - 0.02), a.z), Vec3::new(b.x, y.max(b.y - 0.02), b.z)));
    }
    for w in rib.windows(2) {
        m.quad(w[0].1, w[0].0, w[1].0, w[1].1, Color::WHITE);
    }
    // waterfall: from the last point straight down past the underside
    let (la, lb) = *rib.last().unwrap();
    let out = Vec3::new(pts.last().unwrap().x, 0.0, pts.last().unwrap().y).normalize_or_zero();
    let mut prev = (la, lb);
    for k in 1..=10 {
        let t = k as f32 / 10.0;
        let drop = t * t * 11.0 + t * 1.0;
        let push = out * (0.35 * t + 0.15);
        let spread = 1.0 + t * 0.6;
        let mid = (la + lb) * 0.5;
        let half = (la - lb) * 0.5 * spread;
        let a = mid + half + push - Vec3::Y * drop;
        let b = mid - half + push - Vec3::Y * drop;
        m.quad(prev.1, prev.0, a, b, Color::WHITE);
        prev = (a, b);
    }
    let mesh = cx.world.resource_mut::<Assets<Mesh>>().add(m.build());
    let mat = water_material(cx, Color::srgba(0.62, 0.82, 0.92, 0.78), false);
    cx.spawn((
        Name::new("Brook"),
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::IDENTITY,
        bevy::light::NotShadowCaster,
    ));
}
