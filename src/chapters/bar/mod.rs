//! Chapter 4 — The Tippler (STUB). Registered so the flow works end to
//! end; the chapter agent replaces it with the full chapter following the
//! original's design. See chapters/b612 for a complete API example.

use crate::core::prelude::*;

pub struct Bar;

const ID: &str = "bar";

pub const STRINGS: StringTable = &[
    ("bar.title", "酒鬼", "The Tippler"),
    ("bar.subtitle", "为了忘记而喝", "Drinking in order to forget"),
    ("bar.line", "一盏昏暗的灯，一张摆满瓶子的桌子。", "A dim lamp and a table crowded with bottles."),
    ("bar.visited", "那张桌子旁的人，还在一圈一圈地喝。", "The man at the table is still drinking, round and round."),
    ("bar.loading", "夜色很浓，只有一扇窗亮着……", "The night is deep; only one window is lit…"),
    ("bar.ev.meet", "酒馆", "The tavern"),
    ("bar.st.meet", "去和酒鬼说话", "Speak to the tippler"),
    ("bar.verb.talk", "坐下", "Sit"),
    ("bar.l.npc", "我喝酒，是为了忘记。", "I drink so that I may forget."),
    ("bar.l.prince", "忘记什么呢？", "Forget what?"),
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
            map_pos: Vec2::new(840.0, 400.0),
            map_radius: 48.0,
            map_colors: (hex(0x7d7a6a), hex(0xf2b45a)),
            music: "music.bar",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("bar.npc", npc);
    }

    fn build(&self, cx: &mut ChapterCx) {
        let mut p = PlanetParams::sized(10.0);
        p.ground = hex(0x7d7a6a);
        p.ground2 = hex(0x6b685a);
        p.mods = vec![TerrainMod::Flatten { center: Vec2::new(0.0, 5.0), radius: 2.0, blend: 1.5, height: None }];
        cx.planet(p);
        cx.sky(20.5, 0.0, None, SkyPalette::default());
        let npc_p = cx.ground(0.0, 5.0);
        let npc = cx.model("model.bar.npc", npc_p, 180.0, 1.0);
        cx.collider(npc, Collider::cylinder(0.35, 1.5));
        cx.interactable(npc, Interactable::new("npc", "bar.verb.talk").radius(2.2).focus(Vec3::Y * 1.0).prompt_height(0.8));
        cx.anchor("npc", npc_p, 180.0);
        cx.scatter("model.rock", 6, 4.0, 9.0, 4, (0.5, 1.1), &[(Vec2::new(0.0, 5.0), 2.0)]);
        let start = cx.ground(0.0, -2.0);
        cx.player(start, 0.0);
    }

    fn story(&self) -> Story {
        Story {
            events: vec![event("bar.ev.meet", &["bar.st.meet"])],
            hidden: None,
            script: vec![
                card(),
                start_event(0),
                target(Pos::anchor("npc"), "bar.ev.meet"),
                wait_use("npc"),
                face(Pos::anchor("npc")),
                say(speakers::DRINKER, "bar.l.npc"),
                say(speakers::PRINCE, "bar.l.prince"),
                complete_event(0),
                departure(),
            ],
        }
    }
}

fn npc(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.36, 0.0), (0.33, 0.55), (0.24, 1.05), (0.13, 1.3), (0.0, 1.34)], 10, hex(0x50607a));
    m.sphere(Vec3::Y * 1.5, 0.21, 4, 8, hex(0xf2d2b6));
    m.ellipsoid(Vec3::new(0.0, 1.56, -0.03), Vec3::new(0.22, 0.14, 0.22), 3, 8, hex(0x8a8a8a));
    p.add(&m);
}
