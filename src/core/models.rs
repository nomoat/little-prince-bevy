//! Model spawning through the manifest. `commands.spawn_model("model.x", tf)`
//! spawns an entity whose children are either the glTF scene named by the
//! manifest entry, or the parts of a registered procedural generator.
//! If a glTF fails to load, the procedural fallback is spawned instead.

use super::manifest::Manifest;
use super::meshkit::{MeshBuilder, hex};
use bevy::asset::LoadState;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot};
use std::collections::HashMap;

/// Material of one procedural part.
#[derive(Clone)]
pub enum PartMat {
    /// Lit, vertex-coloured (the default for everything).
    Vertex,
    /// Unlit, vertex-coloured (glows, sky bits).
    Unlit,
    /// Vertex colour plus emissive glow (lamps, windows). `strength` multiplies.
    Emissive(Color, f32),
    /// Transparent glass tinted by `Color` (alpha = opacity).
    Glass(Color),
    /// Anything else.
    Custom(StandardMaterial),
}

pub struct Part {
    pub mesh: Mesh,
    pub transform: Transform,
    pub mat: PartMat,
    /// Named parts get a [`PartName`] so chapter code can find and animate them.
    pub name: Option<String>,
    pub cast_shadows: bool,
}

/// What a procedural generator fills in.
#[derive(Default)]
pub struct ModelParts {
    pub parts: Vec<Part>,
}

impl ModelParts {
    /// Adds a vertex-coloured lit part.
    pub fn add(&mut self, mb: &MeshBuilder) -> &mut Self {
        self.add_part(None, mb, Transform::IDENTITY, PartMat::Vertex)
    }
    pub fn add_named(&mut self, name: &str, mb: &MeshBuilder, t: Transform) -> &mut Self {
        self.add_part(Some(name), mb, t, PartMat::Vertex)
    }
    pub fn add_part(&mut self, name: Option<&str>, mb: &MeshBuilder, t: Transform, mat: PartMat) -> &mut Self {
        if !mb.is_empty() {
            self.parts.push(Part {
                mesh: mb.build(),
                transform: t,
                mat,
                name: name.map(|s| s.to_string()),
                cast_shadows: true,
            });
        }
        self
    }
}

pub type ModelFn = fn(&mut ModelParts);

/// Registry of procedural generators, keyed by the manifest's `procedural` id.
#[derive(Resource, Default)]
pub struct ProceduralModels(pub HashMap<String, ModelFn>);

pub trait ProceduralAppExt {
    /// Registers a procedural generator under `id` (used by manifest entries
    /// `procedural: "id"`, or directly as a key if no manifest entry exists).
    fn register_model(&mut self, id: &str, f: ModelFn) -> &mut Self;
}

impl ProceduralAppExt for App {
    fn register_model(&mut self, id: &str, f: ModelFn) -> &mut Self {
        self.world_mut()
            .get_resource_or_insert_with(ProceduralModels::default)
            .0
            .insert(id.to_string(), f);
        self
    }
}

/// Shared materials.
#[derive(Resource, Clone)]
pub struct SharedMaterials {
    pub vertex: Handle<StandardMaterial>,
    pub unlit: Handle<StandardMaterial>,
}

/// A part's name inside a spawned model.
#[derive(Component, Clone, Debug)]
pub struct PartName(pub String);

/// The manifest key a model entity was spawned from.
#[derive(Component, Clone, Debug)]
pub struct ModelKey(pub String);

/// Waiting to be resolved into children by [`resolve_pending_models`].
#[derive(Component)]
pub struct PendingModel(pub String);

/// A glTF child we watch so we can fall back to procedural on failure.
#[derive(Component)]
pub(crate) struct GltfWatch {
    key: String,
    handle: Handle<WorldAsset>,
}

#[derive(Resource, Default)]
pub(crate) struct ModelCache(HashMap<String, Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform, Option<String>, bool)>>);

pub trait SpawnModelExt {
    /// Spawns a model by manifest key; returns the root entity's commands.
    fn spawn_model(&mut self, key: &str, transform: Transform) -> EntityCommands<'_>;
}

impl SpawnModelExt for Commands<'_, '_> {
    fn spawn_model(&mut self, key: &str, transform: Transform) -> EntityCommands<'_> {
        self.spawn((transform, Visibility::default(), ModelKey(key.to_string()), PendingModel(key.to_string())))
    }
}

