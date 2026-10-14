//! Chapter 1 — Asteroid B-612 (STUB, but a complete worked example of the
//! core API; see docs/CORE_API.md). The chapter agent replaces this with the
//! full chapter following the original's design.
//!
//! Shows: planet params + terrain mods, sky + sunset, manifest models with a
//! procedural fallback, colliders, interactables (press + hold), anchors,
//! a chapter system reacting to `Interacted` and setting a story flag,
//! events/steps, dialogue, a choice, a hidden goal, camera shot, test hints
//! and departure.

use crate::core::prelude::*;
use crate::core::story::found_hidden;

pub struct B612;

const ID: &str = "b612";

pub const STRINGS: StringTable = &[
    ("b612.title", "小行星 B612", "Asteroid B-612"),
    ("b612.subtitle", "三座火山与一朵玫瑰", "Three volcanoes and a rose"),
    ("b612.line", "这是小王子的家。一颗比房子大不了多少的星星。", "The little prince's home: a star hardly bigger than a house."),
    ("b612.visited", "火山已经打扫干净，玫瑰在玻璃罩下睡着。", "The volcanoes are swept; the rose sleeps under her glass."),
    ("b612.loading", "清晨，太阳从东边升起……", "Morning. The sun rises in the east…"),
    // events & steps
    ("b612.ev.wake", "醒来", "Waking up"),
    ("b612.st.rose", "去看看玫瑰", "Go and see the rose"),
    ("b612.ev.chores", "每天的功课", "The daily chores"),
    ("b612.st.volcano", "打扫火山（按住 E）", "Sweep the volcano (hold E)"),
    ("b612.st.sprouts", "拔掉猴面包树的幼苗（0/3）", "Pull up the baobab sprouts (0/3)"),
    ("b612.ev.sunset", "看日落", "Watching the sunset"),
    ("b612.st.chair", "坐到椅子上", "Sit in the chair"),
    ("b612.hidden", "在星球的另一边看看远方", "Look out from the far side of the planet"),
    // targets & verbs
    ("b612.target.rose", "玫瑰", "The rose"),
    ("b612.target.volcano", "火山", "Volcano"),
    ("b612.target.chair", "椅子", "Chair"),
    ("b612.verb.talk", "说话", "Talk"),
    ("b612.verb.sweep", "打扫", "Sweep"),
    ("b612.verb.pull", "拔掉", "Pull up"),
    ("b612.verb.sit", "坐下", "Sit"),
    // dialogue (our own words)
    ("b612.l.rose1", "你醒啦。今天早上的风有点凉。", "You're awake. The wind is a little cold this morning."),
    ("b612.l.prince1", "我先去把火山打扫干净，再回来陪你。", "I'll sweep the volcanoes first, then come back to you."),
    ("b612.l.rose2", "别忘了那些猴面包树的小苗。", "Don't forget those little baobab sprouts."),
    ("b612.l.sprout_done", "好了，它们再也长不成大树了。", "There. They won't grow into trees now."),
    ("b612.l.sit", "在这颗小星星上，只要把椅子挪几步，就能再看一次日落。", "On a star this small, moving the chair a few steps gives you another sunset."),
    ("b612.l.rose_q", "你喜欢日落，对吗？", "You love sunsets, don't you?"),
    ("b612.opt.yes", "人难过的时候，就会喜欢日落。", "When one is sad, one loves sunsets."),
    ("b612.opt.quiet", "（不说话，只是看着天边）", "(Say nothing, just watch the sky)"),
    ("b612.l.rose_yes", "那今天，你难过吗？", "And today — are you sad?"),
    ("b612.l.rose_quiet", "……天边真好看。", "…The sky is lovely."),
    ("b612.mono.sunset", "太阳慢慢沉下去了。", "The sun sinks slowly away."),
    ("b612.l.depart", "鸟群从远处飞来了。也许，是时候出发去看看别的星星。", "A flock of birds is coming. Perhaps it is time to go and see other stars."),
];

