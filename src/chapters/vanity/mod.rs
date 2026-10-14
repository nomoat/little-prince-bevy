//! Chapter 3 — The Conceited Man (STUB). Registered so the flow works end to
//! end; the chapter agent replaces it with the full chapter following the
//! original's design. See chapters/b612 for a complete API example.

use crate::core::prelude::*;

pub struct Vanity;

const ID: &str = "vanity";

pub const STRINGS: StringTable = &[
    ("vanity.title", "爱虚荣的人", "The Conceited Man"),
    ("vanity.subtitle", "他只听得见掌声", "He only ever hears applause"),
    ("vanity.line", "一座小小的剧场，台上的人一直在等观众。", "A tiny theatre; the man on stage is always waiting for an audience."),
    ("vanity.visited", "帽子还在举着，座位还是空的。", "The hat is still raised; the seats are still empty."),
    ("vanity.loading", "远处传来稀稀落落的掌声……", "Faint applause drifts from far away…"),
    ("vanity.ev.meet", "剧场", "The theatre"),
    ("vanity.st.meet", "去和台上的人说话", "Speak to the man on stage"),
    ("vanity.verb.talk", "说话", "Talk"),
    ("vanity.l.npc", "一位崇拜者！快，为我鼓掌吧！", "An admirer! Quick — applaud me!"),
    ("vanity.l.prince", "鼓掌之后，又该做什么呢？", "And after the applause, what then?"),
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
        app.register_model("vanity.npc", npc);
    }

    fn build(&self, cx: &mut ChapterCx) {
        let mut p = PlanetParams::sized(11.0);
        p.ground = hex(0xb98fb0);
        p.ground2 = hex(0xa77d9e);
        p.mods = vec![TerrainMod::Flatten { center: Vec2::new(0.0, 5.0), radius: 2.0, blend: 1.5, height: None }];
        cx.planet(p);
        cx.sky(15.0, 0.5, Some(17.5), SkyPalette::default());
        let npc_p = cx.ground(0.0, 5.0);
        let npc = cx.model("model.vanity.npc", npc_p, 180.0, 1.0);
        cx.collider(npc, Collider::cylinder(0.35, 1.5));
        cx.interactable(npc, Interactable::new("npc", "vanity.verb.talk").radius(2.2).focus(Vec3::Y * 1.0).prompt_height(0.8));
        cx.anchor("npc", npc_p, 180.0);
        cx.scatter("model.rock", 6, 4.0, 10.0, 3, (0.5, 1.1), &[(Vec2::new(0.0, 5.0), 2.0)]);
        let start = cx.ground(0.0, -2.0);
        cx.player(start, 0.0);
    }

    fn story(&self) -> Story {
        Story {
            events: vec![event("vanity.ev.meet", &["vanity.st.meet"])],
            hidden: None,
            script: vec![
                card(),
                start_event(0),
                target(Pos::anchor("npc"), "vanity.ev.meet"),
                wait_use("npc"),
                face(Pos::anchor("npc")),
                say(speakers::VAIN_MAN, "vanity.l.npc"),
                say(speakers::PRINCE, "vanity.l.prince"),
                complete_event(0),
                departure(),
            ],
        }
    }
}

fn npc(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.36, 0.0), (0.33, 0.55), (0.24, 1.05), (0.13, 1.3), (0.0, 1.34)], 10, hex(0xc0507a));
    m.sphere(Vec3::Y * 1.5, 0.21, 4, 8, hex(0xf2d2b6));
    m.ellipsoid(Vec3::new(0.0, 1.56, -0.03), Vec3::new(0.22, 0.14, 0.22), 3, 8, hex(0x8a8a8a));
    p.add(&m);
}