/// Resolves `PendingModel` into a glTF scene child or procedural parts.
pub(crate) fn resolve_pending_models(
    mut commands: Commands,
    q: Query<(Entity, &PendingModel)>,
    manifest: Res<Manifest>,
    procs: Res<ProceduralModels>,
    server: Res<AssetServer>,
    mut cache: ResMut<ModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    shared: Res<SharedMaterials>,
) {
    for (e, pm) in &q {
        commands.entity(e).remove::<PendingModel>();
        let key = pm.0.as_str();
        let entry = manifest.get(key);
        if let Some(path) = entry.and_then(|en| en.variants.first().cloned().or(en.path.clone())) {
            let en = entry.unwrap();
            let scene_path = if path.contains('#') { path.clone() } else { GltfAssetLabel::Scene(0).from_asset(path.clone()).to_string() };
            let handle: Handle<WorldAsset> = server.load(scene_path);
            let t = Transform::from_translation(Vec3::from(en.offset))
                .with_rotation(Quat::from_rotation_y(en.yaw.to_radians()))
                .with_scale(Vec3::splat(en.scale));
            let child = commands
                .spawn((WorldAssetRoot(handle.clone()), t, GltfWatch { key: key.to_string(), handle }))
                .id();
            commands.entity(e).add_child(child);
            continue;
        }
        spawn_procedural_children(&mut commands, e, key, entry.and_then(|en| en.procedural.as_deref()), &procs, &mut cache, &mut meshes, &mut mats, &shared);
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_procedural_children(
    commands: &mut Commands,
    parent: Entity,
    key: &str,
    proc_id: Option<&str>,
    procs: &ProceduralModels,
    cache: &mut ModelCache,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    shared: &SharedMaterials,
) {
    let id = proc_id.unwrap_or(key).to_string();
    if !cache.0.contains_key(&id) {
        let f = procs
            .0
            .get(&id)
            .or_else(|| procs.0.get(key))
            .copied()
            .unwrap_or_else(|| {
                warn!("model '{key}': no file and no procedural generator '{id}', using placeholder");
                placeholder as ModelFn
            });
        let mut parts = ModelParts::default();
        f(&mut parts);
        let built = parts
            .parts
            .into_iter()
            .map(|p| {
                let mat = match p.mat {
                    PartMat::Vertex => shared.vertex.clone(),
                    PartMat::Unlit => shared.unlit.clone(),
                    PartMat::Emissive(c, k) => mats.add(StandardMaterial {
                        base_color: Color::WHITE,
                        emissive: c.to_linear() * k,
                        perceptual_roughness: 0.8,
                        ..default()
                    }),
                    PartMat::Glass(c) => mats.add(StandardMaterial {
                        base_color: c,
                        alpha_mode: AlphaMode::Blend,
                        perceptual_roughness: 0.08,
                        reflectance: 0.6,
                        ..default()
                    }),
                    PartMat::Custom(m) => mats.add(m),
                };
                (meshes.add(p.mesh), mat, p.transform, p.name, p.cast_shadows)
            })
            .collect();
        cache.0.insert(id.clone(), built);
    }
    let parts = cache.0.get(&id).unwrap();
    for (mesh, mat, t, name, shadows) in parts {
        let mut ec = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), *t));
        if let Some(n) = name {
            ec.insert((PartName(n.clone()), Name::new(n.clone())));
        }
        if !shadows {
            ec.insert(bevy::light::NotShadowCaster);
        }
        let c = ec.id();
        commands.entity(parent).add_child(c);
    }
}

/// glTF load failed → spawn the procedural fallback next to it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn gltf_fallback(
    mut commands: Commands,
    q: Query<(Entity, &GltfWatch, &ChildOf)>,
    server: Res<AssetServer>,
    manifest: Res<Manifest>,
    procs: Res<ProceduralModels>,
    mut cache: ResMut<ModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    shared: Res<SharedMaterials>,
) {
    for (e, w, parent) in &q {
        match server.load_state(&w.handle) {
            LoadState::Failed(_) => {
                warn!("model '{}': file failed to load, using procedural fallback", w.key);
                commands.entity(e).despawn();
                let proc_id = manifest.get(&w.key).and_then(|en| en.procedural.clone());
                spawn_procedural_children(&mut commands, parent.parent(), &w.key, proc_id.as_deref(), &procs, &mut cache, &mut meshes, &mut mats, &shared);
            }
            LoadState::Loaded => {
                commands.entity(e).remove::<GltfWatch>();
            }
            _ => {}
        }
    }
}

// ------------------------------------------------- core procedural models --

pub fn placeholder(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.5, Vec3::ONE, hex(0xd070c0));
    p.add(&m);
}