impl Chapter for B612 {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 1,
            title: "b612.title",
            subtitle: "b612.subtitle",
            line: "b612.line",
            visited_line: "b612.visited",
            loading: "b612.loading",
            map_pos: Vec2::new(210.0, 610.0),
            map_radius: 60.0,
            map_colors: (hex(0x9fb36c), hex(0xf0c23a)),
            music: "music.b612",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("b612.volcano", volcano)
            .register_model("b612.rose", rose)
            .register_model("b612.sprout", sprout)
            .add_systems(Update, (pull_sprouts, lookout_hidden).in_set(CoreSet::Gameplay).run_if(in_chapter(ID)));
    }

    fn build(&self, cx: &mut ChapterCx) {
        // --- the planet: a small domed terrain with a knoll for the rose
        let rose_at = Vec2::new(3.0, 4.0);
        let chair_at = Vec2::new(-4.5, -2.0);
        let mut p = PlanetParams::sized(14.0);
        p.mods = vec![
            TerrainMod::Mound { center: rose_at, radius: 2.6, height: 0.35 },
            TerrainMod::Flatten { center: chair_at, radius: 1.2, blend: 1.0, height: None },
            TerrainMod::Paint { center: rose_at, radius: 1.2, blend: 0.8, color: hex(0x7c6a52) },
        ];
        cx.planet(p);
        cx.sky(8.5, 0.5, Some(16.5), SkyPalette::default());
        cx.ambience("amb.wind");

        // --- props
        let rose_p = cx.ground(rose_at.x, rose_at.y);
        let rose = cx.model("model.b612.rose", rose_p, 0.0, 1.0);
        cx.collider(rose, Collider::cylinder(0.25, 0.7).no_camera());
        cx.interactable(rose, Interactable::new("rose", "b612.verb.talk").radius(2.2).focus(Vec3::Y * 0.4).prompt_height(0.6));
        cx.anchor("rose", rose_p, 0.0);

        for (i, (a, r)) in [(150.0f32, 7.0f32), (205.0, 8.5), (250.0, 6.5)].iter().enumerate() {
            let pos = cx.polar(*a, *r);
            let v = cx.model("model.b612.volcano", pos, *a, if i == 2 { 0.8 } else { 1.0 });
            cx.collider(v, Collider::cylinder(0.75, 0.9));
            if i == 0 {
                cx.interactable(v, Interactable::new("volcano", "b612.verb.sweep").radius(2.4).hold(1.2).focus(Vec3::Y * 0.6).disabled());
                cx.anchor("volcano", pos, 0.0);
            }
        }
        for i in 0..3 {
            let pos = cx.polar(40.0 + i as f32 * 35.0, 9.0 + i as f32 * 0.8);
            let s = cx.model("model.b612.sprout", pos, i as f32 * 70.0, 1.0);
            cx.interactable(s, Interactable::new(&format!("sprout_{i}"), "b612.verb.pull").radius(1.6).prompt_height(0.5).disabled().once());
            if i == 0 {
                cx.anchor("sprouts", pos, 0.0);
            }
        }
        let chair_p = cx.ground(chair_at.x, chair_at.y);
        let chair = cx.model("model.b612.chair", chair_p, 250.0, 1.0);
        cx.collider(chair, Collider::cuboid(Vec3::new(0.5, 0.45, 0.5)).no_camera());
        cx.interactable(chair, Interactable::new("chair", "b612.verb.sit").radius(1.8).focus(Vec3::Y * 0.4).disabled());
        cx.anchor("chair", chair_p, 250.0);
        cx.anchor("seat", chair_p + Vec3::Y * 0.47, 250.0);
        cx.anchor("lookout", cx.polar(320.0, 12.5), 0.0);

        cx.scatter("model.rock", 7, 6.0, 13.0, 3, (0.5, 1.2), &[(rose_at, 2.0), (chair_at, 2.0)]);
        cx.scatter("model.grass_tuft", 60, 1.5, 13.5, 5, (0.8, 1.4), &[(rose_at, 1.0)]);
        cx.scatter("model.flower", 10, 3.0, 12.0, 9, (0.8, 1.2), &[(rose_at, 1.5)]);

        // --- the prince wakes near the centre, facing east (toward the sunrise)
        let start = cx.ground(0.0, -3.0);
        cx.player(start, 90.0);
    }

    fn story(&self) -> Story {
        use crate::core::story::Pos;
        Story {
            events: vec![
                event("b612.ev.wake", &["b612.st.rose"]),
                event("b612.ev.chores", &["b612.st.volcano", "b612.st.sprouts"]),
                event("b612.ev.sunset", &["b612.st.chair"]),
            ],
            hidden: Some("b612.hidden"),
            script: vec![
                card(),
                teach(&["W", "A", "S", "D"], "core.teach.move"),
                start_event(0),
                target(Pos::anchor("rose"), "b612.target.rose"),
                wait_near(Pos::anchor("rose"), 2.4),
                face(Pos::anchor("rose")),
                say(speakers::ROSE, "b612.l.rose1"),
                say(speakers::PRINCE, "b612.l.prince1"),
                say(speakers::ROSE, "b612.l.rose2"),
                complete_event(0),
                Cmd::HiddenPending,
                start_event(1),
                teach(&["E"], "core.teach.interact"),
                target(Pos::anchor("volcano"), "b612.target.volcano"),
                wait_use("volcano"),
                sfx("sfx.wood"),
                step(1),
                enable("sprout_0"),
                enable("sprout_1"),
                enable("sprout_2"),
                target(Pos::anchor("sprouts"), "b612.target.volcano"),
                wait_flag("sprouts_done", vec![t_use("sprout_0"), t_use("sprout_1"), t_use("sprout_2")]),
                say(speakers::PRINCE, "b612.l.sprout_done"),
                complete_event(1),
                start_event(2),
                target(Pos::anchor("chair"), "b612.target.chair"),
                wait_use("chair"),
                disable("chair"),
                clear_target(),
                teleport(Pos::anchor("seat"), 250.0),
                pose(Pose::Sit),
                say_for(speakers::PRINCE, "b612.l.sit", 2.5),
                shot(Pos::Player(Vec3::new(2.5, 1.6, 2.5)), Pos::Player(Vec3::new(-6.0, 1.5, -2.0)), 2.0),
                sunset(12.0),
                mono("b612.mono.sunset"),
                wait_sunset(),
                end_shot(),
                choice_test(Some((speakers::ROSE, "b612.l.rose_q")), &[("b612.opt.yes", "yes"), ("b612.opt.quiet", "quiet")], 0),
                label("yes"),
                say(speakers::ROSE, "b612.l.rose_yes"),
                goto("after"),
                label("quiet"),
                say(speakers::ROSE, "b612.l.rose_quiet"),
                label("after"),
                pose(Pose::None),
                complete_event(2),
                say(speakers::PRINCE, "b612.l.depart"),
                departure(),
            ],
        }
    }
}

