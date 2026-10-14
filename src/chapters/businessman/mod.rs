//! Chapter 5 — The Businessman (the original's `counting` chapter, scene
//! `StarObservatory`; design notes in docs/original/businessman.md).
//!
//! A small night island with a round glass-domed observatory; 28 rocks
//! float around it. The prince pushes open the double door, talks with the
//! man who owns the stars across his desk (four camera beats: two-shot,
//! ledger, the look up at the dome, the stumped merchant), and may find his
//! own planet through the brass telescope (hidden goal). Numbers follow the
//! original: spawn, door, desk, stance, camera specs, lens set and the B-612
//! offset.

pub mod kit;
mod models;
mod scope;
mod world;

use crate::core::camera::CameraTuning;
use crate::core::prelude::*;
use kit::{dolly_to, shot_fov, start_dolly, talk};
use world::*;

pub struct Businessman;

const ID: &str = "businessman";

pub const STRINGS: StringTable = &[
    ("businessman.title", "商人的星球", "The Businessman's Planet"),
    ("businessman.subtitle", "一个忙得没空抬头的人", "A man too busy to look up"),
    ("businessman.line", "小行星 328。这里住着一个商人，他整夜数星星，还说它们都归他所有。", "Asteroid 328. A businessman counts the stars all night long, and says they all belong to him."),
    ("businessman.visited", "小王子说起他的花和火山，商人张了张嘴，什么也答不上来。", "The prince spoke of his flower and his volcanoes, and the businessman had no answer."),
    ("businessman.loading", "夜里，一座玻璃圆顶下亮着灯……", "At night, a lamp burns under a glass dome…"),
    // event & steps
    ("businessman.ev.visit", "拜访数星星的人", "Visit the man who counts the stars"),
    ("businessman.st.door", "推开天文台的门", "Open the observatory door"),
    ("businessman.st.talk", "和商人说话", "Talk to the businessman"),
    ("businessman.hidden", "在星空里找到 B612", "Find B-612 among the stars"),
    ("businessman.t.door", "天文台", "The observatory"),
    ("businessman.t.merchant", "商人", "The businessman"),
    // verbs
    ("businessman.verb.open", "推开门", "Open the door"),
    ("businessman.verb.close", "关上门", "Close the door"),
    ("businessman.verb.talk", "说话", "Talk"),
    ("businessman.verb.scope", "看星星", "Look at the stars"),
    // the talk (our own words)
    ("businessman.l.greet", "晚上好。", "Good evening."),
    ("businessman.l1", "三加五得八，八加七十五……再加二十二……一共是五亿零一百六十二万二千七百三十一。", "Three and five make eight, and seventy-five… and twenty-two… that makes five hundred and one million, six hundred and twenty-two thousand, seven hundred and thirty-one."),
    ("businessman.l2", "五亿……五亿什么？", "Five hundred million… five hundred million what?"),
    ("businessman.l3", "嗯？还在这儿？就是天上那些亮闪闪的小东西，让闲人做梦的那些——星星！", "Hm? Still here? Those little glittering things in the sky that set idle people dreaming — stars!"),
    ("businessman.l4", "那你拿这五亿颗星星做什么呢？", "And what do you do with five hundred million stars?"),
    ("businessman.l5", "什么也不做。我拥有它们。", "Nothing. I own them."),
    ("businessman.l6", "拥有星星，对你有什么用呢？", "And what good does owning the stars do you?"),
    ("businessman.l7", "让我富有。我把星星的数目写在一张小纸条上，再把纸条锁进抽屉里。", "It makes me rich. I write the number of my stars on a little slip of paper, and lock the slip in a drawer."),
    ("businessman.l8", "就这样？", "And that's all?"),
    ("businessman.l9", "这就足够了！", "That is quite enough!"),
    ("businessman.l10", "我有一朵花，我每天给她浇水。我有三座火山，每个星期都把它们打扫干净。", "I have a flower, and I water her every day. I have three volcanoes, and I sweep them out every week."),
    ("businessman.l11", "我拥有它们，对它们是有用的。可你对星星，一点用处也没有。", "It is of some use to them that I own them. But you are of no use to the stars."),
    ("businessman.l.stumped", "……", "……"),
    ("businessman.mono.after", "大人们真是奇怪得很。", "Grown-ups really are very odd."),
    // telescope
    ("businessman.scope.lens", "换镜", "Lens"),
    ("businessman.scope.help", "拖动 / WASD 瞄准 · 滚轮或 C 换镜头 · E 离开", "Drag / WASD to aim · wheel or C to change lens · E to leave"),
    ("businessman.scope.warm", "那边有一颗暖黄色的星……换个更强的镜头看看。", "A warm yellow star over there… try a stronger lens."),
    ("businessman.scope.found", "三座火山，一朵玫瑰……是 B612！", "Three volcanoes and a rose… it's B-612!"),
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
        models::register(app);
        app.init_resource::<kit::Dolly>()
            .init_resource::<kit::SavedCamTuning>()
            .init_resource::<Door>()
            .init_resource::<Merchant>()
            .init_resource::<CamZone>()
            .init_resource::<scope::Scope>()
            .add_systems(Update, kit::restore_cam_tuning)
            .add_systems(Update, kit::hide_prince_near_camera.after(crate::core::camera::update_camera).run_if(in_chapter(ID)))
            .add_systems(
                Update,
                (
                    kit::update_dolly,
                    door_system,
                    merchant_anim,
                    (scope::on_use, scope::update, scope::visuals).chain(),
                    camera_zone,
                    floating_rocks,
                    motes,
                    lantern_flicker,
                    fill_light,
                )
                    .in_set(CoreSet::Gameplay)
                    .run_if(in_chapter(ID)),
            );
    }

    fn build(&self, cx: &mut ChapterCx) {
        // per-chapter resources
        cx.world.insert_resource(Door::default());
        cx.world.insert_resource(Merchant::default());
        cx.world.insert_resource(CamZone::default());
        cx.world.resource_mut::<kit::Dolly>().0 = None;

        // ---- the island: a flat floating rock (domed terrain, nearly level)
        let mut p = PlanetParams::sized(11.0);
        p.cap_flatten = 0.8;
        p.cap_radius = 60.0;
        p.noise_amp = 0.08;
        p.ground = hex(0x5d6a5a);
        p.ground2 = hex(0x4e5a52);
        p.dry = hex(0x6a6a70);
        p.rock = hex(0x5d5866);
        p.mods = vec![
            TerrainMod::Flatten { center: Vec2::ZERO, radius: 4.4, blend: 1.6, height: Some(0.0) },
            TerrainMod::Flatten { center: Vec2::new(0.0, -4.6), radius: 1.6, blend: 1.4, height: Some(0.0) },
            TerrainMod::Paint { center: Vec2::new(0.0, -6.2), radius: 1.0, blend: 2.5, color: hex(0x7a7468) },
            TerrainMod::Paint { center: Vec2::new(0.0, -4.4), radius: 0.9, blend: 1.0, color: hex(0x7a7468) },
            TerrainMod::Paint { center: Vec2::ZERO, radius: 4.0, blend: 0.6, color: hex(0x6e6a64) },
        ];
        cx.planet(p);
        // footsteps: star rug in the middle of the round room, boards round it
        cx.step_zone(Vec3::new(0.0, 1.2, 0.0), Vec3::new(1.4, 1.5, 1.4), "sfx.step_carpet");
        cx.step_zone(Vec3::new(0.0, 1.2, 0.0), Vec3::new(3.3, 1.5, 3.3), "sfx.step_wood");
        // night: the sun far below the horizon, time frozen
        let pal = SkyPalette {
            night_zenith: hex(0x05081c),
            night_horizon: Color::srgb(0.11, 0.16, 0.32),
            fog: 0.45,
            ..default()
        };
        cx.sky(0.3, 0.0, None, pal);
        cx.ambience("amb.businessman.night");

        // camera rig of this chapter (pivot 1.0, distance 3.2 [1.8, 5], pitch 12° [-30, 60], fov 50)
        kit::save_cam_tuning(cx.world);
        {
            let mut t = cx.world.resource_mut::<CameraTuning>();
            t.pivot_height = 1.0;
            t.default_distance = 3.2;
            t.min_distance = 1.8;
            t.max_distance = 5.0;
            t.default_pitch = 12.0;
            t.min_pitch = -30.0;
            t.max_pitch = 60.0;
            t.fov = 50.0;
        }
        cx.world.resource_mut::<crate::core::player::PlayerTuning>().walk_speed = 0.9;

        // ---- the observatory
        let floor = 0.36;
        cx.model("model.businessman.observatory", Vec3::ZERO, 0.0, 1.0);
        cx.model("model.businessman.dome", Vec3::ZERO, 0.0, 1.0);
        // plinth (walkable floor 0.36 m up), doorstep, wall ring with the south doorway
        let plinth = cx.empty(Vec3::ZERO, 0.0);
        cx.collider(plinth, Collider::cylinder(3.62, floor).no_camera());
        let step = cx.empty(Vec3::new(0.0, 0.0, -3.92), 0.0);
        cx.collider(step, Collider::cuboid(Vec3::new(1.9, 0.18, 0.5)).no_camera());
        for i in 0..24 {
            let a = i as f32 * 15.0 + 7.5;
            if (a - 180.0).abs() < 10.0 {
                continue;
            }
            let pos = Vec3::new(a.to_radians().sin() * 3.375, 0.0, a.to_radians().cos() * 3.375);
            let e = cx.empty(pos, a);
            cx.collider(e, Collider::cuboid(Vec3::new(0.95, 3.0, 0.3)));
        }
        for x in [-0.9f32, 0.9] {
            let e = cx.empty(Vec3::new(x, 0.0, -3.36), 0.0);
            cx.collider(e, Collider::cuboid(Vec3::new(0.12, 3.0, 0.32)));
        }
        // the double door: hinges at x = ±0.85, z = -3.2; leaves swing inward
        for (x, base_yaw, sign) in [(-0.85f32, 0.0f32, -1.0f32), (0.85, 180.0, 1.0)] {
            let e = cx.model("model.businessman.door_leaf", Vec3::new(x, floor + 0.01, -3.2), base_yaw, 1.0);
            cx.world.entity_mut(e).insert(DoorLeaf { sign, base_yaw });
        }
        let blocker = cx.empty(Vec3::new(0.0, floor, -3.2), 0.0);
        cx.collider(blocker, Collider::cuboid(Vec3::new(1.7, 2.25, 0.14)));
        cx.world.entity_mut(blocker).insert(DoorBlocker);
        let door_i = cx.empty(Vec3::new(0.0, floor, -3.2), 180.0);
        let mut it = Interactable::new("door", "businessman.verb.open").radius(1.8).focus(Vec3::Y * 1.1).prompt_height(0.4).priority(3);
        it.max_facing_angle = 110.0;
        cx.interactable(door_i, it);
        cx.anchor("door", Vec3::new(0.0, 0.0, -3.6), 0.0);
        cx.anchor("door_outside", Vec3::new(0.0, 0.0, -6.0), 0.0);

        // ---- interior
        let at = |x: f32, z: f32| Vec3::new(x, floor, z);
        cx.model("model.businessman.rug", at(0.0, 0.0) + Vec3::Y * 0.005, 0.0, 1.0);
        let tel = cx.model("model.businessman.telescope", at(0.0, 0.0) + Vec3::Y * 0.01, scope::TUBE_AZ, 1.0);
        cx.collider(tel, Collider::cylinder(0.32, 1.5).no_camera());
        // the telescope interactable faces the eyepiece side (±75°), radius 1.5, facing 100°, priority 2
        let scope_i = cx.empty(at(0.0, 0.0), scope::TUBE_AZ + 180.0);
        let mut it = Interactable::new("telescope", "businessman.verb.scope").radius(1.5).prompt_height(0.35).priority(2);
        it.max_facing_angle = 100.0;
        it.side_half_angle = Some(75.0);
        it.focus_offset = kit::rot_y(scope::TUBE_AZ, Vec3::new(0.0, 1.085, -0.451));
        cx.interactable(scope_i, it);
        scope::build(cx, at(0.0, 0.0));
        let stance = cx.world.resource::<scope::Scope>().stance;
        cx.anchor("scope_stance", stance, scope::TUBE_AZ);

        let desk = cx.model("model.businessman.desk", at(0.0, 2.0), 0.0, 1.0);
        cx.world.entity_mut(desk).insert(DeskRoot);
        cx.collider(desk, Collider::cuboid(Vec3::new(1.5, 0.8, 0.78)));
        let chair = cx.model("model.businessman.chair", at(0.0, 2.58), 180.0, 1.0);
        cx.collider(chair, Collider::cylinder(0.42, 1.4));
        let merchant = cx.model("model.businessman.merchant", at(0.0, 2.58) + Vec3::Y * 0.0, 180.0, 1.0);
        cx.world.entity_mut(merchant).insert(MerchantRoot);
        cx.anchor("merchant", at(0.0, 2.58) + Vec3::Y * 1.1, 180.0);
        cx.anchor("desk_front", at(0.0, 0.9), 0.0);
        cx.anchor("talk_stance", at(0.0, 1.0), 0.0);
        // "Talk" within 2.1 m of the desk front, front side only
        let talk_i = cx.empty(at(0.0, 0.9), 180.0);
        let mut it = Interactable::new("talk", "businessman.verb.talk").radius(2.1).focus(Vec3::Y * 0.9).prompt_height(0.7).priority(1).disabled();
        it.side_half_angle = Some(80.0);
        cx.interactable(talk_i, it);

        // furniture ring (bookshelf NE, archive NW, side desk E, table W, low cabinet SE, bench cabinet SW)
        let ring = |a: f32, r: f32| Vec3::new(a.to_radians().sin() * r, floor, a.to_radians().cos() * r);
        let furn = [
            ("model.businessman.bookshelf", 40.0f32, 2.95f32, Vec3::new(1.0, 2.1, 0.36)),
            ("model.businessman.cabinet", -42.0, 2.95, Vec3::new(0.9, 1.6, 0.44)),
            ("model.businessman.side_desk", 92.0, 2.75, Vec3::new(1.1, 0.8, 0.55)),
            ("model.businessman.table", -95.0, 2.55, Vec3::new(0.8, 0.75, 0.8)),
            ("model.businessman.low_cabinet", 132.0, 2.95, Vec3::new(1.0, 0.85, 0.44)),
            ("model.businessman.low_cabinet", -132.0, 2.95, Vec3::new(1.0, 0.85, 0.44)),
        ];
        let mut tops = Vec::new();
        for (key, a, r, size) in furn {
            let pos = ring(a, r);
            let e = cx.model(key, pos, a + 180.0, 1.0);
            cx.collider(e, Collider::cuboid(size).not_walkable());
            tops.push((pos, a, size.y));
        }
        let stool = cx.model("model.businessman.stool", ring(72.0, 2.1), 0.0, 1.0);
        cx.collider(stool, Collider::cylinder(0.2, 0.5).no_camera());
        // armillary spheres: on the side desk, on the round table, on the low cabinet
        cx.model("model.businessman.armillary", tops[2].0 + Vec3::Y * 0.765 + kit::rot_y(tops[2].1 + 180.0, Vec3::new(0.3, 0.0, 0.0)), 20.0, 1.0);
        cx.model("model.businessman.armillary", tops[3].0 + Vec3::Y * 0.72, 0.0, 1.1);
        cx.model("model.businessman.armillary", tops[4].0 + Vec3::Y * 0.84, 60.0, 0.9);
        cx.model("model.businessman.sextant", tops[5].0 + Vec3::Y * 0.84, 40.0, 1.0);
        cx.model("model.businessman.scrolls", tops[3].0 + Vec3::Y * 0.72 + Vec3::new(0.15, 0.0, 0.18), 0.0, 1.0);
        cx.model("model.businessman.oil_lamp", at(0.55, 2.2) + Vec3::Y * 0.79, 0.0, 1.0);
        cx.model("model.businessman.candles", tops[2].0 + Vec3::Y * 0.765 + kit::rot_y(tops[2].1 + 180.0, Vec3::new(-0.4, 0.0, 0.05)), 0.0, 1.0);
        cx.model("model.businessman.boxes", ring(-165.0, 2.7), 15.0, 1.0);
        let bx = cx.empty(ring(-165.0, 2.7), 15.0);
        cx.collider(bx, Collider::cuboid(Vec3::new(0.5, 0.65, 0.42)).not_walkable());
        // five wall charts: north, moon, constellation, zodiac, orbits
        for (key, a) in [
            ("model.businessman.chart_north", 0.0f32),
            ("model.businessman.chart_moon", 66.0),
            ("model.businessman.chart_constellation", -68.0),
            ("model.businessman.chart_zodiac", 112.0),
            ("model.businessman.chart_orbits", -112.0),
        ] {
            let pos = ring(a, 3.23) + Vec3::Y * 1.75;
            cx.model(key, pos, a + 180.0, 1.0);
        }
        // lanterns: 4 outside (r 3.9, h 2.1), 4 inside (h 2.2); flicker
        for (i, a) in [45.0f32, 135.0, 225.0, 315.0].iter().enumerate() {
            let pos = Vec3::new(a.to_radians().sin() * 3.52, 2.1, a.to_radians().cos() * 3.52);
            cx.model("model.businessman.lantern", pos, *a, 1.0);
            if i % 2 == 0 {
                let lp = Vec3::new(a.to_radians().sin() * 3.9, 2.05, a.to_radians().cos() * 3.9);
                let l = cx.point_light(lp, hex(0xffb868), 26000.0, 7.0);
                cx.world.entity_mut(l).insert(Flicker { base: 26000.0, seed: i as f32 * 1.7 });
            }
        }
        for (i, a) in [20.0f32, 160.0, 200.0, 340.0].iter().enumerate() {
            let pos = Vec3::new(a.to_radians().sin() * 3.22, floor + 1.95, a.to_radians().cos() * 3.22);
            cx.model("model.businessman.lantern", pos, a + 180.0, 1.0);
            if i % 2 == 1 {
                let lp = Vec3::new(a.to_radians().sin() * 2.95, floor + 1.85, a.to_radians().cos() * 2.95);
                let l = cx.point_light(lp, hex(0xffb060), 30000.0, 6.5);
                cx.world.entity_mut(l).insert(Flicker { base: 30000.0, seed: 5.0 + i as f32 });
            }
        }
        let desk_lamp = cx.point_light(at(0.5, 2.15) + Vec3::Y * 1.05, hex(0xffc070), 22000.0, 4.5);
        cx.world.entity_mut(desk_lamp).insert(Flicker { base: 22000.0, seed: 9.0 });
        let fill = cx.point_light(Vec3::ZERO, hex(0xffd0a0), 2600.0, 3.0);
        cx.world.entity_mut(fill).insert(FillLight);

        // dust motes in the lamplight (130 max, 6-16 mm, warm, alpha 0.45)
        {
            let w = &mut *cx.world;
            let mesh = w.resource_mut::<Assets<Mesh>>().add(Sphere::new(0.005).mesh().ico(1).unwrap());
            let mat = w.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.86, 0.6, 0.45),
                emissive: LinearRgba::rgb(0.6, 0.45, 0.25),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });
            let mut rng = Rng::new(130);
            for _ in 0..130 {
                let p = Vec3::new(rng.range(-2.7, 2.7), floor + rng.range(0.2, 2.6), rng.range(-2.7, 2.7));
                let life = rng.range(14.0, 22.0);
                let size = rng.range(1.2, 3.2);
                w.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(p).with_scale(Vec3::splat(size)),
                    Mote { vel: Vec3::new(rng.range(-0.03, 0.03), rng.range(-0.01, 0.02), rng.range(-0.03, 0.03)), life, age: rng.range(0.0, life) },
                    bevy::light::NotShadowCaster,
                    ChapterScoped,
                ));
            }
        }

        // ---- 28 floating rocks drifting around the island (bob 0.08-0.14 m / 7-7.6 s, spin 2.4-2.5°/s)
        let mut rng = Rng::new(28);
        for i in 0..28 {
            let a = rng.range(0.0, 360.0);
            let r = rng.range(15.0, 34.0);
            let base = Vec3::new(a.to_radians().sin() * r, rng.range(-7.0, 7.0), a.to_radians().cos() * r);
            let scale = rng.range(0.6, 2.6) * if i % 7 == 0 { 1.8 } else { 1.0 };
            let e = cx.model("model.businessman.float_rock", base, rng.range(0.0, 360.0), scale);
            let axis = Vec3::new(rng.range(-1.0, 1.0), rng.range(-1.0, 1.0), rng.range(-1.0, 1.0)).normalize_or(Vec3::Y);
            cx.world.entity_mut(e).insert(FloatingRock { base, amp: rng.range(0.08, 0.14), period: rng.range(7.0, 7.6), phase: rng.range(0.0, 6.28), axis, spin: rng.range(2.4, 2.5) });
        }
        // a few stones and tufts on the island
        cx.scatter("model.rock_small", 9, 5.6, 10.0, 41, (0.7, 1.4), &[(Vec2::new(0.0, -6.0), 2.0)]);
        cx.scatter("model.grass", 26, 5.0, 10.2, 42, (0.8, 1.3), &[(Vec2::new(0.0, -6.0), 1.6)]);

        // ---- the prince lands south of the door, facing +Z; start framing pitch 3°, distance 4.2
        let start = cx.ground(-1.8, -8.2);
        cx.player(start, 0.0);
        let mut rig = cx.world.resource_mut::<CameraRig>();
        rig.pitch = 3f32.to_radians();
        rig.target_distance = 4.2;
        rig.distance = 4.2;
    }

    fn story(&self) -> Story {
        let t = |who, key| talk(STRINGS, who, key);
        let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
        Story {
            events: vec![event("businessman.ev.visit", &["businessman.st.door", "businessman.st.talk"])],
            hidden: Some("businessman.hidden"),
            script: vec![
                // opening: chapter card over the landing shot with a slow 3 m push, then a 4 s descent
                lock(),
                run(|w: &mut World| {
                    let (cam, look) = (Vec3::new(8.0, 8.5, -14.0), Vec3::new(0.93, 3.71, -5.56));
                    w.resource_mut::<CameraRig>().begin_shot(Shot { position: cam, look_at: look, blend: 0.0, fov: Some(45.0) });
                    let d = (look - cam).normalize() * 3.0;
                    start_dolly(w, cam + d, look + d, 45.0, 3.6);
                }),
                card(),
                run(|w: &mut World| w.resource_mut::<kit::Dolly>().0 = None),
                Cmd::EndShot(4.0),
                wait(3.2),
                run(|w: &mut World| w.resource_mut::<CamZone>().ease = 1.5),
                unlock(),
                start_event(0),
                Cmd::HiddenPending,
                target(Pos::anchor("door"), "businessman.t.door"),
                wait_flag("bm_door_open", vec![t_use("door")]),
                step(1),
                enable("talk"),
                target(Pos::anchor("merchant"), "businessman.t.merchant"),
                wait_use("talk"),
                disable("talk"),
                clear_target(),
                // the talk: the prince walks (0.9 m/s) to the stance across the desk
                lock(),
                run(|w: &mut World| {
                    // go round the telescope (it stands between the door and the desk)
                    let mut q = w.query_filtered::<&Transform, With<Player>>();
                    let pp = q.iter(w).next().map(|t| t.translation).unwrap_or(Vec3::ZERO);
                    let way = if pp.z < 0.45 && pp.x.abs() < 0.9 {
                        Vec3::new(if pp.x < -0.05 { -0.8 } else { 0.8 }, 0.36, if pp.z < -0.2 { -0.3 } else { 0.55 })
                    } else {
                        pp
                    };
                    w.resource_mut::<crate::core::story::Anchors>().0.insert("talk_way".into(), (way, 0.0));
                }),
                walk_to(Pos::anchor("talk_way")),
                walk_to(Pos::anchor("talk_stance")),
                face(Pos::anchor("merchant")),
                // beat 1: two-shot from the armillary side, pushing in over the greeting
                shot_fov(p(1.5, 1.75, 0.6), p(-0.1, 1.2, 1.7), 44.0, 1.0),
                dolly_to(p(1.32, 1.72, 0.82), p(-0.1, 1.2, 1.7), 42.0, 3.0),
                t(speakers::PRINCE, "businessman.l.greet"),
                shot_fov(p(0.3, 1.62, 1.25), p(0.0, 1.25, 2.4), 35.0, 0.8),
                t(speakers::MERCHANT, "businessman.l1"),
                t(speakers::PRINCE, "businessman.l2"),
                t(speakers::MERCHANT, "businessman.l3"),
                t(speakers::PRINCE, "businessman.l4"),
                t(speakers::MERCHANT, "businessman.l5"),
                t(speakers::PRINCE, "businessman.l6"),
                // beat 2: the ledger — the number goes on a slip, the slip into a drawer
                shot_fov(p(0.45, 1.75, 1.7), p(-0.05, 1.12, 2.2), 30.0, 0.9),
                run(|w: &mut World| set_mode(w, MerchantMode::Lock)),
                t(speakers::MERCHANT, "businessman.l7"),
                t(speakers::PRINCE, "businessman.l8"),
                t(speakers::MERCHANT, "businessman.l9"),
                // beat 3: wide — the prince looks up at the glass dome (in 0.8, hold 1.2, out 0.6)
                shot_fov(p(1.9, 1.35, -0.15), p(-0.2, 2.0, 1.8), 52.0, 1.0),
                pose(Pose::LookUp),
                wait(2.0),
                pose(Pose::None),
                lock(),
                wait(0.6),
                shot_fov(p(0.6, 1.78, 2.85), p(0.0, 1.25, 1.0), 33.0, 0.8),
                t(speakers::PRINCE, "businessman.l10"),
                t(speakers::PRINCE, "businessman.l11"),
                // beat 4: over the prince's right shoulder — the merchant is stumped
                shot_fov(p(0.36, 1.5, 0.3), p(-0.05, 1.3, 2.45), 40.0, 0.7),
                run(|w: &mut World| set_mode(w, MerchantMode::Stumped)),
                wait(1.3),
                say_for(speakers::MERCHANT, "businessman.l.stumped", 1.4),
                wait(1.0),
                // return: the play camera behind the prince, the telescope filling the view
                face(Pos::World(p(0.0, 1.0, -1.0))),
                run(|w: &mut World| {
                    w.resource_mut::<CameraRig>().snap_behind(std::f32::consts::PI);
                    w.resource_mut::<CameraRig>().end_shot(1.4);
                }),
                wait(1.2),
                unlock(),
                complete_event(0),
                mono("businessman.mono.after"),
                // test route: also find B-612 through the telescope (optional hidden goal)
                test_only(|w: &mut World| w.resource_mut::<StoryRunner>().set_flag("bm_test")),
                if_not_flag("bm_test", "bm_depart"),
                wait_until(
                    |w: &World| w.resource::<StoryRunner>().hidden_done,
                    vec![t_run(scope::test_to_stance), t_use("telescope"), t_wait(3.2), t_run(scope::test_aim), t_wait(3.0)],
                ),
                wait(1.0),
                run(scope::force_leave),
                wait_until(|w: &World| w.resource::<scope::Scope>().state == scope::ScopeState::Idle, vec![t_wait(5.0)]),
                // step away from the telescope so holding E departs instead of looking again
                test_only(|w: &mut World| {
                    let mut q = w.query::<(&mut Transform, &mut Player)>();
                    if let Some((mut t, mut p)) = q.iter_mut(w).next() {
                        t.translation = Vec3::new(1.3, 0.38, -1.6);
                        p.yaw = std::f32::consts::PI;
                    }
                }),
                label("bm_depart"),
                departure(),
            ],
        }
    }
}
