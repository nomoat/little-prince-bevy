//! Chapter 6 — The Lamplighter (the original's `lamp` chapter, scene
//! `Lighter_Planet`; design notes in docs/original/lamp.md).
//!
//! A tiny rocky island in a sunset sea that turns once a minute. The path
//! climbs from the landing stone on the south shore to the crown, where one
//! street lamp stands with its lamplighter. The one-minute day runs on its
//! own clock (day.rs); the lamplighter lights the lamp at every dusk and puts
//! it out at every dawn. The talk takes control of the sky: it rushes to
//! dawn, to dusk, holds in the night, and rushes to dawn again, while the
//! prince learns about the orders. Side activity: the sunset seat, where the
//! day runs faster near the horizon.

mod day;
mod models;

use super::businessman::kit::{self, dir_from, shot_fov, start_dolly, talk};
use crate::core::camera::CameraTuning;
use crate::core::prelude::*;
use day::*;

pub struct Lamp;

const ID: &str = "lamp";

pub const STRINGS: StringTable = &[
    ("lamp.title", "点灯人的星球", "The Lamplighter's Planet"),
    ("lamp.subtitle", "一分钟转一圈的星球", "A planet that turns once a minute"),
    ("lamp.line", "小行星 329。所有星星里最小的一颗：一盏路灯，一个从不休息的人。", "Asteroid 329. The smallest of them all: one street lamp, and one man who never rests."),
    ("lamp.visited", "他本可以做我的朋友。可他的星球太小了，容不下两个人。", "He could have been my friend. But his planet is far too small for two."),
    ("lamp.loading", "太阳刚刚落进海里，又要升起来了……", "The sun has only just sunk into the sea, and already it rises…"),
    ("lamp.ev.lamp", "去看看那盏路灯", "Go and see the lamp"),
    ("lamp.st.talk", "和点灯人说话", "Talk to the lamplighter"),
    ("lamp.t.lamp", "路灯", "The lamp"),
    ("lamp.verb.talk", "说话", "Talk"),
    ("lamp.verb.sit", "坐下", "Sit down"),
    ("lamp.verb.stand", "站起来", "Stand up"),
    // the talk (our own words)
    ("lamp.l1", "早上好。你为什么刚刚把灯熄了？", "Good morning. Why did you just put out your lamp?"),
    ("lamp.l2", "这是命令。早上好。", "Those are the orders. Good morning."),
    ("lamp.l3", "那你为什么又把它点亮了？", "Then why have you just lit it again?"),
    ("lamp.l4", "没什么好明白的。命令就是命令。晚上好。", "There is nothing to understand. Orders are orders. Good evening."),
    ("lamp.l5", "我这份差事真够受的。从前还说得过去：早上熄灯，晚上点灯，白天剩下的时间休息，夜里剩下的时间睡觉。", "Mine is a terrible job. Once it made sense: out in the morning, lit at night, the rest of the day to rest and the rest of the night to sleep."),
    ("lamp.l6", "可这颗星球一年比一年转得快，命令却一个字也没改。", "But every year the planet turned faster, and the orders never changed a word."),
    ("lamp.l7", "现在它一分钟就转一圈，我连一秒钟也歇不下来。每分钟都要点一次灯，熄一次灯！", "Now it turns once a minute, and I haven't a second's rest. I light and put out the lamp once every minute!"),
    ("lamp.l8", "别人都会瞧不起他……可只有他不让我觉得可笑。也许是因为，他想着的是自己以外的东西。", "The others would all look down on him… yet he is the only one who doesn't seem ridiculous to me. Perhaps because he thinks of something besides himself."),
    ("lamp.mono.sunsets", "在这里，坐着就能看一次又一次的日落。", "Here, you can sit and watch one sunset after another."),
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
        models::register(app);
        app.init_resource::<DayClock>()
            .init_resource::<LampState>()
            .init_resource::<Lamplighter>()
            .init_resource::<Seat>()
            .init_resource::<Guide>()
            .add_systems(Update, guide.in_set(CoreSet::Gameplay).run_if(in_chapter(ID)))
            .add_systems(
                Update,
                (kit::update_dolly, tick_clock, lamplighter, lamp_glow, seat_system, boundary, surf)
                    .chain()
                    .in_set(CoreSet::Gameplay)
                    .run_if(in_chapter(ID)),
            )
            .add_systems(
                Update,
                drive_sky
                    .after(crate::core::sky::tick_sky_clock)
                    .before(crate::core::sky::apply_sky)
                    .run_if(in_chapter(ID)),
            )
            .add_systems(Update, sea_colour.after(crate::core::sky::apply_sky).run_if(in_chapter(ID)))
            .add_systems(Update, kit::hide_prince_near_camera.after(crate::core::camera::update_camera).run_if(in_chapter(ID)));
    }

    fn build(&self, cx: &mut ChapterCx) {
        cx.world.insert_resource(DayClock::default());
        cx.world.insert_resource(LampState::default());
        cx.world.insert_resource(Lamplighter::default());
        cx.world.insert_resource(Seat::default());
        cx.world.insert_resource(Guide::default());
        cx.world.resource_mut::<kit::Dolly>().0 = None;

        // ---- the island: a domed base below the sea, raised by a chain of
        // overlapping pads from the landing (south, ~0.7 m) to the crown (4.5 m)
        let mut p = PlanetParams::sized(12.0);
        p.top_y = -1.4;
        p.noise_amp = 0.12;
        p.noise_wavelength = 2.6;
        p.ground = hex(0x7a8452);
        p.ground2 = hex(0x8b8162);
        p.dry = hex(0x8a8070);
        p.rock = hex(0x5e564e);
        let pad = |x: f32, z: f32, r: f32, b: f32, h: f32| TerrainMod::Flatten { center: Vec2::new(x, z), radius: r, blend: b, height: Some(h) };
        p.mods = vec![
            pad(0.55, -8.7, 1.6, 2.2, 0.67),
            pad(0.4, -6.6, 1.6, 2.4, 1.25),
            pad(0.45, -4.4, 1.7, 2.4, 1.85),
            pad(0.5, -2.2, 1.7, 2.4, 2.5),
            pad(0.45, 0.0, 1.7, 2.4, 3.15),
            pad(0.35, 2.3, 1.7, 2.4, 3.85),
            pad(0.2, 4.7, 2.1, 2.4, 4.5),
            pad(2.45, -2.2, 0.9, 1.0, 2.35), // sunset ledge
            pad(2.35, 0.45, 0.9, 0.9, 3.0),  // tide pool shelf
            TerrainMod::Paint { center: Vec2::new(0.5, -8.6), radius: 1.6, blend: 1.4, color: hex(0xb7a77e) },
            TerrainMod::Paint { center: Vec2::new(0.4, -5.0), radius: 0.5, blend: 0.7, color: hex(0x9a8e72) },
            TerrainMod::Paint { center: Vec2::new(0.5, -1.0), radius: 0.5, blend: 0.7, color: hex(0x9a8e72) },
            TerrainMod::Paint { center: Vec2::new(0.3, 2.6), radius: 0.5, blend: 0.7, color: hex(0x9a8e72) },
            TerrainMod::Paint { center: Vec2::new(0.2, 4.6), radius: 1.2, blend: 1.0, color: hex(0x8e8a78) },
        ];
        cx.planet(p);
        // footsteps: bare rock all over the little island
        cx.step_zone(Vec3::new(0.0, 2.0, -1.4), Vec3::new(6.0, 8.0, 9.0), "sfx.step_stone");
        // the sky is driven by the one-minute day clock (day.rs)
        let pal = SkyPalette {
            dusk_horizon: hex(0xff8a4c),
            dusk_zenith: hex(0x3a4478),
            golden_horizon: hex(0xffc98a),
            fog: 1.0,
            ..default()
        };
        cx.sky(16.66, 0.0, None, pal);
        cx.sky_mut().latitude = 63.0;
        cx.ambience("amb.lamp.sea");
        {
            // the opening holds the clock so the sun meets the sea right after control
            let mut c = cx.world.resource_mut::<DayClock>();
            let first = c.dusk_cross - 4.6;
            c.s = first as f64;
            c.frozen = true;
            let (el, az) = c.eval(c.s);
            c.el = el;
            c.az = az;
        }

        // camera: start framing pitch 3°, distance 3.8
        kit::save_cam_tuning(cx.world);
        {
            let mut t = cx.world.resource_mut::<CameraTuning>();
            t.default_distance = 3.8;
            t.fov = 50.0;
        }
        cx.world.resource_mut::<crate::core::player::PlayerTuning>().walk_speed = 0.9;

        // ---- the sea (and its surf ring) to the horizon
        {
            let w = &mut *cx.world;
            let sea_mesh = w.resource_mut::<Assets<Mesh>>().add(bevy::mesh::MeshBuilder::build(&Circle::new(390.0).mesh().resolution(96)));
            let sea_mat = w.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
                base_color: hex(0x2a4a6a),
                perceptual_roughness: 0.16,
                reflectance: 0.55,
                ..default()
            });
            w.spawn((
                Name::new("Sea"),
                Sea,
                Mesh3d(sea_mesh),
                MeshMaterial3d(sea_mat),
                Transform::from_xyz(0.4, 0.0, -2.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                bevy::light::NotShadowCaster,
                ChapterScoped,
            ));
        }
        build_surf(cx);

        // ---- the crown: street lamp + lamplighter
        let lamp_p = cx.ground(0.2, 4.6);
        let lamp = cx.model("model.lamp.street_lamp", lamp_p, 0.0, 1.0);
        cx.world.entity_mut(lamp).insert(LampRoot);
        cx.collider(lamp, Collider::cylinder(0.26, 3.2));
        let ll = cx.point_light(lamp_p + Vec3::Y * 2.92, hex(0xffb35c), 0.0, 9.0);
        cx.world.entity_mut(ll).insert(LampLight);
        cx.anchor("lamp", lamp_p + Vec3::Y * 1.5, 0.0);
        cx.anchor("wick", Vec3::new(0.27, lamp_p.y + 2.92, 4.61), 0.0);
        let ll_p = cx.ground(1.23, 4.55);
        let man = cx.model("model.lamp.lamplighter", ll_p, -90.0, 1.0);
        cx.world.entity_mut(man).insert(LamplighterRoot);
        cx.collider(man, Collider::cylinder(0.3, 1.7));
        cx.interactable(man, Interactable::new("lamplighter", "lamp.verb.talk").radius(3.2).focus(Vec3::Y * 1.0).prompt_height(0.9));
        cx.anchor("lamplighter", ll_p + Vec3::Y * 1.45, -90.0);
        cx.anchor("approach", cx.ground(0.2, 2.95), 0.0);
        cx.anchor("talk_stance", cx.ground(0.35, 3.2), 0.0);

        // ---- the sunset seat (ledge), tide pool, landing stone, path stones
        let seat_p = cx.ground(2.52, -2.29);
        let seat = cx.model("model.lamp.seat", seat_p, 40.0, 1.0);
        let mut it = Interactable::new("seat", "lamp.verb.sit").radius(1.5).focus(Vec3::Y * 0.42).prompt_height(0.5).any_facing();
        it.priority = 1;
        cx.interactable(seat, it);
        cx.anchor("seat", seat_p + Vec3::Y * 0.42, 40.0);
        cx.anchor("seat_front", seat_p + kit::rot_y(40.0, Vec3::new(0.0, 0.0, 0.9)), 40.0);
        cx.model("model.lamp.tide_pool", cx.ground(2.4, 0.45), 0.0, 1.0);
        let pool = cx.empty(cx.ground(2.4, 0.45), 0.0);
        cx.collider(pool, Collider::cylinder(0.95, 0.2).not_walkable().no_camera());
        cx.model("model.lamp.stone", cx.ground(0.6, -8.55) + Vec3::Y * 0.02, 10.0, 1.6);
        for (i, (x, z)) in [(0.35f32, -7.3f32), (0.6, -6.2), (0.3, -5.2), (0.55, -4.0), (0.4, -2.9), (0.6, -1.8), (0.35, -0.6), (0.5, 0.6), (0.3, 1.7), (0.4, 2.8)].iter().enumerate() {
            cx.model("model.lamp.stone", cx.ground(*x, *z) + Vec3::Y * 0.02, i as f32 * 37.0, 0.8 + (i % 3) as f32 * 0.12);
        }
        // boulders on the slopes, grass and bushes where the rock is gentle
        let mut rng = Rng::new(329);
        let terrain = cx.terrain().cloned();
        if let Some(t) = terrain {
            let mut placed = 0;
            for _ in 0..400 {
                let (x, z) = (rng.range(-2.6, 3.4), rng.range(-9.5, 6.5));
                let h = t.height(x, z);
                let path = path_dist(Vec2::new(x, z));
                let near = |px: f32, pz: f32, r: f32| Vec2::new(x - px, z - pz).length() < r;
                if h < 0.35 || near(0.2, 4.6, 1.4) || near(1.23, 4.55, 1.0) || near(2.52, -2.29, 1.2) || near(2.4, 0.45, 1.3) || near(0.6, -8.55, 1.2) {
                    continue;
                }
                let slope = t.slope_deg(x, z);
                let key = if slope > 32.0 {
                    if rng.f() < 0.35 { "model.lamp.boulder" } else { continue }
                } else if path < 0.7 {
                    continue;
                } else if rng.f() < 0.6 {
                    "model.grass"
                } else if rng.f() < 0.5 {
                    "model.lamp.bush"
                } else {
                    "model.flower_yellow"
                };
                let s = rng.range(0.6, 1.2);
                cx.model(key, t.ground(x, z), rng.range(0.0, 360.0), s);
                placed += 1;
                if placed > 80 {
                    break;
                }
            }
        }

        // ---- distant cloud sea at the horizon and the "first star"
        let mut rng = Rng::new(61);
        for i in 0..22 {
            let a = i as f32 / 22.0 * 360.0 + rng.range(-6.0, 6.0);
            let r = rng.range(230.0, 330.0);
            let pos = Vec3::new(a.to_radians().sin() * r, rng.range(4.0, 22.0), a.to_radians().cos() * r);
            cx.model("model.lamp.cloud", pos, a + 90.0, rng.range(9.0, 18.0));
        }
        {
            let w = &mut *cx.world;
            let glow = kit::texture(w, "tex.lamp.star", "flare");
            let quad = w.resource_mut::<Assets<Mesh>>().add(kit::quad_mesh());
            let mat = w.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.9, 0.75, 0.0),
                base_color_texture: Some(glow),
                unlit: true,
                fog_enabled: false,
                alpha_mode: AlphaMode::Add,
                cull_mode: None,
                ..default()
            });
            let p = Vec3::new(-55.0, 35.0, 318.0);
            w.spawn((Mesh3d(quad), MeshMaterial3d(mat), Transform::from_translation(p).with_rotation(kit::face_toward(p, Vec3::new(0.0, 5.0, 0.0))).with_scale(Vec3::splat(9.0)), FirstStar, Visibility::Hidden, ChapterScoped));
        }

        // ---- the prince lands on the stone, facing +Z with the lamp up the hill
        let start = cx.ground(0.6, -8.55);
        cx.anchor("landing", start, 0.0);
        cx.player(start, 0.0);
        let mut rig = cx.world.resource_mut::<CameraRig>();
        rig.pitch = 3f32.to_radians();
        rig.target_distance = 3.8;
        rig.distance = 3.8;
    }

    fn story(&self) -> Story {
        let t = |who, key| talk(STRINGS, who, key);
        let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
        Story {
            events: vec![event("lamp.ev.lamp", &["lamp.st.talk"])],
            hidden: None,
            script: vec![
                // opening: card over the landing shot from the sea, push 3 m, 4 s descent;
                // the sun meets the sea just after control
                lock(),
                run(|w: &mut World| {
                    let (cam, look) = (Vec3::new(-16.0, 8.5, -18.0), Vec3::new(-1.0, 3.0, -2.0));
                    w.resource_mut::<CameraRig>().begin_shot(Shot { position: cam, look_at: look, blend: 0.0, fov: Some(55.0) });
                    let d = (look - cam).normalize() * 3.0;
                    start_dolly(w, cam + d, look + d, 55.0, 3.6);
                }),
                card(),
                run(|w: &mut World| {
                    w.resource_mut::<kit::Dolly>().0 = None;
                    w.resource_mut::<DayClock>().frozen = false;
                }),
                Cmd::EndShot(4.0),
                wait(3.6),
                unlock(),
                start_event(0),
                target(Pos::anchor("lamp"), "lamp.t.lamp"),
                // test route: try the sunset seat first (side activity)
                test_only(|w: &mut World| w.resource_mut::<StoryRunner>().set_flag("lamp_test")),
                if_not_flag("lamp_test", "lamp_talk"),
                wait_until(|w: &World| w.resource::<Seat>().seated, vec![t_goto(Pos::anchor("seat_front")), t_use("seat")]),
                wait(2.0),
                wait_until(|w: &World| !w.resource::<Seat>().seated, vec![t_use("seat")]),
                label("lamp_talk"),
                wait_use("lamplighter"),
                disable("lamplighter"),
                disable("seat"),
                clear_target(),
                lock(),
                run(|w: &mut World| {
                    w.resource_mut::<Lamplighter>().scripted = true;
                    start_guide(w);
                }),
                wait_until(|w: &World| w.resource::<Guide>().points.is_empty(), vec![]),
                face(Pos::anchor("lamplighter")),
                // beat 1: dawn two-shot — the sky rushes to the first dawn (2.2 s), the lamp goes out
                shot_fov(p(-3.45, 5.79, 1.91), p(0.79, 5.95, 3.88), 48.0, 1.0),
                run(|w: &mut World| {
                    let mut c = w.resource_mut::<DayClock>();
                    let to = c.dawn_cross - (RAISE + 1.0);
                    c.rush_to(to, 2.2);
                }),
                wait_until(|w: &World| w.resource::<DayClock>().held(), vec![]),
                run(|w: &mut World| {
                    // he puts it out on the sea line; the sky holds 4 s after sunrise
                    let c = w.resource::<DayClock>().dawn_cross + 4.0;
                    w.resource_mut::<DayClock>().hold_at(c);
                    start_act(w, Act::PutOut, false);
                }),
                wait(2.8),
                t(speakers::PRINCE, "lamp.l1"),
                t(speakers::LAMPLIGHTER, "lamp.l2"),
                // beat 2: dusk lamp shot — rush 2.6 s to the next dusk, he lights the lamp
                shot_fov(p(-2.55, 6.22, 1.88), p(0.79, 6.5, 3.88), 55.0, 1.0),
                run(|w: &mut World| {
                    let mut c = w.resource_mut::<DayClock>();
                    let to = c.dusk_cross - (RAISE + 1.333);
                    c.rush_to(to, 2.6);
                }),
                wait_until(|w: &World| w.resource::<DayClock>().held(), vec![]),
                run(|w: &mut World| {
                    // the sky settles 8 s after sunset (night)
                    let c = w.resource::<DayClock>().dusk_cross + 8.0;
                    w.resource_mut::<DayClock>().hold_at(c);
                    start_act(w, Act::Light, false);
                }),
                wait(3.2),
                t(speakers::PRINCE, "lamp.l3"),
                t(speakers::LAMPLIGHTER, "lamp.l4"),
                // beat 3: night face shot
                shot_fov(p(-1.58, 5.9, 3.24), p(0.98, 5.8, 4.6), 45.0, 1.0),
                t(speakers::LAMPLIGHTER, "lamp.l5"),
                t(speakers::LAMPLIGHTER, "lamp.l6"),
                t(speakers::LAMPLIGHTER, "lamp.l7"),
                // beat 4: dawn again; the prince faces the sunrise, the lamplighter looks up behind him
                shot_fov(p(-2.22, 5.79, 2.36), p(0.75, 5.79, 3.81), 52.0, 1.2),
                run(|w: &mut World| {
                    let mut c = w.resource_mut::<DayClock>();
                    let to = c.dawn_cross - (RAISE + 1.0);
                    c.rush_to(to, 2.2);
                }),
                wait_until(|w: &World| w.resource::<DayClock>().held(), vec![]),
                run(|w: &mut World| {
                    let c = w.resource::<DayClock>().dawn_cross + 3.0;
                    w.resource_mut::<DayClock>().hold_at(c);
                    start_act(w, Act::PutOut, false);
                    let sunrise = w.resource::<DayClock>().eval(w.resource::<DayClock>().dawn_cross as f64).1;
                    let mut q = w.query::<&mut Player>();
                    if let Some(mut pl) = q.iter_mut(w).next() {
                        pl.face_target = Some(sunrise.to_radians());
                    }
                }),
                wait(2.8),
                run(|w: &mut World| w.resource_mut::<Lamplighter>().look_up = true),
                Cmd::Mono("lamp.l8", 7.5),
                wait(7.5),
                // back to the turning day
                run(|w: &mut World| {
                    w.resource_mut::<Lamplighter>().look_up = false;
                    w.resource_mut::<Lamplighter>().scripted = false;
                    w.resource_mut::<DayClock>().mode = ClockMode::Free;
                    let yaw = {
                        let mut q = w.query::<&Player>();
                        q.iter(w).next().map(|p| p.yaw).unwrap_or(0.0)
                    };
                    w.resource_mut::<CameraRig>().snap_behind(yaw);
                    w.resource_mut::<CameraRig>().end_shot(1.4);
                }),
                wait(1.2),
                enable("seat"),
                unlock(),
                complete_event(0),
                departure(),
            ],
        }
    }
}

