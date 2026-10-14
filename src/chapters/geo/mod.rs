//! Chapter 7 — The Geographer ("MapLibrary" in the original).
//!
//! Follows the original GeographyStory / GeographerConversation design
//! (docs/original/geo.md), in the original's world coordinates:
//! - a Mediterranean map library (hall + tower) on a small island above a sea
//!   of clouds (y = −55); golden late afternoon, sun at 30°; front garden with
//!   raised beds and cypresses; entrance steps up to the floor (y 0); front
//!   doors fixed open 100° inward; hall x ±4, z ≈ 0.6..12.2; back opening to a
//!   balcony whose far rail is at z = 15.3 (usable ±3.4 m); shelves of globes
//!   and books along both walls; the map table at (−1.95, 6.5) with the map,
//!   armillary sphere, books, inkwell and pencil stand; the geographer at
//!   (−2.17, 5.55) facing +z;
//! - camera rig: pivot 1.0, distance 3.6 (1.2–5.0), default pitch −8°
//!   (−25..60), FOV 55; inside the hall the pitch eases to 10° over 1.2 s
//!   when below 4° (GeoCameraPitchAssist);
//! - the talk: prompt within 2.0 m of the map table (facing it); the prince
//!   walks (0.9 m/s) to the spot across the table (−2.15, 7.6), round the
//!   table corner (−0.72, 7.6) when coming from the room side; ten lines with
//!   the geographer's clips (SpotExplorer, PleaseTell, PickWrite 0.5 s into
//!   line 3 with the pencil taken up, PencilStops + ≥ 0.9 s pen-stop beat,
//!   TalkCalm, BackToBook 1.0 s into line 10), the 1.2 s stunned beat with the
//!   prince's head lowered 12°, the shot sequence (two-shot pushing in, write,
//!   stop, calm, the side glide, over-the-shoulder, wide shot held 1.8 s), a
//!   warm fill light on the prince's face, and the return view (wide heading
//!   −50°, pitch 10°, 2.4 s);
//! - side activity "Gaze out" at the balcony rail (1.7 m, |x| ≤ 3.4): an
//!   over-the-shoulder look-out with free look (yaw −60..45, pitch −12..30,
//!   FOV 50, enter 1.4 s, exit 1.2 s).

use super::vanity::kit::*;
use crate::core::camera::{CameraRig, CameraTuning};
use crate::core::prelude::*;
use crate::core::state::AppState;

pub struct Geo;

const ID: &str = "geo";

const GEO_POS: Vec3 = Vec3::new(-2.17, 0.0, 5.55);
const TABLE: Vec3 = Vec3::new(-1.95, 0.0, 6.5);
const TABLE_Y: f32 = 0.8;
const SPOT: Vec3 = Vec3::new(-2.15, 0.0, 7.6);
const CORNER: Vec3 = Vec3::new(-0.72, 0.0, 7.6);
const RAIL_Z: f32 = 15.3;
/// Pencil stand (world) → geographer-local.
const STAND_L: Vec3 = Vec3::new(0.4, 0.81, 0.48);
const BOOK_L: Vec3 = Vec3::new(0.02, 0.83, 0.55);

pub const STRINGS: StringTable = &[
    ("geo.title", "地理学家的星球", "The Geographer's Planet"),
    ("geo.subtitle", "一位记录山川大海、却从没亲眼见过它们的地理学家", "A geographer who records mountains and seas he has never seen"),
    ("geo.line", "他从不出门，却记下了远方的一切。", "He never goes out, yet he records every faraway place."),
    ("geo.visited", "小王子第一次为留在家里的花担心。他听了地理学家的话，去了地球。", "For the first time the prince worried about the flower left at home. On the geographer's advice, he went on to Earth."),
    ("geo.loading", "书页翻动的声音……", "The rustle of turning pages…"),
    ("geo.n1", "第七颗星球大得多。一位老先生住在这里，写着一本又一本厚厚的书。", "The seventh planet was much bigger. An old gentleman lived here, writing one enormous book after another."),
    ("geo.ev.meet", "在地图馆里看看", "Look around the map library"),
    ("geo.st.meet", "去地图桌那边，和老先生说话", "Go to the map table and speak to the old gentleman"),
    ("geo.target.table", "地图桌", "The map table"),
    ("geo.verb.talk", "说话", "Talk"),
    ("geo.verb.gaze", "眺望", "Gaze out"),
    ("geo.verb.leave", "离开", "Leave"),
    // the conversation (our own wording)
    ("geo.l1", "咦！来了一位探险家！", "Well, well! Here comes an explorer!"),
    ("geo.l2", "你是从很远的地方来的吧！快，给我讲讲你的星球。", "You have come from far away! Quick — tell me about your planet."),
    ("geo.l3", "我的星球很小。有三座火山，两座还在冒烟，一座已经熄了。", "My planet is very small. There are three volcanoes — two still smoking, one gone quiet."),
    ("geo.l4", "我还有一朵花。", "I have a flower, too."),
    ("geo.l5", "花，我们是不记的。", "We do not record flowers."),
    ("geo.l6", "为什么？花是最美的呀！", "Why not? Flowers are the loveliest of all!"),
    ("geo.l7", "因为花是「短暂的」。", "Because flowers are \"ephemeral\"."),
    ("geo.l8", "「短暂的」是什么意思？", "What does \"ephemeral\" mean?"),
    ("geo.l9", "意思是：随时可能消失。", "It means: in danger of disappearing at any moment."),
    ("geo.l10", "我的花是短暂的……她只有四根刺保护自己，可我却把她独自留在了家里。", "My flower is ephemeral… she has only four thorns to defend herself, and I left her all alone at home."),
];

