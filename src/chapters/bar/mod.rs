//! Chapter 4 — The Tippler ("Bar_Main" in the original).
//!
//! Follows the original BarStory / BarTableScene / DrinkerActor /
//! DrinkerAutoPlay / BarDoor / BarSeat design (docs/original/bar.md):
//! - a NIGHT island with a lit tavern, the moon top right, chimney smoke;
//!   a low beamed hall, a stairwell and an upstairs floor, a walled yard;
//! - the front door: two-palm push (approach 0.9 m/s, align 0.25 s, push
//!   ≈ 2.2 s, opens 100°, leaf 1.3 m); the hall lights brighten with the
//!   opening angle (full at 75°); a quick back door to the yard; doors stay
//!   wedged open;
//! - the drinker at a table of full and empty bottles; ambient sips every
//!   9–16 s and looks to the window (paused during the story); the bottle's
//!   fill drops 0.025/s while drinking, never below 0.35;
//! - SITTING on the high guest chair (seat 0.46 m) starts the scene — there is
//!   no talk prompt: two-shot → 22 s push-in, his first sip 0.8 s after the
//!   prince sits, the eight-line exchange with the drink / head-down /
//!   outburst beats (the last line is locked ~1 s, no click), the side-on
//!   silence shot with 3 s of silence and a last sip, then back to the
//!   seated view (2.5 s). Then "stand up" and depart. All four camera shots
//!   use the original's positions (table-scene frame) and FOVs.

use super::vanity::kit::*;
use crate::core::camera::CameraRig;
use crate::core::prelude::*;

pub struct Bar;

const ID: &str = "bar";

/// Tavern origin (front door plane) in the world. Interior x ∈ [−4, 4],
/// z ∈ [0.4, 6.4] (local); the table-scene shots are in this frame.
const T: Vec3 = Vec3::new(-6.5, 0.0, -6.3);
fn l(x: f32, y: f32, z: f32) -> Vec3 {
    T + Vec3::new(x, y, z)
}
const CHAIR: Vec2 = Vec2::new(-1.95, 1.62);
const DRINKER: Vec2 = Vec2::new(-1.95, 3.6);
const TABLE: Vec2 = Vec2::new(-1.95, 2.6);
const HIGH_SEAT: f32 = 0.46;
const CEIL: f32 = 2.6;
const FLOOR2: f32 = 2.9;

pub const DRINKER_SPK: Speaker = speakers::DRINKER;

pub const STRINGS: StringTable = &[
    ("bar.title", "酒鬼的星球", "The Tippler's Planet"),
    ("bar.subtitle", "一个为了忘记一切而喝酒的人", "A man who drinks to forget everything"),
    ("bar.line", "夜里的小酒馆亮着灯，桌上摆满了瓶子。", "A little tavern glows in the night; the table is covered in bottles."),
    ("bar.visited", "那盏小灯还在黑暗里摇摇晃晃地亮着。", "The small lamp still flickers on in the dark."),
    ("bar.loading", "夜很深，酒瓶碰出清脆的一声……", "Deep night; a bottle clinks somewhere…"),
    ("bar.n1", "这颗星球上住着一个酒鬼。他喝酒，是为了忘记一件让他羞愧的事——他喝酒。", "A tippler lives on this planet. He drinks to forget something that shames him: his drinking."),
    ("bar.ev.sit", "到小酒馆里坐一坐", "Go and sit in the tavern"),
    ("bar.st.sit", "在酒鬼对面坐下", "Sit down across from the tippler"),
    ("bar.target.drinker", "酒鬼", "The tippler"),
    ("bar.verb.push", "推门", "Push the door"),
    ("bar.verb.sit", "坐下", "Sit down"),
    ("bar.verb.stand", "站起来", "Stand up"),
    // the exchange (our own wording)
    ("bar.l1", "你在这儿做什么呢？", "What are you doing here?"),
    ("bar.l2", "我在喝酒。", "I am drinking."),
    ("bar.l3", "你为什么要喝酒呢？", "Why are you drinking?"),
    ("bar.l4", "为了忘记。", "So that I may forget."),
    ("bar.l5", "忘记什么呢？", "Forget what?"),
    ("bar.l6", "忘记我很羞愧。", "Forget that I am ashamed."),
    ("bar.l7", "你羞愧什么呢？", "Ashamed of what?"),
    ("bar.l8", "羞愧我喝酒！", "Ashamed of drinking!"),
];

