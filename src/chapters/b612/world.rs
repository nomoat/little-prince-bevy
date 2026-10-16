//! B-612 uses the original baked terrain, scene graph and interaction anchors.
use super::original::{self, SceneData};
use super::systems::*;
use crate::core::camera::CameraTuning;
use crate::core::prelude::*;
use crate::core::sky::Sky;

// Unity X is reflected consistently for the Bevy right-handed world.
pub const NOOK: Vec2 = Vec2::new(1.3, 17.6);
pub const ROSE: Vec2 = Vec2::new(0.0, -5.2);
pub const POND: Vec2 = Vec2::new(-11.2, 0.8);
pub const ROSE_YAW: f32 = -150.0;
pub const CAN_YAW: f32 = 98.0;
pub const SPAWN_YAW: f32 = -25.0;
// The reference hand-over shot looks north from the foot of the bed. Its
// heading is independent of the prince, who faces the sunrise at 25° in Unity.
pub const OPENING_VIEW_YAW: f32 = 0.0;
pub const CHAIR_YAW: f32 = 174.0;

pub fn dir_az(az_deg: f32) -> Vec2 {
    let a = az_deg.to_radians();
    Vec2::new(a.sin(), a.cos())
}

pub fn planet_params() -> PlanetParams {
    PlanetParams {
        top_y: 5.0,
        cap_radius: 60.0,
        cap_flatten: 0.0,
        walk_radius: 22.5,
        soft_radius: 21.9,
        hard_radius: 23.2,
        failsafe_radius: 23.9,
        lip_start: 22.9,
        hole_radius: 24.6,
        lip_depth: 0.9,
        underside_depth: 17.0,
        rim_noise: 1.3,
        rim_segments: 320,
        ..default()
    }
}