impl Chapter for Geo {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 7,
            title: "geo.title",
            subtitle: "geo.subtitle",
            line: "geo.line",
            visited_line: "geo.visited",
            loading: "geo.loading",
            map_pos: Vec2::new(1390.0, 560.0),
            map_radius: 64.0,
            map_colors: (hex(0x9fb08a), hex(0xe8dcb4)),
            music: "music.geo",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("geo.library", library_model)
            .register_model("geo.door", door_leaf)
            .register_model("geo.tower", tower)
            .register_model("geo.table", table_model)
            .register_model("geo.geographer", geographer)
            .register_model("geo.cypress", cypress)
            .register_model("geo.bed", garden_bed)
            .register_model("geo.clouds", cloud_sea)
            .register_model("geo.beam", sunbeam)
            .init_resource::<GState>();
        super::vanity::kit::register(app, ID);
        app.add_systems(Update, (talk_trigger, pitch_assist, gaze, animate_geographer, fill_light, drift_clouds).chain().in_set(CoreSet::Gameplay).run_if(in_chapter(ID)));
        app.add_systems(OnExit(AppState::Chapter), restore_camera);
    }

    fn build(&self, cx: &mut ChapterCx) {
        cx.world.insert_resource(GState::default());
        cx.world.insert_resource(PrinceOverride::default());
        cx.world.resource_mut::<crate::core::player::PlayerTuning>().walk_speed = 0.9;
        // GeoCamera rig (restored on exit)
        {
            let mut t = cx.world.resource_mut::<CameraTuning>();
            t.pivot_height = 1.0;
            t.default_distance = 3.6;
            t.min_distance = 1.2;
            t.max_distance = 5.0;
            t.default_pitch = -8.0;
            t.min_pitch = -25.0;
            t.max_pitch = 60.0;
            t.fov = 55.0;
        }
        let tuning = cx.world.resource::<CameraTuning>().clone();
        cx.world.resource_mut::<CameraRig>().reset(&tuning);
        cx.world.resource_mut::<GState>().tuned = true;

        // island: garden at y −0.7, the library on its plinth (floor y 0)
        let mut p = PlanetParams::sized(15.5);
        p.center = Vec2::new(0.0, 1.0);
        p.top_y = -0.7;
        p.cap_flatten = 0.6;
        p.soft_radius = 15.2;
        p.hard_radius = 16.2;
        p.failsafe_radius = 17.0;
        p.lip_start = 16.2;
        p.hole_radius = 19.5;
        p.underside_depth = 14.0;
        p.ground = hex(0x9aae6a);
        p.ground2 = hex(0x86a05c);
        p.dry = hex(0xd8c8a0);
        p.rock = hex(0xb09a7e);
        p.mods = vec![
            TerrainMod::Flatten { center: Vec2::new(0.0, 6.5), radius: 10.5, blend: 3.0, height: Some(-0.7) },
            TerrainMod::Flatten { center: Vec2::new(0.0, -8.0), radius: 6.0, blend: 3.0, height: Some(-0.7) },
            TerrainMod::Paint { center: Vec2::new(0.0, -2.5), radius: 1.4, blend: 0.6, color: hex(0xd8c8a0) },
            TerrainMod::Paint { center: Vec2::new(0.0, -5.0), radius: 1.3, blend: 0.6, color: hex(0xd8c8a0) },
            TerrainMod::Paint { center: Vec2::new(0.0, -7.5), radius: 1.3, blend: 0.6, color: hex(0xd8c8a0) },
            TerrainMod::Paint { center: Vec2::new(0.0, -10.0), radius: 1.3, blend: 0.6, color: hex(0xd8c8a0) },
        ];
        cx.planet(p);
        // footsteps: stone floor in the hall and on the balcony
        cx.step_zone(Vec3::new(0.0, 1.5, 8.5), Vec3::new(3.5, 3.0, 7.2), "sfx.step_stone");
        let pal = SkyPalette {
            day_zenith: hex(0x5a8fd0),
            day_horizon: hex(0xf2dcb8),
            golden_horizon: hex(0xf7c88e),
            fog: 0.25,
            ..default()
        };
        cx.sky(16.0, 0.2, None, pal);
        {
            let mut sky = cx.sky_mut();
            let h = sky.hour_for_elevation(30.0, true);
            sky.set_hour(h);
            sky.set_rate(0.2, Some(h + 0.6));
        }
        cx.ambience("amb.wind");

        // the library (layout → mesh + colliders) and the tower
        cx.model("model.geo.library", Vec3::ZERO, 0.0, 1.0);
        spawn_colliders(cx, Vec3::ZERO, &library_layout());
        cx.spawn((Transform::from_xyz(0.0, 5.0, 6.3), Collider::cuboid(Vec3::new(8.8, 0.4, 12.4))));
        let tw = cx.model("model.geo.tower", Vec3::new(7.0, -0.7, 11.6), 0.0, 1.0);
        cx.collider(tw, Collider::cylinder(1.9, 9.5));
        // front doors, fixed open 100° inward
        for (x, sign) in [(-1.3f32, -1.0f32), (1.3, 1.0)] {
            let h = cx.empty(Vec3::new(x, 0.0, 0.36), sign * 100.0);
            let leaf = cx.model("model.geo.door", Vec3::new(-sign * 0.65, 0.0, 0.0), 0.0, 1.0);
            cx.world.entity_mut(h).add_child(leaf);
            let col = cx.spawn((Transform::from_xyz(-sign * 0.65, 0.0, 0.0), Collider::cuboid(Vec3::new(1.3, 2.8, 0.08)).no_camera()));
            cx.world.entity_mut(h).add_child(col);
        }

        // the map table and the geographer
        let tb = cx.model("model.geo.table", TABLE, 0.0, 1.0);
        cx.collider(tb, Collider::cuboid(Vec3::new(1.6, TABLE_Y, 1.1)).not_walkable().no_camera());
        cx.interactable(tb, Interactable::new("geo_talk", "geo.verb.talk").radius(2.0).focus(Vec3::Y * 0.9).prompt_height(0.4).priority(1).max_angle(100.0));
        cx.anchor("table", TABLE + Vec3::Y * 0.9, 0.0);
        let g = cx.model("model.geo.geographer", GEO_POS, 0.0, 1.0);
        cx.world.entity_mut(g).insert((Rig::default(), Geographer::default()));
        cx.collider(g, Collider::cylinder(0.38, 1.3).no_camera());
        cx.anchor("geo_head", GEO_POS + Vec3::new(0.0, 1.2, 0.05), 0.0);

        // the gaze-out spot slides along the far rail
        let gz = cx.empty(Vec3::new(0.0, 0.0, RAIL_Z - 0.3), 0.0);
        cx.interactable(gz, Interactable::new("gaze", "geo.verb.gaze").radius(1.7).focus(Vec3::Y * 0.9).prompt_height(0.5).priority(-1));
        cx.world.entity_mut(gz).insert(GazeSpot);

        // light: lanterns, candles (flicker), the fill light for the talk
        lamp(cx, Vec3::new(0.0, 3.4, 3.0), hex(0xffd29a), 60_000.0, 9.0, 0.06, 0.2);
        lamp(cx, Vec3::new(-2.0, 2.0, 6.4), hex(0xffc27a), 30_000.0, 5.0, 0.15, 1.7);
        lamp(cx, Vec3::new(1.5, 3.4, 9.6), hex(0xffd29a), 50_000.0, 9.0, 0.06, 3.3);
        lamp(cx, Vec3::new(0.0, 2.9, -0.6), hex(0xffc98a), 20_000.0, 6.0, 0.08, 4.5);
        let fl = cx.point_light(SPOT + Vec3::Y * 1.3, hex(0xffc690), 0.0, 2.5);
        cx.world.entity_mut(fl).insert(FillLight);
        cx.model("model.geo.beam", Vec3::new(0.0, 5.0, 6.2), 0.0, 1.0);

        // garden: raised beds, cypresses, olives, shrubs; the sea of clouds below
        for (x, z) in [(-3.2f32, -12.5f32), (3.2, -12.5), (-3.2, -9.6), (3.2, -9.6)] {
            let b = cx.model("model.geo.bed", Vec3::new(x, -0.7, z), 0.0, 1.0);
            cx.collider(b, Collider::cuboid(Vec3::new(2.4, 0.45, 1.8)).no_camera());
        }
        for (x, z, s) in [(-7.0f32, -3.0f32, 1.0f32), (7.0, -3.0, 1.1), (-7.0, 2.5, 0.95), (7.0, 3.0, 1.0), (-6.6, 9.0, 1.05), (-5.4, -9.5, 0.9), (5.6, -9.0, 0.9)] {
            let at = cx.ground(x, z);
            let c = cx.model("model.geo.cypress", at, x * 20.0, s);
            cx.collider(c, Collider::cylinder(0.45, 4.0).no_camera());
        }
        for (x, z) in [(-9.0f32, -6.0f32), (9.5, -6.5), (10.0, 6.0)] {
            let at = cx.ground(x, z);
            let t = cx.model("model.tree_oak", at, x * 13.0, 1.0);
            cx.collider(t, Collider::cylinder(0.35, 3.0).no_camera());
        }
        let avoid = [(Vec2::new(0.0, 6.3), 9.0), (Vec2::new(0.0, -6.0), 1.5), (Vec2::new(0.0, -11.0), 4.6), (Vec2::new(7.0, 11.6), 2.6)];
        cx.scatter("model.bush", 14, 7.0, 15.0, 71, (0.7, 1.2), &avoid);
        cx.scatter("model.grass_tuft", 110, 2.0, 15.0, 72, (0.8, 1.3), &avoid);
        cx.scatter("model.flower_yellow", 16, 3.0, 15.0, 73, (0.7, 1.1), &avoid);
        cx.scatter("model.flower_red", 12, 3.0, 15.0, 74, (0.7, 1.1), &avoid);
        cx.scatter("model.rock", 6, 12.0, 15.5, 75, (0.6, 1.2), &avoid);
        let c = cx.model("model.geo.clouds", Vec3::new(0.0, -55.0, 0.0), 0.0, 1.0);
        cx.world.entity_mut(c).insert(CloudSea);

        cx.player(Vec3::new(0.0, -0.7, -8.0), 0.0);
    }

    fn story(&self) -> Story {
        let cam0 = Vec3::new(27.0, 13.5, -43.0);
        let look0 = Vec3::new(15.68, 10.0, -33.8);
        let push = (look0 - cam0).normalize() * 1.5;
        let wide_cam = Vec3::new(2.6, 3.4, 2.4);
        let wide_look = Vec3::new(-1.9, 0.7, 7.0);
        let wide_heading = yaw_to(wide_cam, wide_look);
        Story {
            events: vec![event("geo.ev.meet", &["geo.st.meet"])],
            hidden: None,
            script: vec![
                // landing shot (world), 1.5 m push, 4 s descent
                push_cmd((cam0, look0, 45.0), (cam0 + push, look0 + push - Vec3::Y * 1.5, 45.0), 4.0),
                card(),
                run(|w: &mut World| end_shot_to(w, Some(0.0), Some(-8.0), 1.8)),
                narrate(&["geo.n1"]),
                start_event(0),
                target(Pos::anchor("table"), "geo.target.table"),
                wait_use("geo_talk"),
                // ---- the talk
                clear_target(),
                run(|w: &mut World| {
                    w.resource_mut::<GState>().talking = true;
                    // from the room side: round the table's east end through the corner
                    if let Some(p) = player_pos(w) {
                        if p.z < 7.35 {
                            let a = Vec3::new(CORNER.x + 0.07, 0.0, p.z.clamp(4.6, 7.6));
                            w.resource_mut::<crate::core::story::Anchors>().0.insert("geo_a".into(), (a, 0.0));
                            crate::core::story::set_flag(w, "geo_via_corner");
                        }
                    }
                }),
                if_not_flag("geo_via_corner", "direct"),
                walk_to(Pos::anchor("geo_a")),
                walk_to(Pos::World(CORNER)),
                label("direct"),
                walk_to(Pos::World(SPOT)),
                run(|w: &mut World| {
                    set_drive(w, Drive::External);
                    face_yaw(w, 180.0);
                    w.resource_mut::<GState>().fill = true;
                }),
                // L1: spots the explorer — two-shot, pushing in
                run(|w: &mut World| geo_clip(w, Clip::SpotExplorer)),
                shot_cmd(Vec3::new(-1.05, 1.45, 9.5), Vec3::new(-2.16, 1.05, 6.4), 0.8, 45.0),
                wait(0.8),
                shot_cmd(Vec3::new(-1.15, 1.42, 9.1), Vec3::new(-2.16, 1.05, 6.4), 14.0, 43.0),
                say(speakers::GEOGRAPHER, "geo.l1"),
                run(|w: &mut World| geo_clip(w, Clip::PleaseTell)),
                say(speakers::GEOGRAPHER, "geo.l2"),
                // L3: PickWrite 0.5 s into the line — write shot of the book and hand
                run(|w: &mut World| geo_clip_in(w, Clip::PickWrite, 0.5)),
                shot_cmd(Vec3::new(-1.35, 1.5, 7.05), Vec3::new(-2.05, 0.85, 6.1), 1.2, 40.0),
                say(speakers::PRINCE, "geo.l3"),
                // L4: the pencil stops; ≥ 0.9 s pen-stop beat; up to his face
                run(|w: &mut World| geo_clip(w, Clip::PencilStops)),
                shot_cmd(Vec3::new(-1.75, 1.3, 7.15), Vec3::new(-2.17, 1.25, 5.6), 1.4, 38.0),
                say(speakers::PRINCE, "geo.l4"),
                wait(0.9),
                // L5–L9: calm talk from the prince's side
                run(|w: &mut World| geo_clip(w, Clip::TalkCalm)),
                shot_cmd(Vec3::new(-2.75, 1.4, 8.45), Vec3::new(-2.17, 1.1, 5.6), 1.2, 44.0),
                say(speakers::GEOGRAPHER, "geo.l5"),
                say(speakers::PRINCE, "geo.l6"),
                say(speakers::GEOGRAPHER, "geo.l7"),
                say(speakers::PRINCE, "geo.l8"),
                say(speakers::GEOGRAPHER, "geo.l9"),
                // stunned beat 1.2 s: head lowers 12°; the side shot glides round to his face
                run(|w: &mut World| {
                    w.resource_mut::<PrinceOverride>().head = Some(Quat::from_rotation_x(12f32.to_radians()));
                    shot_fov(w, Vec3::new(-0.75, 1.3, 7.3), Vec3::new(-2.15, 1.05, 7.6), 1.6, 42.0);
                }),
                wait(1.2),
                // over the geographer's shoulder (balcony bright behind the prince)
                shot_cmd(Vec3::new(-2.55, 1.6, 5.1), Vec3::new(-2.15, 1.1, 7.6), 1.0, 40.0),
                run(|w: &mut World| geo_clip_in(w, Clip::BackToBook, 1.0)),
                say(speakers::PRINCE, "geo.l10"),
                // wide shot holds 1.8 s, then the banner (complete delay 1 s)
                shot_cmd(wide_cam, wide_look, 1.0, 42.0),
                wait(1.8),
                wait(1.0),
                complete_event(0),
                // return view: wide heading −50°, pitch 10°, over 2.4 s
                run(move |w: &mut World| {
                    w.resource_mut::<PrinceOverride>().head = None;
                    let mut s = w.resource_mut::<GState>();
                    s.fill = false;
                    s.talking = false;
                    set_drive(w, Drive::Player);
                    end_shot_to(w, Some(wide_heading - 50.0), Some(10.0), 2.4);
                }),
                departure(),
            ],
        }
    }
}

