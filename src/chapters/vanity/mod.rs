//! Chapter 3 — The Conceited Man ("Theater_Main" in the original).
//!
//! Follows the original VanityStory / VanityClap / VanityDoor / VanityPerformer
//! design (docs/original/vanity.md):
//! - an old tent-like theatre on a meadow island; door plane z = 0, hall
//!   interior x ∈ [−7.4, 7.4], z ∈ [0.6, 17.6]; inward double door (105° in
//!   1.25 s; push from outside, pull from inside, close; never through the
//!   player); child-size audience chairs (seat 0.30 m) you can sit in; a stage
//!   with steps; the vain man on stage (z ≈ 14.5); a big mirror on the +x side;
//!   warm flickering lamps;
//! - clapping with C (inside the hall only, standing or seated): a 2.03 s clip
//!   with palm contacts at 0.63 / 0.93 / 1.27 s, a different dry clap sound per
//!   contact; the vain man answers EVERY clap with a hat salute (on the third
//!   contact);
//! - the talk (prompt within 4.6 m, inside the hall): the prince walks to the
//!   stance (0, 11.15) at 0.9 m/s, two-shot → 16 s push-in, "clap your hands",
//!   two claps each answered by a salute (if the player does not clap within
//!   4 s, the prince claps by himself), the admire exchange, the look back at
//!   the empty rows (L6 2.4 s into it), the shrug 2.2 s into L7, and the final
//!   hall shot: the prince turns away, the vain man raises his hat to the
//!   empty hall. All six camera shots use the original's world positions/FOV.

pub mod kit;

use crate::core::interact::InteractFocus;
use crate::core::prelude::*;
use kit::*;

pub struct Vanity;

const ID: &str = "vanity";

/// Hall interior (VanityClap.interiorX/Z).
const HALL_X: f32 = 7.4;
const HALL_Z: (f32, f32) = (0.6, 17.6);
/// The talk stance (foot of the stage steps) and the vain man's spot.
const STANCE: Vec3 = Vec3::new(0.0, 0.0, 11.15);
const VAIN_POS: Vec3 = Vec3::new(0.0, 0.9, 14.5);
const STAGE_Y: f32 = 0.9;
/// Clap clip (LP_Clap: 62 frames @ 30 fps) and its palm contacts.
const CLAP_LEN: f32 = 2.03;
const CLAP_CONTACTS: [f32; 3] = [0.63, 0.93, 1.27];
/// Seat rows (z) and seats per side (|x|).
const ROWS: [f32; 5] = [3.0, 4.3, 5.6, 6.9, 8.2];
const SEAT_X: [f32; 6] = [1.4, 2.4, 3.4, 4.4, 5.4, 6.4];
const SEAT_H: f32 = 0.30;

pub const STRINGS: StringTable = &[
    ("vanity.title", "爱虚荣的人的星球", "The Conceited Man's Planet"),
    ("vanity.subtitle", "一颗从来没有人经过的星星，住着一个爱虚荣的人", "A star nobody ever passes, home to a conceited man"),
    ("vanity.line", "一座旧剧场，台上的人一直在等观众。", "An old theatre; the man on stage is always waiting for an audience."),
    ("vanity.visited", "帽子还在举着，座位还是空的。", "The hat is still raised; the seats are still empty."),
    ("vanity.loading", "远处传来稀稀落落的掌声……", "Faint applause drifts from far away…"),
    ("vanity.n1", "这里住着一个爱虚荣的人。在他的耳朵里，世上只有称赞。", "A conceited man lives here. To his ears, there is nothing in the world but praise."),
    ("vanity.loc", "— 旧剧场 —", "— The Old Theatre —"),
    ("vanity.ev.meet", "去见见台上的那个人", "Go and see the man on the stage"),
    ("vanity.st.meet", "走进剧场，和台上的人说话", "Go into the theatre and speak to the man on stage"),
    ("vanity.target.vain", "爱虚荣的人", "The conceited man"),
    ("vanity.verb.talk", "说话", "Talk"),
    ("vanity.verb.push", "推门", "Push the door"),
    ("vanity.verb.pull", "拉门", "Pull the door"),
    ("vanity.verb.close", "关门", "Close the door"),
    ("vanity.verb.sit", "坐下", "Sit down"),
    ("vanity.verb.stand", "站起来", "Stand up"),
    ("vanity.teach.clap", "鼓掌", "Clap"),
    // the talk (our own wording)
    ("vanity.l1", "啊！啊！终于来了一个崇拜我的人！", "Ah! Ah! At last, an admirer has come to see me!"),
    ("vanity.l2", "来，拍拍手吧！", "Come now — clap your hands!"),
    ("vanity.l3", "你真的非常非常仰慕我，对不对？", "You really do admire me ever so much, don't you?"),
    ("vanity.l4", "「仰慕」是什么意思呀？", "What does \"admire\" mean?"),
    ("vanity.l5", "仰慕嘛，就是承认我是这颗星球上最英俊、穿得最体面、最有钱、也最聪明的人。", "To admire means to agree that I am the handsomest, the best-dressed, the richest and the cleverest man on this planet."),
    ("vanity.l6", "可是，这颗星球上只有你一个人呀！", "But you are the only person on this planet!"),
    ("vanity.l7", "求求你，就让我高兴一下——照样仰慕我吧！", "Please, do me this kindness — admire me all the same!"),
    ("vanity.l8", "我仰慕你。可是，这对你又有什么要紧的呢？", "I admire you. But what difference could that possibly make to you?"),
];