impl Chapter for Bar {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 4,
            title: "bar.title",
            subtitle: "bar.subtitle",
            line: "bar.line",
            visited_line: "bar.visited",
            loading: "bar.loading",
            map_pos: Vec2::new(860.0, 470.0),
            map_radius: 48.0,
            map_colors: (hex(0x5a6688), hex(0xf0b45a)),
            music: "music.bar",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("bar.tavern", tavern_model)
            .register_model("bar.door", door_leaf)
            .register_model("bar.door_back", door_back)
            .register_model("bar.table", table_model)
            .register_model("bar.chair_high", chair_high)
            .register_model("bar.chair", chair_low)
            .register_model("bar.drinker", drinker_model)
            .register_model("bar.moon", moon)
            .register_model("bar.smoke", smoke)
            .register_model("bar.barrel", barrel)
            .init_resource::<BState>();
        super::vanity::kit::register(app, ID);
        app.add_systems(Update, (doors, seat, drinker_auto, animate_drinker, smoke_puffs, lights_by_door).chain().in_set(CoreSet::Gameplay).run_if(in_chapter(ID)));
    }

    fn build(&self, cx: &mut ChapterCx) {
        cx.world.insert_resource(BState::default());
        cx.world.insert_resource(PrinceOverride::default());
        cx.world.resource_mut::<crate::core::player::PlayerTuning>().walk_speed = 0.9;

        let mut p = PlanetParams::sized(16.0);
        p.center = Vec2::new(-6.0, -6.0);
        p.cap_flatten = 0.45;
        p.ground = hex(0x4f6a55);
        p.ground2 = hex(0x445e4c);
        p.dry = hex(0x6f6a58);
        p.rock = hex(0x55505e);
        p.mods = vec![
            TerrainMod::Flatten { center: Vec2::new(T.x, T.z + 5.0), radius: 7.6, blend: 3.0, height: Some(0.0) },
            TerrainMod::Flatten { center: Vec2::new(T.x, T.z - 1.5), radius: 2.2, blend: 1.5, height: Some(0.0) },
            TerrainMod::Paint { center: Vec2::new(T.x, T.z - 2.0), radius: 1.4, blend: 1.0, color: hex(0x6b5e4a) },
            TerrainMod::Paint { center: Vec2::new(T.x + 0.6, T.z - 5.5), radius: 1.1, blend: 1.0, color: hex(0x6b5e4a) },
            TerrainMod::Paint { center: Vec2::new(T.x + 1.2, T.z - 9.0), radius: 1.0, blend: 1.0, color: hex(0x6b5e4a) },
        ];
        cx.planet(p);
        let pal = SkyPalette {
            night_zenith: hex(0x0a1030),
            night_horizon: hex(0x22305a),
            fog: 0.45,
            ..default()
        };
        cx.sky(23.2, 0.0, None, pal);
        cx.sky_mut().fixed_night = Some(1.0);
        cx.ambience("amb.night");

        // tavern shell (layout → mesh + colliders)
        cx.model("model.bar.tavern", T, 0.0, 1.0);
        spawn_colliders(cx, T, &tavern_layout());
        // camera lids: the hall ceiling (around the stairwell) and the roof
        cx.spawn((Transform::from_translation(l(0.0, 5.2, 3.4)), Collider::cuboid(Vec3::new(8.8, 0.4, 6.8))));

        // doors: the front (push, 100°, ≈2.2 s) and the back door to the yard (quick)
        spawn_door(cx, Door::new("front", 0.0, -1.0, 100.0, 1.3, 2.2, Vec2::Y), l(-0.65, 0.0, 0.2), "model.bar.door", Vec3::new(0.65, 0.0, 0.0), Vec3::new(1.3, 2.2, 0.08));
        spawn_door(cx, Door::new("back", 0.0, -1.0, 95.0, 1.0, 0.7, Vec2::Y), l(2.6, 0.0, 6.6), "model.bar.door_back", Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.0, 2.2, 0.08));
        let d = cx.empty(l(0.0, 0.0, 0.2), 0.0);
        cx.interactable(d, Interactable::new("bar_door", "bar.verb.push").radius(2.0).focus(Vec3::Y * 1.1).prompt_height(0.5));
        let d = cx.empty(l(3.1, 0.0, 6.6), 0.0);
        cx.interactable(d, Interactable::new("bar_back", "bar.verb.push").radius(1.6).focus(Vec3::Y * 1.1).prompt_height(0.5));

        // the table scene
        let tb = cx.model("model.bar.table", l(TABLE.x, 0.0, TABLE.y), 0.0, 1.0);
        cx.collider(tb, Collider::cuboid(Vec3::new(1.0, 0.76, 0.86)).not_walkable().no_camera());
        let ch = cx.model("model.bar.chair_high", l(CHAIR.x, 0.0, CHAIR.y), 0.0, 1.0);
        cx.collider(ch, Collider::cuboid(Vec3::new(0.46, 0.5, 0.42)).not_walkable().no_camera());
        let seat = cx.empty(l(CHAIR.x, HIGH_SEAT, CHAIR.y), 0.0);
        cx.interactable(seat, Interactable::new("guest_chair", "bar.verb.sit").radius(1.4).prompt_height(0.7).any_facing());
        cx.anchor("guest_chair", l(CHAIR.x, 0.0, CHAIR.y), 0.0);
        let dc = cx.model("model.bar.chair", l(DRINKER.x, 0.0, DRINKER.y + 0.05), 180.0, 1.0);
        cx.collider(dc, Collider::cuboid(Vec3::new(0.6, 0.6, 0.6)).not_walkable().no_camera());
        let dr = cx.model("model.bar.drinker", l(DRINKER.x, 0.0, DRINKER.y), 180.0, 1.0);
        cx.world.entity_mut(dr).insert((Rig::default(), Drinker::default()));
        cx.anchor("drinker", l(DRINKER.x, 1.1, DRINKER.y), 180.0);

        // lights: pendant over the table, fireplace, counter lamp, door lantern
        let a = lamp(cx, l(TABLE.x, 2.25, TABLE.y), hex(0xffb35c), 70_000.0, 7.0, 0.1, 0.4);
        let b = lamp(cx, l(-2.5, 0.7, 5.7), hex(0xff7a32), 60_000.0, 6.0, 0.35, 2.0);
        let c = lamp(cx, l(1.3, 2.1, 4.9), hex(0xffc37a), 40_000.0, 6.0, 0.08, 3.1);
        let e = lamp(cx, l(-1.6, 2.4, 3.5), hex(0xffb466), 25_000.0, 6.0, 0.1, 5.2);
        for x in [a, b, c, e] {
            cx.world.entity_mut(x).insert(HallLight);
        }
        // soft fill from the room side so the prince's back reads in the two-shot
        lamp(cx, l(-0.9, 1.7, 0.9), hex(0xffa860), 9_000.0, 3.0, 0.05, 7.7);
        lamp(cx, l(1.1, 2.3, -0.5), hex(0xffc37a), 30_000.0, 7.0, 0.12, 4.4);
        lamp(cx, l(0.0, 4.2, 3.4), hex(0xffb466), 18_000.0, 6.0, 0.1, 6.2);

        // the moon (top right of the opening view) and chimney smoke
        cx.model("model.bar.moon", T + Vec3::new(-70.0, 62.0, 110.0), 0.0, 1.0);
        for i in 0..12 {
            let s = cx.model("model.bar.smoke", l(-3.0, 8.6, 6.0), 0.0, 0.3);
            cx.world.entity_mut(s).insert(Puff(i as f32 / 12.0));
        }

        // yard dressing and the night meadow
        for (x, z, s) in [(-3.4f32, 8.0f32, 1.0f32), (-2.6, 8.3, 0.9), (3.2, 10.6, 1.0)] {
            let b = cx.model("model.bar.barrel", l(x, 0.0, z), x * 40.0, s);
            cx.collider(b, Collider::cylinder(0.38 * s, 0.9).no_camera());
        }
        let avoid = [(Vec2::new(T.x, T.z + 5.0), 7.2), (Vec2::new(T.x, T.z - 2.0), 1.6), (Vec2::new(T.x + 0.6, T.z - 5.5), 1.4), (Vec2::new(T.x + 1.2, T.z - 9.0), 1.3)];
        cx.scatter("model.grass_tuft", 90, 1.0, 15.5, 41, (0.8, 1.4), &avoid);
        cx.scatter("model.rock", 9, 9.0, 15.5, 42, (0.6, 1.4), &avoid);
        cx.scatter("model.tree_pine", 6, 10.0, 15.0, 43, (0.8, 1.2), &avoid);
        cx.scatter("model.mushroom", 8, 6.0, 15.0, 44, (0.8, 1.2), &avoid);

        let start = cx.ground(T.x + 1.5, T.z - 10.5);
        cx.player(start, -8.0);
    }

    fn story(&self) -> Story {
        let cam0 = Vec3::new(-10.0, 4.5, -26.0);
        let look0 = Vec3::new(-6.53, 3.98, -6.31);
        let push = (look0 - cam0).normalize() * 3.0;
        Story {
            events: vec![event("bar.ev.sit", &["bar.st.sit"])],
            hidden: None,
            script: vec![
                push_cmd((cam0, look0, 52.0), (cam0 + push, look0 + push, 52.0), 4.0),
                card(),
                run(|w: &mut World| end_shot_to(w, Some(-8.0), Some(14.0), 1.6)),
                narrate(&["bar.n1"]),
                start_event(0),
                target(Pos::anchor("guest_chair"), "bar.target.drinker"),
                // sitting on the guest chair starts the scene (no talk prompt)
                wait_flag(
                    "b_seated",
                    vec![t_use("bar_door"), t_goto(Pos::World(l(0.0, 0.0, 1.3))), t_use("guest_chair")],
                ),
                // ---- phase talk
                clear_target(),
                run(|w: &mut World| w.resource_mut::<BState>().scene = true),
                wait_until(|w: &World| drinker_idle(w), vec![t_wait(13.0)]),
                shot_cmd(l(-1.24, 1.4, 0.75), l(-1.97, 0.92, 2.55), 0.8, 50.0),
                wait(0.8),
                shot_cmd(l(-1.37, 1.32, 1.07), l(-1.97, 0.92, 2.55), 22.0, 48.0),
                run(|w: &mut World| drink(w, DGest::Drink)),
                wait(1.0),
                wait_until(|w: &World| drinker_idle(w), vec![t_wait(8.0)]),
                say(speakers::PRINCE, "bar.l1"),
                say(DRINKER_SPK, "bar.l2"),
                say(speakers::PRINCE, "bar.l3"),
                say(DRINKER_SPK, "bar.l4"),
                say(speakers::PRINCE, "bar.l5"),
                run(|w: &mut World| drink(w, DGest::Drink)),
                shot_cmd(l(-1.35, 1.3, 2.4), l(-2.03, 1.04, 3.38), 1.8, 42.0),
                wait(1.0),
                wait_until(|w: &World| drinker_idle(w), vec![t_wait(8.0)]),
                say(DRINKER_SPK, "bar.l6"),
                say(speakers::PRINCE, "bar.l7"),
                run(|w: &mut World| drink(w, DGest::HeadDown)),
                wait(2.5),
                run(|w: &mut World| drink(w, DGest::Outburst)),
                wait(0.35),
                say_for(DRINKER_SPK, "bar.l8", 1.6),
                wait(1.2),
                // side-on silence: both heads down, then his last sip
                shot_cmd(l(0.4, 1.25, 2.7), l(-1.2, 0.99, 2.73), 2.2, 48.0),
                run(|w: &mut World| {
                    drink(w, DGest::HeadDown);
                    w.resource_mut::<BState>().sad = true;
                }),
                wait(3.0),
                run(|w: &mut World| drink(w, DGest::Drink)),
                wait(1.0),
                wait_until(|w: &World| drinker_idle(w), vec![t_wait(8.0)]),
                // ---- done: back to the seated view
                run(|w: &mut World| {
                    w.resource_mut::<BState>().sad = false;
                    end_shot_to(w, Some(0.0), Some(12.0), 2.5);
                }),
                wait(2.5),
                run(|w: &mut World| {
                    let mut s = w.resource_mut::<BState>();
                    s.scene = false;
                    s.done = true;
                }),
                complete_event(0),
                // the autopilot cannot walk while seated: stand up first
                test_only(|w: &mut World| stand(w)),
                departure(),
            ],
        }
    }
}

