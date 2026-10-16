//! Portable environmental shading for the imported scene. Vegetation bends
//! on the GPU; the original meshes, textures and instance locations stay intact.
use super::{CoreSet, player::Player, sky::Sky};
use bevy::{
    gltf::GltfMaterialName,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};
use std::collections::HashMap;

pub type EnvironmentMaterial = ExtendedMaterial<StandardMaterial, EnvironmentExtension>;

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct EnvironmentUniform {
    motion: Vec4,
    sun: Vec4,
    tint: Vec4,
    flow: Vec4,
    actor: Vec4,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct EnvironmentExtension {
    #[uniform(100)]
    #[reflect(ignore)]
    uniform: EnvironmentUniform,
}

impl MaterialExtension for EnvironmentExtension {
    fn vertex_shader() -> ShaderRef {
        "shaders/environment_vertex.wesl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/environment_fragment.wesl".into()
    }
}

impl EnvironmentExtension {
    pub fn vegetation(height: f32) -> Self {
        Self {
            uniform: EnvironmentUniform {
                motion: Vec4::new(0.0, 0.0, 0.12, height.max(0.1)),
                sun: Vec4::new(0.0, 1.0, 0.0, 1.0),
                // M_B612_Grass_Meadow: _ColorTrans and _TransAmount.
                tint: Vec4::new(0.80, 0.82, 0.48, 0.22),
                flow: Vec4::new(0.85, 0.53, 0.0, 0.92),
                actor: Vec4::new(0.0, -100.0, 0.0, 0.65),
            },
        }
    }
    fn water(name: &str) -> Self {
        let mut effect = Self::vegetation(1.0);
        // Source _ScatterColor, _Roughness and flow/phase speeds. The
        // shader itself is a portable reimplementation, not Unity shader code.
        let (kind, speed, roughness, tint) = match name {
            "M_B612_Pond" => (1.0, 0.44, 0.035, Vec3::new(0.16, 0.34, 0.33)),
            "M_B612_Waterfall" => (3.0, 2.6, 0.12, Vec3::new(0.78, 0.86, 0.90)),
            _ => (2.0, 1.5, 0.06, Vec3::new(0.15, 0.34, 0.33)),
        };
        effect.uniform.motion.x = kind;
        effect.uniform.tint = tint.extend(0.7);
        effect.uniform.flow = Vec4::new(-0.6, -0.8, speed, roughness);
        effect
    }
}

#[derive(Component)]
struct Converted;

pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<EnvironmentMaterial>::default())
        .add_systems(
            Update,
            (convert_water, animate_environment)
                .chain()
                .in_set(CoreSet::Presentation),
        );
}

fn convert_water(
    mut commands: Commands,
    query: Query<
        (Entity, &GltfMaterialName, &MeshMaterial3d<StandardMaterial>),
        Without<Converted>,
    >,
    standard: Res<Assets<StandardMaterial>>,
    mut extended: ResMut<Assets<EnvironmentMaterial>>,
    mut cache: Local<HashMap<AssetId<StandardMaterial>, Handle<EnvironmentMaterial>>>,
) {
    for (entity, name, material) in &query {
        if !matches!(
            name.0.as_str(),
            "M_B612_Water" | "M_B612_Pond" | "M_B612_Brook" | "M_B612_Waterfall"
        ) {
            commands.entity(entity).insert(Converted);
            continue;
        }
        let Some(base) = standard.get(&material.0) else {
            continue;
        };
        let handle = cache
            .entry(material.0.id())
            .or_insert_with(|| {
                let mut base = base.clone();
                let extension = EnvironmentExtension::water(&name.0);
                base.perceptual_roughness = extension.uniform.flow.w;
                base.metallic = 0.0;
                base.reflectance = 0.35;
                extended.add(ExtendedMaterial { base, extension })
            })
            .clone();
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((MeshMaterial3d(handle), Converted));
    }
}

fn animate_environment(
    time: Res<Time>,
    sky: Res<Sky>,
    player: Query<&GlobalTransform, With<Player>>,
    mut materials: ResMut<Assets<EnvironmentMaterial>>,
) {
    let position = player
        .iter()
        .next()
        .map(|p| p.translation())
        .unwrap_or(Vec3::splat(-1000.0));
    for (_, material) in materials.iter_mut() {
        let u = &mut material.extension.uniform;
        u.motion.y = time.elapsed_secs_wrapped();
        u.sun = sky.sun_dir.extend(1.0 - sky.night * 0.92);
        u.actor = position.extend(0.65);
    }
}
