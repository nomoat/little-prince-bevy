//! The core engine shared by all chapters. See docs/CORE_API.md.

pub mod audio;
pub mod camera;
pub mod chapter;
pub mod i18n;
pub mod input;
pub mod interact;
pub mod manifest;
pub mod meshkit;
pub mod models;
pub mod params;
pub mod physics;
pub mod planet;
pub mod platform;
pub mod player;
pub mod save;
pub mod sky;
pub mod state;
pub mod story;
pub mod synth;
pub mod testing;
pub mod ui;

/// Everything a chapter usually needs.
pub mod prelude {
    pub use super::audio::AudioCmd;
    pub use super::camera::{CameraRig, Shot};
    pub use super::chapter::{Chapter, ChapterCx, ChapterInfo, in_chapter};
    pub use super::i18n::{I18n, Lang, StringTable, Tr};
    pub use super::input::InputFrame;
    pub use super::interact::{Interactable, Interacted};
    pub use super::meshkit::{MeshBuilder, Rng, hex, mix_color, scale_color};
    pub use super::models::{ModelParts, PartMat, PartName, ProceduralAppExt, SpawnModelExt};
    pub use super::physics::Collider;
    pub use super::planet::{PlanetParams, Terrain, TerrainMod};
    pub use super::player::{Drive, Player, Pose};
    pub use super::sky::{DayPhase, Sky, SkyPalette};
    pub use super::state::ChapterScoped;
    pub use super::story::cmd::*;
    pub use super::story::{Cmd, EventDef, Pos, Story, StoryRunner, TestAction, event, found_hidden};
    pub use super::ui::hud::{Hud, Speaker, speakers};
    pub use super::CoreSet;
    pub use bevy::prelude::*;
}

use bevy::asset::AssetMetaCheck;
use bevy::pbr::DistanceFog;
use bevy::prelude::*;
use bevy::ui::IsDefaultUiCamera;
use camera::MainCamera;
use manifest::{Manifest, ManifestFile, ManifestLoader, ManifestLoading};
use params::LaunchParams;
use save::{Quality, SaveData, SavePersist};
use state::AppState;

/// Update-schedule ordering. Chapter systems normally go in `Gameplay`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreSet {
    Input,
    Physics,
    Gameplay,
    Story,
    Camera,
    Presentation,
}

/// Builds the app: window, plugins, core resources and systems.
pub fn build_app(app: &mut App) {
    let params = LaunchParams::from_query(&platform::query_string());
    let mut save = if params.fresh { SaveData::default() } else { SaveData::load() };
    if let Some(l) = params.lang.as_deref().and_then(i18n::Lang::parse) {
        save.lang = l;
    }
    if params.free {
        save.settings.free_travel = true;
    }
    if let Some(q) = params.quality.as_deref() {
        save.settings.quality = match q {
            "low" => Quality::Low,
            "high" => Quality::High,
            _ => Quality::Medium,
        };
    }
    let test = params.test;
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "小王子 · 星辰之旅".into(),
                    canvas: Some("#bevy".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin { meta_check: AssetMetaCheck::Never, file_path: asset_root(), ..default() })
            .set(bevy::log::LogPlugin { filter: "wgpu=error,naga=warn,bevy_render=warn,bevy_asset=warn".into(), ..default() }),
    );
    let mut i18n = i18n::I18n::default();
    i18n.lang = save.lang;
    i18n.add_table(i18n::CORE_STRINGS);
    i18n.add_table(ui::hud::SPEAKER_STRINGS);
    i18n.add_table(ui::menu::MENU_STRINGS);
    let speed = params.speed.unwrap_or(if test { 1.5 } else { 1.0 });
    app.insert_resource(SavePersist(!params.fresh))
        .insert_resource(save)
        .insert_resource(i18n)
        .insert_resource(params)
        .insert_resource(manifest::embedded_manifest())
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.17)))
        .insert_resource(ui::hud::Fade { alpha: 1.0, target: 1.0, speed: 1.0, ..default() })
        .init_asset::<ManifestFile>()
        .register_asset_loader(ManifestLoader)
        .init_state::<AppState>()
        .add_message::<state::Flow>()
        .add_message::<interact::Interacted>()
        .add_message::<audio::AudioCmd>()
        .init_resource::<save::SaveTimer>()
        .init_resource::<ManifestLoading>()
        .init_resource::<state::ActiveChapter>()
        .init_resource::<state::Transition>()
        .init_resource::<state::DepartHold>()
        .init_resource::<input::InputFrame>()
        .init_resource::<input::InputGate>()
        .init_resource::<input::AgentInput>()
        .init_resource::<input::TouchControls>()
        .init_resource::<input::MouseState>()
        .init_resource::<input::PointerOverUi>()
        .init_resource::<interact::InteractFocus>()
        .init_resource::<physics::ColliderWorld>()
        .init_resource::<player::PlayerTuning>()
        .init_resource::<camera::CameraTuning>()
        .init_resource::<camera::CameraRig>()
        .init_resource::<sky::Sky>()
        .init_resource::<story::StoryRunner>()
        .init_resource::<story::StoryLock>()
        .init_resource::<story::Anchors>()
        .init_resource::<audio::AudioState>()
        .init_resource::<models::ModelCache>()
        .init_resource::<models::ProceduralModels>()
        .init_resource::<testing::Autopilot>()
        .insert_resource(Time::<Virtual>::from_max_delta(std::time::Duration::from_millis(250)))
        .configure_sets(Update, (CoreSet::Input, CoreSet::Physics, CoreSet::Gameplay, CoreSet::Story, CoreSet::Camera, CoreSet::Presentation).chain());
    app.world_mut().resource_mut::<Time<Virtual>>().set_relative_speed(speed);
    models::register_core_models(app);
    ui::plugin(app);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Last, testing::native_shots);
    app.add_systems(Startup, (models::setup_shared_materials, boot_fonts, start_manifest_loading, setup_camera, sky::setup_sky))
        .add_systems(Update, state::boot.run_if(in_state(AppState::Boot)))
        .add_systems(Update, (merge_runtime_manifests, state::handle_flow, testing::flow_driver, testing::probe, testing::exit_when_done, save::persist_save, apply_quality, i18n::apply_translations))
        .add_systems(OnEnter(AppState::Loading), state::loading_hop)
        .add_systems(OnEnter(AppState::Chapter), (state::enter_chapter, reset_autopilot))
        .add_systems(OnExit(AppState::Chapter), state::exit_chapter)
        .add_systems(Update, (story::update_input_gate, input::gather_input).chain().in_set(CoreSet::Input))
        .add_systems(Update, (models::resolve_pending_models, models::gltf_fallback, physics::rebuild_collider_world, player::player_controller).chain().in_set(CoreSet::Physics))
        .add_systems(
            Update,
            (interact::update_interaction, story::collect_interactions, story::run_story, story::stall_marker, state::departure, testing::autopilot)
                .chain()
                .in_set(CoreSet::Story)
                .run_if(in_state(AppState::Chapter)),
        )
        .add_systems(Update, camera::update_camera.in_set(CoreSet::Camera))
        .add_systems(
            Update,
            (sky::tick_sky_clock, sky::apply_sky, player::animate_prince, audio::handle_audio, audio::update_tracks, audio::footsteps).chain().in_set(CoreSet::Presentation),
        );
}

