//! Chapter 1 — Asteroid B-612, the last day at home (the original's
//! B612Story). Beats, in order:
//!
//! 1. The chapter card plays over the sleeping prince; he wakes, the camera
//!    rises and orbits from behind to his front, then hands over (B612Opening).
//! 2. Free roam + the three-step controls tutorial (move / look / run).
//! 3. Coming within 4 m of the rose, she calls him: he is walked to 1.3 m,
//!    medium shot, two lines (RoseCalls).
//! 4. "Look after the rose": can from the tool shed → fill it at the spring
//!    (four fetch slabs) → water her one last time (1.8 s soak).
//! 5. Farewell (part 1), the glass-cloche ritual (part 2) and the hold-E put-back.
//! 6. "Watch the last sunset": sit on the chair at the west edge → the clock
//!    hops to the golden hour (3.5 s) and the sun sets (22 s) → departure.
//!
//! Layout and props: world.rs. Mechanics: systems.rs. Lens helper: director.rs.

pub mod director;
mod models;
pub mod systems;
pub mod world;

use crate::core::prelude::*;
use systems::{RosePose, set_rose_pose, tremble};

pub struct B612;

const ID: &str = "b612";

pub const NARR: Speaker = speakers::NARRATOR;

pub const STRINGS: StringTable = &[
    ("b612.title", "小行星 B612", "Asteroid B-612"),
    ("b612.subtitle", "在 B612 的最后一天", "The last day on B-612"),
    (
        "b612.line",
        "这是小王子的家。一颗比房子大不了多少的星星。",
        "The little prince's home: a star hardly bigger than a house.",
    ),
    (
        "b612.visited",
        "火山还在冒着轻烟，玫瑰独自守着她的星球。",
        "The volcanoes still smoke a little; the rose keeps the planet alone.",
    ),
    ("b612.loading", "清晨，太阳从东边升起……", "Morning. The sun rises in the east…"),
    ("b612.free", "到处走走，熟悉一下你的星球", "Walk around and get to know your planet"),
    ("b612.st.see_rose", "去看看玫瑰", "Go and see the rose"),
    // events & steps
    ("b612.ev.rose", "照顾玫瑰", "Look after the rose"),
    ("b612.st.can", "去工具棚取来水壶", "Fetch the watering can from the tool shed"),
    ("b612.st.fill", "在泉边把水壶灌满", "Fill it at the spring"),
    ("b612.st.water", "最后一次给她浇水", "Water her one last time"),
    ("b612.ev.sunset", "看最后一次日落", "Watch the last sunset"),
    ("b612.st.chair", "去星球边上的椅子坐下", "Go and sit on the chair at the planet's edge"),
    // tutorial
    ("b612.key.mouse", "鼠标", "Mouse"),
    (
        "b612.teach.look",
        "点一下画面，再移动鼠标环顾四周",
        "Click the view, then move the mouse to look around",
    ),
    ("b612.teach.run", "按住奔跑", "Hold to run"),
    ("b612.teach.tasks", "按住查看本章目标", "Hold to see the chapter goals"),
    ("b612.teach.touch_move", "用左下角的摇杆走动", "Use the stick at the lower left to walk"),
    (
        "b612.teach.touch_look",
        "在屏幕右侧拖动，环顾四周",
        "Drag on the right side of the screen to look around",
    ),
    (
        "b612.teach.touch_tasks",
        "点“任务”按钮查看本章目标",
        "Tap the tasks button to see the chapter goals",
    ),
    // targets & verbs
    ("b612.target.rose", "玫瑰", "The rose"),
    ("b612.target.can", "水壶", "Watering can"),
    ("b612.target.spring", "泉水", "The spring"),
    ("b612.target.cloche", "玻璃罩", "The glass cloche"),
    ("b612.target.chair", "椅子", "The chair"),
    ("b612.target.spot", "她身边", "Beside her"),
    ("b612.verb.pick_can", "拿起水壶", "Pick up the can"),
    ("b612.verb.put_back", "把水壶放回去", "Put the can back"),
    ("b612.verb.fill", "灌水", "Fill"),
    ("b612.verb.water", "浇水", "Water"),
    ("b612.verb.can_empty", "水壶是空的", "The can is empty"),
    ("b612.verb.lift_glass", "拿起玻璃罩", "Lift the glass"),
    ("b612.verb.put_glass", "把玻璃罩放回去", "Put the glass back"),
    ("b612.verb.put_spot", "把水壶放在花旁边", "Set the can down beside her"),
    ("b612.verb.sit", "坐下", "Sit down"),
    // inner voice
    (
        "b612.mono.empty",
        "水壶是空的。先去泉边灌满水吧。",
        "The can is empty. Fill it at the spring first.",
    ),
    ("b612.mono.put_first", "先把水壶放在她身边吧。", "Set the can down beside her first."),
    ("b612.mono.chair", "我很喜欢看日落。", "I'm very fond of sunsets."),
    ("b612.mono.day_over", "今天的太阳已经落下了。", "The sun has set for today."),
    // the rose calls
    ("b612.l.call1", "我想……现在该是吃早饭的时候了吧。", "I do believe… it is time for my breakfast."),
    (
        "b612.l.call2",
        "能不能劳驾你，好心地想着我一点？",
        "Would you be so kind as to think of me a little?",
    ),
    // farewell, part 1
    ("b612.l.silent", "……", "…"),
    ("b612.l.fw_bye", "再见了。", "Goodbye."),
    ("b612.l.fw_planets", "我想去看看别的星球。", "I want to go and see other planets."),
    (
        "b612.l.fw_cough",
        "（她轻轻咳了一声。不是因为着凉。）",
        "(She gives a small cough. It is not from a cold.)",
    ),
    ("b612.l.fw_silly", "我以前真傻。请你原谅我。", "I have been silly. Please forgive me."),
    (
        "b612.l.fw_love",
        "是的，我爱你。你一直都不知道，那是我的错。",
        "Yes, I love you. It was my fault you never knew.",
    ),
    ("b612.l.fw_you_too", "不过，你也和我一样傻。", "But you were just as silly as I was."),
    // the glass, part 2
    (
        "b612.l.c_leave",
        "把玻璃罩放下吧，我不想要它了。",
        "Put the glass down. I don't want it any more.",
    ),
    ("b612.l.c_wind", "可是风……", "But the wind…"),
    (
        "b612.l.c_air",
        "夜里的凉风对我有好处。我是一朵花呀。",
        "The cool night air will do me good. I am a flower.",
    ),
    ("b612.l.c_animals", "可是那些小动物……", "But the animals…"),
    ("b612.l.c_claws", "我什么也不怕。我有爪子呢。", "I'm afraid of nothing. I have my claws."),
    (
        "b612.l.c_thorns",
        "（她天真地亮出了自己的四根刺。）",
        "(Quite innocently, she shows her four thorns.)",
    ),
    (
        "b612.l.c_linger",
        "别再这样磨磨蹭蹭的了，真叫人心烦。",
        "Don't linger like this. It's tiresome.",
    ),
    ("b612.l.c_go", "你已经决定要走了。那就走吧。", "You have decided to go. So go."),
    // the last sunset
    ("b612.l.sad", "一个人难过的时候，就会喜欢看日落。", "When one is very sad, one loves sunsets."),
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
        models::register(app);
        app.add_systems(
            Update,
            (
                systems::tick,
                systems::trail,
                systems::birds,
                systems::butterflies,
                systems::wings,
                systems::particles,
            )
                .chain()
                .in_set(CoreSet::Gameplay)
                .run_if(in_chapter(ID)),
        )
        // the lens helper is shared with the king chapter (idle when unused)
        .add_systems(Update, director::apply_fov.in_set(CoreSet::Presentation));
    }

    fn build(&self, cx: &mut ChapterCx) {
        world::build(cx);
    }

    fn story(&self) -> Story {
        let rose = || Pos::anchor("rose");
        Story {
            events: vec![
                event("b612.ev.rose", &["b612.st.can", "b612.st.fill", "b612.st.water"]),
                event("b612.ev.sunset", &["b612.st.chair"]),
            ],
            hidden: None,
            script: vec![
                // 1. card over the sleeping prince, wake-up, orbit
                card(),
                wait_flag("opened", vec![t_wait(60.0)]),
                // 2. free roam + tutorial; the rose calls at 4 m whenever he comes close
                run(systems::start_explore),
                wait_until(
                    |w| w.resource::<StoryRunner>().has_flag("tut_done") || systems::near(w, w.resource::<systems::Sites>().rose, 4.0),
                    vec![t_goto(Pos::anchor("rose"))],
                ),
                run(|w| {
                    let r = w.resource::<systems::Sites>().rose;
                    if systems::near(w, r, 4.0) {
                        w.resource_mut::<StoryRunner>().set_flag("near_rose");
                    }
                }),
                if_flag("near_rose", "call"),
                run(systems::guide_to_rose),
                wait_near(rose(), 4.0),
                // 3. the rose calls
                label("call"),
                run(systems::call_begin),
                walk_to(Pos::anchor("call_stand")),
                face(rose()),
                run(systems::call_shot),
                wait(0.7),
                say(speakers::ROSE, "b612.l.call1"),
                say(speakers::ROSE, "b612.l.call2"),
                run(systems::call_end),
                // 4. look after the rose (steps follow the can: shed → spring → rose)
                start_event(0),
                wait_flag("watered", vec![t_use("can"), t_use("spring_0"), t_wait(6.0), t_use("rose_water")]),
                // 5. farewell
                run(systems::farewell_begin),
                if_not_flag("carrying", "farewell"),
                wait_use("can_spot"),
                label("farewell"),
                run(systems::farewell_walk),
                walk_to(Pos::anchor("fw_stand")),
                face(rose()),
                run(systems::two_shot),
                wait(1.0),
                say(speakers::PRINCE, "b612.l.fw_bye"),
                run(|w| {
                    set_rose_pose(w, RosePose::Droop);
                    tremble(w, 0.7);
                }),
                say_for(speakers::ROSE, "b612.l.silent", 2.6),
                say(speakers::PRINCE, "b612.l.fw_planets"),
                run(|w| tremble(w, 0.6)),
                say_for(NARR, "b612.l.fw_cough", 1.8),
                run(|w| {
                    set_rose_pose(w, RosePose::Lift);
                    systems::rose_close(w);
                }),
                say(speakers::ROSE, "b612.l.fw_silly"),
                run(systems::prince_close),
                say_for(speakers::PRINCE, "b612.l.silent", 1.4),
                run(systems::rose_close),
                say(speakers::ROSE, "b612.l.fw_love"),
                run(|w| {
                    set_rose_pose(w, RosePose::Rest);
                    systems::two_shot(w);
                }),
                say(speakers::ROSE, "b612.l.fw_you_too"),
                // the glass ritual
                run(systems::ritual_offer),
                wait_use("cloche"),
                wait_flag("cloche_held", vec![]),
                say(speakers::ROSE, "b612.l.c_leave"),
                say(speakers::PRINCE, "b612.l.c_wind"),
                run(|w| set_rose_pose(w, RosePose::Proud)),
                say(speakers::ROSE, "b612.l.c_air"),
                say(speakers::PRINCE, "b612.l.c_animals"),
                say(speakers::ROSE, "b612.l.c_claws"),
                say_for(NARR, "b612.l.c_thorns", 1.6),
                run(|w| set_rose_pose(w, RosePose::Droop)),
                say(speakers::ROSE, "b612.l.c_linger"),
                say(speakers::ROSE, "b612.l.c_go"),
                run(|w| set_rose_pose(w, RosePose::TurnAway)),
                wait(0.8),
                run(systems::place_offer),
                wait_use("cloche_place"),
                wait_flag("cloche_done", vec![]),
                run(systems::to_sunset),
                complete_event(0),
                // 6. the last sunset
                start_event(1),
                target(Pos::anchor("chair"), "b612.target.chair"),
                wait_use("chair"),
                wait_flag("seated", vec![]),
                clear_target(),
                wait(5.0),
                say_for(speakers::PRINCE, "b612.l.sad", 5.0),
                wait_flag("day_over", vec![]),
                run(systems::stand_up),
                complete_event(1),
                departure(),
            ],
        }
    }
}
