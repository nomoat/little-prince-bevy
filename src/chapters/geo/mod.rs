//! Chapter 7 — The Geographer (STUB). Registered so the flow works end to
//! end; the chapter agent replaces it with the full chapter following the
//! original's design. See chapters/b612 for a complete API example.

use crate::core::prelude::*;

pub struct Geo;

const ID: &str = "geo";

pub const STRINGS: StringTable = &[
    ("geo.title", "地理学家", "The Geographer"),
    ("geo.subtitle", "只记录永恒的事物", "Recording only what never changes"),
    ("geo.line", "一颗很大的星球，住着一位从不出门的学者。", "A big planet, home to a scholar who never goes outside."),
    ("geo.visited", "“转瞬即逝。”小王子第一次开始想念他的玫瑰。", "Ephemeral. For the first time, the prince misses his rose."),
    ("geo.loading", "书页翻动的声音……", "The rustle of turning pages…"),
    ("geo.ev.meet", "书房", "The study"),
    ("geo.st.meet", "去和地理学家说话", "Speak to the geographer"),
    ("geo.verb.talk", "说话", "Talk"),
    ("geo.l.npc", "一位探险家！快，说说你的星球。", "An explorer! Quick, tell me about your planet."),
    ("geo.l.prince", "我有三座火山，还有一朵花。", "I have three volcanoes, and a flower."),
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
        app.register_model("geo.npc", npc);
    }

    fn build(&self, cx: &mut ChapterCx) {
        let mut p = PlanetParams::sized(20.0);
        p.ground = hex(0x9fb08a);
        p.ground2 = hex(0x8ea079);
        p.mods = vec![TerrainMod::Flatten { center: Vec2::new(0.0, 5.0), radius: 2.0, blend: 1.5, height: None }];
        cx.planet(p);
        cx.sky(9.5, 0.5, Some(16.0), SkyPalette::default());
        let npc_p = cx.ground(0.0, 5.0);
        let npc = cx.model("model.geo.npc", npc_p, 180.0, 1.0);
        cx.collider(npc, Collider::cylinder(0.35, 1.5));
        cx.interactable(npc, Interactable::new("npc", "geo.verb.talk").radius(2.2).focus(Vec3::Y * 1.0).prompt_height(0.8));
        cx.anchor("npc", npc_p, 180.0);
        cx.scatter("model.rock", 6, 4.0, 19.0, 7, (0.5, 1.1), &[(Vec2::new(0.0, 5.0), 2.0)]);
        let start = cx.ground(0.0, -2.0);
        cx.player(start, 0.0);
    }

    fn story(&self) -> Story {
        Story {
            events: vec![event("geo.ev.meet", &["geo.st.meet"])],
            hidden: None,
            script: vec![
                card(),
                start_event(0),
                target(Pos::anchor("npc"), "geo.ev.meet"),
                wait_use("npc"),
                face(Pos::anchor("npc")),
                say(speakers::GEOGRAPHER, "geo.l.npc"),
                say(speakers::PRINCE, "geo.l.prince"),
                complete_event(0),
                departure(),
            ],
        }
    }
}

fn npc(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.36, 0.0), (0.33, 0.55), (0.24, 1.05), (0.13, 1.3), (0.0, 1.34)], 10, hex(0x5f8a62));
    m.sphere(Vec3::Y * 1.5, 0.21, 4, 8, hex(0xf2d2b6));
    m.ellipsoid(Vec3::new(0.0, 1.56, -0.03), Vec3::new(0.22, 0.14, 0.22), 3, 8, hex(0x8a8a8a));
    p.add(&m);
}
