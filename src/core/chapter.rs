//! The chapter contract. Each of the eight planets is a [`Chapter`]
//! implementation living in `src/chapters/<id>/`. The core owns the flow
//! (title → map → chapter → departure → map … → ending); a chapter only
//! builds its world ([`Chapter::build`]), returns its story
//! ([`Chapter::story`]) and may add its own systems ([`Chapter::setup_app`])
//! guarded by [`in_chapter`].

use super::i18n::StringTable;
use super::interact::Interactable;
use super::meshkit::hex;
use super::models::{PendingModel, ModelKey};
use super::physics::Collider;
use super::planet::{PlanetGround, PlanetParams, Terrain};
use super::sky::{Sky, SkyPalette};
use super::state::{ActiveChapter, AppState, ChapterScoped};
use super::story::{Anchors, Story};
use bevy::prelude::*;
use std::sync::Arc;

/// Static information (journey map, cards, loading page).
#[derive(Clone, Debug)]
pub struct ChapterInfo {
    pub id: &'static str,
    /// 1..=8, also the journey order.
    pub number: u32,
    /// i18n keys.
    pub title: &'static str,
    pub subtitle: &'static str,
    /// Map narration line (before the first visit) and after visiting.
    pub line: &'static str,
    pub visited_line: &'static str,
    /// Loading page line.
    pub loading: &'static str,
    /// Position on the journey map stage (1672 × 941 px, origin top-left), and radius.
    pub map_pos: Vec2,
    pub map_radius: f32,
    /// Planet colours on the map (body, accent).
    pub map_colors: (Color, Color),
    /// Default music key for the chapter.
    pub music: &'static str,
}

pub trait Chapter: Send + Sync + 'static {
    fn info(&self) -> ChapterInfo;
    /// The chapter's string table (`<id>.*` keys).
    fn strings(&self) -> StringTable {
        &[]
    }
    /// Register systems (use `.run_if(in_chapter("<id>"))`), procedural
    /// models (`app.register_model(..)`) and resources.
    fn setup_app(&self, _app: &mut App) {}
    /// Build the world: planet, sky, props, NPCs, interactables, anchors, player.
    fn build(&self, cx: &mut ChapterCx);
    /// The story script.
    fn story(&self) -> Story;
}

/// All chapters in journey order.
#[derive(Resource, Clone, Default)]
pub struct ChapterRegistry(pub Vec<Arc<dyn Chapter>>);

impl ChapterRegistry {
    pub fn get(&self, id: &str) -> Option<Arc<dyn Chapter>> {
        self.0.iter().find(|c| c.info().id == id).cloned()
    }
    pub fn info(&self, id: &str) -> Option<ChapterInfo> {
        self.get(id).map(|c| c.info())
    }
    pub fn infos(&self) -> Vec<ChapterInfo> {
        self.0.iter().map(|c| c.info()).collect()
    }
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.0.iter().position(|c| c.info().id == id)
    }
    pub fn first_id(&self) -> &'static str {
        self.0.first().map(|c| c.info().id).unwrap_or("b612")
    }
    pub fn last_id(&self) -> &'static str {
        self.0.last().map(|c| c.info().id).unwrap_or("earth")
    }
}

/// Run condition: this chapter is the active, running one.
pub fn in_chapter(id: &'static str) -> impl FnMut(Option<Res<ActiveChapter>>, Res<State<AppState>>) -> bool + Clone {
    move |active: Option<Res<ActiveChapter>>, state: Res<State<AppState>>| {
        *state.get() == AppState::Chapter && active.map(|a| a.id.as_deref() == Some(id)).unwrap_or(false)
    }
}

/// Builder handed to [`Chapter::build`]. Everything spawned through it is
/// despawned automatically when the chapter ends.
pub struct ChapterCx<'w> {
    pub world: &'w mut World,
    pub id: &'static str,
    pub test: bool,
}