// --------------------------------------------------------------- state ----

#[derive(Resource, Default)]
struct BState {
    seated: bool,
    scene: bool,
    done: bool,
    sad: bool,
    sit_seq: Option<f32>,
    door_seq: Option<(u8, f32)>,
    lights: f32,
}

#[derive(Component)]
struct HallLight;

#[derive(Component)]
struct Puff(f32);

fn stand(w: &mut World) {
    w.resource_mut::<BState>().seated = false;
    stand_player(w, l(CHAIR.x + 0.62, 0.0, CHAIR.y), 90.0);
}

// --------------------------------------------------------------- doors ----

#[allow(clippy::too_many_arguments)]
fn doors(
    time: Res<Time>,
    mut msgs: MessageReader<Interacted>,
    mut st: ResMut<BState>,
    mut ds: Query<&mut Door>,
    mut its: Query<&mut Interactable>,
    mut player: Query<(&mut Player, &Transform)>,
    mut audio: MessageWriter<AudioCmd>,
) {
    let dt = time.delta_secs();
    let Ok((mut pl, ptf)) = player.single_mut() else { return };
    for m in msgs.read() {
        match m.id.as_str() {
            "bar_door" if st.door_seq.is_none() => {
                pl.drive = Drive::WalkTo { target: Vec2::new(T.x, T.z - 0.7), speed: 0.9, stop_radius: 0.12 };
                st.door_seq = Some((0, 0.0));
            }
            "bar_back" => {
                // a quick push away from the prince
                let inside = ptf.translation.z < T.z + 6.6;
                for mut d in &mut ds {
                    if d.id == "back" {
                        d.sign = if inside { -1.0 } else { 1.0 };
                        d.inward = if inside { Vec2::Y } else { -Vec2::Y };
                        d.target = 1.0;
                    }
                }
                audio.write(AudioCmd::Sfx("sfx.creak".into(), 0.4));
            }
            _ => {}
        }
    }
    if let Some((stage, t)) = st.door_seq {
        let t = t + dt;
        let mut next = Some((stage, t));
        match stage {
            0 if pl.is_arrived() || t > 5.0 => {
                pl.drive = Drive::External;
                pl.face_target = Some(0.0);
                next = Some((1, 0.0));
            }
            1 if t > 0.25 => {
                pl.pose = Pose::Reach;
                for mut d in &mut ds {
                    if d.id == "front" {
                        d.target = 1.0;
                    }
                }
                audio.write(AudioCmd::Sfx("sfx.creak".into(), 0.7));
                next = Some((2, 0.0));
            }
            2 if t > 2.0 => {
                pl.pose = Pose::None;
                pl.drive = Drive::Player;
                next = None;
            }
            _ => {}
        }
        st.door_seq = next;
    }
    let front_open = ds.iter().any(|d| d.id == "front" && d.target > 0.5);
    let back_open = ds.iter().any(|d| d.id == "back" && d.target > 0.5);
    for mut it in &mut its {
        match it.id.as_str() {
            "bar_door" => it.enabled = !front_open && st.door_seq.is_none(),
            "bar_back" => it.enabled = !back_open,
            _ => {}
        }
    }
}

/// The hall lights brighten with the front door's opening angle (full at 75°).
fn lights_by_door(mut st: ResMut<BState>, ds: Query<&Door>, player: Query<&Transform, With<Player>>, mut ls: Query<&mut Flicker, With<HallLight>>) {
    let ang = ds.iter().map(|d| d.angle_deg()).fold(0.0f32, f32::max);
    let inside = player.single().map(|p| p.translation.z > T.z + 0.5).unwrap_or(false);
    let k = if inside { 1.0 } else { smooth(0.0, 75.0, ang) };
    st.lights = st.lights.max(0.3 + 0.7 * k);
    for mut f in &mut ls {
        f.scale = st.lights;
    }
}

// ---------------------------------------------------------------- seat ----