/// Chapter system: pulling a sprout despawns it; when all three are gone the
/// story flag `sprouts_done` is set (the script waits on it).
fn pull_sprouts(mut commands: Commands, mut msgs: MessageReader<Interacted>, mut runner: ResMut<StoryRunner>, mut audio: MessageWriter<AudioCmd>) {
    for m in msgs.read() {
        if m.id.starts_with("sprout_") {
            commands.entity(m.entity).despawn();
            audio.write(AudioCmd::sfx("sfx.pop"));
            runner.set_flag(&m.id);
            if (0..3).all(|i| runner.has_flag(&format!("sprout_{i}"))) {
                runner.set_flag("sprouts_done");
            }
        }
    }
}

/// Hidden goal: walk to the far side of the planet.
fn lookout_hidden(mut commands: Commands, runner: Res<StoryRunner>, anchors: Res<crate::core::story::Anchors>, player: Query<&Transform, With<Player>>) {
    if runner.hidden_done || !runner.hidden_shown {
        return;
    }
    let (Some((a, _)), Ok(p)) = (anchors.get("lookout"), player.single()) else { return };
    if Vec2::new(a.x - p.translation.x, a.z - p.translation.z).length() < 2.0 {
        commands.queue(|w: &mut World| found_hidden(w));
    }
}

// ------------------------------------------------- procedural fallbacks ----

fn volcano(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.85, 0.0), (0.7, 0.35), (0.42, 0.75), (0.3, 0.82), (0.22, 0.7), (0.0, 0.62)], 9, hex(0x7f6f66));
    p.add(&m);
    let mut g = MeshBuilder::new();
    g.frustum(Vec3::Y * 0.6, 0.2, 0.0, 0.05, 8, hex(0xff8a3d));
    p.add_part(Some("glow"), &g, Transform::IDENTITY, PartMat::Emissive(hex(0xff7a30), 3.0));
}

fn rose(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::Y * 0.42, 0.018, 0.014, 5, hex(0x4f8a45));
    m.with(Transform::from_xyz(0.0, 0.2, 0.0).with_rotation(Quat::from_rotation_z(0.9)), |m| {
        m.ellipsoid(Vec3::new(0.0, 0.06, 0.0), Vec3::new(0.03, 0.07, 0.015), 2, 5, hex(0x5d9c4e));
    });
    for i in 0..4 {
        m.tube(Vec3::Y * (0.1 + i as f32 * 0.07), Vec3::Y * (0.1 + i as f32 * 0.07) + Vec3::new(0.03, 0.02, 0.0), 0.006, 0.0, 3, hex(0x3f6e3a));
    }
    m.sphere(Vec3::Y * 0.5, 0.085, 4, 7, hex(0xd8434e));
    m.lathe(Vec3::Y * 0.43, &[(0.0, 0.0), (0.07, 0.04), (0.11, 0.1), (0.1, 0.14)], 6, hex(0xc93a46));
    p.add(&m);
}

fn sprout(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.tube(Vec3::ZERO, Vec3::Y * 0.25, 0.03, 0.02, 5, hex(0x6c8f3e));
    for i in 0..3 {
        let a = i as f32 * 2.1;
        m.with(Transform::from_xyz(0.0, 0.24, 0.0).with_rotation(Quat::from_rotation_y(a) * Quat::from_rotation_z(0.8)), |m| {
            m.ellipsoid(Vec3::new(0.0, 0.07, 0.0), Vec3::new(0.05, 0.08, 0.02), 2, 5, hex(0x7fae4e));
        });
    }
    p.add(&m);
}