impl<'w> ChapterCx<'w> {
    /// Creates the domed-terrain planet (ground + rim meshes, Terrain resource).
    pub fn planet(&mut self, params: PlanetParams) -> &Terrain {
        let terrain = Terrain::new(params);
        let (top, rim) = terrain.build_meshes();
        let tex = terrain.params.ground_texture.clone();
        let shared = self.world.resource::<super::models::SharedMaterials>().clone();
        let mut ground_mat = shared.vertex.clone();
        if let Some(key) = tex {
            let path = self.world.resource::<super::manifest::Manifest>().path(&key).map(|s| s.to_string());
            if let Some(p) = path {
                let img: Handle<Image> = self.world.resource::<AssetServer>().load_builder().with_settings(|s: &mut bevy::image::ImageLoaderSettings| {
                    s.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                        address_mode_u: bevy::image::ImageAddressMode::Repeat,
                        address_mode_v: bevy::image::ImageAddressMode::Repeat,
                        ..bevy::image::ImageSamplerDescriptor::linear()
                    });
                }).load(p);
                ground_mat = self.world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial { base_color_texture: Some(img), perceptual_roughness: 0.95, reflectance: 0.2, ..default() });
            }
        }
        let top_h = self.world.resource_mut::<Assets<Mesh>>().add(top);
        let rim_h = self.world.resource_mut::<Assets<Mesh>>().add(rim);
        self.world.spawn((Name::new("PlanetTop"), PlanetGround, Mesh3d(top_h), MeshMaterial3d(ground_mat), Transform::default(), ChapterScoped));
        self.world.spawn((Name::new("PlanetRim"), PlanetGround, Mesh3d(rim_h), MeshMaterial3d(shared.vertex.clone()), Transform::default(), ChapterScoped));
        self.world.insert_resource(terrain);
        self.world.resource::<Terrain>()
    }

    pub fn terrain(&self) -> Option<&Terrain> {
        self.world.get_resource::<Terrain>()
    }

    /// Ground point at (x, z) (0 when no planet).
    pub fn ground(&self, x: f32, z: f32) -> Vec3 {
        self.terrain().map(|t| t.ground(x, z)).unwrap_or(Vec3::new(x, 0.0, z))
    }

    /// Ground point at polar coordinates (angle like a yaw: 0 = +Z, 90 = +X).
    pub fn polar(&self, angle_deg: f32, r: f32) -> Vec3 {
        let a = angle_deg.to_radians();
        let c = self.terrain().map(|t| t.params.center).unwrap_or(Vec2::ZERO);
        self.ground(c.x + a.sin() * r, c.y + a.cos() * r)
    }

    /// Sky set-up: starting hour, sun-hours per minute, free-play cap, palette.
    pub fn sky(&mut self, hour: f64, hours_per_minute: f32, cap_hour: Option<f64>, palette: SkyPalette) {
        let mut sky = self.world.resource_mut::<Sky>();
        *sky = Sky::default();
        sky.palette = palette;
        sky.set_hour(hour);
        sky.set_rate(hours_per_minute, cap_hour);
    }

    pub fn sky_mut(&mut self) -> Mut<'_, Sky> {
        self.world.resource_mut::<Sky>()
    }

    /// Spawns the player at a ground position facing `yaw_deg`; the camera snaps behind.
    pub fn player(&mut self, pos: Vec3, yaw_deg: f32) -> Entity {
        let e = super::player::spawn_player(self.world, pos, yaw_deg);
        self.world.resource_mut::<super::camera::CameraRig>().snap_behind(yaw_deg.to_radians());
        e
    }

    /// Spawns a model by manifest key on the ground at (x, z) with yaw (deg) and scale.
    pub fn model(&mut self, key: &str, pos: Vec3, yaw_deg: f32, scale: f32) -> Entity {
        self.world
            .spawn((
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw_deg.to_radians())).with_scale(Vec3::splat(scale)),
                Visibility::default(),
                ModelKey(key.to_string()),
                PendingModel(key.to_string()),
                ChapterScoped,
            ))
            .id()
    }

    /// An empty scoped entity (group / pivot).
    pub fn empty(&mut self, pos: Vec3, yaw_deg: f32) -> Entity {
        self.world.spawn((Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw_deg.to_radians())), Visibility::default(), ChapterScoped)).id()
    }

    /// Spawns any bundle, scoped to the chapter.
    pub fn spawn(&mut self, bundle: impl Bundle) -> Entity {
        self.world.spawn((bundle, ChapterScoped)).id()
    }

    pub fn collider(&mut self, e: Entity, c: Collider) -> &mut Self {
        self.world.entity_mut(e).insert(c);
        self
    }

    pub fn interactable(&mut self, e: Entity, it: Interactable) -> &mut Self {
        self.world.entity_mut(e).insert(it);
        self
    }

    /// Registers a named anchor (position + yaw) for scripts (`Pos::anchor("name")`).
    pub fn anchor(&mut self, name: &str, pos: Vec3, yaw_deg: f32) -> &mut Self {
        self.world.resource_mut::<Anchors>().0.insert(name.to_string(), (pos, yaw_deg));
        self
    }

    /// A point light (lamps, windows).
    pub fn point_light(&mut self, pos: Vec3, color: Color, intensity: f32, range: f32) -> Entity {
        self.spawn((PointLight { color, intensity, range, shadow_maps_enabled: false, ..default() }, Transform::from_translation(pos)))
    }

    /// Scatters `count` copies of a model in a ring [r0, r1] avoiding `avoid` circles.
    pub fn scatter(&mut self, key: &str, count: u32, r0: f32, r1: f32, seed: u64, scale: (f32, f32), avoid: &[(Vec2, f32)]) {
        let mut rng = super::meshkit::Rng::new(seed);
        let mut placed = 0;
        let mut tries = 0;
        while placed < count && tries < count * 20 {
            tries += 1;
            let a = rng.range(0.0, 360.0);
            // uniform over the annulus area
            let r = (r0 * r0 + rng.f() * (r1 * r1 - r0 * r0)).sqrt();
            let p = self.polar(a, r);
            let p2 = Vec2::new(p.x, p.z);
            if avoid.iter().any(|(c, rad)| p2.distance(*c) < *rad) {
                continue;
            }
            let s = rng.range(scale.0, scale.1);
            self.model(key, p, rng.range(0.0, 360.0), s);
            placed += 1;
        }
    }

    /// Default chapter music / ambience.
    pub fn music(&mut self, key: &str) {
        self.world.write_message(super::audio::AudioCmd::Music(Some(key.to_string())));
    }
    pub fn ambience(&mut self, key: &str) {
        self.world.write_message(super::audio::AudioCmd::Ambience(Some(key.to_string())));
    }

    /// Chapter-persistent value (saved): `<chapter>.<name>`.
    pub fn saved(&self, name: &str) -> Option<String> {
        self.world.resource::<super::save::SaveData>().chapter_data.get(&format!("{}.{}", self.id, name)).cloned()
    }
}

/// Is this chapter active? (for code holding `&World`).
pub fn active_chapter(world: &World) -> Option<String> {
    world.get_resource::<ActiveChapter>().and_then(|a| a.id.clone())
}

pub fn default_map_colors() -> (Color, Color) {
    (hex(0x9fb36c), hex(0xf0c23a))
}