// ------------------------------------------------------------- the island --

/// The walkable spine of the island (landing → crown).
const PATH: [(f32, f32); 7] = [(0.55, -8.7), (0.4, -6.6), (0.45, -4.4), (0.5, -2.2), (0.45, 0.0), (0.35, 2.3), (0.2, 4.7)];

fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> (f32, Vec2) {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
    let q = a + ab * t;
    (p.distance(q), q)
}

fn path_dist(p: Vec2) -> f32 {
    PATH.windows(2).map(|w| seg_dist(p, Vec2::new(w[0].0, w[0].1), Vec2::new(w[1].0, w[1].1)).0).fold(f32::MAX, f32::min)
}

/// The invisible edge (the original's ring of 128 edge colliders): soft push
/// back toward the walkable rock, hard clamp, and fall recovery.
fn boundary(time: Res<Time>, seat: Res<Seat>, anchors: Res<crate::core::story::Anchors>, mut player: Query<(&mut Transform, &mut Player)>) {
    let Ok((mut tf, mut pl)) = player.single_mut() else { return };
    if seat.seated || matches!(pl.drive, Drive::External) {
        return;
    }
    let p = Vec2::new(tf.translation.x, tf.translation.z);
    // shapes: (closest point, soft radius, hard radius)
    let mut best: Option<(f32, Vec2, f32, f32)> = None;
    let mut consider = |d: f32, q: Vec2, soft: f32, hard: f32| {
        let excess = d - soft;
        if best.map(|b| excess < b.0).unwrap_or(true) {
            best = Some((excess, q, soft, hard));
        }
    };
    for w in PATH.windows(2) {
        let (d, q) = seg_dist(p, Vec2::new(w[0].0, w[0].1), Vec2::new(w[1].0, w[1].1));
        consider(d, q, 1.7, 2.3);
    }
    for (c, soft, hard) in [(Vec2::new(0.2, 4.7), 2.1, 2.6), (Vec2::new(2.45, -2.2), 0.9, 1.3), (Vec2::new(2.35, 0.45), 0.9, 1.3)] {
        consider(p.distance(c), c, soft, hard);
    }
    if let Some((excess, q, soft, hard)) = best {
        if excess > 0.0 {
            let dir = (q - p).normalize_or_zero();
            let k = (excess / (hard - soft)).clamp(0.0, 1.0);
            let push = 0.4 + (4.2 - 0.4) * k * k;
            let mut np = p + dir * push * time.delta_secs();
            let d = np.distance(q);
            if d > hard {
                np = q - dir * hard;
            }
            tf.translation.x = np.x;
            tf.translation.z = np.y;
        }
    }
    // fall recovery: into the sea → back on the landing stone
    if tf.translation.y < -0.4 {
        if let Some((l, _)) = anchors.get("landing") {
            tf.translation = l + Vec3::Y * 0.1;
            pl.velocity = Vec3::ZERO;
        }
    }
}