#[allow(clippy::too_many_arguments)]
fn seat(
    time: Res<Time>,
    mut commands: Commands,
    mut msgs: MessageReader<Interacted>,
    mut st: ResMut<BState>,
    mut its: Query<&mut Interactable>,
    mut player: Query<&mut Player>,
    mut ov: ResMut<PrinceOverride>,
    mut rig: ResMut<CameraRig>,
) {
    let dt = time.delta_secs();
    for m in msgs.read() {
        if m.id != "guest_chair" {
            continue;
        }
        if st.seated {
            commands.queue(stand);
            rig.yaw = std::f32::consts::FRAC_PI_2;
        } else if st.sit_seq.is_none() {
            // BarSeatAssist: walk to the standing spot beside the chair, then sit
            if let Ok(mut pl) = player.single_mut() {
                pl.drive = Drive::WalkTo { target: Vec2::new(T.x + CHAIR.x + 0.62, T.z + CHAIR.y), speed: 0.9, stop_radius: 0.15 };
            }
            st.sit_seq = Some(0.0);
        }
    }
    if let Some(t) = st.sit_seq {
        let t = t + dt;
        let arrived = player.single().map(|p| p.is_arrived()).unwrap_or(true);
        if arrived || t > 4.0 {
            st.sit_seq = None;
            st.seated = true;
            commands.queue(|w: &mut World| {
                sit_player(w, l(CHAIR.x, HIGH_SEAT, CHAIR.y), 0.0);
                w.resource_mut::<CameraRig>().yaw = 0.0;
                crate::core::story::set_flag(w, "b_seated");
            });
        } else {
            st.sit_seq = Some(t);
        }
    }
    let (seated, scene, done) = (st.seated, st.scene, st.done);
    for mut it in &mut its {
        if it.id == "guest_chair" {
            // seated: "stand up" once the scene is over; standing: sit until it is done
            let key = if seated { "bar.verb.stand" } else { "bar.verb.sit" };
            if it.prompt != key {
                it.prompt = key.to_string();
            }
            it.enabled = if seated { done } else { !done && st.sit_seq.is_none() && !scene };
        }
    }
    // the prince looks at the drinker; in the silence a sad seated pose
    if seated {
        let sad = if st.sad { 1.0 } else { 0.0 };
        ov.head = Some(Quat::from_rotation_x(0.05 + 0.35 * sad));
        ov.arm_l = Some(Quat::from_rotation_x(-0.5 + 0.35 * sad) * Quat::from_rotation_z(0.05));
        ov.arm_r = Some(Quat::from_rotation_x(-0.5 + 0.35 * sad) * Quat::from_rotation_z(-0.05));
    } else {
        ov.head = None;
        ov.arm_l = None;
        ov.arm_r = None;
    }
}

// ------------------------------------------------------------ drinker ----

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum DGest {
    #[default]
    Idle,
    Drink,
    HeadDown,
    Outburst,
    LookWindow,
}

impl DGest {
    fn duration(self) -> f32 {
        match self {
            DGest::Idle => f32::MAX,
            DGest::Drink => 3.2,
            DGest::HeadDown => 2.5,
            DGest::Outburst => 1.8,
            DGest::LookWindow => 3.0,
        }
    }
}

#[derive(Component, Debug)]
struct Drinker {
    g: DGest,
    t: f32,
    next_sip: f32,
    fill: f32,
    cur: Option<DPose>,
    clock: f32,
    hold: bool,
}

impl Default for Drinker {
    fn default() -> Self {
        Drinker { g: DGest::Idle, t: 0.0, next_sip: 6.0, fill: 0.8, cur: None, clock: 0.0, hold: false }
    }
}

fn drink(w: &mut World, g: DGest) {
    let mut q = w.query::<&mut Drinker>();
    if let Some(mut d) = q.iter_mut(w).next() {
        d.g = g;
        d.t = 0.0;
        d.hold = g == DGest::HeadDown;
    }
}

fn drinker_idle(w: &World) -> bool {
    let mut ok = true;
    for e in w.iter_entities() {
        if let Some(d) = e.get::<Drinker>() {
            ok = d.g == DGest::Idle || d.hold;
        }
    }
    ok
}