impl Chapter for Vanity {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 3,
            title: "vanity.title",
            subtitle: "vanity.subtitle",
            line: "vanity.line",
            visited_line: "vanity.visited",
            loading: "vanity.loading",
            map_pos: Vec2::new(650.0, 640.0),
            map_radius: 50.0,
            map_colors: (hex(0xb98fb0), hex(0xf3c86b)),
            music: "music.vanity",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("vanity.theatre", theatre_model)
            .register_model("vanity.door_leaf", door_leaf)
            .register_model("vanity.seat_row", seat_row)
            .register_model("vanity.vain", vain_man)
            .register_model("vanity.mirror", mirror)
            .register_model("vanity.footlights", footlights)
            .register_model("vanity.poster", poster)
            .register_model("vanity.pennant", pennant)
            .init_resource::<VState>();
        kit::register(app, ID);
        app.add_systems(Update, (door_logic, seats, clap, talk_gate, animate_vain).chain().in_set(CoreSet::Gameplay).run_if(in_chapter(ID)));
    }

    fn build(&self, cx: &mut ChapterCx) {
        cx.world.insert_resource(VState::default());
        cx.world.insert_resource(PrinceOverride::default());
        cx.world.resource_mut::<crate::core::player::PlayerTuning>().walk_speed = 0.9;

        // --- the island: a meadow dome, flattened under the theatre
        let mut p = PlanetParams::sized(22.0);
        p.center = Vec2::new(0.0, 6.0);
        p.cap_flatten = 0.5;
        p.ground = hex(0x93ad68);
        p.ground2 = hex(0x7f9c58);
        p.dry = hex(0xc9b98a);
        p.rock = hex(0x8d7a86);
        p.mods = vec![
            TerrainMod::Flatten { center: Vec2::new(0.0, 9.0), radius: 12.0, blend: 3.5, height: Some(0.0) },
            TerrainMod::Flatten { center: Vec2::new(0.0, -2.5), radius: 3.0, blend: 2.0, height: Some(0.0) },
            TerrainMod::Paint { center: Vec2::new(0.0, -5.0), radius: 1.2, blend: 0.8, color: hex(0xc4ae86) },
            TerrainMod::Paint { center: Vec2::new(0.6, -8.0), radius: 1.1, blend: 0.8, color: hex(0xc4ae86) },
            TerrainMod::Paint { center: Vec2::new(1.4, -10.8), radius: 1.0, blend: 0.8, color: hex(0xc4ae86) },
            TerrainMod::Paint { center: Vec2::new(0.0, -2.0), radius: 2.2, blend: 1.0, color: hex(0xc4ae86) },
        ];
        cx.planet(p);
        // footsteps: the old theatre's wooden floor
        cx.step_zone(Vec3::new(0.0, 2.0, (HALL_Z.0 + HALL_Z.1) * 0.5), Vec3::new(HALL_X, 3.0, (HALL_Z.1 - HALL_Z.0) * 0.5), "sfx.step_wood");
        let mut pal = SkyPalette::default();
        pal.day_horizon = hex(0xe9d9e6);
        pal.golden_horizon = hex(0xf5c7a0);
        cx.sky(15.0, 0.5, Some(17.5), pal);
        cx.ambience("amb.wind");

        // --- the theatre (one layout table → mesh + colliders)
        cx.model("model.vanity.theatre", Vec3::ZERO, 0.0, 1.0);
        spawn_colliders(cx, Vec3::ZERO, &theatre_layout());
        // ceiling collider keeps the camera inside the hall
        cx.spawn((Transform::from_xyz(0.0, 5.0, 9.0), Collider::cuboid(Vec3::new(15.6, 0.4, 18.0))));

        // double door: hinges at x = ±1.6 on the door plane, inward (+z)
        let leaf = Vec3::new(1.6, 3.1, 0.08);
        spawn_door(cx, Door::new("door_l", 0.0, -1.0, 105.0, 1.6, 1.25, Vec2::Y), Vec3::new(-1.6, 0.0, 0.3), "model.vanity.door_leaf", Vec3::new(0.8, 0.0, 0.0), leaf);
        spawn_door(cx, Door::new("door_r", 0.0, 1.0, 105.0, 1.6, 1.25, Vec2::Y), Vec3::new(1.6, 0.0, 0.3), "model.vanity.door_leaf", Vec3::new(-0.8, 0.0, 0.0), leaf);
        let door = cx.empty(Vec3::new(0.0, 0.0, 0.3), 0.0);
        cx.interactable(door, Interactable::new("door", "vanity.verb.push").radius(2.3).focus(Vec3::Y * 1.2).prompt_height(0.6).max_angle(80.0));
        cx.anchor("door_out", Vec3::new(0.0, 0.0, -0.75), 0.0);

        // seats: one model per half-row, one sit interactable per chair
        for (ri, z) in ROWS.iter().enumerate() {
            for side in [-1.0f32, 1.0] {
                let e = cx.model("model.vanity.seat_row", Vec3::new(side * 3.9, 0.0, *z), 0.0, 1.0);
                let _ = e;
                cx.spawn((Transform::from_xyz(side * 3.9, 0.0, *z - 0.05), Collider::cuboid(Vec3::new(5.6, 0.75, 0.6)).not_walkable().no_camera()));
                for (si, x) in SEAT_X.iter().enumerate() {
                    let id = format!("seat_{ri}_{}_{si}", if side < 0.0 { "l" } else { "r" });
                    let s = cx.empty(Vec3::new(side * x, 0.0, *z), 0.0);
                    cx.interactable(s, Interactable::new(&id, "vanity.verb.sit").radius(1.05).focus(Vec3::Y * SEAT_H).prompt_height(0.7));
                }
            }
        }

        // stage dressing
        cx.model("model.vanity.footlights", Vec3::new(0.0, STAGE_Y, 12.42), 0.0, 1.0);
        let m = cx.model("model.vanity.mirror", Vec3::new(5.0, STAGE_Y, 15.6), -125.0, 1.0);
        cx.collider(m, Collider::cylinder(0.55, 2.6).no_camera());

        // the vain man (faces the hall, −z)
        let vain = cx.model("model.vanity.vain", VAIN_POS, 180.0, 1.0);
        cx.world.entity_mut(vain).insert((Rig::default(), VainMan::default()));
        cx.collider(vain, Collider::cylinder(0.4, 1.8));
        cx.interactable(vain, Interactable::new("vain", "vanity.verb.talk").radius(4.6).focus(Vec3::Y * 1.1).prompt_height(1.1).disabled().max_angle(70.0));
        cx.anchor("vain", VAIN_POS, 180.0);
        cx.anchor("vain_head", VAIN_POS + Vec3::Y * 1.55, 180.0);

        // warm lamps (flicker), stage light, door lanterns
        lamp(cx, Vec3::new(0.0, 4.0, 14.0), hex(0xffc27a), 600_000.0, 16.0, 0.06, 0.0);
        lamp(cx, Vec3::new(-3.0, 4.2, 6.0), hex(0xffb466), 260_000.0, 14.0, 0.12, 1.3);
        lamp(cx, Vec3::new(3.0, 4.2, 6.0), hex(0xffb466), 260_000.0, 14.0, 0.12, 2.9);
        lamp(cx, Vec3::new(0.0, 3.4, -1.0), hex(0xffc98a), 40_000.0, 7.0, 0.15, 4.1);

        // posters, pennant
        cx.model("model.vanity.poster", Vec3::new(-4.2, 1.2, -0.03), 180.0, 1.0);
        cx.model("model.vanity.poster", Vec3::new(4.2, 1.2, -0.03), 180.0, 1.0);
        cx.model("model.vanity.pennant", Vec3::new(0.0, 8.2, 0.0), 0.0, 1.0);

        // the meadow (VanityMeadowInstances)
        let avoid = [(Vec2::new(0.0, 9.0), 10.8), (Vec2::new(0.0, -2.0), 2.4), (Vec2::new(0.6, -7.0), 1.6), (Vec2::new(1.4, -10.5), 1.4)];
        cx.scatter("model.grass_tuft", 150, 2.0, 21.0, 31, (0.8, 1.5), &avoid);
        cx.scatter("model.flower_purple", 20, 4.0, 20.0, 32, (0.7, 1.1), &avoid);
        cx.scatter("model.flower_yellow", 18, 4.0, 20.0, 33, (0.7, 1.1), &avoid);
        cx.scatter("model.flower", 24, 4.0, 20.0, 34, (0.8, 1.2), &avoid);
        cx.scatter("model.rock", 8, 13.0, 21.0, 35, (0.6, 1.3), &avoid);
        cx.scatter("model.bush", 7, 12.0, 20.0, 36, (0.8, 1.3), &avoid);

        let start = cx.ground(1.6, -12.0);
        cx.player(start, -6.0);
    }

    fn story(&self) -> Story {
        let cam0 = Vec3::new(29.0, 13.0, -29.0);
        let look0 = Vec3::new(2.6, 4.3, -14.7);
        let push = (look0 - cam0).normalize() * 3.0;
        Story {
            events: vec![event("vanity.ev.meet", &["vanity.st.meet"])],
            hidden: None,
            script: vec![
                // opening shot (world): push 3 m, settle 4 s, under the card
                push_cmd((cam0, look0, 55.0), (cam0 + push, look0 + push, 55.0), 4.0),
                card(),
                run(|w: &mut World| end_shot_to(w, Some(-6.0), Some(14.0), 1.6)),
                narrate(&["vanity.n1"]),
                Cmd::Mono("vanity.loc", 3.0),
                start_event(0),
                target(Pos::anchor("vain_head"), "vanity.target.vain"),
                flag("talk_ready"),
                wait_flag(
                    "v_talk",
                    vec![t_use("door"), t_goto(Pos::World(Vec3::new(0.0, 0.0, 6.0))), t_use("vain")],
                ),
                // ---- phase talk
                clear_target(),
                run(|w: &mut World| {
                    w.resource_mut::<VState>().talking = true;
                }),
                walk_to(Pos::World(STANCE)),
                run(|w: &mut World| set_drive(w, Drive::External)),
                face(Pos::anchor("vain_head")),
                shot_cmd(Vec3::new(1.75, 1.6, 8.2), Vec3::new(0.15, 1.95, 13.0), 0.9, 44.0),
                wait(1.0),
                // slow push-in over 16 s through L1-L2 and the claps
                shot_cmd(Vec3::new(1.5, 1.6, 8.7), Vec3::new(0.1, 2.0, 13.3), 16.0, 42.0),
                run(|w: &mut World| gesture(w, Gesture::TeachClap)),
                say(speakers::VAIN_MAN, "vanity.l1"),
                say_for(speakers::VAIN_MAN, "vanity.l2", 2.3),
                Cmd::Teach(vec!["C"], "vanity.teach.clap", 8.0),
                run(|w: &mut World| clap_wait(w, 1)),
                wait_until(|w: &World| w.resource::<VState>().salutes >= 1, vec![TestAction::PressAction2, t_wait(7.0)]),
                run(|w: &mut World| clap_wait(w, 2)),
                wait_until(|w: &World| w.resource::<VState>().salutes >= 2, vec![TestAction::PressAction2, t_wait(7.0)]),
                run(|w: &mut World| clap_wait(w, 0)),
                wait(0.7),
                shot_cmd(Vec3::new(1.05, 1.95, 10.3), Vec3::new(0.0, 2.75, 14.4), 2.2, 34.0),
                run(|w: &mut World| gesture(w, Gesture::Salute)),
                say(speakers::VAIN_MAN, "vanity.l3"),
                say(speakers::PRINCE, "vanity.l4"),
                run(|w: &mut World| gesture(w, Gesture::Plead)),
                say(speakers::VAIN_MAN, "vanity.l5"),
                // the look back at the empty rows
                shot_cmd(Vec3::new(2.3, 1.8, 11.9), Vec3::new(0.0, 1.3, 11.0), 1.1, 46.0),
                face(Pos::World(Vec3::new(0.0, 0.0, 4.0))),
                run(|w: &mut World| gesture(w, Gesture::Idle)),
                wait(1.1),
                shot_cmd(Vec3::new(1.5, 2.55, 13.3), Vec3::new(0.3, 0.95, 9.4), 1.3, 48.0),
                wait(1.3),
                say(speakers::PRINCE, "vanity.l6"),
                face(Pos::anchor("vain_head")),
                run(|w: &mut World| {
                    gesture(w, Gesture::Plead);
                    w.resource_mut::<VState>().shrug_at = Some(2.2);
                }),
                say(speakers::VAIN_MAN, "vanity.l7"),
                say(speakers::PRINCE, "vanity.l8"),
                wait(1.2),
                // hall shot: behind the vain man's left shoulder; the prince turns away
                shot_cmd(Vec3::new(1.4, 2.95, 16.15), Vec3::new(-0.55, 1.4, 8.0), 2.5, 54.0),
                face(Pos::World(Vec3::new(0.0, 0.0, 2.0))),
                run(|w: &mut World| gesture(w, Gesture::SaluteHall)),
                wait(2.5),
                // ---- phase depart
                run(|w: &mut World| {
                    w.resource_mut::<VState>().talking = false;
                    set_drive(w, Drive::Player);
                    end_shot_to(w, Some(180.0), Some(26.0), 1.6);
                }),
                complete_event(0),
                departure(),
            ],
        }
    }
}


