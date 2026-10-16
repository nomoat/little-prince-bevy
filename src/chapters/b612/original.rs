//! B-612's authored scene. The import preserves source coordinates and scale.
//! Vegetation streams from the original instance table and reuses glTF meshes.
use crate::core::environment::{EnvironmentExtension, EnvironmentMaterial};
use crate::core::prelude::*;
use bevy::gltf::{GltfAssetLabel, GltfMaterial, GltfMesh};
use bevy::light::NotShadowCaster;
use bevy::pbr::ExtendedMaterial;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Resource)]
struct Preload(Vec<Handle<bevy::gltf::Gltf>>);

/// Keep the loading screen up until the authored scene and its textures exist.
pub fn preload(world: &mut World) -> bool {
    if !world.contains_resource::<Preload>() {
        let data = SceneData::load();
        let mut paths = vec![
            "original/b612/b612_world.glb".to_string(),
            "original/b612/b612_terrain.glb".to_string(),
            "original/shared/prince.glb".to_string(),
        ];
        for key in ["rose", "can", "cloche", "chair"] {
            paths.push(format!("original/b612/b612_{key}_authored.glb"));
        }
        for layer in data.vegetation {
            paths.extend(layer.paths);
        }
        paths.sort();
        paths.dedup();
        let server = world.resource::<AssetServer>();
        let handles = paths.into_iter().map(|p| server.load(p)).collect();
        world.insert_resource(Preload(handles));
        return false;
    }
    let server = world.resource::<AssetServer>();
    world
        .resource::<Preload>()
        .0
        .iter()
        .all(|h| server.is_loaded_with_dependencies(h.id()))
}

pub fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Vegetation>();
    commands.remove_resource::<SceneData>();
    commands.remove_resource::<Preload>();
}

#[derive(Deserialize)]
pub struct Placement {
    pub position: [f32; 3],
    pub yaw: f32,
    pub scale: [f32; 3],
}
#[derive(Deserialize)]
pub struct Anchor {
    pub x: f32,
    pub z: f32,
    pub height: f32,
    pub yaw: f32,
}
#[derive(Deserialize)]
pub struct Slab {
    pub position: [f32; 3],
    pub yaw: f32,
}
#[derive(Deserialize)]
pub struct BoxShape {
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub yaw: f32,
}
#[derive(Deserialize)]
pub struct ReviewCamera {
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub absolute: bool,
    pub fov: f32,
}
#[derive(Deserialize)]
pub struct VegetationLayer {
    paths: Vec<String>,
    start: usize,
    count: usize,
    lod_distance: f32,
    thin_start: f32,
    thin_power: f32,
    fade_end: f32,
}
#[derive(Deserialize, Resource)]
pub struct SceneData {
    pub models: HashMap<String, Placement>,
    pub anchors: HashMap<String, Anchor>,
    pub slabs: Vec<Slab>,
    pub colliders: Vec<BoxShape>,
    pub water_y: f32,
    pub cameras: HashMap<String, ReviewCamera>,
    vegetation: Vec<VegetationLayer>,
}

impl SceneData {
    pub fn load() -> Self {
        ron::from_str(include_str!("../../../assets/original/b612/scene.ron"))
            .expect("imported B-612 scene metadata")
    }
    pub fn anchor(&self, cx: &ChapterCx, key: &str) -> (Vec3, f32) {
        let a = &self.anchors[key];
        (cx.ground(a.x, a.z) + Vec3::Y * a.height, a.yaw)
    }
    pub fn model(&self, cx: &mut ChapterCx, key: &str) -> Entity {
        let p = &self.models[key];
        let e = cx.model(
            &format!("model.b612.{key}_authored"),
            p.position.into(),
            p.yaw,
            1.0,
        );
        cx.world.get_mut::<Transform>(e).unwrap().scale = p.scale.into();
        e
    }
}

pub fn terrain() -> Terrain {
    Terrain::from_b612_heightmap(
        super::world::planet_params(),
        include_bytes!("../../../assets/original/b612/terrain.f32"),
    )
}