/// DrinkerAutoPlay: a sip every 9–16 s, sometimes a look to the window; paused in the story.
fn drinker_auto(time: Res<Time>, st: Res<BState>, mut q: Query<&mut Drinker>, mut rng: Local<Option<Rng>>) {
    let rng = rng.get_or_insert_with(|| Rng::new(44));
    let dt = time.delta_secs();
    for mut d in &mut q {
        if st.scene {
            d.next_sip = d.next_sip.max(4.0);
            continue;
        }
        if d.g != DGest::Idle {
            continue;
        }
        if d.hold {
            d.hold = false;
        }
        d.next_sip -= dt;
        if d.next_sip <= 0.0 {
            d.g = if rng.f() < 0.3 { DGest::LookWindow } else { DGest::Drink };
            d.t = 0.0;
            d.next_sip = rng.range(9.0, 16.0);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DPose {
    torso: Quat,
    head: Quat,
    arm_l: Quat,
    arm_r: Quat,
    /// 0 = bottle resting on the table, 1 = at the mouth.
    sip: f32,
}

impl DPose {
    fn lerp(&self, o: &DPose, k: f32) -> DPose {
        DPose {
            torso: self.torso.slerp(o.torso, k),
            head: self.head.slerp(o.head, k),
            arm_l: self.arm_l.slerp(o.arm_l, k),
            arm_r: self.arm_r.slerp(o.arm_r, k),
            sip: self.sip + (o.sip - self.sip) * k,
        }
    }
}

const D_HIP: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const D_NECK: Vec3 = Vec3::new(0.0, 1.08, 0.0);
const D_SH_L: Vec3 = Vec3::new(0.22, 1.02, 0.0);
const D_SH_R: Vec3 = Vec3::new(-0.22, 1.02, 0.0);
const D_ARM: f32 = 0.55;

fn d_idle(c: f32) -> DPose {
    let b = (c * 1.1).sin();
    DPose {
        torso: Quat::from_rotation_x(0.32 + b * 0.015),
        head: Quat::from_rotation_x(0.22) * Quat::from_rotation_y((c * 0.3).sin() * 0.1),
        arm_l: Quat::from_rotation_x(-0.75) * Quat::from_rotation_z(-0.35),
        arm_r: Quat::from_rotation_x(-0.8) * Quat::from_rotation_z(0.3),
        sip: 0.0,
    }
}

fn d_pose(g: DGest, t: f32, idle: DPose) -> DPose {
    match g {
        DGest::Idle => idle,
        DGest::Drink => {
            let up = smooth(0.2, 0.9, t) * (1.0 - smooth(2.3, 3.0, t));
            DPose {
                torso: idle.torso.slerp(Quat::from_rotation_x(0.05), up),
                head: idle.head.slerp(Quat::from_rotation_x(-0.45), up),
                arm_l: idle.arm_l,
                arm_r: idle.arm_r.slerp(Quat::from_rotation_x(-2.35) * Quat::from_rotation_z(-0.45), up),
                sip: up,
            }
        }
        DGest::HeadDown => {
            let k = smooth(0.0, 0.8, t);
            DPose {
                torso: idle.torso.slerp(Quat::from_rotation_x(0.55), k),
                head: idle.head.slerp(Quat::from_rotation_x(0.6), k),
                arm_l: idle.arm_l.slerp(Quat::from_rotation_x(-1.05) * Quat::from_rotation_z(-0.55), k),
                arm_r: idle.arm_r.slerp(Quat::from_rotation_x(-1.05) * Quat::from_rotation_z(0.55), k),
                sip: 0.0,
            }
        }
        DGest::Outburst => {
            let k = smooth(0.0, 0.18, t) * (1.0 - smooth(1.2, 1.8, t));
            DPose {
                torso: idle.torso.slerp(Quat::from_rotation_x(-0.12), k),
                head: idle.head.slerp(Quat::from_rotation_x(-0.25), k),
                arm_l: idle.arm_l.slerp(Quat::from_rotation_x(-1.6) * Quat::from_rotation_z(0.7), k),
                arm_r: idle.arm_r.slerp(Quat::from_rotation_x(-1.6) * Quat::from_rotation_z(-0.7), k),
                sip: 0.0,
            }
        }
        DGest::LookWindow => {
            let k = smooth(0.0, 0.6, t) * (1.0 - smooth(2.3, 3.0, t));
            DPose { head: idle.head.slerp(Quat::from_rotation_y(-1.0) * Quat::from_rotation_x(-0.1), k), torso: idle.torso.slerp(Quat::from_rotation_y(-0.15) * Quat::from_rotation_x(0.2), k), ..idle }
        }
    }
}

fn animate_drinker(time: Res<Time>, mut q: Query<(&mut Drinker, &Rig)>, mut tfs: PartTfs, mut audio: MessageWriter<AudioCmd>) {
    let dt = time.delta_secs();
    for (mut d, rig) in &mut q {
        d.t += dt;
        d.clock += dt;
        if d.g != DGest::Idle && d.t >= d.g.duration() && !(d.hold && d.g == DGest::HeadDown) {
            d.g = DGest::Idle;
            d.hold = false;
        }
        // sounds: a gulp at the top of the sip, a clink when the bottle lands
        if d.g == DGest::Drink {
            if d.t - dt < 1.0 && d.t >= 1.0 {
                audio.write(AudioCmd::Sfx("sfx.bar.gulp".into(), 0.5));
            }
            if d.t - dt < 2.95 && d.t >= 2.95 {
                audio.write(AudioCmd::Sfx("sfx.bar.clink".into(), 0.35));
            }
        }
        if d.g == DGest::Outburst && d.t - dt < 0.12 && d.t >= 0.12 {
            audio.write(AudioCmd::Sfx("sfx.plank".into(), 0.7));
        }
        let idle = d_idle(d.clock);
        let target = d_pose(d.g, d.t, idle);
        let k = 1.0 - (-dt / 0.09).exp();
        let pose = match d.cur {
            Some(c) => c.lerp(&target, k),
            None => target,
        };
        d.cur = Some(pose);
        // BottleLiquid: drops 0.025/s while the mouth is under the surface, never below 0.35
        if pose.sip > 0.8 {
            d.fill = (d.fill - 0.025 * dt).max(0.35);
        }
        let r = pose.torso;
        let at = |p: Vec3| D_HIP + r * (p - D_HIP);
        place_part(&mut tfs, rig, "torso", Transform::from_translation(D_HIP).with_rotation(r));
        place_part(&mut tfs, rig, "head", Transform::from_translation(at(D_NECK)).with_rotation(r * pose.head));
        let arm_l = r * pose.arm_l;
        let arm_r = r * pose.arm_r;
        place_part(&mut tfs, rig, "arm_l", Transform::from_translation(at(D_SH_L)).with_rotation(arm_l));
        place_part(&mut tfs, rig, "arm_r", Transform::from_translation(at(D_SH_R)).with_rotation(arm_r));
        let hand = at(D_SH_R) + arm_r * Vec3::new(0.0, -D_ARM - 0.02, 0.0);
        let brot = arm_r * Quat::from_rotation_x(1.0 * (1.0 - pose.sip) + 0.35 * pose.sip);
        let btf = Transform::from_translation(hand + brot * Vec3::new(0.0, -0.08, 0.0)).with_rotation(brot);
        place_part(&mut tfs, rig, "bottle", btf);
        place_part(&mut tfs, rig, "liquid", btf.with_scale(Vec3::new(1.0, d.fill, 1.0)));
    }
}

fn smoke_puffs(time: Res<Time>, mut q: Query<(&mut Puff, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut p, mut tf) in &mut q {
        p.0 = (p.0 + dt / 9.0) % 1.0;
        let k = p.0;
        tf.translation = l(-3.0 + k * 1.6 + (k * 9.0).sin() * 0.15, 8.6 + k * 4.5, 6.0 + k * 0.8);
        tf.scale = Vec3::splat((0.25 + k * 1.1) * (1.0 - smooth(0.75, 1.0, k)).max(0.02));
    }
}

// ------------------------------------------------------------- models ----

const PLASTER: u32 = 0xb9a58a;
const TIMBER: u32 = 0x4a3426;
const FLOORW: u32 = 0x6a4e38;

/// The tavern layout (local: front door plane z = 0, interior x ±4, z 0.4..6.4).
fn tavern_layout() -> Vec<Bx> {
    let w = hex(PLASTER);
    let tb = hex(TIMBER);
    let mut v = vec![
        // front wall: door gap x ±0.65 (2.2 m), glowing windows are visual panes
        bmm(Vec3::new(-4.4, 0.0, 0.0), Vec3::new(-0.65, 5.2, 0.4), w),
        bmm(Vec3::new(0.65, 0.0, 0.0), Vec3::new(4.4, 5.2, 0.4), w),
        bmm(Vec3::new(-0.65, 2.2, 0.0), Vec3::new(0.65, 5.2, 0.4), w),
        // left wall with the open window (z 2..3.4, y 1..2)
        bmm(Vec3::new(-4.4, 0.0, 0.4), Vec3::new(-4.0, 5.2, 2.0), w),
        bmm(Vec3::new(-4.4, 0.0, 3.4), Vec3::new(-4.0, 5.2, 6.4), w),
        bmm(Vec3::new(-4.4, 0.0, 2.0), Vec3::new(-4.0, 1.0, 3.4), w),
        bmm(Vec3::new(-4.4, 2.0, 2.0), Vec3::new(-4.0, 5.2, 3.4), w),
        // right wall, back wall with the back door (x 2.6..3.6)
        bmm(Vec3::new(4.0, 0.0, 0.4), Vec3::new(4.4, 5.2, 6.4), w),
        bmm(Vec3::new(-4.4, 0.0, 6.4), Vec3::new(2.6, 5.2, 6.8), w),
        bmm(Vec3::new(3.6, 0.0, 6.4), Vec3::new(4.4, 5.2, 6.8), w),
        bmm(Vec3::new(2.6, 2.2, 6.4), Vec3::new(3.6, 5.2, 6.8), w),
        // ceiling / upstairs floor around the stairwell (x 2.8..4, z 0.4..4.1)
        bmm(Vec3::new(-4.0, CEIL, 0.4), Vec3::new(2.8, FLOOR2, 6.4), hex(0x5a4030)),
        bmm(Vec3::new(2.8, CEIL, 4.1), Vec3::new(4.0, FLOOR2, 6.4), hex(0x5a4030)),
        // stair rail upstairs
        bmm(Vec3::new(2.7, FLOOR2, 0.4), Vec3::new(2.8, FLOOR2 + 0.9, 4.1), tb).nocam(),
        // counter, fireplace
        bmm(Vec3::new(0.2, 0.0, 5.2), Vec3::new(2.4, 1.05, 5.8), hex(0x5b3b28)),
        bmm(Vec3::new(-3.3, 0.0, 5.9), Vec3::new(-1.7, 1.4, 6.4), hex(0x7a6e66)),
        // upstairs bed and chest
        bmm(Vec3::new(-3.9, FLOOR2, 4.6), Vec3::new(-2.3, FLOOR2 + 0.45, 6.3), hex(0x8a5a4a)).furniture(),
        bmm(Vec3::new(-0.6, FLOOR2, 5.7), Vec3::new(0.4, FLOOR2 + 0.5, 6.3), hex(0x6a4a30)).furniture(),
        // yard wall (gap on the east side, z 9..10.2)
        bmm(Vec3::new(-4.4, 0.0, 6.8), Vec3::new(-4.1, 0.9, 11.6), hex(0x7d7a74)).nocam(),
        bmm(Vec3::new(-4.4, 0.0, 11.3), Vec3::new(4.4, 0.9, 11.6), hex(0x7d7a74)).nocam(),
        bmm(Vec3::new(4.1, 0.0, 6.8), Vec3::new(4.4, 0.9, 9.0), hex(0x7d7a74)).nocam(),
        bmm(Vec3::new(4.1, 0.0, 10.2), Vec3::new(4.4, 0.9, 11.6), hex(0x7d7a74)).nocam(),
    ];
    // stairs: 10 steps of 0.29 m rising toward +z along the right wall
    for k in 1..=10 {
        let z0 = 0.9 + 0.3 * (k - 1) as f32;
        v.push(bmm(Vec3::new(2.9, 0.0, z0), Vec3::new(3.95, 0.29 * k as f32, 4.1), hex(0x6e4c34)));
    }
    // visual: floors, beams, timber framing, shelves of bottles, hearth glow, steps outside
    v.extend([
        bmm(Vec3::new(-4.0, -0.1, 0.4), Vec3::new(4.0, 0.004, 6.4), hex(FLOORW)).vis(),
        bmm(Vec3::new(-0.9, -0.1, -0.9), Vec3::new(0.9, 0.06, 0.0), hex(0x6b6058)).vis(),
        bmm(Vec3::new(-3.1, 0.05, 5.9), Vec3::new(-1.9, 0.75, 6.1), hex(0x1a1210)).vis(),
        bmm(Vec3::new(-3.4, 1.4, 5.8), Vec3::new(-1.6, 1.55, 6.4), hex(TIMBER)).vis(),
        bmm(Vec3::new(0.0, 1.05, 5.15), Vec3::new(2.6, 1.12, 5.85), hex(TIMBER)).vis(),
        bmm(Vec3::new(0.2, 1.45, 6.2), Vec3::new(2.4, 1.5, 6.4), hex(TIMBER)).vis(),
        bmm(Vec3::new(0.2, 1.95, 6.2), Vec3::new(2.4, 2.0, 6.4), hex(TIMBER)).vis(),
    ]);
    for z in [1.5f32, 3.0, 4.5, 6.0] {
        v.push(bmm(Vec3::new(-4.0, 2.4, z - 0.09), Vec3::new(2.8, CEIL, z + 0.09), hex(TIMBER)).vis());
    }
    // exterior timber framing (Tudor-ish)
    for x in [-4.45f32, -2.2, 2.2, 4.25] {
        v.push(bmm(Vec3::new(x, 0.0, -0.05), Vec3::new(x + 0.2, 5.2, 0.0), tb).vis());
    }
    v.push(bmm(Vec3::new(-4.45, 2.55, -0.06), Vec3::new(4.45, 2.75, 0.0), tb).vis());
    v.push(bmm(Vec3::new(-4.45, 5.0, -0.06), Vec3::new(4.45, 5.2, 0.0), tb).vis());
    v.push(bmm(Vec3::new(-0.85, 0.0, -0.06), Vec3::new(-0.65, 2.4, 0.0), tb).vis());
    v.push(bmm(Vec3::new(0.65, 0.0, -0.06), Vec3::new(0.85, 2.4, 0.0), tb).vis());
    v.push(bmm(Vec3::new(-0.85, 2.2, -0.06), Vec3::new(0.85, 2.4, 0.0), tb).vis());
    v
}

fn tavern_model(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    mesh_boxes(&mut m, &tavern_layout());
    // roof: ridge along x, slopes front/back, dark shingles; side gables
    m.with(Transform::from_xyz(0.0, 0.0, 3.4).with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)), |m| {
        roof_slab(m, -3.9, 5.1, 7.3, -4.8, 4.8, 0.18, hex(0x3d3640));
        roof_slab(m, 3.9, 5.1, 7.3, -4.8, 4.8, 0.18, hex(0x3d3640));
        gable(m, 3.4, 5.2, 7.25, -4.42, hex(PLASTER));
        gable(m, 3.4, 5.2, 7.25, 4.42, hex(PLASTER));
    });
    // chimney through the roof
    m.cuboid(Vec3::new(-3.0, 6.5, 6.0), Vec3::new(0.7, 4.2, 0.7), hex(0x6e625c));
    m.cuboid(Vec3::new(-3.0, 8.6, 6.0), Vec3::new(0.85, 0.15, 0.85), hex(0x5a504a));
    // shelf bottles behind the counter
    for (i, c) in [0x3f7a4a, 0x7a3a2a, 0x2f4f6a, 0x8a6a2a, 0x3f7a4a, 0x5a2a4a, 0x2f6a5a, 0x7a5a2a].iter().enumerate() {
        let x = 0.4 + i as f32 * 0.26;
        m.frustum(Vec3::new(x, 1.5, 6.3), 0.05, 0.05, 0.22, 6, hex(*c));
        m.frustum(Vec3::new(x, 1.72, 6.3), 0.05, 0.015, 0.1, 6, hex(*c));
        m.frustum(Vec3::new(x + 0.1, 2.0, 6.3), 0.045, 0.045, 0.2, 6, hex(*c));
    }
    // a lamp sign over the door
    m.tube(Vec3::new(1.1, 2.7, -0.05), Vec3::new(1.1, 2.7, -0.6), 0.03, 0.03, 4, hex(0x2a2a2a));
    m.cuboid(Vec3::new(-1.6, 3.1, -0.12), Vec3::new(1.4, 0.5, 0.06), hex(0x5a3a26));
    m.cuboid(Vec3::new(-1.6, 3.1, -0.16), Vec3::new(0.3, 0.3, 0.03), hex(0xd8b04a));
    p.add(&m);
    // glowing windows (front, upstairs, side), the hearth fire, lanterns
    let mut g = MeshBuilder::new();
    for x in [-2.9f32, 2.9] {
        g.cuboid(Vec3::new(x, 1.45, 0.2), Vec3::new(1.0, 0.9, 0.44), hex(0xffc070));
        g.cuboid(Vec3::new(x, 3.9, 0.2), Vec3::new(0.8, 0.7, 0.44), hex(0xffb060));
    }
    g.cuboid(Vec3::new(4.2, 3.9, 3.4), Vec3::new(0.44, 0.7, 0.9), hex(0xffb060));
    g.cuboid(Vec3::new(1.1, 2.45, -0.6), Vec3::new(0.18, 0.26, 0.18), hex(0xffd090));
    p.add_part(None, &g, Transform::IDENTITY, PartMat::Emissive(hex(0xffb060), 2.2));
    let mut f = MeshBuilder::new();
    for i in 0..5 {
        let x = -2.85 + i as f32 * 0.2;
        f.frustum(Vec3::new(x, 0.08, 6.05), 0.1, 0.0, 0.35 + (i % 2) as f32 * 0.15, 5, hex(0xff8a30));
    }
    f.cuboid(Vec3::new(-2.5, 0.08, 6.1), Vec3::new(0.9, 0.1, 0.2), hex(0x3a2010));
    p.add_part(None, &f, Transform::IDENTITY, PartMat::Emissive(hex(0xff7020), 3.5));
    // window frames (cross bars) on the front panes
    let mut fr = MeshBuilder::new();
    for x in [-2.9f32, 2.9] {
        fr.cuboid(Vec3::new(x, 1.45, -0.03), Vec3::new(0.06, 0.95, 0.04), hex(TIMBER));
        fr.cuboid(Vec3::new(x, 1.45, -0.03), Vec3::new(1.05, 0.06, 0.04), hex(TIMBER));
        fr.cuboid(Vec3::new(x, 3.9, -0.03), Vec3::new(0.06, 0.75, 0.04), hex(TIMBER));
    }
    p.add(&fr);
}

fn leaf(p: &mut ModelParts, w: f32) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 1.1, 0.0), Vec3::new(w - 0.02, 2.2, 0.08), hex(0x5a3a26));
    for y in [0.4f32, 1.1, 1.8] {
        m.cuboid(Vec3::new(0.0, y, 0.0), Vec3::new(w, 0.1, 0.11), hex(0x3a2418));
    }
    m.sphere(Vec3::new(w * 0.36, 1.05, -0.08), 0.04, 2, 5, hex(0xb08a40));
    m.sphere(Vec3::new(w * 0.36, 1.05, 0.08), 0.04, 2, 5, hex(0xb08a40));
    p.add(&m);
}