// ------------------------------------------------------------- state ----

#[derive(Resource, Default)]
struct VState {
    /// Clap clip time (None = not clapping) and contacts already sounded.
    clap_t: Option<f32>,
    contacts: usize,
    last_sound: usize,
    claps: u32,
    salutes: u32,
    /// Talk clap phase: wanted clap count (0 = off) and time waiting for it.
    clap_target: u32,
    clap_wait_t: f32,
    talking: bool,
    shrug_at: Option<f32>,
    shrug_t: f32,
    /// Door push/pull sequence.
    door_seq: Option<DoorSeq>,
    seated: Option<(Vec3, f32)>,
    in_hall_once: bool,
}

#[derive(Clone, Copy, Debug)]
struct DoorSeq {
    stage: u8,
    t: f32,
    pull: bool,
}

fn clap_wait(w: &mut World, n: u32) {
    let mut s = w.resource_mut::<VState>();
    s.clap_target = n;
    s.clap_wait_t = 0.0;
}

fn in_hall(p: Vec3) -> bool {
    p.x.abs() < HALL_X && p.z > HALL_Z.0 && p.z < HALL_Z.1
}

// --------------------------------------------------------------- door ----

#[allow(clippy::too_many_arguments)]
fn door_logic(
    time: Res<Time>,
    mut msgs: MessageReader<Interacted>,
    mut st: ResMut<VState>,
    mut doors: Query<&mut Door>,
    mut its: Query<&mut Interactable>,
    mut player: Query<(&mut Player, &Transform)>,
    mut audio: MessageWriter<AudioCmd>,
) {
    let dt = time.delta_secs();
    let Ok((mut pl, ptf)) = player.single_mut() else { return };
    let pp = ptf.translation;
    let open = doors.iter().any(|d| d.is_open());
    let moving = doors.iter().any(|d| (d.open - d.target).abs() > 1e-3);
    for m in msgs.read() {
        if m.id != "door" || st.door_seq.is_some() || moving {
            continue;
        }
        if open {
            for mut d in &mut doors {
                d.target = 0.0;
            }
            audio.write(AudioCmd::Sfx("sfx.creak".into(), 0.5));
        } else if pp.z < 0.3 {
            // outside: walk to the push stance, align, two-hand push
            pl.drive = Drive::WalkTo { target: Vec2::new(0.0, -0.75), speed: 0.9, stop_radius: 0.12 };
            st.door_seq = Some(DoorSeq { stage: 0, t: 0.0, pull: false });
        } else {
            // inside: step clear of the swing, then pull
            pl.drive = Drive::WalkTo { target: Vec2::new(pp.x.clamp(-0.6, 0.6), 2.45), speed: 0.9, stop_radius: 0.12 };
            st.door_seq = Some(DoorSeq { stage: 0, t: 0.0, pull: true });
        }
    }
    if let Some(mut seq) = st.door_seq {
        seq.t += dt;
        match seq.stage {
            0 => {
                if pl.is_arrived() || seq.t > 4.0 {
                    pl.drive = Drive::External;
                    pl.face_target = Some(if seq.pull { std::f32::consts::PI } else { 0.0 });
                    seq.stage = 1;
                    seq.t = 0.0;
                }
            }
            1 => {
                // align 0.25 s, then the leaves swing (1.25 s)
                if seq.t > 0.25 {
                    pl.pose = Pose::Reach;
                    for mut d in &mut doors {
                        d.target = 1.0;
                    }
                    audio.write(AudioCmd::Sfx("sfx.creak".into(), 0.6));
                    seq.stage = 2;
                    seq.t = 0.0;
                }
            }
            _ => {
                if seq.t > 1.0 {
                    pl.pose = Pose::None;
                    pl.drive = Drive::Player;
                    st.door_seq = None;
                    return;
                }
            }
        }
        st.door_seq = Some(seq);
    }
    // the prompt follows the state and side
    for mut it in &mut its {
        if it.id == "door" {
            let key = if open { "vanity.verb.close" } else if pp.z < 0.3 { "vanity.verb.push" } else { "vanity.verb.pull" };
            if it.prompt != key {
                it.prompt = key.to_string();
            }
            it.enabled = !st.talking && st.door_seq.is_none() && !moving;
        }
    }
}