struct Plant {
    transform: Transform,
    sample: f32,
    entity: Option<(Entity, usize)>,
}
struct Layer {
    spec: VegetationLayer,
    meshes: [Handle<GltfMesh>; 2],
    plants: Vec<Plant>,
    ready: [Option<(Handle<Mesh>, Handle<EnvironmentMaterial>)>; 2],
}
#[derive(Resource)]
pub struct Vegetation {
    layers: Vec<Layer>,
    sweep: Option<Sweep>,
    last_origin: Vec3,
    last_quality: f32,
}
struct Sweep {
    layer: usize,
    plant: usize,
    origin: Vec3,
    quality: f32,
}

pub fn vegetation(cx: &mut ChapterCx, data: &mut SceneData) {
    let bytes = include_bytes!("../../../assets/original/b612/vegetation.bin");
    let server = cx.world.resource::<AssetServer>();
    let mut layers = Vec::new();
    for spec in std::mem::take(&mut data.vegetation) {
        let meshes = std::array::from_fn(|i| {
            server.load(GltfAssetLabel::Mesh(0).from_asset(spec.paths[i].clone()))
        });
        let mut plants = Vec::with_capacity(spec.count);
        for index in spec.start..spec.start + spec.count {
            let b = &bytes[index * 48..(index + 1) * 48];
            let f: Vec<f32> = b
                .chunks_exact(4)
                .map(|x| f32::from_le_bytes(x.try_into().unwrap()))
                .collect();
            let transform = Transform::from_xyz(-f[0], f[1], f[2])
                .with_rotation(Quat::from_xyzw(f[4], -f[5], -f[6], f[7]).normalize())
                .with_scale(Vec3::splat(f[3]));
            // Stable sample: thinning never moves plants or invents locations.
            let hash = (index as u32)
                .wrapping_mul(747796405)
                .wrapping_add(2891336453);
            let hash = ((hash >> ((hash >> 28) + 4)) ^ hash).wrapping_mul(277803737);
            let sample = ((hash >> 22) ^ hash) as f32 / u32::MAX as f32;
            plants.push(Plant {
                transform,
                sample,
                entity: None,
            });
        }
        layers.push(Layer {
            spec,
            meshes,
            plants,
            ready: [None, None],
        });
    }
    cx.world.insert_resource(Vegetation {
        layers,
        sweep: None,
        last_origin: Vec3::splat(f32::INFINITY),
        last_quality: -1.0,
    });
}

pub fn stream_vegetation(
    mut commands: Commands,
    mut vegetation: Option<ResMut<Vegetation>>,
    camera: Res<CameraRig>,
    meshes: Res<Assets<GltfMesh>>,
    gltf_mats: Res<Assets<GltfMaterial>>,
    mut materials: ResMut<Assets<EnvironmentMaterial>>,
    geometry: Res<Assets<Mesh>>,
    save: Res<crate::core::save::SaveData>,
) {
    let Some(v) = vegetation.as_deref_mut() else {
        return;
    };
    let quality = match save.settings.quality {
        crate::core::save::Quality::Low => 0.35,
        crate::core::save::Quality::Medium => 0.7,
        crate::core::save::Quality::High => 1.0,
    };
    // Stationary scenes need no population scan. Moving cameras get a bounded
    // sweep instead of spawning/despawning thousands of entities in one frame.
    if v.sweep.is_none() {
        if v.last_quality == quality
            && v.last_origin.distance_squared(camera.last_pos) < 0.75 * 0.75
        {
            return;
        }
        v.sweep = Some(Sweep {
            layer: 0,
            plant: 0,
            origin: camera.last_pos,
            quality,
        });
    }
    for layer in &mut v.layers {
        for lod in 0..2 {
            if layer.ready[lod].is_some() {
                continue;
            }
            let Some(mesh) = meshes.get(&layer.meshes[lod]) else {
                continue;
            };
            let Some(primitive) = mesh.primitives.first() else {
                continue;
            };
            let Some(mat) = primitive.material.as_ref().and_then(|h| gltf_mats.get(h)) else {
                continue;
            };
            let mut mat = bevy::pbr::gltf::standard_material_from_gltf_material(mat);
            mat.perceptual_roughness = 0.95;
            mat.reflectance = 0.15;
            let height = geometry
                .get(&primitive.mesh)
                .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
                .and_then(|a| a.as_float3())
                .map(|v| v.iter().map(|p| p[1]).fold(0.1, f32::max))
                .unwrap_or(1.0);
            let material = materials.add(ExtendedMaterial {
                base: mat,
                extension: EnvironmentExtension::vegetation(height),
            });
            layer.ready[lod] = Some((primitive.mesh.clone(), material));
        }
    }
    let mut sweep = v.sweep.take().unwrap();
    let mut scanned = 0;
    let mut changes = 0;
    while sweep.layer < v.layers.len() && scanned < 4096 && changes < 128 {
        let layer = &mut v.layers[sweep.layer];
        if layer.ready.iter().any(Option::is_none) {
            v.sweep = Some(sweep);
            return;
        }
        if sweep.plant >= layer.plants.len() {
            sweep.layer += 1;
            sweep.plant = 0;
            continue;
        }
        let plant = &mut layer.plants[sweep.plant];
        sweep.plant += 1;
        scanned += 1;
        let distance = sweep.origin.distance(plant.transform.translation);
        let density = (layer.spec.thin_start / distance.max(1.0))
            .powf(layer.spec.thin_power)
            .min(1.0)
            * sweep.quality;
        let visible = distance < layer.spec.fade_end.min(38.0) && plant.sample < density;
        if !visible {
            if let Some((e, _)) = plant.entity.take() {
                commands.entity(e).despawn();
                changes += 1;
            }
            continue;
        }
        let lod = usize::from(distance > layer.spec.lod_distance);
        let (mesh, mat) = layer.ready[lod].as_ref().unwrap();
        if let Some((e, old_lod)) = plant.entity {
            if old_lod != lod {
                commands
                    .entity(e)
                    .insert((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone())));
                plant.entity = Some((e, lod));
                changes += 1;
            }
        } else {
            let e = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    plant.transform,
                    NotShadowCaster,
                    ChapterScoped,
                ))
                .id();
            plant.entity = Some((e, lod));
            changes += 1;
        }
    }
    if sweep.layer == v.layers.len() {
        v.last_origin = sweep.origin;
        v.last_quality = sweep.quality;
    } else {
        v.sweep = Some(sweep);
    }
}