// -------------------------------------------------------------- the seat --

#[derive(Resource, Default)]
pub struct Seat {
    pub seated: bool,
    shown_mono: bool,
}

/// Sit / stand on the sunset seat. Seated view: yaw -24° from the seat's
/// facing, pitch -1°, distance 3.2, height 1.1, fov 55.
#[allow(clippy::too_many_arguments)]
fn seat_system(
    mut msgs: MessageReader<Interacted>,
    mut seat: ResMut<Seat>,
    mut clock: ResMut<DayClock>,
    anchors: Res<crate::core::story::Anchors>,
    mut rig: ResMut<CameraRig>,
    mut hud: ResMut<Hud>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut its: Query<&mut Interactable>,
) {
    for m in msgs.read() {
        if m.id != "seat" {
            continue;
        }
        let Ok((mut tf, mut pl)) = player.single_mut() else { continue };
        let Some((sp, yaw)) = anchors.get("seat") else { continue };
        if !seat.seated {
            seat.seated = true;
            tf.translation = sp;
            pl.yaw = yaw.to_radians();
            pl.velocity = Vec3::ZERO;
            pl.pose = Pose::Sit;
            pl.drive = Drive::External;
            clock.seated = true;
            let view_yaw = yaw - 24.0;
            let eye = sp + Vec3::Y * 1.1 - dir_from(view_yaw, -1.0) * 3.2;
            rig.begin_shot(Shot { position: eye, look_at: eye + dir_from(view_yaw, -1.0) * 10.0, blend: 1.2, fov: Some(55.0) });
            if !seat.shown_mono {
                seat.shown_mono = true;
                hud.monologue("lamp.mono.sunsets", 4.0);
            }
        } else {
            seat.seated = false;
            pl.pose = Pose::None;
            pl.drive = Drive::Player;
            clock.seated = false;
            tf.translation += kit::rot_y(yaw, Vec3::new(0.0, 0.05, 0.55));
            rig.snap_behind(pl.yaw);
            rig.end_shot(1.0);
        }
    }
    let prompt = if seat.seated { "lamp.verb.stand" } else { "lamp.verb.sit" };
    for mut it in &mut its {
        if it.id == "seat" && it.prompt != prompt {
            it.prompt = prompt.into();
        }
    }
}