// -------------------------------------------------------------- seats ----

fn seats(
    mut commands: Commands,
    mut msgs: MessageReader<Interacted>,
    mut st: ResMut<VState>,
    mut its: Query<(&mut Interactable, &GlobalTransform)>,
) {
    for m in msgs.read() {
        if !m.id.starts_with("seat_") {
            continue;
        }
        let Ok((_, gt)) = its.get(m.entity) else { continue };
        let p = gt.translation();
        if st.seated.is_some() {
            let (seat, _) = st.seated.take().unwrap();
            commands.queue(move |w: &mut World| stand_player(w, Vec3::new(seat.x, 0.0, seat.z + 0.68), 0.0));
        } else {
            st.seated = Some((p, 0.0));
            commands.queue(move |w: &mut World| sit_player(w, Vec3::new(p.x, SEAT_H, p.z + 0.02), 0.0));
        }
    }
    // while seated only "my" chair is offered (as "stand up")
    let seated = st.seated.map(|s| s.0);
    for (mut it, gt) in &mut its {
        if !it.id.starts_with("seat_") {
            continue;
        }
        match seated {
            Some(s) => {
                let mine = gt.translation().distance(s) < 0.05;
                it.enabled = mine && !st.talking;
                let key = "vanity.verb.stand";
                if mine && it.prompt != key {
                    it.prompt = key.to_string();
                }
            }
            None => {
                it.enabled = !st.talking;
                if it.prompt != "vanity.verb.sit" {
                    it.prompt = "vanity.verb.sit".to_string();
                }
            }
        }
    }
}

// --------------------------------------------------------------- clap ----

#[allow(clippy::too_many_arguments)]
fn clap(
    time: Res<Time>,
    input: Res<InputFrame>,
    lock: Res<crate::core::story::StoryLock>,
    mut touch: ResMut<crate::core::input::TouchControls>,
    mut st: ResMut<VState>,
    mut ov: ResMut<PrinceOverride>,
    mut hud: ResMut<Hud>,
    mut audio: MessageWriter<AudioCmd>,
    player: Query<&Transform, With<Player>>,
    mut vain: Query<&mut VainMan>,
    mut rng: Local<Option<Rng>>,
) {
    let dt = time.delta_secs();
    let rng = rng.get_or_insert_with(|| Rng::new(612));
    let Ok(ptf) = player.single() else { return };
    let inside = in_hall(ptf.translation);
    let label = if inside { Some("vanity.teach.clap") } else { None };
    if touch.action2_label != label {
        touch.action2_label = label;
    }
    if inside && !st.in_hall_once {
        st.in_hall_once = true;
    }
    // talk phase: wait ≤ 4 s for the player's clap, then the prince claps himself
    let mut start = input.action2 && inside && !lock.0;
    if st.clap_target > 0 && st.claps < st.clap_target && st.clap_t.is_none() {
        st.clap_wait_t += dt;
        if st.clap_wait_t > 4.0 {
            start = true;
        }
    }
    if start && st.clap_t.is_none() {
        st.clap_t = Some(0.0);
        st.contacts = 0;
        st.clap_wait_t = 0.0;
    }
    if let Some(t) = st.clap_t {
        let t = t + dt;
        while st.contacts < CLAP_CONTACTS.len() && t >= CLAP_CONTACTS[st.contacts] {
            // a random dry clap, never the same twice in a row
            let mut i = (rng.next_u32() % 4) as usize;
            if i == st.last_sound {
                i = (i + 1) % 4;
            }
            st.last_sound = i;
            audio.write(AudioCmd::Sfx(format!("sfx.vanity.clap{i}"), 0.6));
            st.contacts += 1;
            if st.contacts == 3 {
                st.claps += 1;
                // the vain man answers every clap with a salute
                if let Ok(mut v) = vain.single_mut() {
                    v.play(Gesture::Salute);
                    v.counting = true;
                }
            }
        }
        let (l, r) = clap_arms(t, &CLAP_CONTACTS);
        ov.arm_l = Some(l);
        ov.arm_r = Some(r);
        st.clap_t = if t >= CLAP_LEN { None } else { Some(t) };
        if st.clap_t.is_none() {
            ov.arm_l = None;
            ov.arm_r = None;
        }
    }
    // salutes finished → counted
    if let Ok(mut v) = vain.single_mut() {
        if v.counting && v.gesture == Gesture::Salute && v.t >= v.gesture.duration() {
            v.counting = false;
            st.salutes += 1;
        }
    }
    // the shrug, 2.2 s into L7
    if let Some(at) = st.shrug_at {
        st.shrug_t += dt;
        let t = st.shrug_t - at;
        if t >= 0.0 {
            let k = smooth(0.0, 0.25, t) * (1.0 - smooth(1.0, 1.4, t));
            if t > 1.4 {
                st.shrug_at = None;
                st.shrug_t = 0.0;
                ov.arm_l = None;
                ov.arm_r = None;
                ov.head = None;
            } else {
                ov.arm_l = Some(Quat::from_rotation_x(-0.35 * k) * Quat::from_rotation_z(0.75 * k));
                ov.arm_r = Some(Quat::from_rotation_x(-0.35 * k) * Quat::from_rotation_z(-0.75 * k));
                ov.head = Some(Quat::from_rotation_z(0.18 * k));
            }
        }
    }
    let _ = &mut hud;
}

// ------------------------------------------------------------- talking ----

fn talk_gate(
    runner: Res<StoryRunner>,
    st: Res<VState>,
    mut msgs: MessageReader<Interacted>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut its: Query<&mut Interactable>,
    focus: Res<InteractFocus>,
) {
    let Ok(ptf) = player.single() else { return };
    let ok = runner.has_flag("talk_ready") && !runner.has_flag("v_talk") && in_hall(ptf.translation) && st.seated.is_none();
    for mut it in &mut its {
        if it.id == "vain" && it.enabled != ok {
            it.enabled = ok;
        }
    }
    for m in msgs.read() {
        if m.id == "vain" {
            commands.queue(|w: &mut World| crate::core::story::set_flag(w, "v_talk"));
        }
    }
    let _ = focus;
}

// ------------------------------------------------------- the vain man ----

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Gesture {
    #[default]
    Idle,
    TeachClap,
    Salute,
    Plead,
    SaluteHall,
}

impl Gesture {
    fn duration(self) -> f32 {
        match self {
            Gesture::Idle => f32::MAX,
            Gesture::TeachClap => 2.6,
            Gesture::Salute => 1.9,
            Gesture::Plead => 3.2,
            Gesture::SaluteHall => f32::MAX,
        }
    }
}