/// Reuse the original scene's review cameras for repeatable visual comparisons.
/// This is opt-in (?review=CAM_Rose_Main); normal play keeps the story camera.
pub fn review_camera(world: &mut World) {
    let Some(name) = world
        .resource::<crate::core::params::LaunchParams>()
        .get("review")
        .map(str::to_string)
    else {
        return;
    };
    let Some(data) = world.get_resource::<SceneData>() else {
        return;
    };
    let (position, look_at, fov) = if name == "opening" {
        let feet = world.resource::<super::systems::Sites>().feet;
        let yaw = super::world::OPENING_VIEW_YAW.to_radians();
        let pivot = feet + Vec3::Y;
        (
            pivot + Vec3::new(yaw.sin() * 3.4, 0.237, yaw.cos() * 3.4),
            pivot,
            50.0,
        )
    } else {
        let Some(c) = data.cameras.get(&name) else {
            return;
        };
        let mut p = Vec3::from(c.position);
        let mut l = Vec3::from(c.look_at);
        if !c.absolute {
            let t = world.resource::<Terrain>();
            p.y += t.height(p.x, p.z);
            l.y += t.height(l.x, l.z);
        }
        (p, l, c.fov)
    };
    if name == "opening" {
        // The source review shot is the standing frame after waking up.
        let feet = world.resource::<super::systems::Sites>().feet;
        let entity = world.resource::<super::systems::B612State>().player;
        if let Some(mut p) = world.get_mut::<Player>(entity) {
            p.yaw = super::world::SPAWN_YAW.to_radians();
            p.pose = Pose::None;
            p.grounded = true;
        }
        if let Some(mut t) = world.get_mut::<Transform>(entity) {
            t.translation = feet;
            t.rotation = Quat::IDENTITY;
        }
    }
    super::director::shot(world, position, look_at, 0.0, fov);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_ground_and_interactions_agree() {
        let data = SceneData::load();
        let t = terrain();
        let rose = &data.models["rose"];
        assert!((t.height(rose.position[0], rose.position[2]) - rose.position[1]).abs() < 0.12);
        let can = &data.models["can"];
        assert!((t.height(can.position[0], can.position[2]) - can.position[1]).abs() < 0.12);
        assert!(
            data.slabs
                .iter()
                .all(|s| (s.position[1] - data.water_y).abs() < 0.4)
        );
        assert_eq!(
            data.vegetation.iter().map(|s| s.count).sum::<usize>(),
            83955
        );
        assert!(t.height(0.0, 0.0) > 4.8);
    }
}