fn door_leaf(p: &mut ModelParts) {
    leaf(p, 1.3);
}

fn door_back(p: &mut ModelParts) {
    leaf(p, 1.0);
}

/// The drinker's table: worn top, legs, a candle stub and a crowd of bottles.
fn table_model(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.72, 0.0), Vec3::new(1.0, 0.06, 0.86), hex(0x6e4a32));
    for (x, z) in [(-0.44f32, -0.36f32), (0.44, -0.36), (-0.44, 0.36), (0.44, 0.36)] {
        m.cuboid(Vec3::new(x, 0.35, z), Vec3::new(0.07, 0.7, 0.07), hex(0x4a3020));
    }
    let bottles = [(-0.35f32, 0.25f32, 0x3f7a4a, true), (-0.2, 0.3, 0x7a3a2a, false), (0.3, 0.28, 0x2f4f6a, true), (0.38, 0.05, 0x3f7a4a, false), (-0.38, -0.05, 0x8a6a2a, true), (0.12, 0.32, 0x5a2a4a, false), (0.25, -0.25, 0x3f7a4a, true)];
    for (x, z, c, standing) in bottles {
        if standing {
            m.frustum(Vec3::new(x, 0.75, z), 0.045, 0.045, 0.2, 7, hex(c));
            m.frustum(Vec3::new(x, 0.95, z), 0.045, 0.014, 0.1, 7, hex(c));
        } else {
            m.with(Transform::from_xyz(x, 0.795, z).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * Quat::from_rotation_x(x * 3.0)), |m| {
                m.frustum(Vec3::new(0.0, -0.15, 0.0), 0.045, 0.045, 0.2, 7, hex(c));
                m.frustum(Vec3::new(0.0, 0.05, 0.0), 0.045, 0.014, 0.1, 7, hex(c));
            });
        }
    }
    // two bottles on the floor
    for (x, z) in [(0.6f32, 0.5f32), (-0.65, -0.4)] {
        m.with(Transform::from_xyz(x, 0.045, z).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)), |m| {
            m.frustum(Vec3::new(0.0, -0.1, 0.0), 0.045, 0.045, 0.2, 7, hex(0x3f7a4a));
        });
    }
    m.frustum(Vec3::new(-0.05, 0.75, -0.1), 0.04, 0.04, 0.07, 6, hex(0xefe6d0));
    m.frustum(Vec3::new(-0.05, 0.75, 0.1), 0.06, 0.05, 0.1, 7, hex(0xc8d0d0));
    p.add(&m);
    // the pendant lamp above
    let mut lm = MeshBuilder::new();
    lm.tube(Vec3::new(0.0, 1.95, 0.0), Vec3::new(0.0, 2.6, 0.0), 0.012, 0.012, 4, hex(0x2a2a2a));
    lm.frustum(Vec3::new(0.0, 1.85, 0.0), 0.28, 0.08, 0.16, 10, hex(0x3a4a3a));
    p.add(&lm);
    let mut g = MeshBuilder::new();
    g.sphere(Vec3::new(0.0, 1.84, 0.0), 0.07, 3, 6, hex(0xffd090));
    g.frustum(Vec3::new(-0.05, 0.82, -0.1), 0.012, 0.0, 0.04, 4, hex(0xffc060));
    p.add_part(None, &g, Transform::IDENTITY, PartMat::Emissive(hex(0xffc070), 4.0));
}