// ----------------------------------------------------------- sea & surf --

#[derive(Component)]
pub struct Sea;
#[derive(Component)]
pub struct Surf(f32);

/// Breaking-wave rings along the shore (LighterSurf): the shoreline is found
/// by marching out from the island's middle until the rock meets the sea.
fn build_surf(cx: &mut ChapterCx) {
    let Some(t) = cx.terrain().cloned() else { return };
    let c = Vec2::new(0.4, -2.0);
    let n = 96;
    let mut shore = Vec::with_capacity(n);
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        let d = Vec2::new(a.sin(), a.cos());
        let mut r = 0.5;
        while r < 18.0 && t.height(c.x + d.x * r, c.y + d.y * r) > 0.02 {
            r += 0.1;
        }
        shore.push((d, r));
    }
    let w = &mut *cx.world;
    for (k, (inner, outer)) in [(-0.1f32, 0.55f32), (0.5, 1.4)].iter().enumerate() {
        let mut m = MeshBuilder::new();
        for i in 0..n {
            let (d0, r0) = shore[i];
            let (d1, r1) = shore[(i + 1) % n];
            let y = 0.03 + k as f32 * 0.005;
            let a0 = Vec3::new(c.x + d0.x * (r0 + inner), y, c.y + d0.y * (r0 + inner));
            let b0 = Vec3::new(c.x + d0.x * (r0 + outer), y, c.y + d0.y * (r0 + outer));
            let a1 = Vec3::new(c.x + d1.x * (r1 + inner), y, c.y + d1.y * (r1 + inner));
            let b1 = Vec3::new(c.x + d1.x * (r1 + outer), y, c.y + d1.y * (r1 + outer));
            m.quad(a0, a1, b1, b0, hex(0xffffff));
            m.quad(a0, b0, b1, a1, hex(0xffffff));
        }
        let mesh = w.resource_mut::<Assets<Mesh>>().add(m.build());
        let mat = w.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.4),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        w.spawn((Mesh3d(mesh), MeshMaterial3d(mat), Transform::default(), Surf(k as f32 * 1.7), bevy::light::NotShadowCaster, ChapterScoped));
    }
}