// --------------------------------------------------------------- state ----

#[derive(Resource, Default)]
struct GState {
    tuned: bool,
    talking: bool,
    fill: bool,
    fill_k: f32,
    /// Gaze-out: active, look yaw/pitch (deg, relative), seconds idle.
    gazing: bool,
    gaze_yaw: f32,
    gaze_pitch: f32,
    gaze_idle: f32,
    leaving: f32,
}

#[derive(Component)]
struct GazeSpot;
#[derive(Component)]
struct FillLight;
#[derive(Component)]
struct CloudSea;

fn restore_camera(st: Option<ResMut<GState>>, mut t: ResMut<CameraTuning>) {
    if let Some(mut st) = st {
        if st.tuned {
            *t = CameraTuning::default();
            st.tuned = false;
        }
    }
}

fn talk_trigger(st: Res<GState>, mut its: Query<&mut Interactable>, runner: Res<StoryRunner>) {
    for mut it in &mut its {
        if it.id == "geo_talk" && (st.talking || runner.done.first().copied().unwrap_or(false)) && it.enabled {
            it.enabled = false;
        }
        if it.id == "gaze" {
            it.enabled = !st.gazing && !st.talking && st.leaving <= 0.0;
        }
    }
}

/// GeoCameraPitchAssist: in the hall, ease the pitch to 10° over 1.2 s when below 4°.
fn pitch_assist(time: Res<Time>, mut rig: ResMut<CameraRig>, player: Query<&Transform, With<Player>>, st: Res<GState>, mut easing: Local<bool>) {
    let Ok(p) = player.single() else { return };
    let q = p.translation;
    let in_hall = q.x.abs() < 4.0 && q.z > 0.8 && q.z < 11.8 && q.y > -0.2;
    if !in_hall || st.gazing || rig.shot.is_some() {
        *easing = false;
        return;
    }
    if rig.pitch < 4f32.to_radians() {
        *easing = true;
    }
    if *easing {
        let goal = 10f32.to_radians();
        let step = (10.0 + 8.0) / 1.2 * time.delta_secs();
        rig.pitch = (rig.pitch + step.to_radians()).min(goal);
        if rig.pitch >= goal - 1e-3 {
            *easing = false;
        }
    }
}

