//! Chapter 8 — The Earth (STUB). Registered so the flow works end to
//! end; the chapter agent replaces it with the full chapter following the
//! original's design. See chapters/b612 for a complete API example.

use crate::core::prelude::*;

pub struct Earth;

const ID: &str = "earth";

/// Earth-only cast member.
pub const FOX: Speaker = Speaker::new("earth.fox", Color::srgb(0.85, 0.5, 0.22));

pub const STRINGS: StringTable = &[
    ("earth.title", "地球", "The Earth"),
    ("earth.subtitle", "沙漠、狐狸与水井", "The desert, the fox and the well"),
    ("earth.line", "一颗非常大的星球。小王子落在了沙漠里。", "A very large planet. The little prince lands in the desert."),
    ("earth.visited", "沙漠之所以美丽，是因为某处藏着一口井。", "The desert is beautiful because somewhere it hides a well."),
    ("earth.loading", "一望无际的沙丘……", "Dunes as far as the eye can see…"),
    ("earth.ev.meet", "沙漠", "The desert"),
    ("earth.st.meet", "去找那只狐狸", "Find the fox"),
    ("earth.verb.talk", "靠近", "Approach"),
    ("earth.l.npc", "来驯服我吧。", "Please — tame me."),
    ("earth.l.prince", "驯服是什么意思？", "What does tame mean?"),
    ("earth.fox", "狐狸", "The Fox"),
];

impl Chapter for Earth {
    fn info(&self) -> ChapterInfo {
        ChapterInfo {
            id: ID,
            number: 8,
            title: "earth.title",
            subtitle: "earth.subtitle",
            line: "earth.line",
            visited_line: "earth.visited",
            loading: "earth.loading",
            map_pos: Vec2::new(1530.0, 250.0),
            map_radius: 78.0,
            map_colors: (hex(0xe2c08d), hex(0x86aad3)),
            music: "music.earth",
        }
    }

    fn strings(&self) -> StringTable {
        STRINGS
    }

    fn setup_app(&self, app: &mut App) {
        app.register_model("earth.npc", npc);
    }

    fn build(&self, cx: &mut ChapterCx) {
        let mut p = PlanetParams::sized(30.0);
        p.ground = hex(0xe2c08d);
        p.ground2 = hex(0xd4b07c);
        p.mods = vec![TerrainMod::Flatten { center: Vec2::new(0.0, 5.0), radius: 2.0, blend: 1.5, height: None }];
        cx.planet(p);
        cx.sky(9.0, 0.5, Some(16.5), SkyPalette::default());
        let npc_p = cx.ground(0.0, 5.0);
        let npc = cx.model("model.earth.npc", npc_p, 180.0, 1.0);
        cx.collider(npc, Collider::cylinder(0.35, 1.5));
        cx.interactable(npc, Interactable::new("npc", "earth.verb.talk").radius(2.2).focus(Vec3::Y * 1.0).prompt_height(0.8));
        cx.anchor("npc", npc_p, 180.0);
        cx.scatter("model.rock", 6, 4.0, 29.0, 8, (0.5, 1.1), &[(Vec2::new(0.0, 5.0), 2.0)]);
        let start = cx.ground(0.0, -2.0);
        cx.player(start, 0.0);
    }

    fn story(&self) -> Story {
        Story {
            events: vec![event("earth.ev.meet", &["earth.st.meet"])],
            hidden: None,
            script: vec![
                card(),
                start_event(0),
                target(Pos::anchor("npc"), "earth.ev.meet"),
                wait_use("npc"),
                face(Pos::anchor("npc")),
                say(FOX, "earth.l.npc"),
                say(speakers::PRINCE, "earth.l.prince"),
                complete_event(0),
                departure(),
            ],
        }
    }
}

fn npc(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.lathe(Vec3::ZERO, &[(0.36, 0.0), (0.33, 0.55), (0.24, 1.05), (0.13, 1.3), (0.0, 1.34)], 10, hex(0xd9823a));
    m.sphere(Vec3::Y * 1.5, 0.21, 4, 8, hex(0xf2d2b6));
    m.ellipsoid(Vec3::new(0.0, 1.56, -0.03), Vec3::new(0.22, 0.14, 0.22), 3, 8, hex(0x8a8a8a));
    p.add(&m);
}