/// High adult chair (seat 0.46 m), facing +z.
fn chair_high(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let c = hex(0x5e3e2a);
    m.cuboid(Vec3::new(0.0, HIGH_SEAT - 0.03, 0.0), Vec3::new(0.44, 0.06, 0.42), c);
    for (x, z) in [(-0.19f32, -0.18f32), (0.19, -0.18), (-0.19, 0.18), (0.19, 0.18)] {
        m.cuboid(Vec3::new(x, HIGH_SEAT * 0.5, z), Vec3::new(0.05, HIGH_SEAT, 0.05), c);
    }
    m.cuboid(Vec3::new(-0.19, 0.75, -0.19), Vec3::new(0.05, 0.6, 0.05), c);
    m.cuboid(Vec3::new(0.19, 0.75, -0.19), Vec3::new(0.05, 0.6, 0.05), c);
    m.cuboid(Vec3::new(0.0, 0.95, -0.19), Vec3::new(0.44, 0.14, 0.04), c);
    m.cuboid(Vec3::new(0.0, 0.16, 0.0), Vec3::new(0.4, 0.03, 0.03), c);
    p.add(&m);
}

fn chair_low(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let c = hex(0x4e3424);
    m.cuboid(Vec3::new(0.0, 0.44, 0.0), Vec3::new(0.5, 0.06, 0.48), c);
    for (x, z) in [(-0.21f32, -0.2f32), (0.21, -0.2), (-0.21, 0.2), (0.21, 0.2)] {
        m.cuboid(Vec3::new(x, 0.21, z), Vec3::new(0.05, 0.42, 0.05), c);
    }
    m.cuboid(Vec3::new(0.0, 0.85, -0.22), Vec3::new(0.48, 0.7, 0.05), c);
    p.add(&m);
}

