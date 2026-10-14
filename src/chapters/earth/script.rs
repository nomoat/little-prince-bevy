//! The Earth story script (docs/original/earth.md): Valley beats V1–V4 with
//! the optional flower / echo, the railway page, then Dunes beats D1–D7 and
//! the end card. Beat line ids follow the original's constants.

use super::actors::{self, FoxState, PilotPose, SnakeState};
use super::fx::{self, glance, look};
use super::overlay::{self, Sketch};
use super::state::{Dusk, EarthState, Leg, Scene, WellRig};
use super::strings::*;
use super::world::{self, *};
use crate::core::camera::Shot;
use crate::core::i18n::I18n;
use crate::core::player::PlayerTuning;
use crate::core::prelude::*;
use crate::core::story::{Anchors, set_flag};
use crate::core::ui::hud::Fade;

fn test(w: &World) -> bool {
    w.resource::<crate::core::params::LaunchParams>().test
}

fn player(w: &mut World) -> Option<(Vec3, f32)> {
    let mut q = w.query::<(&Transform, &Player)>();
    q.iter(w).next().map(|(t, p)| (t.translation, p.yaw))
}

/// Places the prince (no camera snap) facing `face` (world point).
fn place_prince(w: &mut World, at: Vec2, face: Vec3) {
    let g = world::ground_of(w, at);
    let mut q = w.query::<(&mut Player, &mut Transform)>();
    if let Some((mut p, mut t)) = q.iter_mut(w).next() {
        t.translation = Vec3::new(at.x, t.translation.y.max(g.y), at.y);
        t.translation.y = g.y;
        let d = face - t.translation;
        p.yaw = d.x.atan2(d.z);
        p.face_target = Some(p.yaw);
        p.velocity = Vec3::ZERO;
    }
}

fn anchor(w: &World, name: &str) -> Vec3 {
    w.resource::<Anchors>().get(name).map(|a| a.0).unwrap_or(Vec3::ZERO)
}

fn walk_speed(s: f32) -> Cmd {
    run(move |w: &mut World| w.resource_mut::<PlayerTuning>().walk_speed = s)
}

fn shot_fov(w: &mut World, from: Vec3, look_at: Vec3, blend: f32, fov: Option<f32>) {
    w.resource_mut::<CameraRig>().begin_shot(Shot { position: from, look_at, blend, fov });
}

/// Side-on two-shot of the prince and `other`; `left_bias` pushes them toward the left of the frame.
fn two_shot(w: &mut World, other: Vec3, out: f32, left_bias: f32, blend: f32) {
    let Some((pp, _)) = player(w) else { return };
    let m = (pp + other) * 0.5;
    let u = (other - pp).with_y(0.0).normalize_or(Vec3::X);
    let mut perp = Vec3::new(-u.z, 0.0, u.x);
    // pick the side the current camera is on
    let cam = w.resource::<CameraRig>().last_pos;
    if perp.dot(cam - m) < 0.0 {
        perp = -perp;
    }
    let from = m + perp * out + Vec3::Y * 1.35 - u * 0.4;
    let f = (m - from).normalize();
    let right = f.cross(Vec3::Y).normalize_or_zero();
    let look_at = m + Vec3::Y * 1.0 + right * left_bias;
    shot_fov(w, from, look_at, blend, Some(45.0));
}

fn pilot_pos(w: &mut World) -> Vec3 {
    let mut q = w.query_filtered::<&Transform, With<actors::Pilot>>();
    q.iter(w).next().map(|t| t.translation).unwrap_or(Vec3::ZERO)
}

fn check_hidden(w: &mut World) {
    let r = w.resource::<StoryRunner>();
    if r.has_flag("flower_done") && r.has_flag("echo_done") {
        found_hidden(w);
    }
}

fn set_leg(w: &mut World, leg: Leg) {
    w.resource_mut::<EarthState>().leg = leg;
}

fn any_flag(flags: &'static [&'static str]) -> impl Fn(&World) -> bool + Send + Sync + 'static {
    move |w: &World| {
        let r = w.resource::<StoryRunner>();
        flags.iter().any(|f| r.has_flag(f))
    }
}

/// Side-talk dispatch at the top of every valley wait (flower / echo are optional).
fn side_dispatch() -> Vec<Cmd> {
    vec![if_flag("u_flower", "talk_flower"), if_flag("u_echo", "talk_echo")]
}

