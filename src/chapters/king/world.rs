//! The King's planet (RoyalCourt): a small island planet with a palace whose
//! one-leaf door the prince pushes open, a long throne hall along +Z with a
//! red carpet, polished floor, columns, high windows, candelabra and a dais
//! with steps. The King sits on the throne facing the door (-Z).

use super::systems::{self, KingSites};
use crate::chapters::b612::director::{self, local_to_world};
use crate::chapters::b612::world::resolve_flattens;
use crate::core::prelude::*;

/// Hall interior half width, front/back wall z, wall height.
pub const HX: f32 = 3.4;
pub const Z0: f32 = -3.5;
pub const Z1: f32 = 10.0;
pub const WH: f32 = 5.2;
pub const KING_Z: f32 = 7.3;
pub const KING_YAW: f32 = 180.0;
pub const DAIS_H: f32 = 0.42;

fn stone() -> Color {
    hex(0xece5da)
}

/// Box helper: visual + collider, centred at `c` (world), size `s`.
fn block(cx: &mut ChapterCx, mb: &mut MeshBuilder, c: Vec3, s: Vec3, col: Color, collide: bool) {
    mb.cuboid(c, s, col);
    if collide {
        let e = cx.empty(c - Vec3::Y * s.y * 0.5, 0.0);
        cx.collider(e, Collider::cuboid(s));
    }
}