/// "Gaze out" at the balcony rail.
#[allow(clippy::too_many_arguments)]
fn gaze(
    time: Res<Time>,
    input: Res<InputFrame>,
    mut st: ResMut<GState>,
    mut msgs: MessageReader<Interacted>,
    mut rig: ResMut<CameraRig>,
    mut hud: ResMut<Hud>,
    mut player: Query<(&mut Player, &Transform), Without<GazeSpot>>,
    mut spot: Query<&mut Transform, With<GazeSpot>>,
) {
    let dt = time.delta_secs();
    let Ok((mut pl, ptf)) = player.single_mut() else { return };
    let pp = ptf.translation;
    if let Ok(mut s) = spot.single_mut() {
        s.translation.x = pp.x.clamp(-3.4, 3.4);
    }
    st.leaving = (st.leaving - dt).max(0.0);
    for m in msgs.read() {
        if m.id == "gaze" && !st.gazing {
            st.gazing = true;
            st.gaze_yaw = -14.0;
            st.gaze_pitch = 9.0;
            st.gaze_idle = 0.0;
            pl.drive = Drive::External;
            pl.face_target = Some(0.0);
            let (pos, look) = gaze_view(pp, st.gaze_yaw, st.gaze_pitch);
            rig.begin_shot(Shot { position: pos, look_at: look, blend: 1.4, fov: Some(50.0) });
            hud.teach(vec!["E"], "geo.verb.leave", 4.0);
            return;
        }
    }
    if !st.gazing {
        return;
    }
    st.gaze_yaw = (st.gaze_yaw - input.look.x).clamp(-60.0, 45.0);
    st.gaze_pitch = (st.gaze_pitch - input.look.y).clamp(-12.0, 30.0);
    if input.look.length() > 0.01 {
        st.gaze_idle = 0.0;
    } else {
        st.gaze_idle += dt;
    }
    let (pos, look) = gaze_view(pp, st.gaze_yaw, st.gaze_pitch);
    if let Some(s) = rig.shot.as_mut() {
        s.position = pos;
        s.look_at = look;
    }
    if input.interact || input.jump {
        st.gazing = false;
        st.leaving = 0.6;
        pl.drive = Drive::Player;
        rig.yaw = st.gaze_yaw.to_radians();
        rig.end_shot(1.2);
    }
}

/// Over-the-shoulder look-out camera: right 0.75, up 1.75, back 1.7.
fn gaze_view(p: Vec3, yaw: f32, pitch: f32) -> (Vec3, Vec3) {
    let pos = p + Vec3::new(-0.75, 1.75, -1.7);
    let (y, pi) = (yaw.to_radians(), pitch.to_radians());
    let dir = Vec3::new(y.sin() * pi.cos(), -pi.sin(), y.cos() * pi.cos());
    (pos, pos + dir * 10.0)
}

fn fill_light(time: Res<Time>, mut st: ResMut<GState>, rig: Res<CameraRig>, player: Query<&Transform, (With<Player>, Without<FillLight>)>, mut q: Query<(&mut PointLight, &mut Transform), With<FillLight>>) {
    let goal = if st.fill { 1.0 } else { 0.0 };
    st.fill_k += (goal - st.fill_k).clamp(-time.delta_secs(), time.delta_secs());
    let Ok(p) = player.single() else { return };
    for (mut l, mut tf) in &mut q {
        let face = p.translation + Vec3::Y * 1.0;
        let to_cam = (rig.last_pos - face).normalize_or_zero();
        tf.translation = face + to_cam * 0.9 + Vec3::Y * 0.25;
        l.intensity = 9_000.0 * st.fill_k;
    }
}

fn drift_clouds(time: Res<Time>, mut q: Query<&mut Transform, With<CloudSea>>) {
    for mut t in &mut q {
        t.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.004);
    }
}

// --------------------------------------------------------- geographer ----

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Clip {
    #[default]
    ReadIdle,
    SpotExplorer,
    PleaseTell,
    PickWrite,
    PencilStops,
    TalkCalm,
    BackToBook,
}

impl Clip {
    /// Clip lengths at 30 fps (GEO_*): 90, 90, 146, 75, loop, 151 frames.
    fn duration(self) -> f32 {
        match self {
            Clip::ReadIdle | Clip::TalkCalm => f32::MAX,
            Clip::SpotExplorer | Clip::PleaseTell => 3.0,
            Clip::PickWrite => 146.0 / 30.0,
            Clip::PencilStops => 75.0 / 30.0,
            Clip::BackToBook => 151.0 / 30.0,
        }
    }
    /// What follows a one-shot clip.
    fn next(self) -> Clip {
        match self {
            Clip::PickWrite => Clip::PickWrite,
            Clip::PencilStops => Clip::PencilStops,
            Clip::BackToBook => Clip::ReadIdle,
            _ => Clip::TalkCalm,
        }
    }
}

#[derive(Component, Default)]
struct Geographer {
    clip: Clip,
    t: f32,
    clock: f32,
    /// A clip queued to start after a delay (e.g. 0.5 s into a line).
    queued: Option<(Clip, f32)>,
    pencil_in_hand: bool,
    cur: Option<GPose>,
    scratch: f32,
}

fn geo_clip(w: &mut World, c: Clip) {
    geo_clip_in(w, c, 0.0);
}

fn geo_clip_in(w: &mut World, c: Clip, delay: f32) {
    let mut q = w.query::<&mut Geographer>();
    if let Some(mut g) = q.iter_mut(w).next() {
        g.queued = Some((c, delay));
    }
}

#[derive(Clone, Copy, Debug)]
struct GPose {
    torso: Quat,
    head: Quat,
    /// Hand targets (geographer-local); arms reach toward them.
    hand_l: Vec3,
    hand_r: Vec3,
}

impl GPose {
    fn lerp(&self, o: &GPose, k: f32) -> GPose {
        GPose { torso: self.torso.slerp(o.torso, k), head: self.head.slerp(o.head, k), hand_l: self.hand_l.lerp(o.hand_l, k), hand_r: self.hand_r.lerp(o.hand_r, k) }
    }
}

const G_HIP: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const G_NECK: Vec3 = Vec3::new(0.0, 1.07, 0.0);
const G_SH_L: Vec3 = Vec3::new(0.21, 1.0, 0.0);
const G_SH_R: Vec3 = Vec3::new(-0.21, 1.0, 0.0);
const G_ARM: f32 = 0.56;

fn read_pose(c: f32) -> GPose {
    // reading the huge book: head down, both hands on the pages; a page turn now and then
    let turn = ((c % 10.0) - 8.6).max(0.0) / 1.4;
    let tk = (turn * std::f32::consts::PI).sin();
    GPose {
        torso: Quat::from_rotation_x(0.22 + (c * 1.2).sin() * 0.01),
        head: Quat::from_rotation_x(0.42),
        hand_l: BOOK_L + Vec3::new(0.2 - tk * 0.25, 0.02 + tk * 0.12, -0.05),
        hand_r: BOOK_L + Vec3::new(-0.22, 0.02, -0.06),
    }
}