/// Native: use `./assets` of the working directory when present (so the
/// binary can be run directly from the repo); web: `assets/` next to the page.
fn asset_root() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if std::env::var("BEVY_ASSET_ROOT").is_err() && std::env::var("CARGO_MANIFEST_DIR").is_err() {
            if let Ok(cwd) = std::env::current_dir() {
                let p = cwd.join("assets");
                if p.is_dir() {
                    return p.to_string_lossy().into_owned();
                }
            }
        }
    }
    "assets".into()
}

fn reset_autopilot(mut ap: ResMut<testing::Autopilot>) {
    *ap = testing::Autopilot::default();
}

/// Loads the UI font named by the manifest (`font.ui`).
pub fn boot_fonts(mut commands: Commands, server: Res<AssetServer>, manifest: Res<Manifest>) {
    let path = manifest.path("font.ui").unwrap_or("fonts/ui.ttf").to_string();
    commands.insert_resource(ui::UiFont(server.load(path)));
}

fn start_manifest_loading(server: Res<AssetServer>, mut loading: ResMut<ManifestLoading>) {
    for (name, _) in manifest::EMBEDDED {
        loading.handles.push(server.load::<ManifestFile>(*name));
    }
}

fn merge_runtime_manifests(mut loading: ResMut<ManifestLoading>, server: Res<AssetServer>, files: Res<Assets<ManifestFile>>, mut manifest: ResMut<Manifest>, time: Res<Time<Real>>) {
    if loading.merged {
        return;
    }
    let all_done = loading.handles.iter().all(|h| {
        matches!(server.load_state(h), bevy::asset::LoadState::Loaded | bevy::asset::LoadState::Failed(_))
    });
    if !all_done && time.elapsed_secs() < 6.0 {
        return;
    }
    for h in &loading.handles {
        if let Some(f) = files.get(h) {
            manifest.merge(f);
        }
    }
    loading.merged = true;
    info!("manifest: {} entries", manifest.entries.len());
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("MainCamera"),
        MainCamera,
        Camera3d::default(),
        // A LUT-free tonemapper keeps the wasm small (no ktx2/zstd LUTs).
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        IsDefaultUiCamera,
        Projection::Perspective(PerspectiveProjection { fov: 50f32.to_radians(), near: 0.1, far: 1200.0, ..default() }),
        DistanceFog { color: Color::srgb(0.8, 0.85, 0.9), falloff: bevy::pbr::FogFalloff::Linear { start: 30.0, end: 190.0 }, ..default() },
        Transform::from_xyz(0.0, 3.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn apply_quality(
    save: Res<SaveData>,
    mut done: Local<Option<Quality>>,
    mut commands: Commands,
    cam: Query<Entity, With<MainCamera>>,
    mut lights: Query<&mut DirectionalLight>,
    mut shadow_map: ResMut<bevy::light::DirectionalLightShadowMap>,
) {
    let q = save.settings.quality;
    if *done == Some(q) && !lights.iter().any(|l| l.shadow_maps_enabled != (q != Quality::Low)) {
        return;
    }
    *done = Some(q);
    if let Ok(c) = cam.single() {
        commands.entity(c).insert(match q {
            Quality::Low => Msaa::Off,
            _ => Msaa::Sample4,
        });
    }
    for mut l in &mut lights {
        l.shadow_maps_enabled = q != Quality::Low;
    }
    shadow_map.size = match q {
        Quality::High => 2048,
        _ => 1024,
    };
}