pub fn build(cx: &mut ChapterCx) {
    director::reset(cx.world);
    let mut p = PlanetParams::sized(18.0);
    p.ground = hex(0xb7c08f);
    p.ground2 = hex(0xa4b07c);
    p.dry = hex(0xc9b98a);
    p.seed = 21;
    p.mods = vec![
        TerrainMod::Flatten {
            center: Vec2::new(0.0, 3.2),
            radius: 7.0,
            blend: 3.0,
            height: None,
        },
        TerrainMod::Paint {
            center: Vec2::new(0.0, -5.5),
            radius: 1.6,
            blend: 1.0,
            color: hex(0xcfc3a3),
        },
        TerrainMod::Paint {
            center: Vec2::new(0.0, -8.0),
            radius: 1.2,
            blend: 1.0,
            color: hex(0xcfc3a3),
        },
        TerrainMod::Paint {
            center: Vec2::new(0.0, -10.5),
            radius: 1.0,
            blend: 1.0,
            color: hex(0xcfc3a3),
        },
    ];
    cx.planet(resolve_flattens(p));
    // footsteps: carpet down the centre of the hall, stone elsewhere indoors
    cx.step_zone(Vec3::new(0.0, 2.0, (Z0 + Z1) * 0.5), Vec3::new(0.75, 3.0, (Z1 - Z0) * 0.5), "sfx.step_carpet");
    cx.step_zone(Vec3::new(0.0, 2.0, (Z0 + Z1) * 0.5), Vec3::new(HX, 3.0, (Z1 - Z0) * 0.5), "sfx.step_stone");
    // Late afternoon; the clock does not run here (no sunset mechanic on this planet).
    cx.sky(16.2, 0.0, None, SkyPalette::default());
    let fy = cx.ground(0.0, 3.2).y + 0.14; // floor top

    let shared_vertex = cx.world.resource::<crate::core::models::SharedMaterials>().vertex.clone();
    // ---------------------------------------------------------- palace shell
    let mut walls = MeshBuilder::new();
    let t = 0.3;
    // plinth/floor (walkable top at fy) + entrance step
    let base_h = 1.6;
    block(
        cx,
        &mut walls,
        Vec3::new(0.0, fy - base_h * 0.5, (Z0 + Z1) * 0.5),
        Vec3::new(HX * 2.0 + t * 2.0, base_h, Z1 - Z0 + t * 2.0),
        hex(0xd9cfc2),
        true,
    );
    block(
        cx,
        &mut walls,
        Vec3::new(0.0, fy - 0.08 - 0.5, Z0 - 0.75),
        Vec3::new(2.6, 1.0, 1.0),
        hex(0xd9cfc2),
        true,
    );
    // front wall with the door opening (x in [-0.7, 0.7], height 2.5) and a lintel
    let dz = Z0 - t * 0.5;
    let side_w = HX + t - 0.7;
    block(
        cx,
        &mut walls,
        Vec3::new(-(0.7 + side_w * 0.5), fy + WH * 0.5, dz),
        Vec3::new(side_w, WH, t),
        stone(),
        true,
    );
    block(
        cx,
        &mut walls,
        Vec3::new(0.7 + side_w * 0.5, fy + WH * 0.5, dz),
        Vec3::new(side_w, WH, t),
        stone(),
        true,
    );
    block(
        cx,
        &mut walls,
        Vec3::new(0.0, fy + 2.5 + (WH - 2.5) * 0.5, dz),
        Vec3::new(1.4, WH - 2.5, t),
        stone(),
        true,
    );
    // portal frame
    walls.cuboid(Vec3::new(-0.82, fy + 1.3, dz - 0.12), Vec3::new(0.22, 2.6, 0.12), hex(0xd8c070));
    walls.cuboid(Vec3::new(0.82, fy + 1.3, dz - 0.12), Vec3::new(0.22, 2.6, 0.12), hex(0xd8c070));
    walls.cuboid(Vec3::new(0.0, fy + 2.65, dz - 0.12), Vec3::new(1.86, 0.3, 0.12), hex(0xd8c070));
    walls.lathe(
        Vec3::new(0.0, fy + 2.8, dz - 0.1),
        &[(0.0, 0.0), (0.5, 0.0), (0.35, 0.35), (0.0, 0.45)],
        8,
        hex(0xd8c070),
    );
    // back wall
    block(
        cx,
        &mut walls,
        Vec3::new(0.0, fy + WH * 0.5, Z1 + t * 0.5),
        Vec3::new(HX * 2.0 + t * 2.0, WH, t),
        stone(),
        true,
    );
    // side walls: solid lower band, high windows between piers, upper band
    let len = Z1 - Z0;
    for s in [-1.0f32, 1.0] {
        let x = s * (HX + t * 0.5);
        let e = cx.empty(Vec3::new(x, fy, (Z0 + Z1) * 0.5), 0.0);
        cx.collider(e, Collider::cuboid(Vec3::new(t, WH, len)));
        walls.cuboid(Vec3::new(x, fy + 1.3, (Z0 + Z1) * 0.5), Vec3::new(t, 2.6, len), stone());
        walls.cuboid(Vec3::new(x, fy + 4.65, (Z0 + Z1) * 0.5), Vec3::new(t, 1.1, len), stone());
        let n = 6;
        for i in 0..=n {
            let z = Z0 + i as f32 * len / n as f32;
            walls.cuboid(Vec3::new(x, fy + 3.35, z), Vec3::new(t, 1.5, 0.9), stone());
        }
        // window sills (gold trim) inside
        walls.cuboid(Vec3::new(x - s * 0.17, fy + 2.62, (Z0 + Z1) * 0.5), Vec3::new(0.06, 0.06, len), hex(0xd8c070));
    }
    // ceiling + outside roof
    block(
        cx,
        &mut walls,
        Vec3::new(0.0, fy + WH + 0.15, (Z0 + Z1) * 0.5),
        Vec3::new(HX * 2.0 + t * 2.0, 0.3, len + t * 2.0),
        hex(0xd6cfc4),
        true,
    );
    let mut roof = MeshBuilder::new();
    for s in [-1.0f32, 1.0] {
        roof.with(
            Transform::from_xyz(s * 1.95, fy + WH + 1.05, (Z0 + Z1) * 0.5).with_rotation(Quat::from_rotation_z(-s * 0.5)),
            |m| {
                m.cuboid(Vec3::ZERO, Vec3::new(4.5, 0.2, len + 1.0), hex(0x4a5a9a));
            },
        );
    }
    roof.cuboid(Vec3::new(0.0, fy + WH + 2.05, (Z0 + Z1) * 0.5), Vec3::new(0.3, 0.3, len + 1.0), hex(0xd8c070));
    // gable fronts
    for z in [Z0 - t, Z1 + t] {
        roof.tri(
            Vec3::new(-HX - t, fy + WH + 0.3, z),
            Vec3::new(HX + t, fy + WH + 0.3, z),
            Vec3::new(0.0, fy + WH + 2.1, z),
            stone(),
        );
        roof.tri(
            Vec3::new(HX + t, fy + WH + 0.3, z),
            Vec3::new(-HX - t, fy + WH + 0.3, z),
            Vec3::new(0.0, fy + WH + 2.1, z),
            stone(),
        );
    }
    let wm = cx.world.resource_mut::<Assets<Mesh>>().add(walls.build());
    cx.spawn((Name::new("PalaceShell"), Mesh3d(wm), MeshMaterial3d(shared_vertex.clone()), Transform::IDENTITY));
    let rm = cx.world.resource_mut::<Assets<Mesh>>().add(roof.build());
    cx.spawn((Name::new("PalaceRoof"), Mesh3d(rm), MeshMaterial3d(shared_vertex.clone()), Transform::IDENTITY));
    // towers at the front corners
    for s in [-1.0f32, 1.0] {
        let tp = Vec3::new(s * (HX + 0.9), fy - 0.6, Z0 - 0.4);
        let e = cx.model("model.king.tower", tp, 0.0, 1.0);
        cx.collider(e, Collider::cylinder(1.1, 7.0));
    }

    // ---------------------------------------------------------- polished floor, carpet
    let mut fl = MeshBuilder::new();
    fl.cuboid(Vec3::new(0.0, fy + 0.005, (Z0 + Z1) * 0.5), Vec3::new(HX * 2.0, 0.01, len), Color::WHITE);
    let fm = cx.world.resource_mut::<Assets<Mesh>>().add(fl.build());
    let fmat = cx.world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: hex(0x6e4a4a),
        perceptual_roughness: 0.12,
        reflectance: 0.7,
        metallic: 0.0,
        ..default()
    });
    cx.spawn((Name::new("PolishedFloor"), Mesh3d(fm), MeshMaterial3d(fmat), Transform::IDENTITY));
    let mut carpet = MeshBuilder::new();
    let dais_front = KING_Z - 2.6;
    carpet.cuboid(
        Vec3::new(0.0, fy + 0.015, (Z0 + dais_front) * 0.5),
        Vec3::new(1.3, 0.01, dais_front - Z0),
        hex(0xb02a3a),
    );
    carpet.cuboid(
        Vec3::new(0.62, fy + 0.017, (Z0 + dais_front) * 0.5),
        Vec3::new(0.06, 0.01, dais_front - Z0),
        hex(0xd8b04a),
    );
    carpet.cuboid(
        Vec3::new(-0.62, fy + 0.017, (Z0 + dais_front) * 0.5),
        Vec3::new(0.06, 0.01, dais_front - Z0),
        hex(0xd8b04a),
    );
    let cm = cx.world.resource_mut::<Assets<Mesh>>().add(carpet.build());
    cx.spawn((
        Mesh3d(cm),
        MeshMaterial3d(shared_vertex.clone()),
        Transform::IDENTITY,
        bevy::light::NotShadowCaster,
    ));

    // ---------------------------------------------------------- dais with steps (king space)
    let ko = Vec3::new(0.0, fy + DAIS_H, KING_Z); // king-space origin: dais top under his seat
    let kw = |v: Vec3| local_to_world(ko, KING_YAW, v);
    let mut dais = MeshBuilder::new();
    let floor_k = |x: f32, z: f32| {
        let w = kw(Vec3::new(x, 0.0, z));
        Vec3::new(w.x, fy, w.z)
    };
    // platform: local z in [-1.2, 1.9], |x| < 2.3, top at the dais height
    let pc = floor_k(0.0, 0.35) + Vec3::Y * DAIS_H * 0.5;
    block(cx, &mut dais, pc, Vec3::new(4.6, DAIS_H, 3.1), hex(0xd2c6b6), true);
    // steps (|x| < 1.8): local z [1.9, 2.25] 0.28 high, [2.25, 2.6] 0.14 high
    let s1 = floor_k(0.0, 2.075) + Vec3::Y * 0.14;
    block(cx, &mut dais, s1, Vec3::new(3.6, 0.28, 0.35), hex(0xc9bcaa), true);
    let s2 = floor_k(0.0, 2.425) + Vec3::Y * 0.07;
    block(cx, &mut dais, s2, Vec3::new(3.6, 0.14, 0.35), hex(0xc9bcaa), true);
    // carpet up the steps and on the dais
    dais.cuboid(floor_k(0.0, 2.425) + Vec3::Y * 0.145, Vec3::new(1.3, 0.01, 0.35), hex(0xb02a3a));
    dais.cuboid(floor_k(0.0, 2.075) + Vec3::Y * 0.285, Vec3::new(1.3, 0.01, 0.35), hex(0xb02a3a));
    dais.cuboid(floor_k(0.0, 1.0) + Vec3::Y * (DAIS_H + 0.005), Vec3::new(2.6, 0.01, 1.8), hex(0xb02a3a));
    dais.cuboid(floor_k(0.0, 0.35) + Vec3::Y * (DAIS_H - 0.02), Vec3::new(4.66, 0.05, 3.16), hex(0xd8c070));
    let dm = cx.world.resource_mut::<Assets<Mesh>>().add(dais.build());
    cx.spawn((Name::new("Dais"), Mesh3d(dm), MeshMaterial3d(shared_vertex.clone()), Transform::IDENTITY));

    // throne + king
    let throne = cx.model("model.king.throne", kw(Vec3::new(0.0, 0.0, -0.12)), KING_YAW, 1.0);
    cx.collider(throne, Collider::cuboid(Vec3::new(0.9, 2.0, 0.8)).no_camera());
    let king = cx.model("model.king.king", ko, KING_YAW, 1.0);
    cx.collider(king, Collider::cylinder(0.45, 1.4).no_camera());
    let mut talk = Interactable::new("king", "king.verb.talk")
        .radius(3.1)
        .focus(Vec3::Y * 1.0)
        .prompt_height(0.6)
        .priority(1)
        .disabled();
    talk.max_facing_angle = 70.0;
    talk.side_half_angle = Some(70.0);
    cx.interactable(king, talk);
    cx.anchor("king", ko + Vec3::Y * 0.9, KING_YAW);
    cx.point_light(kw(Vec3::new(0.0, 2.6, 1.4)), hex(0xffd9a0), 9000.0, 7.0);

    // candelabra beside the dais (|x| < 3.25, z ≤ 2.05) and along the hall
    let mut lights = Vec::new();
    for (x, z) in [(2.8f32, 1.5f32), (-2.8, 1.5)] {
        let w = floor_k(x, z);
        let e = cx.model("model.king.candelabrum", w, KING_YAW, 1.0);
        cx.collider(e, Collider::cylinder(0.22, 1.6).no_camera());
        lights.push(cx.point_light(w + Vec3::Y * 1.75, hex(0xffb35a), 26000.0, 9.0));
    }
    for z in [-1.0f32, 2.4] {
        for s in [-1.0f32, 1.0] {
            let w = Vec3::new(s * (HX - 0.55), fy, z);
            let e = cx.model("model.king.candelabrum", w, if s > 0.0 { 270.0 } else { 90.0 }, 1.0);
            cx.collider(e, Collider::cylinder(0.22, 1.6).no_camera());
            lights.push(cx.point_light(w + Vec3::Y * 1.75, hex(0xffb35a), 18000.0, 8.0));
        }
    }
    // columns + banners
    for z in [0.7f32, 4.1] {
        for s in [-1.0f32, 1.0] {
            let e = cx.model("model.king.column", Vec3::new(s * (HX - 0.5), fy, z), 0.0, 1.0);
            cx.collider(e, Collider::cylinder(0.3, 4.9));
        }
    }
    for z in [-0.9f32 + 1.2, 2.4 + 1.2, 5.6] {
        for s in [-1.0f32, 1.0] {
            cx.model(
                "model.king.banner",
                Vec3::new(s * (HX - 0.04), fy + 4.3, z),
                if s > 0.0 { 270.0 } else { 90.0 },
                1.0,
            );
        }
    }
    cx.model("model.king.banner", Vec3::new(-1.6, fy + 4.6, Z1 - 0.04), 180.0, 1.2);
    cx.model("model.king.banner", Vec3::new(1.6, fy + 4.6, Z1 - 0.04), 180.0, 1.2);
    // soft fill light in the hall (light from the high windows)
    cx.point_light(Vec3::new(0.0, fy + 4.4, 1.0), hex(0xdfe4ff), 30000.0, 12.0);
    cx.point_light(Vec3::new(0.0, fy + 4.4, 6.5), hex(0xdfe4ff), 22000.0, 10.0);

    // ---------------------------------------------------------- the door (one leaf, hinge at x = -0.7)
    let hinge = cx.empty(Vec3::new(-0.7, fy, Z0 - 0.06), 0.0);
    let leaf = cx.model("model.king.door", Vec3::new(0.7, 0.0, 0.0), 0.0, 1.0);
    cx.world.entity_mut(hinge).add_child(leaf);
    cx.collider(leaf, Collider::cuboid(Vec3::new(1.4, 2.45, 0.1)));
    let mut it = Interactable::new("door", "king.verb.door")
        .radius(1.9)
        .focus(Vec3::Y * 1.1)
        .prompt_height(0.3)
        .disabled();
    it.max_facing_angle = 70.0;
    cx.interactable(leaf, it);
    cx.anchor("door", Vec3::new(0.0, fy + 1.0, Z0), 0.0);

    // ---------------------------------------------------------- outside dressing
    let avoid = [(Vec2::new(0.0, 3.2), 9.5), (Vec2::new(0.0, -8.0), 2.5)];
    cx.scatter("model.tree", 4, 11.0, 16.5, 4, (0.8, 1.1), &avoid);
    cx.scatter("model.bush", 8, 9.0, 17.0, 6, (0.6, 1.0), &avoid);
    cx.scatter("model.rock", 6, 10.0, 17.0, 8, (0.5, 1.0), &avoid);
    cx.scatter("model.grass_tuft", 60, 6.0, 17.5, 9, (0.7, 1.2), &avoid);
    cx.scatter("model.flower_purple", 14, 7.0, 16.0, 12, (0.8, 1.1), &avoid);
    for s in [-1.0f32, 1.0] {
        let p = cx.ground(s * 1.6, -6.5);
        cx.model("model.bush", p, 0.0, 0.7);
    }

    // ---------------------------------------------------------- the prince arrives
    let start = cx.ground(0.0, -11.0);
    let player = cx.player(start, 0.0);

    let sites = KingSites {
        floor_y: fy,
        king_origin: ko,
        door_stance: Vec3::new(0.0, fy, Z0 - 0.75),
        hall_inside_z: Z0 + 0.6,
    };
    cx.world.insert_resource(sites);
    let test = cx.test && cx.world.resource::<crate::core::params::LaunchParams>().get("king_real").is_none();
    systems::init_state(cx.world, test, player, king, hinge, leaf, lights);
}
