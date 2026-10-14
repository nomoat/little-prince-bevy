//! Chapter 5 — The Businessman (STUB). Registered so the flow works end to
//! end; the chapter agent replaces it with the full chapter following the
//! original's design. See chapters/b612 for a complete API example.

use crate::core::prelude::*;

pub struct Businessman;

const ID: &str = "businessman";

pub const STRINGS: StringTable = &[
    ("businessman.title", "商人", "The Businessman"),
    ("businessman.subtitle", "数星星的人", "The man who counts the stars"),
    ("businessman.line", "一个忙着计算的人，连抬头的时间都没有。", "A man so busy counting he never looks up."),
    ("businessman.visited", "他还在数。五亿零一百六十二万……", "He is still counting. Five hundred and one million…"),
    ("businessman.loading", "算盘的声音在夜里响着……", "The click of an abacus in the night…"),
    ("businessman.ev.meet", "天文台", "The observatory"),
    ("businessman.st.meet", "去和商人说话", "Speak to the businessman"),
    ("businessman.verb.talk", "说话", "Talk"),
    ("businessman.l.npc", "别打扰我！三加五等于八，八加七等于十五……", "Don't bother me! Three and five make eight, eight and seven fifteen…"),
    ("businessman.l.prince", "您数这些星星做什么呢？", "What do you do with all those stars?"),
];

impl Chapter for Businessman {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 5,
            title: "businessman.title",
            subtitle: "businessman.subtitle",
            line: "businessman.line",
            visited_line: "businessman.visited",
            loading: "businessman.loading",
            map_pos: Vec2::new(1050.0, 630.0),
            map_radius: 56.0,
            map_colors: (hex(0x9a9cb4), hex(0xd8e0ff)),
            music: "music.businessman",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("businessman.npc", npc);
    }

    fn build(&self, cx: &mut ChapterCx) {
        let mut p = PlanetParams::sized(13.0);
        p.ground = hex(0x9a9cb4);
        p.ground2 = hex(0x8a8ca4);
        p.mods = vec![TerrainMod::Flatten { center: Vec2::new(0.0, 5.0), radius: 2.0, blend: 1.5, height: None }];
        cx.planet(p);
        cx.sky(21.0, 0.0, None, SkyPalette::default());
        let npc_p = cx.ground(0.0, 5.0);
        let npc = cx.model("model.businessman.npc", npc_p, 180.0, 1.0);
        cx.collider(npc, Collider::cylinder(0.35, 1.5));
        cx.interactable(npc, Interactable::new("npc", "businessman.verb.talk").radius(2.2).focus(Vec3::Y * 1.0).prompt_height(0.8));
        cx.anchor("npc", npc_p, 180.0);
        cx.scatter("model.rock", 6, 4.0, 12.0, 5, (0.5, 1.1), &[(Vec2::new(0.0, 5.0), 2.0)]);
        let start = cx.ground(0.0, -2.0);
        cx.player(start, 0.0);
    }

    fn story(&self) -> Story {
        Story {
            events: vec![event("businessman.ev.meet", &["businessman.st.meet"])],
            hidden: None,
            script: vec![
                card(),
                start_event(0),
                target(Pos::anchor("npc"), "businessman.ev.meet"),
                wait_use("npc"),
                face(Pos::anchor("npc")),
                say(speakers::MERCHANT, "businessman.l.npc"),
                say(speakers::PRINCE, "businessman.l.prince"),
                complete_event(0),
                departure(),
            ],
        }
    }
}

fn npc(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.36, 0.0), (0.33, 0.55), (0.24, 1.05), (0.13, 1.3), (0.0, 1.34)], 10, hex(0x5a6078));
    m.sphere(Vec3::Y * 1.5, 0.21, 4, 8, hex(0xf2d2b6));
    m.ellipsoid(Vec3::new(0.0, 1.56, -0.03), Vec3::new(0.22, 0.14, 0.22), 3, 8, hex(0x8a8a8a));
    p.add(&m);
}