fn clip_pose(c: Clip, t: f32, clock: f32, idle: GPose) -> GPose {
    let talk = GPose {
        torso: Quat::from_rotation_x(0.05),
        head: Quat::from_rotation_x(-0.05),
        hand_l: Vec3::new(0.24, 0.86, 0.36),
        hand_r: Vec3::new(-0.2, 0.86, 0.38),
    };
    match c {
        Clip::ReadIdle => idle,
        Clip::SpotExplorer => {
            // looks up from the book, sits back, hands open in surprise
            let k = smooth(0.0, 0.6, t);
            GPose {
                torso: idle.torso.slerp(Quat::from_rotation_x(-0.08), k),
                head: idle.head.slerp(Quat::from_rotation_x(-0.15), k),
                hand_l: idle.hand_l.lerp(Vec3::new(0.38, 1.05, 0.3), k * (1.0 - smooth(2.2, 3.0, t) * 0.6)),
                hand_r: idle.hand_r.lerp(Vec3::new(-0.38, 1.05, 0.3), k * (1.0 - smooth(2.2, 3.0, t) * 0.6)),
            }
        }
        Clip::PleaseTell => {
            // an open hand toward the prince
            let k = smooth(0.0, 0.5, t) * (1.0 - smooth(2.4, 3.0, t) * 0.7);
            GPose { hand_r: talk.hand_r.lerp(Vec3::new(-0.1, 1.0, 0.6), k), torso: Quat::from_rotation_x(0.12 * k), ..talk }
        }
        Clip::PickWrite => {
            // reach to the pencil stand (pencil in hand from f65), then write in the book
            let reach = smooth(0.4, 1.6, t) * (1.0 - smooth(2.1, 2.9, t));
            let write = smooth(2.6, 3.2, t);
            let wig = Vec3::new(((t - 3.2).max(0.0) * 9.0).sin() * 0.04, 0.0, ((t - 3.2).max(0.0) * 4.0).sin() * 0.03);
            let pen = BOOK_L + Vec3::new(0.1, 0.03, -0.02) + wig * write;
            GPose {
                torso: Quat::from_rotation_x(0.12 + 0.12 * write),
                head: Quat::from_rotation_x(0.1 + 0.35 * write),
                hand_l: talk.hand_l.lerp(STAND_L + Vec3::Y * 0.1, reach).lerp(pen, write),
                hand_r: talk.hand_r.lerp(BOOK_L + Vec3::new(-0.2, 0.02, -0.05), write),
            }
        }
        Clip::PencilStops => {
            // the pencil stops; he lifts his head
            let k = smooth(0.1, 1.0, t);
            let pen = BOOK_L + Vec3::new(0.1, 0.03 + 0.04 * k, -0.02);
            GPose { torso: Quat::from_rotation_x(0.24 - 0.14 * k), head: Quat::from_rotation_x(0.45 - 0.5 * k), hand_l: pen, hand_r: BOOK_L + Vec3::new(-0.2, 0.02, -0.05) }
        }
        Clip::TalkCalm => {
            // calm explaining: the pencil hand makes small gestures (4 s loop)
            let ph = clock * std::f32::consts::TAU / 4.0;
            let g = (ph.sin() * 0.5 + 0.5) * 0.6;
            GPose {
                torso: Quat::from_rotation_x(0.06),
                head: Quat::from_rotation_x(-0.04) * Quat::from_rotation_z(ph.sin() * 0.04),
                hand_l: Vec3::new(0.2, 0.9 + g * 0.12, 0.42 + g * 0.08),
                hand_r: BOOK_L + Vec3::new(-0.2, 0.02, -0.05),
            }
        }
        Clip::BackToBook => {
            // puts the pencil back (out of the hand at f83), head down to the book
            let put = smooth(0.6, 1.8, t) * (1.0 - smooth(2.6, 3.4, t));
            let down = smooth(2.4, 4.4, t);
            let base = GPose { torso: Quat::from_rotation_x(0.06), head: Quat::from_rotation_x(-0.04), hand_l: Vec3::new(0.2, 0.9, 0.42), hand_r: BOOK_L + Vec3::new(-0.2, 0.02, -0.05) };
            let mut p = base.lerp(&idle, down);
            p.hand_l = p.hand_l.lerp(STAND_L + Vec3::Y * 0.1, put);
            p
        }
    }
}

/// Rotation that points an arm (hanging along −Y) from `shoulder` at `target`.
fn reach(shoulder: Vec3, target: Vec3) -> Quat {
    let d = target - shoulder;
    let dir = if d.length() < 1e-3 { -Vec3::Y } else { d.normalize() };
    Quat::from_rotation_arc(-Vec3::Y, dir)
}

#[allow(clippy::too_many_arguments)]
fn animate_geographer(
    time: Res<Time>,
    st: Res<GState>,
    mut q: Query<(&mut Geographer, &Rig, &GlobalTransform)>,
    player: Query<&Transform, (With<Player>, Without<Rig>)>,
    mut tfs: PartTfs,
    mut audio: MessageWriter<AudioCmd>,
) {
    let dt = time.delta_secs();
    let pp = player.single().map(|t| t.translation).ok();
    for (mut g, rig, gt) in &mut q {
        g.t += dt;
        g.clock += dt;
        if let Some((c, d)) = g.queued {
            if d <= 0.0 {
                g.clip = c;
                g.t = 0.0;
                g.queued = None;
            } else {
                g.queued = Some((c, d - dt));
            }
        }
        if g.t > g.clip.duration() {
            g.clip = g.clip.next();
            g.t = g.clip.duration().min(g.t);
            if g.clip == Clip::ReadIdle || g.clip == Clip::TalkCalm {
                g.t = 0.0;
            }
        }
        // pencil: in hand from PickWrite f65 until BackToBook f83
        let was = g.pencil_in_hand;
        g.pencil_in_hand = match g.clip {
            Clip::PickWrite => g.t > 65.0 / 30.0 || was,
            Clip::BackToBook => g.t < 83.0 / 30.0 && was,
            Clip::ReadIdle => false,
            _ => was,
        };
        if g.pencil_in_hand != was {
            audio.write(AudioCmd::Sfx("sfx.wood".into(), if was { 0.5 } else { 0.3 }));
        }
        // pencil scratching while writing (PickWrite f108–146)
        if g.clip == Clip::PickWrite && g.t > 108.0 / 30.0 && g.scratch <= 0.0 {
            audio.write(AudioCmd::Sfx("sfx.geo.pencil".into(), 0.55));
            g.scratch = 1.3;
        }
        g.scratch -= dt;
        let idle = read_pose(g.clock);
        let target = clip_pose(g.clip, g.t, g.clock, idle);
        let k = 1.0 - (-dt / 0.1).exp();
        let pose = match g.cur {
            Some(c) => c.lerp(&target, k),
            None => target,
        };
        g.cur = Some(pose);
        let r = pose.torso;
        let at = |p: Vec3| G_HIP + r * (p - G_HIP);
        // head: during the talk he looks at the prince (unless reading/writing)
        let mut head = r * pose.head;
        if st.talking && matches!(g.clip, Clip::SpotExplorer | Clip::PleaseTell | Clip::TalkCalm | Clip::PencilStops) {
            if let Some(p) = pp {
                let look = head_look(gt, at(G_NECK) + Vec3::Y * 0.15, p + Vec3::Y * 0.95, 6.0, 40.0, (-30.0, 20.0));
                head = head.slerp(look * Quat::from_rotation_x(-0.05), 0.75);
            }
        }
        place_part(&mut tfs, rig, "torso", Transform::from_translation(G_HIP).with_rotation(r));
        place_part(&mut tfs, rig, "head", Transform::from_translation(at(G_NECK)).with_rotation(head));
        let sl = at(G_SH_L);
        let sr = at(G_SH_R);
        let al = reach(sl, pose.hand_l);
        let ar = reach(sr, pose.hand_r);
        place_part(&mut tfs, rig, "arm_l", Transform::from_translation(sl).with_rotation(al));
        place_part(&mut tfs, rig, "arm_r", Transform::from_translation(sr).with_rotation(ar));
        let pencil = if g.pencil_in_hand {
            let hand = sl + al * Vec3::new(0.0, -G_ARM - 0.03, 0.0);
            Transform::from_translation(hand).with_rotation(al * Quat::from_rotation_x(2.6))
        } else {
            Transform::from_translation(STAND_L + Vec3::Y * 0.02).with_rotation(Quat::from_rotation_z(0.12))
        };
        place_part(&mut tfs, rig, "pencil", pencil);
    }
}