pub fn build(cx: &mut ChapterCx) {
    super::director::reset(cx.world);
    cx.world.insert_resource(original::terrain());
    cx.model("model.b612.terrain", Vec3::ZERO, 0.0, 1.0);
    cx.model("model.b612.world", Vec3::ZERO, 0.0, 1.0);
    let mut data = SceneData::load();
    let cap = Sky::default().hour_for_elevation(22.0, true);
    cx.sky(
        8.3,
        0.6,
        Some(cap),
        SkyPalette {
            day_zenith: hex(0x2684be),
            day_horizon: hex(0x62a8d2),
            sun_day: hex(0xffe7c4),
            ..default()
        },
    );
    cx.ambience("amb.wind");
    for c in &data.colliders {
        let e = cx.empty(c.position.into(), c.yaw);
        cx.collider(e, Collider::cuboid(c.size.into()));
    }
    let rose_p = Vec3::from(data.models["rose"].position);
    let rose = data.model(cx, "rose");
    cx.interactable(
        rose,
        Interactable::new("rose_water", "b612.verb.water")
            .radius(1.6)
            .focus(Vec3::Y * 0.3)
            .prompt_height(0.5)
            .disabled(),
    );
    cx.anchor("rose", rose_p, ROSE_YAW);
    let soil = spawn_soil(cx, rose_p);
    let can_home = Vec3::from(data.models["can"].position);
    let can = data.model(cx, "can");
    cx.interactable(
        can,
        Interactable::new("can", "b612.verb.pick_can")
            .radius(1.5)
            .focus(Vec3::Y * 0.15)
            .prompt_height(0.4),
    );
    let home = cx.empty(can_home, CAN_YAW);
    cx.interactable(
        home,
        Interactable::new("can_home", "b612.verb.put_back")
            .radius(1.3)
            .focus(Vec3::Y * 0.15)
            .prompt_height(0.5)
            .disabled(),
    );
    cx.anchor("can_home", can_home, CAN_YAW);
    let cloche_home = Vec3::from(data.models["cloche"].position);
    let cloche = data.model(cx, "cloche");
    cx.interactable(
        cloche,
        Interactable::new("cloche", "b612.verb.lift_glass")
            .radius(1.7)
            .focus(Vec3::Y * 0.25)
            .prompt_height(0.4)
            .disabled(),
    );
    cx.anchor("cloche", cloche_home, data.models["cloche"].yaw);
    let boulder_p = Vec3::new(-0.253, 5.403, -5.898);
    let boulder = cx.empty(boulder_p, 0.0);
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
    let (care_stand, _) = data.anchor(cx, "Rose_CareStand");
    let fw_stand = care_stand;
    let (clo_stand, _) = data.anchor(cx, "Cloche_Stand");
    let spot_q = ROSE + dir_az(-35.0);
    let spot_p = cx.ground(spot_q.x, spot_q.y);
    let spot = cx.empty(spot_p, -125.0);
    cx.interactable(
        spot,
        Interactable::new("can_spot", "b612.verb.put_spot")
            .radius(1.4)
            .focus(Vec3::Y * 0.1)
            .prompt_height(0.5)
            .disabled(),
    );
    cx.anchor("can_spot", spot_p, -125.0);
    let pond_p = cx.ground(POND.x, POND.y);
    let mut slabs = Vec::new();
    for (i, s) in data.slabs.iter().enumerate() {
        let stand = Vec3::from(s.position);
        let e = cx.empty(stand, s.yaw);
        let mut it = Interactable::new(&format!("spring_{i}"), "b612.verb.fill")
            .radius(1.8)
            .focus(Vec3::Y * 0.1)
            .prompt_height(0.5)
            .disabled();
        it.max_facing_angle = 80.0;
        cx.interactable(e, it);
        slabs.push((stand, stand, s.yaw));
    }
    cx.anchor("spring", pond_p, 0.0);
    let chair_p = Vec3::from(data.models["chair"].position);
    let chair = data.model(cx, "chair");
    cx.interactable(
        chair,
        Interactable::new("chair", "b612.verb.sit")
            .radius(1.8)
            .focus(Vec3::Y * 0.4)
            .prompt_height(0.6)
            .disabled(),
    );
    cx.anchor("chair", chair_p, CHAIR_YAW);
    let (feet, _) = data.anchor(cx, "PlayerSpawn");
    let nook = cx.ground(NOOK.x, NOOK.y);
    let player = cx.player(feet, SPAWN_YAW);
    // Original follow camera values, rather than the generic 4.6 m orbit.
    {
        let mut t = cx.world.resource_mut::<CameraTuning>();
        t.pivot_height = 1.0;
        t.default_distance = 3.4;
        t.min_distance = 1.8;
        t.max_distance = 5.0;
        t.default_pitch = 4.0;
        t.fov = 50.0;
        t.ground_clearance = 0.3;
    }
    let tuning = cx.world.resource::<CameraTuning>().clone();
    cx.world.resource_mut::<CameraRig>().reset(&tuning);
    cx.world
        .resource_mut::<CameraRig>()
        .snap_behind(SPAWN_YAW.to_radians());
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
        water_y: data.water_y,
        chair: chair_p,
        nook,
        feet,
        vents: vec![Vec3::new(12.2, 6.634, -6.4), Vec3::new(13.6, 6.004, 1.6)],
        brook_end: cx.ground(-19.5, -16.5),
    };
    let test = cx.test
        && cx
            .world
            .resource::<crate::core::params::LaunchParams>()
            .get("b612_real")
            .is_none();
    cx.world.insert_resource(sites);
    super::systems::init_state(cx.world, test, player, rose, can, cloche, chair, soil);
    super::systems::spawn_life(cx);
    original::vegetation(cx, &mut data);
    cx.world.insert_resource(data);
}

/// Gives every `Flatten { height: None }` an explicit height (sampled the
/// same way the core does: all mounds, plus the flattens before it), so a
/// planet may use several level pads.
pub fn resolve_flattens(mut p: PlanetParams) -> PlanetParams {
    for i in 0..p.mods.len() {
        if let TerrainMod::Flatten {
            center,
            height: None,
            ..
        } = p.mods[i].clone()
        {
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
    let mat = cx
        .world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: hex(0x8a6f50),
            perceptual_roughness: 0.95,
            ..default()
        });
    cx.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_translation(at),
        bevy::light::NotShadowCaster,
    ))
}