#[derive(Component, Default)]
struct VainMan {
    gesture: Gesture,
    t: f32,
    idle_t: f32,
    counting: bool,
    cur: Option<VPose>,
}

impl VainMan {
    fn play(&mut self, g: Gesture) {
        self.gesture = g;
        self.t = 0.0;
    }
}

fn gesture(w: &mut World, g: Gesture) {
    let mut q = w.query::<&mut VainMan>();
    if let Some(mut v) = q.iter_mut(w).next() {
        v.play(g);
    }
}

/// One pose of the vain man's rig.
#[derive(Clone, Copy, Debug)]
struct VPose {
    torso: Quat,
    head: Quat,
    arm_l: Quat,
    arm_r: Quat,
    /// 0 = hat on the head, 1 = hat in the right hand.
    hat: f32,
}

impl VPose {
    fn lerp(&self, o: &VPose, k: f32) -> VPose {
        VPose {
            torso: self.torso.slerp(o.torso, k),
            head: self.head.slerp(o.head, k),
            arm_l: self.arm_l.slerp(o.arm_l, k),
            arm_r: self.arm_r.slerp(o.arm_r, k),
            hat: self.hat + (o.hat - self.hat) * k,
        }
    }
}

const HIP: Vec3 = Vec3::new(0.0, 0.92, 0.0);
const NECK: Vec3 = Vec3::new(0.0, 1.5, 0.0);
const SHOULDER_L: Vec3 = Vec3::new(0.26, 1.43, 0.0);
const SHOULDER_R: Vec3 = Vec3::new(-0.26, 1.43, 0.0);
const ARM_LEN: f32 = 0.6;

fn idle_pose(t: f32) -> VPose {
    // proud idle: chest out, chin up, left hand on the hip (12 s loop)
    let sway = (t * std::f32::consts::TAU / 12.0).sin();
    let breathe = (t * 1.4).sin();
    VPose {
        torso: Quat::from_rotation_x(-0.06 + breathe * 0.01) * Quat::from_rotation_z(sway * 0.03),
        head: Quat::from_rotation_x(-0.12),
        arm_l: Quat::from_rotation_z(0.75) * Quat::from_rotation_x(0.35),
        arm_r: Quat::from_rotation_z(-0.12) * Quat::from_rotation_x(-0.05 + sway * 0.05),
        hat: 0.0,
    }
}

fn gesture_pose(g: Gesture, t: f32, idle: VPose) -> VPose {
    match g {
        Gesture::Idle => idle,
        Gesture::TeachClap => {
            // shows how it is done: two big claps above the chest
            let up = smooth(0.0, 0.35, t) * (1.0 - smooth(2.1, 2.6, t));
            let c = ((t - 0.35).max(0.0) * std::f32::consts::TAU * 1.4).cos() * 0.5 + 0.5;
            let open = 0.15 + 0.5 * c;
            VPose {
                torso: Quat::from_rotation_x(-0.1 * up),
                head: Quat::from_rotation_x(-0.15),
                arm_l: idle.arm_l.slerp(Quat::from_rotation_x(-1.5) * Quat::from_rotation_z(-open), up),
                arm_r: idle.arm_r.slerp(Quat::from_rotation_x(-1.5) * Quat::from_rotation_z(open), up),
                hat: 0.0,
            }
        }
        Gesture::Salute | Gesture::SaluteHall => {
            // grab the hat, lift it high, bow; put it back (held for the hall)
            let hold = g == Gesture::SaluteHall;
            let reach = smooth(0.0, 0.35, t);
            let lift = smooth(0.3, 0.75, t) * if hold { 1.0 } else { 1.0 - smooth(1.15, 1.6, t) };
            let back = if hold { 0.0 } else { smooth(1.4, 1.85, t) };
            let bow = smooth(0.5, 0.9, t) * if hold { 0.6 } else { 1.0 - smooth(1.1, 1.5, t) };
            let to_head = Quat::from_rotation_x(-2.75) * Quat::from_rotation_z(0.35);
            let high = Quat::from_rotation_x(-2.2) * Quat::from_rotation_z(-0.55);
            let mut arm_r = idle.arm_r.slerp(to_head, reach * (1.0 - back));
            arm_r = arm_r.slerp(high, lift);
            VPose {
                torso: Quat::from_rotation_x(0.38 * bow - 0.06 * (1.0 - bow)),
                head: Quat::from_rotation_x(0.2 * bow - 0.1),
                arm_l: idle.arm_l.slerp(Quat::from_rotation_x(0.25) * Quat::from_rotation_z(0.35), bow),
                arm_r,
                hat: smooth(0.25, 0.35, t) * (1.0 - smooth(1.6, 1.7, t)) + if hold { smooth(0.25, 0.35, t) } else { 0.0 },
            }
        }
        Gesture::Plead => {
            // hands clasped in front, leaning toward the prince
            let k = smooth(0.0, 0.4, t);
            VPose {
                torso: Quat::from_rotation_x(0.16 * k),
                head: Quat::from_rotation_x(0.05 * k) * Quat::from_rotation_z(0.08 * k),
                arm_l: idle.arm_l.slerp(Quat::from_rotation_x(-0.95) * Quat::from_rotation_z(-0.45), k),
                arm_r: idle.arm_r.slerp(Quat::from_rotation_x(-0.95) * Quat::from_rotation_z(0.45), k),
                hat: 0.0,
            }
        }
    }
}

fn animate_vain(
    time: Res<Time>,
    mut q: Query<(&mut VainMan, &Rig, &GlobalTransform)>,
    player: Query<&Transform, (With<Player>, Without<Rig>)>,
    mut tfs: PartTfs,
) {
    let dt = time.delta_secs();
    let pp = player.single().map(|t| t.translation).ok();
    for (mut v, rig, gt) in &mut q {
        v.t += dt;
        v.idle_t += dt;
        if v.gesture != Gesture::Idle && v.gesture != Gesture::SaluteHall && v.t > v.gesture.duration() && !v.counting {
            if v.gesture != Gesture::Plead {
                v.gesture = Gesture::Idle;
            }
        }
        let idle = idle_pose(v.idle_t);
        let target = gesture_pose(v.gesture, v.t, idle);
        // gesture fades (0.22 s)
        let k = 1.0 - (-dt / 0.07).exp();
        let pose = match v.cur {
            Some(c) => c.lerp(&target, k),
            None => target,
        };
        v.cur = Some(pose);
        // head look toward the prince (15 m, ±35°, pitch −25..+5)
        let look = pp.map(|p| head_look(gt, NECK + Vec3::Y * 0.12, p + Vec3::Y * 1.1, 15.0, 35.0, (-25.0, 5.0))).unwrap_or(Quat::IDENTITY);
        let look = Quat::IDENTITY.slerp(look, 1.0 - pose.hat.min(1.0) * 0.5);
        // chain: torso about the hip; head and arms ride on it
        let r = pose.torso;
        let at = |p: Vec3| HIP + r * (p - HIP);
        pose_part(&mut tfs, rig, "torso", r, Vec3::ZERO);
        set_abs(&mut tfs, rig, "head", at(NECK), r * look * pose.head);
        let arm_l = r * pose.arm_l;
        let arm_r = r * pose.arm_r;
        set_abs(&mut tfs, rig, "arm_l", at(SHOULDER_L), arm_l);
        set_abs(&mut tfs, rig, "arm_r", at(SHOULDER_R), arm_r);
        // hat: on the head, or in the right hand (crown up, tipped)
        let head_rot = r * look * pose.head;
        let on_head = Transform::from_translation(at(NECK) + head_rot * Vec3::new(0.0, 0.28, -0.01)).with_rotation(head_rot * Quat::from_rotation_x(-0.08));
        let hand = at(SHOULDER_R) + arm_r * Vec3::new(0.0, -ARM_LEN - 0.06, 0.0);
        let in_hand = Transform::from_translation(hand + Vec3::Y * 0.02).with_rotation(arm_r * Quat::from_rotation_x(std::f32::consts::PI * 0.55));
        let h = pose.hat.clamp(0.0, 1.0);
        let tf = Transform::from_translation(on_head.translation.lerp(in_hand.translation, h)).with_rotation(on_head.rotation.slerp(in_hand.rotation, h));
        place_part(&mut tfs, rig, "hat", tf);
    }
}