// ------------------------------------------------------------- models ----

const STONE: u32 = 0xeadfc8;
const STONE_DARK: u32 = 0xcfbf9f;
const TERRACOTTA: u32 = 0xc0663e;
const SHELF: u32 = 0x7a5236;

/// The map library (world = local). Floor y 0 on a plinth over the garden (y −0.7).
fn library_layout() -> Vec<Bx> {
    let w = hex(STONE);
    let mut v = vec![
        // plinth + entrance steps (−0.7 → 0)
        bmm(Vec3::new(-4.4, -0.8, 0.0), Vec3::new(4.4, 0.0, 12.4), hex(STONE_DARK)),
        bmm(Vec3::new(-1.6, -0.8, -1.5), Vec3::new(1.6, -0.467, 0.0), hex(STONE_DARK)),
        bmm(Vec3::new(-1.6, -0.8, -1.0), Vec3::new(1.6, -0.233, 0.0), hex(STONE_DARK)),
        bmm(Vec3::new(-1.6, -0.8, -0.5), Vec3::new(1.6, 0.0, 0.0), hex(STONE_DARK)),
        // front wall with the 2.6 m doorway
        bmm(Vec3::new(-4.4, 0.0, 0.2), Vec3::new(-1.3, 5.0, 0.6), w),
        bmm(Vec3::new(1.3, 0.0, 0.2), Vec3::new(4.4, 5.0, 0.6), w),
        bmm(Vec3::new(-1.3, 2.8, 0.2), Vec3::new(1.3, 5.0, 0.6), w),
        // side walls and the back wall with the opening to the balcony
        bmm(Vec3::new(-4.4, 0.0, 0.6), Vec3::new(-4.0, 5.0, 12.4), w),
        bmm(Vec3::new(4.0, 0.0, 0.6), Vec3::new(4.4, 5.0, 12.4), w),
        bmm(Vec3::new(-4.4, 0.0, 12.0), Vec3::new(-1.6, 5.0, 12.4), w),
        bmm(Vec3::new(1.6, 0.0, 12.0), Vec3::new(4.4, 5.0, 12.4), w),
        bmm(Vec3::new(-1.6, 3.2, 12.0), Vec3::new(1.6, 5.0, 12.4), w),
        // shelves of globes and books along both walls
        bmm(Vec3::new(-4.0, 0.0, 1.6), Vec3::new(-3.5, 3.2, 10.6), hex(SHELF)).furniture().hidden(),
        bmm(Vec3::new(3.5, 0.0, 1.6), Vec3::new(4.0, 3.2, 10.6), hex(SHELF)).furniture().hidden(),
        bmm(Vec3::new(-4.0, 0.0, 1.6), Vec3::new(-3.9, 3.2, 10.6), hex(SHELF)).vis(),
        bmm(Vec3::new(3.9, 0.0, 1.6), Vec3::new(4.0, 3.2, 10.6), hex(SHELF)).vis(),
        // balcony floor and balustrade (far rail at z 15.3)
        bmm(Vec3::new(-3.8, -0.8, 12.4), Vec3::new(3.8, 0.0, 15.6), hex(STONE_DARK)),
        bmm(Vec3::new(-3.8, 0.0, 15.3), Vec3::new(3.8, 0.95, 15.55), w).nocam(),
        bmm(Vec3::new(-3.8, 0.0, 12.4), Vec3::new(-3.55, 0.95, 15.55), w).nocam(),
        bmm(Vec3::new(3.55, 0.0, 12.4), Vec3::new(3.8, 0.95, 15.55), w).nocam(),
    ];
    // visual: floor tiles, shelf boards, cornice, door frame, balcony coping
    v.push(bmm(Vec3::new(-4.0, -0.05, 0.6), Vec3::new(4.0, 0.003, 12.0), hex(0xc99a6a)).vis());
    v.push(bmm(Vec3::new(-1.2, 0.003, 2.0), Vec3::new(1.2, 0.008, 11.0), hex(0x8a3a32)).vis());
    for i in 0..5 {
        let y = 0.2 + i as f32 * 0.7;
        v.push(bmm(Vec3::new(-3.95, y - 0.05, 1.6), Vec3::new(-3.48, y, 10.6), hex(0x5a3a26)).vis());
        v.push(bmm(Vec3::new(3.48, y - 0.05, 1.6), Vec3::new(3.95, y, 10.6), hex(0x5a3a26)).vis());
    }
    for z in [1.6f32, 4.6, 7.6, 10.5] {
        v.push(bmm(Vec3::new(-3.95, 0.0, z), Vec3::new(-3.48, 3.25, z + 0.1), hex(0x5a3a26)).vis());
        v.push(bmm(Vec3::new(3.48, 0.0, z), Vec3::new(3.95, 3.25, z + 0.1), hex(0x5a3a26)).vis());
    }
    v.push(bmm(Vec3::new(-4.5, 4.8, -0.0), Vec3::new(4.5, 5.05, 12.5), hex(STONE_DARK)).vis());
    v.push(bmm(Vec3::new(-1.5, 0.0, 0.12), Vec3::new(-1.3, 3.0, 0.2), hex(STONE_DARK)).vis());
    v.push(bmm(Vec3::new(1.3, 0.0, 0.12), Vec3::new(1.5, 3.0, 0.2), hex(STONE_DARK)).vis());
    v.push(bmm(Vec3::new(-1.5, 2.8, 0.12), Vec3::new(1.5, 3.0, 0.2), hex(STONE_DARK)).vis());
    v.push(bmm(Vec3::new(-3.85, 0.95, 15.25), Vec3::new(3.85, 1.03, 15.6), hex(STONE_DARK)).vis());
    v
}

