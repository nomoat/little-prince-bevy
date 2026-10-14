//! Chapter 6 — The Lamplighter (STUB). Registered so the flow works end to
//! end; the chapter agent replaces it with the full chapter following the
//! original's design. See chapters/b612 for a complete API example.

use crate::core::prelude::*;

pub struct Lamp;

const ID: &str = "lamp";

pub const STRINGS: StringTable = &[
    ("lamp.title", "点灯人", "The Lamplighter"),
    ("lamp.subtitle", "一分钟就是一天", "Where a day lasts one minute"),
    ("lamp.line", "一颗转得飞快的小星星，上面只有一盏路灯。", "A tiny, fast-turning star with a single street lamp."),
    ("lamp.visited", "灯一会儿亮，一会儿灭，他从不休息。", "The lamp goes on, the lamp goes off; he never rests."),
    ("lamp.loading", "太阳刚落下，又要升起来了……", "The sun has just set, and already it rises…"),
    ("lamp.ev.meet", "路灯", "The lamp"),
    ("lamp.st.meet", "去和点灯人说话", "Speak to the lamplighter"),
    ("lamp.verb.talk", "说话", "Talk"),
    ("lamp.l.npc", "早上好。……晚上好。命令就是命令。", "Good morning. …Good evening. Orders are orders."),
    ("lamp.l.prince", "他是唯一一个不只想着自己的人。", "He is the only one who thinks of something besides himself."),
];

impl Chapter for Lamp {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 6,
            title: "lamp.title",
            subtitle: "lamp.subtitle",
            line: "lamp.line",
            visited_line: "lamp.visited",
            loading: "lamp.loading",
            map_pos: Vec2::new(1190.0, 300.0),
            map_radius: 42.0,
            map_colors: (hex(0xc7b48c), hex(0xffc864)),
            music: "music.lamp",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("lamp.npc", npc);
    }

    fn build(&self, cx: &mut ChapterCx) {
        let mut p = PlanetParams::sized(7.0);
        p.ground = hex(0xc7b48c);
        p.ground2 = hex(0xb5a27a);
        p.mods = vec![TerrainMod::Flatten { center: Vec2::new(0.0, 5.0), radius: 2.0, blend: 1.5, height: None }];
        cx.planet(p);
        cx.sky(11.0, 24.0, None, SkyPalette::default());
        let npc_p = cx.ground(0.0, 5.0);
        let npc = cx.model("model.lamp.npc", npc_p, 180.0, 1.0);
        cx.collider(npc, Collider::cylinder(0.35, 1.5));
        cx.interactable(npc, Interactable::new("npc", "lamp.verb.talk").radius(2.2).focus(Vec3::Y * 1.0).prompt_height(0.8));
        cx.anchor("npc", npc_p, 180.0);
        cx.scatter("model.rock", 6, 4.0, 6.0, 6, (0.5, 1.1), &[(Vec2::new(0.0, 5.0), 2.0)]);
        let start = cx.ground(0.0, -2.0);
        cx.player(start, 0.0);
    }

    fn story(&self) -> Story {
        Story {
            events: vec![event("lamp.ev.meet", &["lamp.st.meet"])],
            hidden: None,
            script: vec![
                card(),
                start_event(0),
                target(Pos::anchor("npc"), "lamp.ev.meet"),
                wait_use("npc"),
                face(Pos::anchor("npc")),
                say(speakers::LAMPLIGHTER, "lamp.l.npc"),
                say(speakers::PRINCE, "lamp.l.prince"),
                complete_event(0),
                departure(),
            ],
        }
    }
}

fn npc(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.36, 0.0), (0.33, 0.55), (0.24, 1.05), (0.13, 1.3), (0.0, 1.34)], 10, hex(0xd07a30));
    m.sphere(Vec3::Y * 1.5, 0.21, 4, 8, hex(0xf2d2b6));
    m.ellipsoid(Vec3::new(0.0, 1.56, -0.03), Vec3::new(0.22, 0.14, 0.22), 3, 8, hex(0x8a8a8a));
    p.add(&m);
}