fn surf(
    time: Res<Time>,
    sky: Res<Sky>,
    q: Query<(&Surf, &MeshMaterial3d<StandardMaterial>)>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut audio: MessageWriter<AudioCmd>,
    mut next: Local<f32>,
) {
    let t = time.elapsed_secs();
    let light = 0.35 + 0.65 * (1.0 - sky.night);
    for (s, m) in &q {
        if let Some(mut mat) = mats.get_mut(&m.0) {
            let pulse = ((t * 0.9 + s.0).sin() * 0.5 + 0.5).powf(1.6);
            mat.base_color = Color::srgba(light, light * 0.97, light * 0.94, 0.12 + 0.38 * pulse);
        }
    }
    *next -= time.delta_secs();
    if *next <= 0.0 {
        *next = 4.0 + (t * 3.7).sin().abs() * 3.0;
        audio.write(AudioCmd::Sfx("sfx.lamp.surf".into(), 0.22));
    }
}

/// The sea takes the sky's horizon colour (set by the core sky each frame).
fn sea_colour(
    clear: Res<ClearColor>,
    sky: Res<Sky>,
    q: Query<&MeshMaterial3d<StandardMaterial>, With<Sea>>,
    clouds: Query<(&PartName, &MeshMaterial3d<StandardMaterial>), Without<Sea>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let h = clear.0;
    // the cloud sea glows with the horizon (they are far: no fog, a soft self-light)
    if let Some((_, m)) = clouds.iter().find(|(n, _)| n.0 == "cloud") {
        if let Some(mut mat) = mats.get_mut(&m.0) {
            let glow = mix_color(h, hex(0xffffff), 0.25 * (1.0 - sky.night));
            mat.emissive = glow.to_linear() * (0.55 - 0.4 * sky.night);
        }
    }
    for m in &q {
        if let Some(mut mat) = mats.get_mut(&m.0) {
            let deep = hex(0x163252);
            let c = mix_color(deep, h, 0.45 - 0.2 * sky.night);
            mat.base_color = scale_color(c, 1.0 - 0.55 * sky.night);
        }
    }
}

