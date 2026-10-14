//! Chapter 2 — The King's Planet (STUB). Registered so the flow works end to
//! end; the chapter agent replaces it with the full chapter following the
//! original's design. See chapters/b612 for a complete API example.

use crate::core::prelude::*;

pub struct King;

const ID: &str = "king";

pub const STRINGS: StringTable = &[
    ("king.title", "国王的星球", "The King's Planet"),
    ("king.subtitle", "一位统治一切的君主", "A monarch who rules over everything"),
    ("king.line", "一位国王独自住在这里，他的长袍几乎铺满了整颗星球。", "A king lives here alone; his robe covers almost the whole planet."),
    ("king.visited", "国王还在等待“条件成熟”的日落。", "The king is still waiting for favourable conditions for his sunset."),
    ("king.loading", "一件长袍铺满了地平线……", "A robe stretches to the horizon…"),
    ("king.ev.meet", "觐见国王", "An audience with the king"),
    ("king.st.meet", "去和国王说话", "Speak to the king"),
    ("king.verb.talk", "觐见", "Greet"),
    ("king.l.npc", "啊！来了一个臣民。走近些，让我好好看看你。", "Ah! Here is a subject. Come closer, so I can see you better."),
    ("king.l.prince", "可是……这里只有您一个人呀。", "But… you are all alone here."),
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
        app.register_model("king.npc", npc);
    }

    fn build(&self, cx: &mut ChapterCx) {
        let mut p = PlanetParams::sized(12.0);
        p.ground = hex(0xe9e2ec);
        p.ground2 = hex(0xd9cfe0);
        p.mods = vec![TerrainMod::Flatten { center: Vec2::new(0.0, 5.0), radius: 2.0, blend: 1.5, height: None }];
        cx.planet(p);
        cx.sky(10.5, 0.5, Some(16.5), SkyPalette::default());
        let npc_p = cx.ground(0.0, 5.0);
        let npc = cx.model("model.king.npc", npc_p, 180.0, 1.0);
        cx.collider(npc, Collider::cylinder(0.35, 1.5));
        cx.interactable(npc, Interactable::new("npc", "king.verb.talk").radius(2.2).focus(Vec3::Y * 1.0).prompt_height(0.8));
        cx.anchor("npc", npc_p, 180.0);
        cx.scatter("model.rock", 6, 4.0, 11.0, 2, (0.5, 1.1), &[(Vec2::new(0.0, 5.0), 2.0)]);
        let start = cx.ground(0.0, -2.0);
        cx.player(start, 0.0);
    }

    fn story(&self) -> Story {
        Story {
            events: vec![event("king.ev.meet", &["king.st.meet"])],
            hidden: None,
            script: vec![
                card(),
                start_event(0),
                target(Pos::anchor("npc"), "king.ev.meet"),
                wait_use("npc"),
                face(Pos::anchor("npc")),
                say(speakers::KING, "king.l.npc"),
                say(speakers::PRINCE, "king.l.prince"),
                complete_event(0),
                departure(),
            ],
        }
    }
}

fn npc(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.36, 0.0), (0.33, 0.55), (0.24, 1.05), (0.13, 1.3), (0.0, 1.34)], 10, hex(0x7a5aa8));
    m.sphere(Vec3::Y * 1.5, 0.21, 4, 8, hex(0xf2d2b6));
    m.ellipsoid(Vec3::new(0.0, 1.56, -0.03), Vec3::new(0.22, 0.14, 0.22), 3, 8, hex(0x8a8a8a));
    p.add(&m);
}