fn set_abs(tfs: &mut PartTfs, rig: &Rig, name: &str, pos: Vec3, rot: Quat) {
    place_part(tfs, rig, name, Transform::from_translation(pos).with_rotation(rot));
}

// ------------------------------------------------- procedural models ----

const WOOD: u32 = 0x7a4a3a;
const WOOD_DARK: u32 = 0x5a3428;
const VELVET: u32 = 0x8e2a3a;
const VELVET_DARK: u32 = 0x6a1f2c;
const GOLD: u32 = 0xd8b04a;
const CANVAS_A: u32 = 0xe8d9bf;
const CANVAS_B: u32 = 0xb8445a;

/// The theatre layout (world = local, door plane z = 0). Used for both the
/// mesh and the colliders.
fn theatre_layout() -> Vec<Bx> {
    let wall = hex(WOOD);
    let trim = hex(WOOD_DARK);
    let mut v = vec![
        // front wall with the 3.2 × 3.1 doorway
        bmm(Vec3::new(-7.8, 0.0, 0.0), Vec3::new(-1.6, 5.0, 0.6), wall),
        bmm(Vec3::new(1.6, 0.0, 0.0), Vec3::new(7.8, 5.0, 0.6), wall),
        bmm(Vec3::new(-1.6, 3.1, 0.0), Vec3::new(1.6, 5.0, 0.6), wall),
        // side and back walls
        bmm(Vec3::new(-7.8, 0.0, 0.0), Vec3::new(-7.4, 5.0, 18.0), wall),
        bmm(Vec3::new(7.4, 0.0, 0.0), Vec3::new(7.8, 5.0, 18.0), wall),
        bmm(Vec3::new(-7.8, 0.0, 17.6), Vec3::new(7.8, 5.0, 18.0), wall),
        // stage (walkable top 0.9) and its three steps
        bmm(Vec3::new(-6.8, 0.0, 12.3), Vec3::new(6.8, STAGE_Y, 17.6), hex(0x6b4632)),
        bmm(Vec3::new(-1.2, 0.0, 11.25), Vec3::new(1.2, 0.3, 12.3), hex(0x7d5440)),
        bmm(Vec3::new(-1.2, 0.0, 11.6), Vec3::new(1.2, 0.6, 12.3), hex(0x7d5440)),
        bmm(Vec3::new(-1.2, 0.0, 11.95), Vec3::new(1.2, 0.9, 12.3), hex(0x7d5440)),
        // proscenium pillars (gold-trimmed)
        bmm(Vec3::new(-7.4, 0.0, 12.0), Vec3::new(-6.8, 4.8, 12.5), hex(0x5b2230)),
        bmm(Vec3::new(6.8, 0.0, 12.0), Vec3::new(7.4, 4.8, 12.5), hex(0x5b2230)),
        // threshold step outside
        bmm(Vec3::new(-1.9, -0.2, -0.6), Vec3::new(1.9, 0.04, 0.0), trim).vis(),
    ];
    // visual-only interior: floor, velvet lining, stage edge, curtains, valance
    v.extend([
        bmm(Vec3::new(-7.4, -0.1, 0.6), Vec3::new(7.4, 0.005, 17.6), hex(0x8b6448)).vis(),
        bmm(Vec3::new(-1.0, 0.006, 0.6), Vec3::new(1.0, 0.012, 11.3), hex(VELVET_DARK)).vis(),
        bmm(Vec3::new(-7.4, 0.0, 0.6), Vec3::new(-7.36, 4.95, 17.6), hex(VELVET)).vis(),
        bmm(Vec3::new(7.36, 0.0, 0.6), Vec3::new(7.4, 4.95, 17.6), hex(VELVET)).vis(),
        bmm(Vec3::new(-7.4, 0.0, 0.6), Vec3::new(-1.6, 4.95, 0.64), hex(VELVET)).vis(),
        bmm(Vec3::new(1.6, 0.0, 0.6), Vec3::new(7.4, 4.95, 0.64), hex(VELVET)).vis(),
        bmm(Vec3::new(-7.4, 1.1, 0.64), Vec3::new(7.4, 1.18, 0.7), hex(GOLD)).vis(),
        bmm(Vec3::new(-6.8, STAGE_Y - 0.08, 12.26), Vec3::new(6.8, STAGE_Y + 0.02, 12.34), hex(GOLD)).vis(),
        bmm(Vec3::new(-6.8, STAGE_Y, 17.2), Vec3::new(6.8, 4.9, 17.5), hex(0x6a1a2a)).vis(),
        bmm(Vec3::new(-6.8, STAGE_Y, 12.45), Vec3::new(-5.3, 4.6, 12.75), hex(VELVET)).vis(),
        bmm(Vec3::new(5.3, STAGE_Y, 12.45), Vec3::new(6.8, 4.6, 12.75), hex(VELVET)).vis(),
        bmm(Vec3::new(-6.9, 4.0, 12.2), Vec3::new(6.9, 4.8, 12.6), hex(0x5b2230)).vis(),
        bmm(Vec3::new(-6.9, 3.92, 12.18), Vec3::new(6.9, 4.0, 12.62), hex(GOLD)).vis(),
        bmm(Vec3::new(-7.4, 4.8, 0.6), Vec3::new(7.4, 4.95, 17.6), hex(0x3c2430)).vis(),
        // door frame
        bmm(Vec3::new(-1.8, 0.0, -0.08), Vec3::new(-1.6, 3.3, 0.0), hex(GOLD)).vis(),
        bmm(Vec3::new(1.6, 0.0, -0.08), Vec3::new(1.8, 3.3, 0.0), hex(GOLD)).vis(),
        bmm(Vec3::new(-1.8, 3.1, -0.08), Vec3::new(1.8, 3.3, 0.0), hex(GOLD)).vis(),
        // marquee board over the door
        bmm(Vec3::new(-2.6, 3.5, -0.22), Vec3::new(2.6, 4.5, 0.0), hex(0x3a2030)).vis(),
        bmm(Vec3::new(-2.4, 3.65, -0.24), Vec3::new(2.4, 4.35, -0.2), hex(0xe6c36a)).vis(),
        // wall base trim outside
        bmm(Vec3::new(-7.85, 0.0, -0.05), Vec3::new(7.85, 0.5, 0.0), trim).vis(),
    ]);
    // bunting: alternating little flags under the eaves (front)
    for i in 0..14 {
        let x = -7.0 + i as f32 * 1.08;
        let c = if i % 2 == 0 { hex(CANVAS_B) } else { hex(0xe6c36a) };
        v.push(bmm(Vec3::new(x - 0.2, 4.55, -0.12), Vec3::new(x + 0.2, 4.95, -0.06), c).vis());
    }
    v
}

