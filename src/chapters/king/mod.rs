//! Chapter 2 — The King's planet (the original's KingStory, scene
//! RoyalCourt). Beats, in order:
//!
//! 1. Card over the opening shot of the whole planet (push-in 3 m), then the
//!    camera settles to the play view while the narration plays.
//! 2. "Go and see the King": the prince pushes the palace door open himself
//!    (CourtDoor), walks into the throne hall (arrival hint), and talks to
//!    the King (radius 3.1 m, facing ±70°, front side ±70°).
//! 3. The audience: route to the stance on the King's right, two-shot and a
//!    slow 14 s push-in, the subject lines, the sunset request, the King
//!    consults his calendar (shot "calendar", nod at 5.2 s) and names 7:40,
//!    the prince's aside, the chair line, the hall shot, "I must go", and
//!    after 0.35 m the King leans out and appoints him ambassador.
//! 4. Departure.
//!
//! World: world.rs. Mechanics: systems.rs. Shots use King space
//! (+x = his right, +z = in front of him) exactly as the original's numbers.

mod models;
pub mod systems;
mod world;

use crate::core::prelude::*;
use systems::Gesture;

pub struct King;

const ID: &str = "king";

pub const STRINGS: StringTable = &[
    ("king.title", "国王的星球", "The King's Planet"),
    ("king.subtitle", "一座空荡荡的宫殿，住着一位国王", "An empty palace where a king lives"),
    (
        "king.line",
        "一位国王独自住在这里。在他眼里，所有人都是臣民。",
        "A king lives here alone. To him, everyone is a subject.",
    ),
    (
        "king.visited",
        "国王还坐在王座上，等着下一位臣民。",
        "The king is still on his throne, waiting for the next subject.",
    ),
    ("king.loading", "王宫的大门紧闭着……", "The palace door is shut…"),
    ("king.n1", "这里住着一位国王。", "A king lives here."),
    (
        "king.n2",
        "在他看来，所有人都是他的臣民——连星星也要听他的。",
        "To him, everyone is his subject — even the stars obey him.",
    ),
    ("king.ev.meet", "去见国王", "Go and see the King"),
    ("king.st.meet", "推开宫殿的大门，去见国王", "Push open the palace door and see the King"),
    ("king.target.king", "国王", "The King"),
    ("king.verb.door", "推开宫殿的大门", "Push the palace door"),
    ("king.verb.talk", "说话", "Talk"),
    (
        "king.hint.arrive",
        "大厅尽头的王座上，坐着一位国王。",
        "On the throne at the end of the hall sits a king.",
    ),
    // the audience (our own wording)
    ("king.l.subject", "啊！来了一位臣民！", "Ah! Here comes a subject!"),
    (
        "king.l.how",
        "（他从来没见过我，怎么知道我是他的臣民呢？）",
        "(He has never seen me. How can he know I am his subject?)",
    ),
    (
        "king.l.all",
        "凡是来到朕面前的，都是朕的臣民。这里的一切都服从朕——连星星也不例外。",
        "Whoever comes before me is my subject. Everything here obeys me — even the stars.",
    ),
    (
        "king.l.sunset",
        "那您能让太阳现在就落下去吗？我好想看一次日落。",
        "Then could you make the sun set right now? I would so like to see a sunset.",
    ),
    (
        "king.l.calendar",
        "当然可以。不过命令得在合适的时候下达。让朕查查……嗯，今晚七点四十分左右。到时候，朕会命令它落山。",
        "Of course. But an order must be given at the proper time. Let me see… hm, about twenty to eight this evening. Then I shall order it to set.",
    ),
    (
        "king.l.aside",
        "（到了七点四十分，太阳本来就会落山呀。）",
        "(At twenty to eight it would set anyway.)",
    ),
    (
        "king.l.chair",
        "在我的星球上，只要把椅子挪几步，就能看到日落。",
        "On my planet, I only had to move my chair a few steps to see a sunset.",
    ),
    ("king.l.go", "我该走了。", "I must be going."),
    ("king.l.ambassador", "等等！朕任命你为朕的大使！", "Wait! I appoint you my ambassador!"),
];

impl Chapter for King {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 2,
            title: "king.title",
            subtitle: "king.subtitle",
            line: "king.line",
            visited_line: "king.visited",
            loading: "king.loading",
            map_pos: Vec2::new(430.0, 420.0),
            map_radius: 54.0,
            map_colors: (hex(0xece6ef), hex(0x8a5fb8)),
            music: "music.king",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        models::register(app);
        app.add_systems(Update, systems::tick.in_set(CoreSet::Gameplay).run_if(in_chapter(ID)));
    }

    fn build(&self, cx: &mut ChapterCx) {
        world::build(cx);
    }

    fn story(&self) -> Story {
        let king = || Pos::anchor("king");
        Story {
            events: vec![event("king.ev.meet", &["king.st.meet"])],
            hidden: None,
            script: vec![
                // 1. card over the opening shot, then the camera settles
                run(systems::opening_shot),
                card(),
                run(systems::opening_settle),
                narrate(&["king.n1", "king.n2"]),
                // 2. go and see the King: push the door, walk in, talk
                start_event(0),
                target(king(), "king.target.king"),
                wait_flag("door_open", vec![t_use("door")]),
                wait_use("king"),
                // 3. the audience
                run(systems::audience_begin),
                lock(),
                walk_to(Pos::anchor("aud_a")),
                walk_to(Pos::anchor("aud_b")),
                face(king()),
                run(systems::shot_two),
                run(|w| systems::gesture(w, Gesture::TalkA)),
                wait(1.0),
                run(systems::shot_push),
                say(speakers::KING, "king.l.subject"),
                say(speakers::PRINCE, "king.l.how"),
                run(|w| systems::gesture(w, Gesture::TalkB)),
                say(speakers::KING, "king.l.all"),
                run(systems::shot_king),
                say(speakers::PRINCE, "king.l.sunset"),
                wait(2.2),
                run(|w| {
                    systems::shot_calendar(w);
                    systems::gesture(w, Gesture::Consult);
                }),
                wait(5.2),
                run(|w| {
                    systems::shot_king(w);
                    systems::gesture(w, Gesture::TalkA);
                }),
                say(speakers::KING, "king.l.calendar"),
                run(systems::shot_prince),
                say(speakers::PRINCE, "king.l.aside"),
                say(speakers::PRINCE, "king.l.chair"),
                run(systems::shot_hall),
                wait(1.0),
                say(speakers::PRINCE, "king.l.go"),
                run(systems::leave_walk),
                wait_until(|w| systems::walked(w, 0.35), vec![]),
                run(|w| systems::gesture(w, Gesture::LeanOut)),
                wait(0.35),
                say_for(speakers::KING, "king.l.ambassador", 3.0),
                wait_until(systems::arrived, vec![]),
                run(systems::audience_end),
                unlock(),
                complete_event(0),
                departure(),
            ],
        }
    }
}