// ------------------------------------------------------------ the walk --

/// Story walk to the talk stance that follows the island's spine (so the
/// prince never tries to climb a cliff): nearest point of the path, up the
/// path, round the lamp, then the stance. A step that makes no progress for
/// 2.5 s is skipped by placing him at its end (like the original's
/// ScriptedApproach giving up and snapping).
#[derive(Resource, Default)]
pub struct Guide {
    pub points: Vec<Vec2>,
    stuck: f32,
    best: f32,
}

fn start_guide(w: &mut World) {
    let mut q = w.query_filtered::<&Transform, With<Player>>();
    let pp = q.iter(w).next().map(|t| Vec2::new(t.translation.x, t.translation.z)).unwrap_or(Vec2::ZERO);
    let stance = Vec2::new(0.35, 3.2);
    let mut pts = Vec::new();
    if pp.distance(stance) > 1.2 {
        // nearest point on the spine
        let mut best = (f32::MAX, Vec2::ZERO, 0usize);
        for (i, s) in PATH.windows(2).enumerate() {
            let (d, q) = seg_dist(pp, Vec2::new(s[0].0, s[0].1), Vec2::new(s[1].0, s[1].1));
            if d < best.0 {
                best = (d, q, i);
            }
        }
        if best.0 > 0.4 {
            pts.push(best.1);
        }
        let from_z = best.1.y;
        for (x, z) in PATH.iter() {
            if *z > from_z + 0.3 && *z < stance.y - 0.3 {
                pts.push(Vec2::new(*x, *z));
            }
            if *z >= stance.y && from_z > stance.y {
                // coming from beyond the lamp: go round its west side
                pts.push(Vec2::new(-1.0, 3.6));
                break;
            }
        }
    }
    pts.push(stance);
    let mut g = w.resource_mut::<Guide>();
    g.points = pts;
    g.stuck = 0.0;
    g.best = f32::MAX;
}

fn guide(time: Res<Time>, terrain: Option<Res<Terrain>>, mut g: ResMut<Guide>, mut player: Query<(&mut Transform, &mut Player)>) {
    let Some(&next) = g.points.first() else { return };
    let Ok((mut tf, mut pl)) = player.single_mut() else { return };
    let p = Vec2::new(tf.translation.x, tf.translation.z);
    let d = p.distance(next);
    if d < 0.3 {
        g.points.remove(0);
        g.best = f32::MAX;
        g.stuck = 0.0;
        if g.points.is_empty() {
            pl.drive = Drive::Player;
        }
        return;
    }
    if !matches!(pl.drive, Drive::WalkTo { target, .. } if target == next) {
        pl.drive = Drive::WalkTo { target: next, speed: 0.9, stop_radius: 0.2 };
    }
    if d < g.best - 0.05 {
        g.best = d;
        g.stuck = 0.0;
    } else {
        g.stuck += time.delta_secs();
        if g.stuck > 2.5 {
            let y = terrain.as_ref().map(|t| t.height(next.x, next.y)).unwrap_or(tf.translation.y) + 0.05;
            tf.translation = Vec3::new(next.x, y, next.y);
            pl.velocity = Vec3::ZERO;
        }
    }
}