fn theatre_model(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    mesh_boxes(&mut m, &theatre_layout());
    // the tent-like roof: striped canvas slabs from the eaves to the ridge
    let n = 9;
    for i in 0..n {
        let z0 = -0.4 + i as f32 * (18.8 / n as f32);
        let z1 = z0 + 18.8 / n as f32;
        let c = if i % 2 == 0 { hex(CANVAS_A) } else { hex(CANVAS_B) };
        roof_slab(&mut m, -8.1, 4.9, 8.2, z0, z1, 0.12, c);
        roof_slab(&mut m, 8.1, 4.9, 8.2, z0, z1, 0.12, c);
    }
    gable(&mut m, 7.8, 5.0, 8.15, -0.01, hex(CANVAS_A));
    gable(&mut m, 7.8, 5.0, 8.15, 18.01, hex(CANVAS_A));
    // round vent window in the front gable
    m.with(Transform::from_xyz(0.0, 6.3, -0.05).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)), |m| {
        m.frustum(Vec3::ZERO, 0.62, 0.62, 0.06, 14, hex(GOLD));
    });
    p.add(&m);
    // glowing marquee letters (dots) and window glow
    let mut g = MeshBuilder::new();
    for i in 0..15 {
        let x = -2.25 + i as f32 * 0.32;
        g.sphere(Vec3::new(x, 3.58, -0.26), 0.045, 2, 5, hex(0xffe39a));
        g.sphere(Vec3::new(x, 4.42, -0.26), 0.045, 2, 5, hex(0xffe39a));
    }
    m = MeshBuilder::new();
    m.with(Transform::from_xyz(0.0, 6.3, -0.09).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)), |m| {
        m.frustum(Vec3::ZERO, 0.5, 0.5, 0.04, 14, hex(0xffcf80));
    });
    g.append(&m);
    // chandeliers inside (hoops of candle bulbs)
    for z in [6.0f32, 10.0] {
        for i in 0..10 {
            let a = i as f32 / 10.0 * std::f32::consts::TAU;
            g.sphere(Vec3::new(a.cos() * 0.6, 4.1, z + a.sin() * 0.6), 0.05, 2, 5, hex(0xffd896));
        }
    }
    p.add_part(None, &g, Transform::IDENTITY, PartMat::Emissive(hex(0xffcf80), 3.5));
    let mut h = MeshBuilder::new();
    for z in [6.0f32, 10.0] {
        h.frustum(Vec3::new(0.0, 4.02, z), 0.66, 0.66, 0.05, 12, hex(GOLD));
        h.tube(Vec3::new(0.0, 4.07, z), Vec3::new(0.0, 4.8, z), 0.02, 0.02, 4, hex(0x4a3a2a));
    }
    // lanterns either side of the door
    for x in [-2.3f32, 2.3] {
        h.cuboid(Vec3::new(x, 2.55, -0.2), Vec3::new(0.08, 0.6, 0.08), hex(0x2a2a2a));
    }
    p.add(&h);
    let mut l = MeshBuilder::new();
    for x in [-2.3f32, 2.3] {
        l.cuboid(Vec3::new(x, 2.95, -0.25), Vec3::new(0.24, 0.32, 0.24), hex(0xffcf80));
    }
    p.add_part(None, &l, Transform::IDENTITY, PartMat::Emissive(hex(0xffb860), 2.5));
}

fn door_leaf(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 1.55, 0.0), Vec3::new(1.58, 3.1, 0.08), hex(0x6e3326));
    for y in [0.7f32, 2.2] {
        m.cuboid(Vec3::new(0.0, y, 0.0), Vec3::new(1.2, 0.9, 0.11), hex(0x86412f));
    }
    m.sphere(Vec3::new(0.0, 1.25, -0.09), 0.05, 2, 6, hex(GOLD));
    m.sphere(Vec3::new(0.0, 1.25, 0.09), 0.05, 2, 6, hex(GOLD));
    p.add(&m);
}

/// Half a row of six child-size theatre chairs (seat 0.30 m), centred at x 0.
fn seat_row(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    for i in 0..6 {
        let x = -2.5 + i as f32;
        m.cuboid(Vec3::new(x, SEAT_H - 0.04, 0.0), Vec3::new(0.46, 0.09, 0.4), hex(VELVET));
        m.cuboid(Vec3::new(x, 0.52, -0.22), Vec3::new(0.46, 0.52, 0.08), hex(VELVET));
        m.cuboid(Vec3::new(x, 0.79, -0.22), Vec3::new(0.48, 0.04, 0.1), hex(GOLD));
        m.cuboid(Vec3::new(x - 0.25, 0.22, -0.05), Vec3::new(0.04, 0.44, 0.5), hex(0x3a2a2a));
    }
    m.cuboid(Vec3::new(3.0 - 0.25, 0.22, -0.05), Vec3::new(0.04, 0.44, 0.5), hex(0x3a2a2a));
    m.cuboid(Vec3::new(0.0, 0.04, -0.05), Vec3::new(5.9, 0.08, 0.5), hex(0x3a2a2a));
    p.add(&m);
}

fn footlights(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    let mut g = MeshBuilder::new();
    for i in 0..13 {
        let x = -6.0 + i as f32;
        m.cuboid(Vec3::new(x, 0.06, 0.0), Vec3::new(0.3, 0.12, 0.16), hex(0x2a2224));
        g.sphere(Vec3::new(x, 0.13, 0.03), 0.06, 2, 6, hex(0xfff0c0));
    }
    p.add(&m);
    p.add_part(None, &g, Transform::IDENTITY, PartMat::Emissive(hex(0xffe0a0), 4.0));
}

fn mirror(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    // gilded oval-ish frame on a stand (faces +z)
    m.cuboid(Vec3::new(0.0, 0.05, 0.0), Vec3::new(1.2, 0.1, 0.6), hex(0x5a3a2a));
    for (x, h) in [(-0.72f32, 2.7f32), (0.72, 2.7)] {
        m.cuboid(Vec3::new(x, h * 0.5, 0.0), Vec3::new(0.12, h, 0.12), hex(GOLD));
    }
    m.cuboid(Vec3::new(0.0, 2.72, 0.0), Vec3::new(1.6, 0.16, 0.14), hex(GOLD));
    m.cuboid(Vec3::new(0.0, 0.3, 0.0), Vec3::new(1.5, 0.12, 0.14), hex(GOLD));
    m.sphere(Vec3::new(0.0, 2.92, 0.0), 0.14, 3, 8, hex(GOLD));
    p.add(&m);
    let mut g = MeshBuilder::new();
    g.cuboid(Vec3::new(0.0, 1.5, 0.0), Vec3::new(1.32, 2.3, 0.04), hex(0xdfe8f0));
    p.add_part(
        None,
        &g,
        Transform::IDENTITY,
        PartMat::Custom(StandardMaterial { base_color: hex(0xcfd8e4), metallic: 0.9, perceptual_roughness: 0.06, reflectance: 1.0, ..default() }),
    );
}