pub fn story() -> Story {
    let a = way_dir();
    let mut s: Vec<Cmd> = vec![
        card(),
        run(|w: &mut World| {
            if w.resource::<EarthState>().scene == Scene::Dunes {
                set_flag(w, "start_dunes");
            }
        }),
        if_flag("start_dunes", "dunes_debug"),
        // ================================================= V1 the snake
        start_event(0),
        target(Pos::anchor("snake"), "earth.tg.snake"),
        wait_use("snake"),
        clear_target(),
        lock(),
        run(|w: &mut World| actors::with_snake(w, false, |s, _| s.state = SnakeState::Raising)),
        face(Pos::anchor("snake")),
        wait(0.6),
        say(speakers::PRINCE, "earth.s1"),
        say(SNAKE, "earth.s2"),
        // S3: he looks up at his star (40 m above him, look up to 55°)
        run(|w: &mut World| {
            fx::show_his_star(w);
            look(w, 55.0, 0.0, 0.0, 45.0);
            let (Some((pp, yaw)), Some(star)) = (player(w), fx::his_star_pos(w)) else { return };
            let _ = star;
            let f = Vec3::new(yaw.sin(), 0.0, yaw.cos());
            let right = Vec3::new(-yaw.cos(), 0.0, yaw.sin());
            let cam = pp - f * 2.2 + right * 0.7 + Vec3::Y * 0.35;
            let dir = f * 40f32.to_radians().cos() + Vec3::Y * 40f32.to_radians().sin();
            shot_fov(w, cam, cam + dir * 10.0, 1.4, Some(62.0));
        }),
        wait(1.4),
        say(speakers::PRINCE, "earth.s3"),
        run(|w: &mut World| {
            look(w, 0.0, 0.0, 0.0, 60.0);
            fx::hide_his_star(w);
            w.resource_mut::<CameraRig>().end_shot(1.0);
        }),
        say(SNAKE, "earth.s4"),
        say(speakers::PRINCE, "earth.s5"),
        say(speakers::PRINCE, "earth.s6"),
        say(SNAKE, "earth.s7"),
        say(SNAKE, "earth.s8"),
        say(SNAKE, "earth.s9"),
        run(|w: &mut World| actors::with_snake(w, false, |s, _| s.state = SnakeState::Settling)),
        enable("flower"),
        enable("echo"),
        unlock(),
        complete_event(0),
        Cmd::HiddenPending,
        // ================================================= V2 roses (leg)
        start_event(1),
        target(
            Pos::enter(
                Pos::anchor("garden_gate"),
                Vec2::new(super::world::GX0, super::world::GZ0),
                Vec2::new(super::world::GX1, super::world::GZ1),
                Pos::anchor("garden"),
            ),
            "earth.tg.garden",
        ),
        run(|w: &mut World| set_leg(w, Leg::Roses)),
        enable("roses"),
        test_only(|w: &mut World| {
            set_flag(w, "t_need_flower");
            set_flag(w, "t_need_echo");
        }),
        label("w_dispatch"),
        if_flag("t_need_flower", "t_flower"),
        if_flag("t_need_echo", "t_echo"),
        if_flag("stage_train", "w_train"),
        if_flag("stage_fox", "w_fox"),
        goto("w_roses"),
        // test route step 2: the optional flower and echo
        label("t_flower"),
        unflag("t_need_flower"),
        wait_use("flower"),
        goto("talk_flower"),
        label("t_echo"),
        unflag("t_need_echo"),
        wait_use("echo"),
        goto("talk_echo"),
        // ---- V-a the little flower
        label("talk_flower"),
        unflag("u_flower"),
        disable("flower"),
        lock(),
        face(Pos::anchor("flower")),
        run(|w: &mut World| look(w, -45.0, 0.0, 0.0, 60.0)),
        wait(0.5),
        say(speakers::PRINCE, "earth.l10"),
        say(FLOWER, "earth.l11"),
        run(|w: &mut World| look(w, 0.0, 0.0, 0.0, 60.0)),
        flag("flower_done"),
        run(check_hidden),
        unlock(),
        goto("w_dispatch"),
        // ---- V-b the echo at the summit
        label("talk_echo"),
        unflag("u_echo"),
        disable("echo"),
        walk_speed(1.3),
        walk_to(Pos::anchor("summit")),
        walk_speed(1.0),
        lock(),
        face(Pos::anchor("garden")),
        wait(0.5),
        run(|w: &mut World| {
            let sp = anchor(w, "summit");
            let g = anchor(w, "garden");
            let f = (g - sp).with_y(0.0).normalize_or(Vec3::Z);
            let right = Vec3::new(-f.z, 0.0, f.x);
            shot_fov(w, sp - f * 4.5 + right * 1.4 + Vec3::Y * 1.6, sp + f * 10.0 + Vec3::Y * -1.0, 1.2, None);
        }),
        wait(1.0),
        say(speakers::PRINCE, "earth.l12"),
        sfx("sfx.earth.echo"),
        say_for(ECHO, "earth.l13", 2.6),
        sfx("sfx.earth.echo"),
        say_for(ECHO, "earth.l15", 2.4),
        end_shot(),
        flag("echo_done"),
        run(check_hidden),
        unlock(),
        goto("w_dispatch"),
        // ================================================= V2 the rose garden
        label("w_roses"),
        wait_until(any_flag(&["u_flower", "u_echo", "u_roses"]), vec![t_use("roses")]),
    ];
    s.extend(side_dispatch());
    s.extend([
        unflag("u_roses"),
        disable("roses"),
        clear_target(),
        lock(),
        face(Pos::anchor("roses_look")),
        say(speakers::PRINCE, "earth.r1"),
        say(ROSES, "earth.r2"),
        // he stands still 2.5 s, head down (ground 1.1 m ahead, ≤ 40°), and cries
        run(|w: &mut World| look(w, -40.0, 0.0, 0.0, 40.0)),
        wait(2.5),
        run(|w: &mut World| w.resource_mut::<Hud>().monologue("earth.r3", 4.0)),
        wait(4.2),
        run(|w: &mut World| look(w, 0.0, 0.0, 0.0, 40.0)),
        unlock(),
        flag("stage_fox"),
        complete_event(1),
        start_event(2),
        target(
            Pos::leave(
                Pos::anchor("garden_arch"),
                Vec2::new(super::world::GX0, super::world::GZ0),
                Vec2::new(super::world::GX1, super::world::GZ1),
                Pos::anchor("fox_tree"),
            ),
            "earth.tg.fox",
        ),
        run(|w: &mut World| set_leg(w, Leg::Fox)),
        enable("fox"),
        goto("w_dispatch"),
        // ================================================= V3 the fox
        label("w_fox"),
        wait_until(any_flag(&["u_flower", "u_echo", "u_fox"]), vec![t_use("fox")]),
    ]);
    s.extend(side_dispatch());
    s.extend([
        unflag("u_fox"),
        disable("fox"),
        clear_target(),
        lock(),
        run(actors::fox_begin_talk),
        wait_flag("fox_seated", vec![]),
        face(Pos::Interactable("fox".into())),
        run(|w: &mut World| {
            let mut q = w.query_filtered::<&Transform, With<actors::Fox>>();
            let fp = q.iter(w).next().map(|t| t.translation).unwrap_or(Vec3::ZERO);
            two_shot(w, fp, 3.4, 0.0, 1.2);
        }),
        say(speakers::PRINCE, "earth.f1"),
        say(FOX, "earth.f2"),
        say(speakers::PRINCE, "earth.f3"),
        say(FOX, "earth.f4"),
        say(FOX, "earth.f5"),
        say(speakers::PRINCE, "earth.f6"),
        say(FOX, "earth.f7"),
        // F8: he glances back toward the rose garden for 2.2 s (head + upper body, feet still)
        run(|w: &mut World| {
            let g = anchor(w, "garden");
            glance(w, g, 2.2);
        }),
        say(speakers::PRINCE, "earth.f8"),
        say(FOX, "earth.f9"),
        say(FOX, "earth.f10"),
        say(FOX, "earth.f11"),
        say(speakers::PRINCE, "earth.f12"),
        // his head comes up to his star over 2 s, holds 2 s
        run(|w: &mut World| {
            fx::show_his_star(w);
            look(w, 55.0, 0.0, 0.0, 27.5);
            let (Some((pp, yaw)), Some(star)) = (player(w), fx::his_star_pos(w)) else { return };
            let _ = star;
            let f = Vec3::new(yaw.sin(), 0.0, yaw.cos());
            let right = Vec3::new(-yaw.cos(), 0.0, yaw.sin());
            let cam = pp - f * 2.2 + right * 0.8 + Vec3::Y * 0.35;
            let dir = f * 40f32.to_radians().cos() + Vec3::Y * 40f32.to_radians().sin();
            shot_fov(w, cam, cam + dir * 10.0, 2.0, Some(62.0));
        }),
        wait(2.0),
        wait(2.0),
        run(|w: &mut World| {
            look(w, 0.0, 0.0, 0.0, 60.0);
            fx::hide_his_star(w);
            actors::fox_watch_wheat(w);
            w.resource_mut::<CameraRig>().end_shot(1.2);
        }),
        unlock(),
        flag("stage_train"),
        complete_event(2),
        start_event(3),
        target(Pos::anchor("board"), "earth.tg.station"),
        run(|w: &mut World| set_leg(w, Leg::Train)),
        enable("train"),
        goto("w_dispatch"),
        // ================================================= V4 the train
        label("w_train"),
        wait_until(any_flag(&["u_flower", "u_echo", "u_train"]), vec![t_use("train")]),
    ]);
    s.extend(side_dispatch());
    s.extend([
        unflag("u_train"),
        disable("train"),
        clear_target(),
        lock(),
        sfx("sfx.earth.whistle"),
        // he steps toward the carriage (0.6 s at 0.9 m/s), then the railway page blooms
        run(|w: &mut World| {
            let door = anchor(w, "door");
            if let Some((pp, _)) = player(w) {
                let d = (door - pp).xz();
                let t = pp.xz() + d.normalize_or_zero() * d.length().min(0.54);
                let mut q = w.query::<&mut Player>();
                if let Some(mut p) = q.iter_mut(w).next() {
                    p.drive = Drive::WalkTo { target: t, speed: 0.9, stop_radius: 0.05 };
                }
            }
        }),
        wait(0.6),
        run(|w: &mut World| {
            w.resource_mut::<EarthState>().leg = Leg::None;
            overlay::page_start(w);
        }),
        wait_flag("page_covered", vec![]),
        run(world::switch_to_dunes),
        run(overlay::page_loaded),
        wait_flag("page_done", vec![]),
        goto("dunes"),
        label("dunes_debug"),
        run(|w: &mut World| {
            for (i, d) in w.resource_mut::<StoryRunner>().done.iter_mut().enumerate() {
                *d = i < 3;
            }
        }),
        // ================================================= D1 arrival
        label("dunes"),
        lock(),
        run(|w: &mut World| {
            fx::start_train_arrival(w);
            let car1 = D_STEPOFF.x + 3.0;
            let h = anchor(w, "stepoff").y;
            let from = Vec3::new(car1 - 5.0, h + 3.0, D_RAIL_Z + 10.0);
            let look_at = Vec3::new(car1, h + 1.5, D_RAIL_Z);
            shot_fov(w, from, look_at, 0.0, Some(38.0));
            // keep the prince hidden aboard
            let mut q = w.query::<(&mut Player, &mut Visibility)>();
            if let Some((mut p, mut v)) = q.iter_mut(w).next() {
                p.drive = Drive::External;
                *v = Visibility::Hidden;
            }
            w.write_message(AudioCmd::sfx("sfx.earth.train"));
        }),
        wait_flag("train_stopped", vec![]),
        wait(0.2),
        run(|w: &mut World| overlay::veil(w, 1.0, 0.5)),
        wait(0.5),
        run(|w: &mut World| {
            let plane = world::ground_of(w, D_PLANE);
            place_prince(w, D_STEPOFF, plane);
            let mut q = w.query::<(&mut Player, &mut Visibility)>();
            if let Some((mut p, mut v)) = q.iter_mut(w).next() {
                p.drive = Drive::Player;
                *v = Visibility::Visible;
            }
            let yaw = world::yaw_to(D_STEPOFF, D_PLANE).to_radians();
            let mut rig = w.resource_mut::<CameraRig>();
            rig.end_shot(0.0);
            rig.snap_behind(yaw);
            overlay::veil(w, 0.0, 0.8);
        }),
        wait(0.8),
        unlock(),
        complete_event(3),
        // ================================================= D2 wake and draw
        start_event(4),
        target(Pos::anchor("pilot"), "earth.tg.pilot"),
        run(|w: &mut World| {
            let p = anchor(w, "pilot");
            w.resource_mut::<EarthState>().trail = Some(p);
        }),
        wait_use("pilot_wake"),
        run(|w: &mut World| w.resource_mut::<EarthState>().trail = None),
        disable("pilot_wake"),
        clear_target(),
        lock(),
        // beside his knees: −70° from his seated facing, 1.35 m
        run(|w: &mut World| {
            let b = anchor(w, "pilot");
            let at = D_BEDROLL + yaw_dir(D_PILOT_YAW - 70.0) * 1.35;
            place_prince(w, at, b);
            actors::with_pilot(w, |p, _| p.set_pose(PilotPose::SitUp, 1.33));
        }),
        wait(1.33),
        run(|w: &mut World| actors::with_pilot(w, |p, _| p.set_pose(PilotPose::Stand, 0.9))),
        wait(0.9),
        // the rest of the talk is standing; the prince 1.3 m in front
        run(|w: &mut World| {
            let at = D_BEDROLL + yaw_dir(D_PILOT_YAW) * 1.3;
            let pp = pilot_pos(w);
            place_prince(w, at, pp);
            let head = world::ground_of(w, at) + Vec3::Y * 1.0;
            actors::with_pilot(w, |p, _| p.look_at = Some(head));
            two_shot(w, pp, 4.6, 1.0, 1.0);
        }),
        wait(0.6),
        say(speakers::PRINCE, "earth.l36"),
        run(|w: &mut World| overlay::panel_show(w, None)),
        wait(2.0),
        run(|w: &mut World| overlay::panel_show(w, Some(Sketch::OldSheep))),
        wait(0.6),
        say(speakers::PRINCE, "earth.l73"),
        wait(1.5),
        run(|w: &mut World| overlay::panel_show(w, Some(Sketch::Box))),
        wait(0.6),
        say(PILOT, "earth.l37"),
        say(speakers::PRINCE, "earth.l38"),
        say(speakers::PRINCE, "earth.l65"),
        run(overlay::panel_hide),
        end_shot(),
        complete_event(4),
        // ================================================= D3 "Eight days later"
        run(|w: &mut World| {
            let (t, sub) = {
                let i = w.resource::<I18n>();
                (i.t("earth.card8").to_string(), i.t("earth.card8.sub").to_string())
            };
            let mut f = w.resource_mut::<Fade>();
            f.text = Some(t);
            f.sub = Some(sub);
            f.to(1.0, 0.6);
        }),
        wait(0.7),
        run(|w: &mut World| {
            // afternoon light; the aviator sits on the seat crate, the prince 60° off his facing toward the big dune
            {
                let mut st = w.resource_mut::<EarthState>();
                st.dusk = Dusk::Fixed(0.12);
                st.share = 0.12;
            }
            let crate_p = world::ground_of(w, D_CRATE);
            let toward = world::yaw_to(D_CRATE, D_DUNE_TOP);
            let pilot_yaw = toward - 60.0;
            actors::with_pilot(w, |p, t| {
                t.translation = crate_p;
                p.yaw = pilot_yaw.to_radians();
                p.yaw_target = None;
                p.set_pose(PilotPose::SitCrate, 0.0);
            });
            let at = D_CRATE + yaw_dir(toward) * 1.35;
            place_prince(w, at, crate_p + Vec3::Y * 0.8);
            let head = world::ground_of(w, at) + Vec3::Y * 1.0;
            actors::with_pilot(w, |p, _| p.look_at = Some(head));
            let yaw = world::yaw_to(at, D_CRATE).to_radians();
            w.resource_mut::<CameraRig>().snap_behind(yaw);
            two_shot(w, crate_p, 3.6, 0.0, 0.0);
        }),
        wait(3.5),
        run(|w: &mut World| {
            let mut f = w.resource_mut::<Fade>();
            f.to(0.0, 0.6);
        }),
        wait(0.6),
        run(|w: &mut World| {
            let mut f = w.resource_mut::<Fade>();
            f.text = None;
            f.sub = None;
        }),
        start_event(5),
        say(PILOT, "earth.l39"),
        run(|w: &mut World| look(w, 45.0, 0.0, 0.0, 50.0)),
        say(speakers::PRINCE, "earth.l71"),
        run(|w: &mut World| look(w, 0.0, 0.0, 0.0, 50.0)),
        say(PILOT, "earth.l74"),
        say(speakers::PRINCE, "earth.l75"),
        say(speakers::PRINCE, "earth.l49"),
        end_shot(),
        unlock(),
        // ================================================= D4 the well
        target(Pos::anchor("well"), "earth.tg.well"),
        wait_use("well"),
        disable("well"),
        clear_target(),
        lock(),
        sfx("sfx.earth.creak"),
        run(|w: &mut World| {
            // instant variant: one creak, the bucket is already on the rim
            let mut q = w.query_filtered::<Entity, With<WellRig>>();
            let Some(well) = q.iter(w).next() else { return };
            let mut parts = w.query::<(&PartName, &ChildOf, &mut Transform)>();
            for (n, c, mut t) in parts.iter_mut(w) {
                if c.parent() != well {
                    continue;
                }
                if n.0 == "bucket" {
                    *t = Transform::from_xyz(0.0, 0.73, 0.68);
                } else if n.0 == "rope" {
                    let d = Vec3::new(0.0, 1.15 - 1.6, 0.68);
                    *t = Transform::from_xyz(0.0, 1.6, 0.0).with_rotation(Quat::from_rotation_arc(Vec3::NEG_Y, d.normalize())).with_scale(Vec3::new(1.0, d.length(), 1.0));
                }
            }
        }),
        face(Pos::anchor("well")),
        say(speakers::PRINCE, "earth.l69"),
        run(|w: &mut World| overlay::memory(w, "earth.l79")),
        wait(3.5),
        unlock(),
        // ================================================= D5 the sacred way, the doorway snake
        step(1),
        target(Pos::anchor("gate"), "earth.tg.gate"),
        run(|w: &mut World| {
            let mut st = w.resource_mut::<EarthState>();
            st.dusk = Dusk::Corridor;
        }),
        wait_use("snake2"),
        disable("snake2"),
        clear_target(),
        lock(),
        run(|w: &mut World| {
            let door = anchor(w, "door");
            let spot = anchor(w, "snake_spot");
            place_prince(w, spot.xz(), door);
            let mut pq = w.query::<(&mut Transform, &Player)>();
            if let Some((mut t, _)) = pq.iter_mut(w).next() {
                t.translation.y = spot.y;
            }
            let share = w.resource::<EarthState>().share;
            w.resource_mut::<EarthState>().dusk = Dusk::Ease { from: share, to: 0.8, t: 0.0, secs: 4.0 };
        }),
        say(SNAKE, "earth.l76"),
        // the snake crawls away toward the far dune under the star; the camera watches 2.7 s
        run(move |w: &mut World| {
            actors::with_snake(w, true, |s, _| {
                s.state = SnakeState::Leaving;
                s.yaw = WAY_YAW.to_radians();
            });
            let spot = anchor(w, "snake_spot");
            let door = anchor(w, "door");
            let back = Vec3::new(-a.x, 0.0, -a.y);
            let right = Vec3::new(-(WAY_YAW.to_radians()).cos(), 0.0, (WAY_YAW.to_radians()).sin());
            shot_fov(w, spot + back * 2.3 + right * 0.7 + Vec3::Y * 1.5, door + Vec3::new(a.x, 0.0, a.y) * 3.0 + Vec3::Y * 0.3, 1.0, None);
        }),
        wait(2.7),
        // ================================================= D6 reunion and farewell
        step(2),
        run(move |w: &mut World| {
            let fw = d_farewell();
            let start = fw - a * 3.5;
            let g = world::ground_of(w, start);
            let spot = anchor(w, "snake_spot");
            actors::with_pilot(w, |p, t| {
                t.translation = g;
                p.yaw = WAY_YAW.to_radians();
                p.set_pose(PilotPose::Walk, 0.0);
                p.walk = Some((fw, 1.2));
                p.yaw_target = Some(WAY_YAW.to_radians());
                p.look_at = Some(spot + Vec3::Y * 1.0);
            });
            // seen over the pilot's shoulder from a raised shot (back 4.0, right 0.9, up 3.0, FOV 42)
            let fwp = anchor(w, "farewell");
            let back = Vec3::new(-a.x, 0.0, -a.y);
            let right = Vec3::new(-(WAY_YAW.to_radians()).cos(), 0.0, (WAY_YAW.to_radians()).sin());
            shot_fov(w, fwp + back * 4.0 + right * 0.9 + Vec3::Y * 3.0, spot + Vec3::Y * 0.8, 1.6, Some(42.0));
            // he turns round to face the way the pilot comes
            let mut q = w.query::<&mut Player>();
            if let Some(mut p) = q.iter_mut(w).next() {
                p.face_target = Some((WAY_YAW + 180.0).to_radians());
            }
        }),
        wait_flag("pilot_arrived", vec![]),
        say(PILOT, "earth.l72"),
        run(|w: &mut World| {
            let mut q = w.query_filtered::<&mut Interactable, With<actors::Pilot>>();
            if let Some(mut it) = q.iter_mut(w).next() {
                it.id = "pilot_bye".into();
                it.prompt = "earth.verb.talk".into();
                it.radius = 4.5;
                it.focus_offset = Vec3::Y * 1.7;
                it.prompt_height = 0.35;
            }
        }),
        unlock(),
        wait_use("pilot_bye"),
        disable("pilot_bye"),
        lock(),
        // he walks out across the top landing (0.85 m/s) and meets the aviator 1.8 m in front of him
        walk_speed(0.85),
        walk_to(Pos::World({
            let m = d_farewell() + way_dir() * 1.8;
            Vec3::new(m.x, 0.0, m.y)
        })),
        walk_speed(1.0),
        face(Pos::anchor("farewell")),
        run(|w: &mut World| {
            let pp = pilot_pos(w);
            two_shot(w, pp, 3.0, 0.0, 1.0);
        }),
        say(speakers::PRINCE, "earth.l50"),
        say(PILOT, "earth.l77"),
        say(speakers::PRINCE, "earth.l51"),
        say(PILOT, "earth.l52"),
        say(speakers::PRINCE, "earth.l53"),
        say(speakers::PRINCE, "earth.l57"),
        say(speakers::PRINCE, "earth.l58"),
        // ================================================= D7 walking out, the star, the end card
        run(move |w: &mut World| {
            if let Some(d) = w.resource_mut::<StoryRunner>().done.get_mut(5) {
                *d = true;
            }
            w.resource_mut::<Hud>().hide_objective();
            let past = D_DOOR + a * 3.2;
            let speed = if test(w) { 3.0 } else { 1.5 };
            let mut q = w.query::<&mut Player>();
            if let Some(mut p) = q.iter_mut(w).next() {
                p.drive = Drive::WalkTo { target: past, speed, stop_radius: 0.1 };
            }
            let fwp = anchor(w, "farewell");
            let back = Vec3::new(-a.x, 0.0, -a.y);
            let right = Vec3::new(-(WAY_YAW.to_radians()).cos(), 0.0, (WAY_YAW.to_radians()).sin());
            let door = anchor(w, "door");
            shot_fov(w, fwp + back * 4.0 + right * 0.9 + Vec3::Y * 3.0, door + Vec3::Y * 0.8, 1.2, Some(42.0));
            w.resource_mut::<EarthState>().end_walk = 1;
            w.resource_mut::<EarthState>().dusk = Dusk::Ease { from: 0.8, to: 0.92, t: 0.0, secs: 6.0 };
            actors::with_pilot(w, |p, _| p.look_at = Some(door + Vec3::Y * 1.0));
        }),
        wait_flag("past_door", vec![]),
        run(overlay::end_start),
        music("music.earth_end"),
        wait_flag("endcard_done", vec![]),
        leave(),
    ]);
    let _ = (FoxState::Wander, Fade::default);
    Story {
        events: vec![
            event("earth.ev.where", &["earth.st.snake"]),
            event("earth.ev.roses", &["earth.st.roses"]),
            event("earth.ev.fox", &["earth.st.fox"]),
            event("earth.ev.train", &["earth.st.train"]),
            event("earth.ev.wake", &["earth.st.wake"]),
            event("earth.ev.home", &["earth.st.home", "earth.st.way", "earth.st.pilot"]),
        ],
        hidden: Some("earth.hidden"),
        script: s,
    }
}
