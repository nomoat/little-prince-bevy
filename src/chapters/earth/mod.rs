//! Chapter 8 — The Earth (original scenes Desert_Valley → Desert_Dunes).
//! Follows docs/original/earth.md: the valley (snake on its rock, the little
//! flower and the echo, the rose garden, the fox under the apple tree, the
//! station) and — after the railway page — the dunes (the sleeping aviator
//! and the sheep drawings, "eight days later", the well, the sacred way and
//! the doorway snake, the farewell, the walk toward the star and the end card).

mod actors;
mod fx;
mod models;
mod overlay;
mod script;
mod state;
pub mod strings;
mod world;

use crate::core::camera::CameraTuning;
use crate::core::prelude::*;
use crate::core::state::AppState;
use state::{EarthState, HisStar, Scene};

pub use strings::FOX;

pub struct Earth;

const ID: &str = "earth";

impl Chapter for Earth {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 8,
            title: "earth.title",
            subtitle: "earth.subtitle",
            line: "earth.line",
            visited_line: "earth.visited",
            loading: "earth.loading",
            map_pos: Vec2::new(1530.0, 250.0),
            map_radius: 78.0,
            map_colors: (hex(0xe2c08d), hex(0x86aad3)),
            music: "music.earth",
        }
    }

    fn strings(&self) -> StringTable {
        strings::STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        models::register(app);
        app.init_resource::<EarthState>()
            .init_resource::<fx::PrinceLook>()
            .add_systems(
                Update,
                (
                    use_flags,
                    actors::snake_update,
                    actors::fox_update,
                    actors::pilot_update,
                    fx::guide_signs,
                    fx::footprints,
                    fx::stardust_trail,
                    fx::dusk_ramp,
                    fx::train_arrival,
                    fx::ending_walk,
                    overlay::update,
                )
                    .in_set(CoreSet::Gameplay)
                    .run_if(in_chapter(ID)),
            )
            .add_systems(
                Update,
                (fx::prince_look.after(crate::core::player::animate_prince), fx::star_billboard).in_set(CoreSet::Presentation).run_if(in_chapter(ID)),
            )
            .add_systems(OnExit(AppState::Chapter), restore_camera);
    }

    fn build(&self, cx: &mut ChapterCx) {
        cx.world.insert_resource(EarthState::default());
        cx.world.insert_resource(fx::PrinceLook::default());
        // Valley camera (original): distance 3.6 (1.8–5.5), pitch 10°, FOV 50.
        let saved = cx.world.resource::<CameraTuning>().clone();
        let mut t = saved.clone();
        t.default_distance = 3.6;
        t.min_distance = 1.8;
        t.max_distance = 5.5;
        t.default_pitch = 10.0;
        t.fov = 50.0;
        cx.world.insert_resource(t.clone());
        cx.world.resource_mut::<CameraRig>().reset(&t);
        cx.world.resource_mut::<EarthState>().saved_camera = Some(saved);

        let dunes = cx.world.resource::<crate::core::params::LaunchParams>().get("earth") == Some("dunes");
        if dunes {
            world::build_dunes(cx);
            let p = cx.ground(world::D_STEPOFF.x, world::D_STEPOFF.y);
            cx.player(p, world::yaw_to(world::D_STEPOFF, world::D_PLANE));
        } else {
            world::build_valley(cx);
            let star = cx.model("model.earth.star", Vec3::new(0.0, -50.0, 0.0), 0.0, 0.001);
            cx.world.entity_mut(star).insert(HisStar);
            world::spawn_valley_player(cx);
            world::mark_valley(cx.world);
            cx.world.resource_mut::<EarthState>().scene = Scene::Valley;
        }
        overlay::spawn(cx.world);
    }

    fn story(&self) -> Story {
        script::story()
    }
}

/// Valley interactables set `u_<id>` flags (the script dispatches optional talks).
fn use_flags(mut msgs: MessageReader<Interacted>, mut runner: ResMut<StoryRunner>) {
    for m in msgs.read() {
        if matches!(m.id.as_str(), "flower" | "echo" | "roses" | "fox" | "train") {
            runner.set_flag(&format!("u_{}", m.id));
        }
    }
}

fn restore_camera(st: Option<ResMut<EarthState>>, mut tuning: ResMut<CameraTuning>, mut look: Option<ResMut<fx::PrinceLook>>) {
    if let Some(mut st) = st {
        if let Some(t) = st.saved_camera.take() {
            *tuning = t;
        }
    }
    if let Some(l) = look.as_deref_mut() {
        *l = fx::PrinceLook::default();
    }
}