/// The drinker, seated (origin = floor under his chair), facing +z.
/// Named parts: torso (hip pivot), head, arm_l, arm_r, bottle, liquid.
fn drinker_model(p: &mut ModelParts) {
    let coat = hex(0x5a5f78);
    let skin = hex(0xe0b8a0);
    let mut legs = MeshBuilder::new();
    for x in [-0.11f32, 0.11] {
        legs.tube(Vec3::new(x, 0.5, 0.0), Vec3::new(x, 0.5, 0.42), 0.075, 0.07, 6, hex(0x4a4238));
        legs.tube(Vec3::new(x, 0.5, 0.42), Vec3::new(x, 0.06, 0.46), 0.065, 0.06, 6, hex(0x4a4238));
        legs.cuboid(Vec3::new(x, 0.04, 0.52), Vec3::new(0.12, 0.08, 0.24), hex(0x2a2018));
    }
    p.add(&legs);
    let mut t = MeshBuilder::new();
    t.with(Transform::from_translation(-D_HIP), |t| {
        t.lathe(D_HIP - Vec3::Y * 0.04, &[(0.0, 0.0), (0.24, 0.0), (0.27, 0.2), (0.25, 0.42), (0.18, 0.58), (0.0, 0.6)], 9, coat);
        t.cuboid(Vec3::new(0.0, 0.85, 0.2), Vec3::new(0.2, 0.3, 0.06), hex(0x8a6a5a));
        t.frustum(Vec3::new(0.0, 1.0, 0.0), 0.16, 0.12, 0.09, 8, hex(0x7a3030));
    });
    p.add_part(Some("torso"), &t, Transform::from_translation(D_HIP), PartMat::Vertex);
    let mut h = MeshBuilder::new();
    h.ellipsoid(Vec3::new(0.0, 0.14, 0.0), Vec3::new(0.14, 0.16, 0.14), 4, 9, skin);
    h.sphere(Vec3::new(0.0, 0.12, 0.15), 0.045, 3, 6, hex(0xd0505a));
    h.ellipsoid(Vec3::new(0.0, 0.05, 0.06), Vec3::new(0.13, 0.07, 0.1), 2, 8, hex(0x6a6058));
    h.sphere(Vec3::new(0.05, 0.18, 0.12), 0.016, 2, 4, hex(0x2b2b3a));
    h.sphere(Vec3::new(-0.05, 0.18, 0.12), 0.016, 2, 4, hex(0x2b2b3a));
    h.cuboid(Vec3::new(0.05, 0.205, 0.125), Vec3::new(0.06, 0.012, 0.02), hex(0x4a3a30));
    h.cuboid(Vec3::new(-0.05, 0.205, 0.125), Vec3::new(0.06, 0.012, 0.02), hex(0x4a3a30));
    h.frustum(Vec3::new(0.0, 0.2, -0.01), 0.15, 0.1, 0.12, 9, hex(0x8a4a3a));
    h.sphere(Vec3::new(0.0, 0.33, -0.01), 0.04, 2, 5, hex(0x8a4a3a));
    p.add_part(Some("head"), &h, Transform::from_translation(D_NECK), PartMat::Vertex);
    let a = arm_mesh(D_ARM, 0.065, 0.05, coat, skin);
    p.add_part(Some("arm_l"), &a, Transform::from_translation(D_SH_L), PartMat::Vertex);
    p.add_part(Some("arm_r"), &a, Transform::from_translation(D_SH_R), PartMat::Vertex);
    // the bottle (origin at its middle) and its liquid (scaled by fill)
    let mut b = MeshBuilder::new();
    b.frustum(Vec3::new(0.0, -0.11, 0.0), 0.048, 0.048, 0.2, 8, hex(0x2f6a4a));
    b.frustum(Vec3::new(0.0, 0.09, 0.0), 0.048, 0.016, 0.12, 8, hex(0x2f6a4a));
    p.add_part(Some("bottle"), &b, Transform::IDENTITY, PartMat::Glass(Color::srgba(0.2, 0.5, 0.32, 0.55)));
    let mut q = MeshBuilder::new();
    q.frustum(Vec3::new(0.0, -0.105, 0.0), 0.04, 0.04, 0.2, 8, hex(0x9a5a20));
    p.add_part(Some("liquid"), &q, Transform::IDENTITY, PartMat::Vertex);
}

fn moon(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.sphere(Vec3::ZERO, 6.0, 8, 14, hex(0xf4ecd0));
    m.sphere(Vec3::new(2.0, 1.5, -5.0), 1.3, 3, 6, hex(0xd8d0b4));
    m.sphere(Vec3::new(-2.2, -1.2, -5.1), 1.0, 3, 6, hex(0xd8d0b4));
    p.add_part(None, &m, Transform::IDENTITY, PartMat::Emissive(hex(0xf4ecd0), 1.4));
}

fn smoke(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.sphere(Vec3::ZERO, 0.5, 3, 7, hex(0x9aa0b0));
    p.add_part(None, &m, Transform::IDENTITY, PartMat::Glass(Color::srgba(0.62, 0.65, 0.72, 0.35)));
}

fn barrel(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.0, 0.0), (0.32, 0.0), (0.38, 0.45), (0.32, 0.9), (0.0, 0.9)], 10, hex(0x7a5232));
    m.frustum(Vec3::Y * 0.18, 0.355, 0.36, 0.05, 10, hex(0x3a3a3a));
    m.frustum(Vec3::Y * 0.7, 0.36, 0.355, 0.05, 10, hex(0x3a3a3a));
    p.add(&m);
}