fn rock(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::Y * 0.35, Vec3::new(0.7, 0.5, 0.6), 3, hex(0x8f8a80));
    p.add(&m);
}

fn rock_small(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.rock(Vec3::Y * 0.12, Vec3::new(0.25, 0.18, 0.22), 7, hex(0x9a948a));
    p.add(&m);
}

fn tree(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.16, 0.1, 1.4, 6, hex(0x7a5636));
    m.sphere(Vec3::Y * 2.0, 0.9, 4, 7, hex(0x5f9a4e));
    m.sphere(Vec3::new(0.35, 1.7, 0.2), 0.6, 3, 6, hex(0x6ea85a));
    p.add(&m);
}

fn bush(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.sphere(Vec3::Y * 0.3, 0.42, 3, 6, hex(0x5d9150));
    m.sphere(Vec3::new(0.3, 0.22, 0.1), 0.3, 3, 6, hex(0x6aa05a));
    p.add(&m);
}

fn grass_tuft(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for i in 0..5 {
        let a = i as f32 * 1.3;
        let base = Vec3::new(a.cos() * 0.06, 0.0, a.sin() * 0.06);
        let tip = base + Vec3::new(a.cos() * 0.08, 0.32 + (i % 2) as f32 * 0.1, a.sin() * 0.08);
        m.tube(base, tip, 0.025, 0.0, 3, hex(0x7fae55));
    }
    let mut parts = MeshBuilder::new();
    parts.append(&m);
    p.parts.push(Part { mesh: parts.build(), transform: Transform::IDENTITY, mat: PartMat::Vertex, name: None, cast_shadows: false });
}

fn flower(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::Y * 0.3, 0.015, 0.012, 4, hex(0x5c8f45));
    for i in 0..5 {
        let a = i as f32 / 5.0 * std::f32::consts::TAU;
        m.sphere(Vec3::new(a.cos() * 0.05, 0.32, a.sin() * 0.05), 0.035, 2, 4, hex(0xf3d36b));
    }
    m.sphere(Vec3::Y * 0.32, 0.03, 2, 4, hex(0xd0763a));
    p.add(&m);
}

fn chair(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let wood = hex(0x9b6b43);
    for (x, z) in [(-0.2, -0.2), (0.2, -0.2), (-0.2, 0.2), (0.2, 0.2)] {
        m.cuboid(Vec3::new(x, 0.22, z), Vec3::new(0.05, 0.44, 0.05), wood);
    }
    m.cuboid(Vec3::new(0.0, 0.46, 0.0), Vec3::new(0.48, 0.05, 0.48), hex(0xb27d50));
    m.cuboid(Vec3::new(0.0, 0.75, -0.22), Vec3::new(0.48, 0.5, 0.05), wood);
    p.add(&m);
}

fn lamp_post(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.12, 0.06, 0.15, 8, hex(0x3d3d48));
    m.frustum(Vec3::Y * 0.15, 0.04, 0.035, 2.1, 8, hex(0x3d3d48));
    m.frustum(Vec3::Y * 2.25, 0.16, 0.0, 0.18, 6, hex(0x3d3d48));
    p.add(&m);
    let mut g = MeshBuilder::new();
    g.frustum(Vec3::Y * 2.0, 0.09, 0.12, 0.25, 6, hex(0xffe2a0));
    p.add_part(Some("flame"), &g, Transform::IDENTITY, PartMat::Emissive(hex(0xffc864), 4.0));
}

fn npc_generic(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.32, 0.0), (0.3, 0.5), (0.22, 1.0), (0.12, 1.25), (0.0, 1.3)], 10, hex(0x6c5a8e));
    m.sphere(Vec3::Y * 1.45, 0.2, 4, 8, hex(0xf2d2b6));
    m.sphere(Vec3::new(0.0, 1.5, -0.04), 0.205, 3, 8, hex(0x8a8a8a));
    p.add(&m);
}

pub(crate) fn register_core_models(app: &mut App) {
    app.register_model("placeholder", placeholder)
        .register_model("rock", rock)
        .register_model("rock_small", rock_small)
        .register_model("tree", tree)
        .register_model("bush", bush)
        .register_model("grass_tuft", grass_tuft)
        .register_model("flower", flower)
        .register_model("chair", chair)
        .register_model("lamp_post", lamp_post)
        .register_model("npc_generic", npc_generic);
}

pub(crate) fn setup_shared_materials(mut commands: Commands, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(SharedMaterials {
        vertex: mats.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 0.92, reflectance: 0.25, ..default() }),
        unlit: mats.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..default() }),
    });
}