fn library_model(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    mesh_boxes(&mut m, &library_layout());
    // terracotta roof with a skylight; gables
    roof_slab(&mut m, -4.7, 5.0, 6.7, -0.3, 4.9, 0.15, hex(TERRACOTTA));
    roof_slab(&mut m, -4.7, 5.0, 6.7, 7.5, 12.7, 0.15, hex(TERRACOTTA));
    roof_slab(&mut m, 4.7, 5.0, 6.7, -0.3, 12.7, 0.15, hex(TERRACOTTA));
    roof_slab(&mut m, -4.7, 5.0, 6.7, 4.9, 7.5, 0.06, hex(0x7a5236));
    gable(&mut m, 4.4, 5.0, 6.65, 0.19, hex(STONE));
    gable(&mut m, 4.4, 5.0, 6.65, 12.41, hex(STONE));
    // balusters
    for i in 0..19 {
        let x = -3.4 + i as f32 * 0.38;
        m.lathe(Vec3::new(x, 0.0, 15.42), &[(0.07, 0.0), (0.05, 0.25), (0.08, 0.5), (0.04, 0.75), (0.06, 0.95)], 6, hex(STONE));
    }
    // globes and book rows on the shelves
    for side in [-1.0f32, 1.0] {
        for i in 0..18 {
            let z = 1.9 + i as f32 * 0.48;
            for (j, y) in [0.2f32, 0.9, 1.6, 2.3].iter().enumerate() {
                let c = [0x8a3a32u32, 0x3a5a7a, 0x6a7a3a, 0x9a7a3a, 0x5a3a6a][(i + j * 2) % 5];
                let h = 0.32 + ((i * 7 + j * 3) % 4) as f32 * 0.05;
                m.cuboid(Vec3::new(side * 3.72, y + h * 0.5, z), Vec3::new(0.3, h, 0.38), hex(c));
            }
        }
        for z in [3.0f32, 6.4, 9.6] {
            m.sphere(Vec3::new(side * 3.7, 3.45, z), 0.22, 4, 9, hex(0x4a7ab0));
            m.frustum(Vec3::new(side * 3.7, 3.2, z), 0.08, 0.05, 0.05, 6, hex(0xc8a040));
        }
    }
    // lanterns on the facade
    for x in [-2.0f32, 2.0] {
        m.cuboid(Vec3::new(x, 2.4, 0.12), Vec3::new(0.05, 0.4, 0.05), hex(0x3a3a3a));
    }
    p.add(&m);
    let mut g = MeshBuilder::new();
    for x in [-2.0f32, 2.0] {
        g.cuboid(Vec3::new(x, 2.7, 0.05), Vec3::new(0.2, 0.28, 0.2), hex(0xffd090));
    }
    g.cuboid(Vec3::new(1.5, 3.4, 9.6), Vec3::new(0.22, 0.3, 0.22), hex(0xffd090));
    g.cuboid(Vec3::new(0.0, 3.4, 3.0), Vec3::new(0.22, 0.3, 0.22), hex(0xffd090));
    p.add_part(None, &g, Transform::IDENTITY, PartMat::Emissive(hex(0xffc070), 3.0));
    // skylight glass
    let mut sk = MeshBuilder::new();
    roof_slab(&mut sk, -4.7, 5.0, 6.7, 5.0, 7.4, 0.04, hex(0xe8f0ff));
    p.add_part(None, &sk, Transform::IDENTITY, PartMat::Emissive(hex(0xfff2d0), 1.2));
}

fn door_leaf(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 1.4, 0.0), Vec3::new(1.28, 2.8, 0.08), hex(0x2f5a6a));
    for y in [0.7f32, 2.0] {
        m.cuboid(Vec3::new(0.0, y, 0.0), Vec3::new(0.9, 0.9, 0.11), hex(0x3d6e80));
    }
    p.add(&m);
}

fn tower(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 1.9, 1.75, 9.0, 12, hex(STONE));
    m.frustum(Vec3::Y * 9.0, 2.0, 2.0, 0.3, 12, hex(STONE_DARK));
    m.frustum(Vec3::Y * 9.3, 2.1, 0.0, 2.6, 12, hex(TERRACOTTA));
    for (y, a) in [(3.0f32, 200.0f32), (6.0, 250.0), (7.6, 160.0)] {
        let r = a.to_radians();
        m.cuboid(Vec3::new(r.sin() * 1.75, y, r.cos() * 1.75), Vec3::new(0.4, 0.8, 0.4), hex(0x3a3040));
    }
    p.add(&m);
}

/// The map table (top 0.8) with the map, the huge book, armillary sphere,
/// book stacks, inkwell, papers and the pencil stand.
fn table_model(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, TABLE_Y - 0.04, 0.0), Vec3::new(1.6, 0.08, 1.1), hex(0x6a4428));
    for (x, z) in [(-0.72f32, -0.47f32), (0.72, -0.47), (-0.72, 0.47), (0.72, 0.47)] {
        m.lathe(Vec3::new(x, 0.0, z), &[(0.06, 0.0), (0.05, 0.3), (0.07, 0.45), (0.05, 0.72)], 6, hex(0x5a3a22));
    }
    // map spread (local to the table: world (−1.8, 0.81, 6.7))
    m.cuboid(Vec3::new(0.15, 0.815, 0.2), Vec3::new(0.9, 0.01, 0.6), hex(0xe8d8a8));
    m.cuboid(Vec3::new(0.0, 0.822, 0.25), Vec3::new(0.3, 0.004, 0.2), hex(0x7aa0b8));
    m.cuboid(Vec3::new(0.35, 0.822, 0.1), Vec3::new(0.25, 0.004, 0.15), hex(0x9aae6a));
    // the huge open book in front of the geographer (world (−2.15, 0.82, 6.1))
    m.with(Transform::from_xyz(-0.2, 0.82, -0.4), |m| {
        m.cuboid(Vec3::new(-0.2, 0.03, 0.0), Vec3::new(0.38, 0.06, 0.5), hex(0xf2ead6));
        m.cuboid(Vec3::new(0.2, 0.03, 0.0), Vec3::new(0.38, 0.06, 0.5), hex(0xf2ead6));
        m.cuboid(Vec3::new(0.0, 0.01, 0.0), Vec3::new(0.82, 0.03, 0.54), hex(0x6a2a22));
        for i in 0..6 {
            m.cuboid(Vec3::new(-0.2, 0.062, -0.15 + i as f32 * 0.06), Vec3::new(0.28, 0.002, 0.012), hex(0x6a6a7a));
        }
    });
    // armillary sphere (world (−1.3, 0.81, 6.9))
    m.with(Transform::from_xyz(0.65, TABLE_Y, 0.4), |m| {
        m.frustum(Vec3::ZERO, 0.08, 0.03, 0.12, 6, hex(0xb08a40));
        for k in 0..3 {
            let rot = Quat::from_rotation_x(k as f32 * 1.0) * Quat::from_rotation_z(0.4);
            m.with(Transform::from_xyz(0.0, 0.28, 0.0).with_rotation(rot), |m| {
                m.frustum_caps(Vec3::new(0.0, -0.005, 0.0), 0.15, 0.15, 0.01, 16, hex(0xd0a848), false, false);
            });
        }
        m.sphere(Vec3::Y * 0.28, 0.04, 2, 6, hex(0x4a7ab0));
    });
    // book stacks, inkwell, papers
    for (i, c) in [0x3a5a7a, 0x8a3a32, 0x6a7a3a].iter().enumerate() {
        m.cuboid(Vec3::new(-0.6, 0.83 + i as f32 * 0.07, 0.3), Vec3::new(0.3 - i as f32 * 0.03, 0.07, 0.22), hex(*c));
    }
    m.frustum(Vec3::new(0.4, 0.8, -0.5), 0.045, 0.035, 0.06, 7, hex(0x1a1a2a));
    m.cuboid(Vec3::new(0.55, 0.815, -0.2), Vec3::new(0.22, 0.004, 0.3), hex(0xf4ecd8));
    // pencil stand (world (−1.77, 0.81, 6.03))
    m.frustum(Vec3::new(0.18, 0.8, -0.47), 0.04, 0.04, 0.1, 7, hex(0x7a5a3a));
    p.add(&m);
    // candles
    let mut c = MeshBuilder::new();
    for (x, z) in [(-0.7f32, -0.35f32), (0.72, -0.3), (0.5, 0.45)] {
        c.frustum(Vec3::new(x, 0.8, z), 0.025, 0.025, 0.14, 6, hex(0xf2ead6));
    }
    p.add(&c);
    let mut f = MeshBuilder::new();
    for (x, z) in [(-0.7f32, -0.35f32), (0.72, -0.3), (0.5, 0.45)] {
        f.sphere(Vec3::new(x, 0.97, z), 0.022, 2, 5, hex(0xffd080));
    }
    p.add_part(None, &f, Transform::IDENTITY, PartMat::Emissive(hex(0xffc060), 5.0));
}