fn poster(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::new(0.0, 0.85, 0.0), Vec3::new(1.3, 1.7, 0.03), hex(0xe9d7a8));
    m.cuboid(Vec3::new(0.0, 1.3, 0.02), Vec3::new(1.1, 0.5, 0.02), hex(CANVAS_B));
    // a little top-hatted silhouette
    m.cuboid(Vec3::new(0.0, 0.62, 0.02), Vec3::new(0.34, 0.6, 0.02), hex(0x4a2f5a));
    m.sphere(Vec3::new(0.0, 1.0, 0.02), 0.13, 3, 8, hex(0xf2d2b6));
    m.cuboid(Vec3::new(0.0, 1.2, 0.02), Vec3::new(0.2, 0.22, 0.03), hex(0x2a2030));
    m.cuboid(Vec3::new(0.0, 0.18, 0.02), Vec3::new(1.0, 0.12, 0.02), hex(GOLD));
    p.add(&m);
}

fn pennant(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::Y * 1.6, 0.04, 0.03, 5, hex(0x4a3a2a));
    m.sphere(Vec3::Y * 1.62, 0.07, 2, 6, hex(GOLD));
    m.tri(Vec3::new(0.0, 1.5, 0.0), Vec3::new(0.0, 1.1, 0.0), Vec3::new(0.0, 1.3, 1.0), hex(CANVAS_B));
    m.tri(Vec3::new(0.0, 1.1, 0.0), Vec3::new(0.0, 1.5, 0.0), Vec3::new(0.0, 1.3, 1.0), hex(CANVAS_B));
    p.add(&m);
}

/// The vain man (~1.95 m with his hat), facing +z, feet at the origin.
/// Named parts: torso (hip pivot), head (neck), arm_l / arm_r (shoulders), hat.
fn vain_man(p: &mut ModelParts) {
    let coat = hex(0x6a3f8a);
    let coat_dark = hex(0x4e2c68);
    let skin = hex(0xf2d2b6);
    let trouser_a = hex(0xe8dcc8);
    let trouser_b = hex(0x9a8a74);
    // legs (static)
    let mut legs = MeshBuilder::new();
    for x in [-0.1f32, 0.1] {
        for i in 0..4 {
            let y0 = 0.08 + i as f32 * 0.21;
            legs.frustum(Vec3::new(x, y0, 0.0), 0.07, 0.075, 0.21, 7, if i % 2 == 0 { trouser_a } else { trouser_b });
        }
        legs.cuboid(Vec3::new(x, 0.05, 0.05), Vec3::new(0.13, 0.1, 0.3), hex(0x1e1a22));
    }
    p.add(&legs);
    // torso (pivot at the hip): tailcoat, waistcoat, bow tie, medals
    let mut t = MeshBuilder::new();
    t.with(Transform::from_translation(-HIP), |t| {
        t.lathe(Vec3::new(0.0, 0.62, 0.0), &[(0.0, 0.0), (0.27, 0.0), (0.24, 0.3), (0.25, 0.55), (0.3, 0.78), (0.2, 0.88), (0.0, 0.9)], 10, coat);
        t.cuboid(Vec3::new(0.0, 1.12, 0.17), Vec3::new(0.24, 0.42, 0.12), hex(0xd8b04a));
        t.cuboid(Vec3::new(0.0, 1.43, 0.2), Vec3::new(0.16, 0.07, 0.06), hex(0xb02a3a));
        t.sphere(Vec3::new(0.0, 1.43, 0.23), 0.03, 2, 5, hex(0xb02a3a));
        // tails
        t.cuboid(Vec3::new(0.09, 0.62, -0.2), Vec3::new(0.12, 0.5, 0.05), coat_dark);
        t.cuboid(Vec3::new(-0.09, 0.62, -0.2), Vec3::new(0.12, 0.5, 0.05), coat_dark);
        // epaulettes with fringe
        for x in [-0.27f32, 0.27] {
            t.ellipsoid(Vec3::new(x, 1.46, 0.0), Vec3::new(0.11, 0.04, 0.1), 2, 7, hex(GOLD));
        }
        // medals
        for (i, c) in [0xe8c040u32, 0xc0c8d0, 0xd07a3a].iter().enumerate() {
            t.cuboid(Vec3::new(0.18 - i as f32 * 0.06, 1.28, 0.24), Vec3::new(0.03, 0.08, 0.02), hex(0x3a5aa0));
            t.sphere(Vec3::new(0.18 - i as f32 * 0.06, 1.21, 0.25), 0.028, 2, 5, hex(*c));
        }
    });
    p.add_part(Some("torso"), &t, Transform::from_translation(HIP), PartMat::Vertex);
    // head (pivot at the neck): big nose, curled moustache, rosy cheeks
    let mut h = MeshBuilder::new();
    h.frustum(Vec3::ZERO, 0.06, 0.06, 0.08, 6, skin);
    h.ellipsoid(Vec3::new(0.0, 0.17, 0.0), Vec3::new(0.15, 0.18, 0.15), 4, 9, skin);
    h.ellipsoid(Vec3::new(0.0, 0.16, 0.15), Vec3::new(0.035, 0.05, 0.07), 2, 6, hex(0xeab8a0));
    h.ellipsoid(Vec3::new(0.065, 0.1, 0.12), Vec3::new(0.06, 0.02, 0.03), 2, 6, hex(0x3a2a22));
    h.ellipsoid(Vec3::new(-0.065, 0.1, 0.12), Vec3::new(0.06, 0.02, 0.03), 2, 6, hex(0x3a2a22));
    h.sphere(Vec3::new(0.11, 0.08, 0.12), 0.012, 2, 4, hex(0x3a2a22));
    h.sphere(Vec3::new(-0.11, 0.08, 0.12), 0.012, 2, 4, hex(0x3a2a22));
    h.sphere(Vec3::new(0.05, 0.21, 0.13), 0.018, 2, 4, hex(0x2b2b3a));
    h.sphere(Vec3::new(-0.05, 0.21, 0.13), 0.018, 2, 4, hex(0x2b2b3a));
    h.sphere(Vec3::new(0.085, 0.13, 0.11), 0.03, 2, 5, hex(0xf0a0a0));
    h.sphere(Vec3::new(-0.085, 0.13, 0.11), 0.03, 2, 5, hex(0xf0a0a0));
    h.ellipsoid(Vec3::new(0.0, 0.2, -0.04), Vec3::new(0.155, 0.12, 0.14), 3, 9, hex(0x3a2a22));
    p.add_part(Some("head"), &h, Transform::from_translation(NECK), PartMat::Vertex);
    // arms
    let a = arm_mesh(ARM_LEN, 0.07, 0.055, coat, hex(0xf6f2ea));
    p.add_part(Some("arm_l"), &a, Transform::from_translation(SHOULDER_L), PartMat::Vertex);
    p.add_part(Some("arm_r"), &a, Transform::from_translation(SHOULDER_R), PartMat::Vertex);
    // the hat (origin at its brim): top hat with a band and a plume
    let mut hat = MeshBuilder::new();
    hat.frustum(Vec3::ZERO, 0.24, 0.24, 0.025, 12, hex(0x2a2030));
    hat.frustum(Vec3::Y * 0.02, 0.15, 0.165, 0.3, 12, hex(0x2a2030));
    hat.frustum(Vec3::Y * 0.04, 0.152, 0.155, 0.06, 12, hex(0xb02a3a));
    hat.tube(Vec3::new(0.12, 0.08, 0.0), Vec3::new(0.2, 0.42, -0.08), 0.035, 0.008, 5, hex(0xf0e6ff));
    hat.tube(Vec3::new(0.12, 0.08, 0.02), Vec3::new(0.24, 0.36, 0.0), 0.03, 0.006, 5, hex(0xd8a0e0));
    p.add_part(Some("hat"), &hat, Transform::from_translation(NECK + Vec3::Y * 0.28), PartMat::Vertex);
}