/// The geographer, seated on a stool (origin on the floor), facing +z.
/// Named parts: torso, head, arm_l (pencil hand), arm_r, pencil.
fn geographer(p: &mut ModelParts) {
    let robe = hex(0x3f5f4a);
    let skin = hex(0xf0d0b8);
    let mut st = MeshBuilder::new();
    st.frustum(Vec3::ZERO, 0.2, 0.17, 0.44, 8, hex(0x5a3a22));
    for x in [-0.1f32, 0.1] {
        st.tube(Vec3::new(x, 0.47, 0.0), Vec3::new(x, 0.47, 0.38), 0.075, 0.07, 6, robe);
        st.tube(Vec3::new(x, 0.47, 0.38), Vec3::new(x, 0.05, 0.42), 0.07, 0.065, 6, robe);
        st.cuboid(Vec3::new(x, 0.04, 0.48), Vec3::new(0.1, 0.08, 0.2), hex(0x4a2a1a));
    }
    p.add(&st);
    let mut t = MeshBuilder::new();
    t.with(Transform::from_translation(-G_HIP), |t| {
        t.lathe(G_HIP - Vec3::Y * 0.05, &[(0.0, 0.0), (0.25, 0.0), (0.24, 0.25), (0.22, 0.45), (0.15, 0.58), (0.0, 0.6)], 9, robe);
        t.frustum(Vec3::new(0.0, 0.98, 0.0), 0.17, 0.12, 0.1, 8, hex(0xd8c8a0));
        t.cuboid(Vec3::new(0.0, 0.7, 0.21), Vec3::new(0.08, 0.45, 0.04), hex(0xc8a040));
    });
    p.add_part(Some("torso"), &t, Transform::from_translation(G_HIP), PartMat::Vertex);
    let mut h = MeshBuilder::new();
    h.ellipsoid(Vec3::new(0.0, 0.14, 0.0), Vec3::new(0.13, 0.15, 0.13), 4, 9, skin);
    // white beard, moustache, spectacles, skullcap
    h.lathe(Vec3::new(0.0, -0.12, 0.08), &[(0.0, 0.0), (0.07, 0.04), (0.11, 0.14), (0.12, 0.22), (0.0, 0.24)], 8, hex(0xf2f0ea));
    h.ellipsoid(Vec3::new(0.0, 0.09, 0.12), Vec3::new(0.08, 0.025, 0.03), 2, 6, hex(0xf2f0ea));
    for x in [-0.05f32, 0.05] {
        h.with(Transform::from_xyz(x, 0.17, 0.125).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)), |m| {
            m.frustum_caps(Vec3::ZERO, 0.035, 0.035, 0.01, 10, hex(0x8a6a30), false, false);
        });
        h.sphere(Vec3::new(x, 0.17, 0.12), 0.012, 2, 4, hex(0x2b2b3a));
    }
    h.cuboid(Vec3::new(0.0, 0.17, 0.13), Vec3::new(0.03, 0.008, 0.008), hex(0x8a6a30));
    h.sphere(Vec3::new(0.0, 0.13, 0.135), 0.025, 2, 5, hex(0xe8b8a0));
    h.ellipsoid(Vec3::new(0.0, 0.24, -0.01), Vec3::new(0.12, 0.06, 0.12), 2, 8, hex(0x6a2a3a));
    h.ellipsoid(Vec3::new(0.0, 0.16, -0.05), Vec3::new(0.135, 0.07, 0.11), 2, 8, hex(0xe8e6e0));
    p.add_part(Some("head"), &h, Transform::from_translation(G_NECK), PartMat::Vertex);
    let a = arm_mesh(G_ARM, 0.07, 0.055, robe, skin);
    p.add_part(Some("arm_l"), &a, Transform::from_translation(G_SH_L), PartMat::Vertex);
    p.add_part(Some("arm_r"), &a, Transform::from_translation(G_SH_R), PartMat::Vertex);
    let mut pc = MeshBuilder::new();
    pc.frustum(Vec3::ZERO, 0.009, 0.009, 0.16, 6, hex(0xe0b030));
    pc.frustum(Vec3::Y * 0.16, 0.009, 0.0, 0.025, 6, hex(0x3a3030));
    p.add_part(Some("pencil"), &pc, Transform::from_translation(STAND_L), PartMat::Vertex);
}

fn cypress(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.frustum(Vec3::ZERO, 0.12, 0.1, 0.5, 6, hex(0x5a4030));
    m.lathe(Vec3::Y * 0.3, &[(0.0, 0.0), (0.45, 0.3), (0.55, 1.4), (0.45, 2.8), (0.22, 3.9), (0.0, 4.5)], 8, hex(0x3f5f34));
    p.add(&m);
}

fn garden_bed(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.2, 0.0), Vec3::new(2.4, 0.4, 1.8), hex(STONE_DARK));
    m.cuboid(Vec3::new(0.0, 0.41, 0.0), Vec3::new(2.2, 0.03, 1.6), hex(0x6a4a30));
    for i in 0..10 {
        let x = -0.9 + (i % 5) as f32 * 0.45;
        let z = if i < 5 { -0.4 } else { 0.4 };
        m.sphere(Vec3::new(x, 0.52, z), 0.16, 2, 6, hex(0x5f8a44));
        m.sphere(Vec3::new(x + 0.05, 0.62, z), 0.05, 2, 5, hex([0xe0a0c0u32, 0xf0d060, 0xd06050][i % 3]));
    }
    p.add(&m);
}

/// The sea of clouds far below (origin at y −55): a soft deck and cumulus heaps.
fn cloud_sea(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let mut rng = Rng::new(330);
    m.frustum_caps(Vec3::new(0.0, -2.0, 0.0), 420.0, 420.0, 0.5, 40, hex(0xf4ecef), false, true);
    for _ in 0..140 {
        let a = rng.range(0.0, std::f32::consts::TAU);
        let r = 35.0 + rng.f().powf(0.6) * 330.0;
        let c = Vec3::new(a.cos() * r, 0.0, a.sin() * r);
        let s = rng.range(6.0, 16.0) * (0.6 + r / 300.0);
        let tint = mix_color(hex(0xffffff), hex(0xf6d6c0), rng.f() * 0.6);
        for k in 0..3 {
            let o = Vec3::new(rng.range(-1.0, 1.0) * s, rng.range(0.0, 0.5) * s * 0.4, rng.range(-1.0, 1.0) * s);
            m.ellipsoid(c + o, Vec3::new(s * (1.0 - k as f32 * 0.2), s * 0.45, s * 0.85), 3, 7, tint);
        }
    }
    p.add_part(None, &m, Transform::IDENTITY, PartMat::Unlit);
}

/// The skylight sunbeam with dust (a faint slanted shaft).
fn sunbeam(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.with(Transform::from_rotation(Quat::from_rotation_x(0.35) * Quat::from_rotation_z(-0.25)), |m| {
        m.frustum_caps(Vec3::new(0.0, -5.2, 0.0), 1.1, 0.75, 5.2, 10, hex(0xfff0c8), false, false);
    });
    p.add_part(None, &m, Transform::IDENTITY, PartMat::Glass(Color::srgba(1.0, 0.94, 0.78, 0.07)));
}
