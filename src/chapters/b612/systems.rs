//! B-612 mechanics (B612Story / B612Opening / B612ClocheRitual /
//! B612RoseActing / FillTarget / WaterTarget / CarryProp / ChairSeat in the
//! original), written as chapter systems. Story beats call the `pub fn`s
//! here through `run(..)`; the per-frame logic lives in [`tick`].

use super::director;
use super::world::*;
use crate::core::interact::Interacted;
use crate::core::meshkit::Rng;
use crate::core::player::PlayerTuning;
use crate::core::prelude::*;
use crate::core::sky::Sky;
use crate::core::story::{Anchors, set_flag};
use bevy::ecs::message::{MessageCursor, Messages};
use bevy::light::NotShadowCaster;
use std::f32::consts::{PI, TAU};

/// Where things are (filled by world::build).
#[derive(Resource, Clone)]
pub struct Sites {
    pub rose: Vec3,
    pub boulder: Vec3,
    pub cloche_home: Vec3,
    pub cloche_stand: Vec3,
    pub can_home: Vec3,
    pub can_spot: Vec3,
    pub fw_stand: Vec3,
    pub care_stand: Vec3,
    /// (slab, stand, facing yaw toward the pool)
    pub slabs: Vec<(Vec3, Vec3, f32)>,
    pub pond: Vec3,
    pub water_y: f32,
    pub chair: Vec3,
    pub nook: Vec3,
    pub feet: Vec3,
    pub vents: Vec<Vec3>,
    pub brook_end: Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Phase {
    #[default]
    Opening,
    Explore,
    Call,
    Care,
    Farewell,
    Ritual,
    Sunset,
    Depart,
}

/// Rose poses (B612RoseActing).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RosePose {
    #[default]
    Rest,
    Lift,
    Droop,
    Proud,
    TurnAway,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Seq {
    #[default]
    None,
    Fill {
        slab: usize,
        t: f32,
        stage: u8,
    },
    Water {
        t: f32,
        stage: u8,
    },
    Ritual {
        t: f32,
        stage: u8,
    },
    Place {
        t: f32,
        stage: u8,
    },
    Sit {
        t: f32,
        stage: u8,
        from: f64,
        to: f64,
    },
}

#[derive(Resource)]
pub struct B612State {
    pub test: bool,
    pub phase: Phase,
    pub open_t: f32,
    pub opened: bool,
    pub tut_stage: usize,
    pub tut_t: f32,
    pub tut_acc: f32,
    pub tut_delay: f32,
    pub player: Entity,
    pub rose: Entity,
    pub rose_head: Option<Entity>,
    pub can: Entity,
    pub cloche: Entity,
    pub chair: Entity,
    pub soil: Entity,
    pub rose_pose: RosePose,
    rose_cur: Vec3,
    pub tremble: f32,
    pub carrying: bool,
    pub fill: f32,
    pub can_down: bool,
    can_rest: (Vec3, f32),
    pub care_step: Option<usize>,
    pub seq: Seq,
    pub wet: f32,
    pub soak: f32,
    pub cloche_held: bool,
    pub chair_line: bool,
    pub trail_t: f32,
    pub trail_persist: bool,
    pub sunset_t: f32,
    pub time: f32,
    emit_t: [f32; 4],
    rng: Rng,
    pub fly_next: f32,
    pub fly_t: f32,
    pub fly_from: Vec3,
    pub fly_to: Vec3,
    pub depart_t: f32,
    assets: FxAssets,
}

#[derive(Clone)]
struct FxAssets {
    ball: Handle<Mesh>,
    smoke: Handle<Mesh>,
    water: Handle<StandardMaterial>,
    steam: Handle<StandardMaterial>,
    spark: Handle<StandardMaterial>,
}

/// Seconds-scale for my own sequences (test runs go faster).
fn speed(w: &World) -> f32 {
    if w.resource::<B612State>().test { 2.5 } else { 1.0 }
}

#[allow(clippy::too_many_arguments)]
pub fn init_state(world: &mut World, test: bool, player: Entity, rose: Entity, can: Entity, cloche: Entity, chair: Entity, soil: Entity) {
    let ball = world.resource_mut::<Assets<Mesh>>().add(Sphere::new(1.0).mesh().ico(1).unwrap());
    let smoke = world.resource_mut::<Assets<Mesh>>().add(Rectangle::new(2.0,2.0));
    let smoke_texture = world.resource::<AssetServer>().load("original/b612/steam.png");
    let mut mats = world.resource_mut::<Assets<StandardMaterial>>();
    let water = mats.add(StandardMaterial {
        base_color: Color::srgba(0.7, 0.86, 0.95, 0.75),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let steam = mats.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.95, 0.86, 0.3),
        base_color_texture: Some(smoke_texture),
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let spark = mats.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.93, 0.6),
        emissive: LinearRgba::rgb(4.0, 3.2, 1.4),
        unlit: true,
        ..default()
    });
    let sites = world.resource::<Sites>().clone();
    world.insert_resource(B612State {
        test,
        phase: Phase::Opening,
        open_t: 0.0,
        opened: false,
        tut_stage: 0,
        tut_t: 0.0,
        tut_acc: 0.0,
        tut_delay: 0.0,
        player,
        rose,
        rose_head: None,
        can,
        cloche,
        chair,
        soil,
        rose_pose: RosePose::Rest,
        rose_cur: Vec3::ZERO,
        tremble: 0.0,
        carrying: false,
        fill: 0.0,
        can_down: false,
        can_rest: (sites.can_home, CAN_YAW),
        care_step: None,
        seq: Seq::None,
        wet: 0.0,
        soak: 0.0,
        cloche_held: false,
        chair_line: false,
        trail_t: 0.0,
        trail_persist: false,
        sunset_t: 0.0,
        time: 0.0,
        emit_t: [0.0; 4],
        rng: Rng::new(612),
        fly_next: 55.0,
        fly_t: -1.0,
        fly_from: Vec3::ZERO,
        fly_to: Vec3::ZERO,
        depart_t: 0.0,
        assets: FxAssets { ball, smoke, water, steam, spark },
    });
    // asleep on the pad: lying on his back, head to the west
    if let Some(mut pl) = world.get_mut::<Player>(player) {
        pl.drive = Drive::External;
        pl.pose = Pose::None;
    }
    if let Some(mut t) = world.get_mut::<Transform>(player) {
        t.rotation = Quat::IDENTITY;
        t.translation = sites.feet;
    }
}

// ------------------------------------------------------------- components

#[derive(Component)]
pub struct Particle {
    billboard: bool,
    vel: Vec3,
    life: f32,
    max: f32,
    gravity: f32,
    size: f32,
    grow: f32,
}

#[derive(Component)]
pub struct TrailDot(usize);

#[derive(Component)]
pub struct Bird {
    idx: usize,
    depart: bool,
}

#[derive(Component)]
pub struct Butterfly {
    center: Vec3,
    ph: f32,
    r: f32,
}

/// Butterflies, trail dots, the two flocks (scoped to the chapter).
pub fn spawn_life(cx: &mut ChapterCx) {
    let st = cx.world.resource::<B612State>().assets.clone();
    for i in 0..14 {
        cx.spawn((
            TrailDot(i),
            Mesh3d(st.ball.clone()),
            MeshMaterial3d(st.spark.clone()),
            Transform::from_scale(Vec3::splat(0.05)),
            Visibility::Hidden,
            NotShadowCaster,
        ));
    }
    let centers = [
        ROSE + Vec2::new(1.5, -1.5),
        Vec2::new(4.0, -4.0),
        Vec2::new(-5.0, 9.0),
        POND + Vec2::new(-2.5, 2.5),
    ];
    for (i, c) in centers.iter().enumerate() {
        let p = cx.ground(c.x, c.y);
        let e = cx.model("model.b612.butterfly", p + Vec3::Y * 0.7, 0.0, 1.4);
        cx.world.entity_mut(e).insert(Butterfly {
            center: p,
            ph: i as f32 * 1.7,
            r: 1.2 + i as f32 * 0.3,
        });
    }
    for i in 0..14 {
        let e = cx.model("model.b612.bird", Vec3::new(0.0, -50.0, 0.0), 0.0, 1.0);
        cx.world.entity_mut(e).insert((Bird { idx: i % 7, depart: i >= 7 }, Visibility::Hidden));
    }
}

// ------------------------------------------------------------- helpers

fn st(w: &mut World) -> Mut<'_, B612State> {
    w.resource_mut::<B612State>()
}

pub fn player_pos(w: &World) -> Vec3 {
    let p = w.resource::<B612State>().player;
    w.get::<Transform>(p).map(|t| t.translation).unwrap_or_default()
}

fn player_yaw(w: &World) -> f32 {
    let p = w.resource::<B612State>().player;
    w.get::<Player>(p).map(|p| p.yaw).unwrap_or(0.0)
}

fn with_player(w: &mut World, f: impl FnOnce(&mut Player, &mut Transform)) {
    let e = w.resource::<B612State>().player;
    let Some(mut t) = w.get::<Transform>(e).copied() else { return };
    {
        let Some(mut pl) = w.get_mut::<Player>(e) else { return };
        f(&mut pl, &mut t);
    }
    if let Some(mut slot) = w.get_mut::<Transform>(e) {
        *slot = t;
    }
}

pub fn near(w: &World, p: Vec3, r: f32) -> bool {
    let q = player_pos(w);
    Vec2::new(p.x - q.x, p.z - q.z).length() <= r
}

fn yaw_to(from: Vec3, to: Vec3) -> f32 {
    let d = to - from;
    d.x.atan2(d.z)
}

fn flat(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

fn set_enabled(w: &mut World, id: &str, on: bool) {
    let mut q = w.query::<&mut Interactable>();
    for mut it in q.iter_mut(w) {
        if it.id == id && it.enabled != on {
            it.enabled = on;
        }
    }
}

fn set_prompt(w: &mut World, id: &str, key: &str) {
    let mut q = w.query::<&mut Interactable>();
    for mut it in q.iter_mut(w) {
        if it.id == id && it.prompt != key {
            it.prompt = key.to_string();
        }
    }
}

fn ground(w: &World, x: f32, z: f32) -> Vec3 {
    w.get_resource::<Terrain>().map(|t| t.ground(x, z)).unwrap_or(Vec3::new(x, 0.0, z))
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn sfx(w: &mut World, key: &str) {
    w.write_message(AudioCmd::sfx(key));
}

pub fn set_rose_pose(w: &mut World, p: RosePose) {
    st(w).rose_pose = p;
}

pub fn tremble(w: &mut World, secs: f32) {
    st(w).tremble = secs;
}

fn objective(w: &mut World, step: usize) {
    let ev = w.resource::<StoryRunner>().events.first().map(|e| e.name);
    let key = w.resource::<StoryRunner>().events.first().and_then(|e| e.steps.get(step).copied());
    {
        let mut r = w.resource_mut::<StoryRunner>();
        r.event_index = Some(0);
        r.step_index = step;
        r.stall_best = f32::MAX;
        r.stall_time = 0.0;
    }
    w.resource_mut::<Hud>().show_objective(ev, key, false);
}

fn set_target(w: &mut World, p: Option<Vec3>, label: Option<&'static str>) {
    let mut r = w.resource_mut::<StoryRunner>();
    r.target = p.map(Pos::World);
    r.target_label = label;
    r.stall_best = f32::MAX;
    r.stall_time = 0.0;
}

// ------------------------------------------------------------- story hooks

/// Free roam starts: the tutorial runs, the objective is the free line.
pub fn start_explore(w: &mut World) {
    st(w).phase = Phase::Explore;
    w.resource_mut::<Hud>().show_objective(None, Some("b612.free"), false);
}

/// GuideToRose (after the tutorial): objective + stardust trail.
pub fn guide_to_rose(w: &mut World) {
    let rose = w.resource::<Sites>().rose;
    w.resource_mut::<Hud>().show_objective(None, Some("b612.st.see_rose"), false);
    set_target(w, Some(rose + Vec3::Y * 0.4), Some("b612.target.rose"));
    st(w).trail_persist = true;
}

/// RoseCalls: stop the tutorial, walk him to 1.3 m from her (1.1 m/s).
pub fn call_begin(w: &mut World) {
    {
        let mut s = st(w);
        s.phase = Phase::Call;
        s.trail_persist = false;
        s.trail_t = 0.0;
        s.tut_stage = 9;
    }
    w.resource_mut::<Hud>().teach = None;
    set_target(w, None, None);
    w.resource_mut::<PlayerTuning>().walk_speed = 1.1;
    let rose = w.resource::<Sites>().rose;
    let p = player_pos(w);
    let mut d = flat(p - rose).normalize_or(Vec3::new(0.0, 0.0, -1.0));
    if d.length() < 0.5 {
        d = Vec3::new(0.0, 0.0, -1.0);
    }
    let stand = rose + d * 1.3;
    let g = ground(w, stand.x, stand.z);
    w.resource_mut::<Anchors>().0.insert("call_stand".into(), (g, 0.0));
}

/// The medium shot from behind him toward her (pitch 14°, 2.6 m, FOV 40, 30° off the line).
pub fn call_shot(w: &mut World) {
    let s = w.resource::<Sites>().clone();
    let p = player_pos(w);
    let look = (p + Vec3::Y * 0.75).lerp(s.rose + Vec3::Y * 0.45, 0.55);
    let dir = flat(s.rose - p).normalize_or(Vec3::Z);
    let pick = |ang: f32| {
        let d = Quat::from_rotation_y(ang.to_radians()) * dir;
        look - d * 2.6 * 14f32.to_radians().cos() + Vec3::Y * 2.6 * 14f32.to_radians().sin()
    };
    let (a, b) = (pick(30.0), pick(-30.0));
    let cam = if a.distance(s.boulder) > b.distance(s.boulder) { a } else { b };
    director::shot(w, cam, look, 1.0, 40.0);
    set_rose_pose(w, RosePose::Lift);
}

pub fn call_end(w: &mut World) {
    set_rose_pose(w, RosePose::Rest);
    director::release(w, 0.3);
    w.resource_mut::<PlayerTuning>().walk_speed = 1.0;
    let mut s = st(w);
    s.phase = Phase::Care;
    s.care_step = None;
}

/// Farewell: set the can down first if carried; then step close to her.
pub fn farewell_begin(w: &mut World) {
    let carrying = {
        let mut s = st(w);
        s.phase = Phase::Farewell;
        s.trail_t = 0.0;
        s.carrying
    };
    set_target(w, None, None);
    if carrying {
        w.resource_mut::<StoryRunner>().set_flag("carrying");
        let spot = w.resource::<Sites>().can_spot;
        set_target(w, Some(spot), Some("b612.target.spot"));
        w.resource_mut::<Hud>().monologue("b612.mono.put_first", 4.0);
    }
}

pub fn farewell_walk(w: &mut World) {
    set_target(w, None, None);
    w.resource_mut::<PlayerTuning>().walk_speed = 0.9;
    let s = w.resource::<Sites>().clone();
    let yaw = yaw_to(s.fw_stand, s.rose).to_degrees();
    w.resource_mut::<Anchors>().0.insert("fw_stand".into(), (s.fw_stand, yaw));
}

/// Two-shot: 2.7 m, FOV 40, camera height 1.0, look height 0.36, turned 15° toward his face.
pub fn two_shot(w: &mut World) {
    let s = w.resource::<Sites>().clone();
    let p = player_pos(w);
    let mid = (p + s.rose) * 0.5;
    let line = flat(s.rose - p).normalize_or(Vec3::Z);
    let mut perp = Vec3::new(-line.z, 0.0, line.x);
    if (mid + perp).distance(s.boulder) < (mid - perp).distance(s.boulder) {
        perp = -perp;
    }
    // turn toward his face (toward the rose's side of the line)
    let ang = 15f32.to_radians() * if perp.cross(line).y > 0.0 { -1.0 } else { 1.0 };
    let dir = Quat::from_rotation_y(ang) * perp;
    let base_y = mid.y;
    let cam = Vec3::new(mid.x + dir.x * 2.7, base_y + 1.0, mid.z + dir.z * 2.7);
    let look = Vec3::new(mid.x, base_y + 0.36 + 0.2, mid.z);
    director::shot(w, cam, look, 1.0, 40.0);
}

/// Close-up on her (FOV 32), over his shoulder.
pub fn rose_close(w: &mut World) {
    let s = w.resource::<Sites>().clone();
    let p = player_pos(w);
    let line = flat(s.rose - p).normalize_or(Vec3::Z);
    let side = Vec3::new(-line.z, 0.0, line.x);
    let side = if (p + side).distance(s.boulder) > (p - side).distance(s.boulder) {
        side
    } else {
        -side
    };
    let cam = p + Vec3::Y * 0.95 - line * 0.55 + side * 0.4;
    director::shot(w, cam, s.rose + Vec3::Y * 0.55, 1.0, 32.0);
}

/// Close-up on him (FOV 32), from beside her.
pub fn prince_close(w: &mut World) {
    let s = w.resource::<Sites>().clone();
    let p = player_pos(w);
    let line = flat(p - s.rose).normalize_or(Vec3::Z);
    let side = Vec3::new(-line.z, 0.0, line.x);
    let side = if (s.rose + side).distance(s.boulder) > (s.rose - side).distance(s.boulder) {
        side
    } else {
        -side
    };
    let cam = s.rose + Vec3::Y * 0.62 - line * 0.35 + side * 0.45;
    director::shot(w, cam, p + Vec3::Y * 0.98, 1.0, 32.0);
}

/// After part 1: ease back, the glass ritual is offered.
pub fn ritual_offer(w: &mut World) {
    director::release(w, 1.2);
    w.resource_mut::<PlayerTuning>().walk_speed = 1.0;
    st(w).phase = Phase::Ritual;
    let c = w.resource::<Sites>().cloche_home;
    set_target(w, Some(c), Some("b612.target.cloche"));
}

pub fn place_offer(w: &mut World) {
    set_enabled(w, "cloche_place", true);
}

/// The glass is back on its boulder: on to the last sunset.
pub fn to_sunset(w: &mut World) {
    let (wet, rose) = (w.resource::<B612State>().wet, w.resource::<Sites>().rose);
    if wet > 0.2 {
        let p = player_pos(w);
        with_player(w, |pl, _| pl.face_target = Some(yaw_to(p, rose)));
    }
    st(w).phase = Phase::Sunset;
}

/// After the sunset: stand up beside the chair, birds are coming.
pub fn stand_up(w: &mut World) {
    let chair = w.resource::<Sites>().chair;
    let g = ground(w, chair.x + 0.75, chair.z + 0.1);
    with_player(w, |pl, t| {
        pl.pose = Pose::None;
        pl.drive = Drive::Player;
        t.translation = g;
    });
    director::release(w, 1.6);
    set_enabled(w, "chair", false);
    st(w).phase = Phase::Depart;
    w.resource_mut::<Hud>().monologue("b612.mono.day_over", 4.0);
}

pub fn sunset_over(w: &World) -> bool {
    let s = w.resource::<B612State>();
    w.resource::<Sky>().sunset_done() || s.sunset_t > 30.0
}

// ------------------------------------------------------------- per frame

/// The chapter's main per-frame logic (exclusive; Gameplay set).
pub fn tick(world: &mut World, mut cursor: Local<MessageCursor<Interacted>>) {
    if !world.contains_resource::<B612State>() || !world.contains_resource::<Sites>() {
        return;
    }
    let dt = world.resource::<Time>().delta_secs();
    if world.resource::<Time<Virtual>>().is_paused() {
        return;
    }
    st(world).time += dt;
    let used: Vec<String> = {
        let msgs = world.resource::<Messages<Interacted>>();
        cursor.read(msgs).map(|m| m.id.clone()).collect()
    };
    for id in used {
        on_use(world, &id);
    }
    opening(world, dt);
    tutorial(world, dt);
    care(world);
    sequences(world, dt);
    place_props(world);
    rose_acting(world, dt);
    chair_line(world);
    emitters(world, dt);
    soil(world, dt);
}

fn on_use(w: &mut World, id: &str) {
    let (phase, carrying, fill, seq) = {
        let s = w.resource::<B612State>();
        (s.phase, s.carrying, s.fill, s.seq)
    };
    match id {
        "can" if !carrying => {
            st(w).carrying = true;
            set_enabled(w, "can", false);
            sfx(w, "sfx.tin");
        }
        "can_home" if carrying => {
            let home = w.resource::<Sites>().can_home;
            let mut s = st(w);
            s.carrying = false;
            s.can_rest = (home, CAN_YAW);
            drop(s);
            sfx(w, "sfx.tin");
        }
        "can_spot" if carrying => {
            let spot = w.resource::<Sites>().can_spot;
            let rose = w.resource::<Sites>().rose;
            let mut s = st(w);
            s.carrying = false;
            s.can_down = true;
            s.can_rest = (spot, yaw_to(spot, rose).to_degrees() + 70.0);
            drop(s);
            set_enabled(w, "can_spot", false);
            set_target(w, None, None);
            sfx(w, "sfx.tin");
        }
        _ if id.starts_with("spring_") && carrying && fill < 1.0 && seq == Seq::None => {
            let i: usize = id[7..].parse().unwrap_or(0);
            st(w).seq = Seq::Fill { slab: i, t: 0.0, stage: 0 };
        }
        "rose_water" if carrying && seq == Seq::None => {
            if fill >= 1.0 {
                st(w).seq = Seq::Water { t: 0.0, stage: 0 };
            } else {
                w.resource_mut::<Hud>().monologue("b612.mono.empty", 3.0);
            }
        }
        "cloche" => {
            set_enabled(w, "cloche", false);
            set_target(w, None, None);
            st(w).seq = Seq::Ritual { t: 0.0, stage: 0 };
        }
        "cloche_place" => {
            set_enabled(w, "cloche_place", false);
            st(w).seq = Seq::Place { t: 0.0, stage: 0 };
        }
        "chair" if phase == Phase::Sunset => {
            set_enabled(w, "chair", false);
            let hour = w.resource::<Sky>().hour;
            let to = w.resource::<Sky>().hour_for_elevation(9.0, true).max(hour);
            st(w).seq = Seq::Sit {
                t: 0.0,
                stage: 0,
                from: hour,
                to,
            };
        }
        _ => {}
    }
}

/// B612Opening: asleep under the card → wake-up → camera orbit → hand-over.
fn opening(w: &mut World, dt: f32) {
    const SLEEP: f32 = 4.5;
    const WAKE: f32 = 3.9 / 1.2; // wake clip at speed 1.2
    const STAND: f32 = 0.3;
    const ORBIT: f32 = 5.2;
    const HOLD: f32 = 1.4;
    let (phase, opened, test) = {
        let s = w.resource::<B612State>();
        (s.phase, s.opened, s.test)
    };
    if phase != Phase::Opening || opened {
        return;
    }
    let k = if test { 5.0 } else { 1.0 };
    let t = {
        let mut s = st(w);
        s.open_t += dt * k;
        s.open_t
    };
    let feet = w.resource::<Sites>().feet;
    let yaw = SPAWN_YAW.to_radians();
    let facing = Quat::from_rotation_y(yaw);
    let behind = feet + facing * Vec3::new(0.0, 1.0 + 3.4 * 4f32.to_radians().sin(), -3.4 * 4f32.to_radians().cos());
    // body
    let u = ((t - SLEEP) / WAKE).clamp(0.0, 1.0);
    with_player(w, |pl, tf| {
        pl.drive = Drive::External;
        pl.yaw = yaw;
        tf.rotation = Quat::IDENTITY;
        tf.translation = feet;
    });
    if t < SLEEP + WAKE + STAND {
        // camera one: from the head side, rising with the clip (riseFrom 0.3 → riseTo 0.95)
        let r = ease((u - 0.3) / 0.65);
        let cam = (feet + facing * Vec3::new(0.75, 1.15, -1.95)).lerp(behind, r);
        let look = (feet + facing * Vec3::new(0.0, 0.3, 0.75)).lerp(feet + Vec3::Y * 1.0, r);
        director::shot(w, cam, look, 0.0, 38.0);
        return;
    }
    let o = ((t - SLEEP - WAKE - STAND) / ORBIT).clamp(0.0, 1.0);
    if o < 1.0 || t < SLEEP + WAKE + STAND + ORBIT + HOLD {
        // Camera two arcs from behind to the original site's hand-over view.
        let eo = o * o * o * (o * (o * 6.0 - 15.0) + 10.0);
        let az = (yaw + PI) + (OPENING_VIEW_YAW.to_radians() + 2.0 * PI - yaw - PI) * eo;
        let pitch = (4.0 + 6.0 * (eo * PI).sin()).to_radians();
        let pivot = feet + Vec3::Y * 1.0;
        let cam = pivot + Vec3::new(az.sin() * pitch.cos(), pitch.sin(), az.cos() * pitch.cos()) * 3.4;
        director::shot(w, cam, pivot, 0.0, 38.0);
        return;
    }
    // hand over to the player camera
    w.resource_mut::<CameraRig>().snap_behind(OPENING_VIEW_YAW.to_radians() + PI);
    director::release(w, 1.6);
    with_player(w, |pl, tf| {
        pl.drive = Drive::Player;
        tf.rotation = Quat::IDENTITY;
        tf.translation = feet;
    });
    st(w).opened = true;
    set_flag(w, "opened");
}

/// TeachControls: move (1.5 s) → look (150°) → run (1 s), 25 s timeout each.
fn tutorial(w: &mut World, dt: f32) {
    let (phase, stage) = {
        let s = w.resource::<B612State>();
        (s.phase, s.tut_stage)
    };
    if phase != Phase::Explore || stage >= 9 {
        return;
    }
    let touch = crate::core::platform::is_touch_device() || w.resource::<crate::core::params::LaunchParams>().touch == Some(true);
    let input = w.resource::<InputFrame>().clone();
    let mut s = st(w);
    if stage == 0 {
        s.tut_delay += dt;
        if s.tut_delay < 1.2 {
            return;
        }
    }
    s.tut_t += dt;
    let moving = input.move_axis.length() > 0.2 || input.move_world.is_some_and(|m| m.length() > 0.2);
    match stage {
        1 => {
            if moving {
                s.tut_acc += dt;
            }
        }
        2 => s.tut_acc += input.look.length(),
        3 => {
            if moving && input.run {
                s.tut_acc += dt;
            }
        }
        _ => {}
    }
    let need = match stage {
        1 => 1.5,
        2 => 150.0,
        3 => 1.0,
        _ => 0.0,
    };
    let next = stage == 0 || (stage <= 3 && (s.tut_acc >= need || s.tut_t >= 25.0)) || (stage == 4 && s.tut_t >= 0.8);
    if !next {
        return;
    }
    let ns = if stage == 2 && touch { 4 } else { stage + 1 };
    s.tut_stage = ns;
    s.tut_t = 0.0;
    s.tut_acc = 0.0;
    drop(s);
    let mut hud = w.resource_mut::<Hud>();
    match (ns, touch) {
        (1, false) => hud.teach(vec!["W", "A", "S", "D"], "core.teach.move", 26.0),
        (1, true) => hud.teach(vec![], "b612.teach.touch_move", 26.0),
        (2, false) => hud.teach(vec!["b612.key.mouse"], "b612.teach.look", 26.0),
        (2, true) => hud.teach(vec![], "b612.teach.touch_look", 26.0),
        (3, _) => hud.teach(vec!["Shift"], "b612.teach.run", 26.0),
        (4, false) => hud.teach(vec!["Tab"], "b612.teach.tasks", 4.0),
        (4, true) => hud.teach(vec![], "b612.teach.touch_tasks", 4.0),
        _ => {
            drop(hud);
            st(w).tut_stage = 9;
            set_flag(w, "tut_done");
            return;
        }
    }
    drop(hud);
    if ns <= 3 {
        sfx(w, "sfx.click");
    }
}

/// The care loop's live objective (B612Story.Update target selection) and
/// which interactables are offered.
fn care(w: &mut World) {
    let (phase, carrying, fill, can_down, seq, step) = {
        let s = w.resource::<B612State>();
        (s.phase, s.carrying, s.fill, s.can_down, s.seq, s.care_step)
    };
    let free = seq == Seq::None;
    let early = matches!(phase, Phase::Explore | Phase::Call | Phase::Care);
    set_enabled(w, "can", !carrying && !can_down && free && early);
    set_enabled(w, "can_home", carrying && free && matches!(phase, Phase::Explore | Phase::Care));
    for i in 0..4 {
        set_enabled(w, &format!("spring_{i}"), carrying && fill < 1.0 && free && phase != Phase::Farewell);
    }
    set_enabled(w, "rose_water", carrying && free && phase == Phase::Care);
    set_prompt(w, "rose_water", if fill >= 1.0 { "b612.verb.water" } else { "b612.verb.can_empty" });
    if phase != Phase::Care || !free {
        return;
    }
    let want = if !carrying {
        0
    } else if fill < 1.0 {
        1
    } else {
        2
    };
    if step == Some(want) {
        return;
    }
    let sites = w.resource::<Sites>().clone();
    let p = player_pos(w);
    let (pos, label) = match want {
        0 => (sites.can_home + Vec3::Y * 0.15, "b612.target.can"),
        1 => {
            let best = sites
                .slabs
                .iter()
                .min_by(|a, b| a.0.distance(p).total_cmp(&b.0.distance(p)))
                .map(|s| s.0)
                .unwrap_or(sites.pond);
            (best, "b612.target.spring")
        }
        _ => (sites.rose + Vec3::Y * 0.4, "b612.target.rose"),
    };
    objective(w, want);
    set_target(w, Some(pos), Some(label));
    let mut s = st(w);
    s.care_step = Some(want);
    if step.is_some() {
        s.trail_t = 7.0;
    }
}

fn sequences(w: &mut World, dt: f32) {
    let seq = w.resource::<B612State>().seq;
    let k = speed(w);
    let dt = dt * k;
    let sites = w.resource::<Sites>().clone();
    match seq {
        Seq::None => {}
        Seq::Fill { slab, t, stage } => {
            let t = t + dt;
            let (_, stand, face) = sites.slabs[slab];
            let face_r = face.to_radians();
            let mut ns = stage;
            let mut nt = t;
            match stage {
                0 => {
                    if t <= dt {
                        with_player(w, |pl, _| {
                            pl.drive = Drive::WalkTo {
                                target: Vec2::new(stand.x, stand.z),
                                speed: 0.9 * k,
                                stop_radius: 0.12,
                            }
                        });
                    }
                    let arrived = {
                        let e = w.resource::<B612State>().player;
                        w.get::<Player>(e).map(|p| p.is_arrived()).unwrap_or(true)
                    };
                    if arrived || t > 3.0 {
                        with_player(w, |pl, tf| {
                            pl.drive = Drive::External;
                            pl.face_target = Some(face_r);
                            tf.translation = stand;
                        });
                        // over his right shoulder (−70° from his facing), pitch 40°, 2.2 m
                        let pivot = stand + Vec3::Y * 0.45;
                        let a = face_r - 70f32.to_radians();
                        let p40 = 40f32.to_radians();
                        let cam = pivot + Vec3::new(a.sin() * p40.cos(), p40.sin(), a.cos() * p40.cos()) * 2.9;
                        let look = pivot + Vec3::new(face_r.sin(), 0.0, face_r.cos()) * 0.4 - Vec3::Y * 0.25;
                        director::shot(w, cam, look, 1.1, 45.0);
                        ns = 1;
                        nt = 0.0;
                    }
                }
                1 => {
                    with_player(w, |pl, _| pl.pose = Pose::Kneel);
                    if t > 0.6 {
                        ns = 2;
                        nt = 0.0;
                        sfx(w, "sfx.water");
                    }
                }
                2 => {
                    let mut s = st(w);
                    s.fill = ((t - 0.4) / 1.6).clamp(0.0, 1.0);
                    if t > 2.2 {
                        s.fill = 1.0;
                        ns = 3;
                        nt = 0.0;
                    }
                }
                _ => {
                    if t > 0.5 {
                        with_player(w, |pl, _| {
                            pl.pose = Pose::None;
                            pl.drive = Drive::Player;
                        });
                        director::release(w, 1.3);
                        st(w).seq = Seq::None;
                        sfx(w, "sfx.chime");
                        return;
                    }
                }
            }
            st(w).seq = Seq::Fill { slab, t: nt, stage: ns };
        }
        Seq::Water { t, stage } => {
            let t = t + dt;
            let (mut ns, mut nt) = (stage, t);
            match stage {
                0 => {
                    if t <= dt {
                        let c = sites.care_stand;
                        with_player(w, |pl, _| {
                            pl.drive = Drive::WalkTo {
                                target: Vec2::new(c.x, c.z),
                                speed: 0.9 * k,
                                stop_radius: 0.12,
                            }
                        });
                    }
                    let arrived = {
                        let e = w.resource::<B612State>().player;
                        w.get::<Player>(e).map(|p| p.is_arrived()).unwrap_or(true)
                    };
                    if arrived || t > 3.0 {
                        let yaw = yaw_to(sites.care_stand, sites.rose);
                        with_player(w, |pl, _| {
                            pl.drive = Drive::External;
                            pl.face_target = Some(yaw);
                            pl.pose = Pose::Reach;
                        });
                        let side = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
                        let mid = (sites.care_stand + sites.rose) * 0.5;
                        let cam = mid + side * 1.9 + Vec3::Y * 1.0 - Vec3::new(yaw.sin(), 0.0, yaw.cos()) * 0.4;
                        director::shot(w, cam, mid + Vec3::Y * 0.35, 1.0, 42.0);
                        ns = 1;
                        nt = 0.0;
                    }
                }
                1 => {
                    if t > 0.7 {
                        ns = 2;
                        nt = 0.0;
                        sfx(w, "sfx.water");
                    }
                }
                2 => {
                    let mut s = st(w);
                    if t > 0.4 {
                        s.soak += dt;
                        s.wet = (s.wet + dt / 1.8).min(1.0);
                    }
                    s.fill = (1.0 - s.soak / 1.8).max(0.0);
                    if s.soak >= 1.8 {
                        s.fill = 0.0;
                        ns = 3;
                        nt = 0.0;
                    }
                    let every = (t / 0.9).floor() != ((t - dt) / 0.9).floor();
                    drop(s);
                    if every && t > 0.5 {
                        sfx(w, "sfx.water");
                    }
                }
                _ => {
                    if t > 0.5 {
                        with_player(w, |pl, _| {
                            pl.pose = Pose::None;
                            pl.drive = Drive::Player;
                        });
                        st(w).seq = Seq::None;
                        set_flag(w, "watered");
                        return;
                    }
                }
            }
            st(w).seq = Seq::Water { t: nt, stage: ns };
        }
        Seq::Ritual { t, stage } => {
            let t = t + dt;
            let (mut ns, mut nt) = (stage, t);
            let stand = sites.cloche_stand;
            let yb = yaw_to(stand, sites.boulder);
            let yr = yaw_to(stand, sites.rose);
            match stage {
                0 => {
                    if t <= dt {
                        with_player(w, |pl, _| {
                            pl.drive = Drive::WalkTo {
                                target: Vec2::new(stand.x, stand.z),
                                speed: 0.9 * k,
                                stop_radius: 0.1,
                            }
                        });
                    }
                    let arrived = {
                        let e = w.resource::<B612State>().player;
                        w.get::<Player>(e).map(|p| p.is_arrived()).unwrap_or(true)
                    };
                    if arrived || t > 2.5 {
                        with_player(w, |pl, tf| {
                            pl.drive = Drive::External;
                            pl.face_target = Some(yb);
                            tf.translation = stand;
                        });
                        // round to his face: stand frame (+z toward the boulder)
                        let ydeg = yb.to_degrees();
                        let cam = director::local_to_world(stand, ydeg, Vec3::new(-0.8, 0.78, -2.05));
                        let look = director::local_to_world(stand, ydeg, Vec3::new(0.0, 0.72, -0.22));
                        director::shot(w, cam, look, 2.4, 38.0);
                        ns = 1;
                        nt = 0.0;
                    }
                }
                1 => {
                    if t > 0.8 {
                        ns = 2;
                        nt = 0.0;
                        with_player(w, |pl, _| pl.pose = Pose::Reach);
                        sfx(w, "sfx.glass");
                    }
                }
                2 => {
                    if t > 1.2 {
                        st(w).cloche_held = true;
                        with_player(w, |pl, _| pl.face_target = Some(yr));
                        ns = 3;
                        nt = 0.0;
                    }
                }
                3 => {
                    if t > 1.2 {
                        ns = 4;
                        nt = 0.0;
                        set_flag(w, "cloche_held");
                    }
                }
                _ => {}
            }
            st(w).seq = Seq::Ritual { t: nt, stage: ns };
        }
        Seq::Place { t, stage } => {
            let t = t + dt;
            let (mut ns, mut nt) = (stage, t);
            let yb = yaw_to(sites.cloche_stand, sites.boulder);
            match stage {
                0 => {
                    with_player(w, |pl, _| pl.face_target = Some(yb));
                    if t > 0.9 {
                        ns = 1;
                        nt = 0.0;
                        st(w).cloche_held = false;
                    }
                }
                1 => {
                    if t > 1.0 {
                        ns = 2;
                        nt = 0.0;
                        sfx(w, "sfx.glass");
                    }
                }
                _ => {
                    with_player(w, |pl, _| {
                        pl.pose = Pose::None;
                        pl.drive = Drive::Player;
                    });
                    director::release(w, 1.8);
                    st(w).seq = Seq::None;
                    set_flag(w, "cloche_done");
                    return;
                }
            }
            st(w).seq = Seq::Place { t: nt, stage: ns };
        }
        Seq::Sit { t, stage, from, to } => {
            let t = t + dt;
            let (mut ns, mut nt) = (stage, t);
            let test = w.resource::<B612State>().test;
            match stage {
                0 => {
                    let chair = sites.chair;
                    with_player(w, |pl, tf| {
                        pl.drive = Drive::External;
                        pl.pose = Pose::Sit;
                        pl.yaw = CHAIR_YAW.to_radians();
                        pl.face_target = Some(CHAIR_YAW.to_radians());
                        tf.translation = chair + Vec3::new(0.0, 0.045, 0.06);
                        tf.rotation = Quat::IDENTITY;
                    });
                    // seated view: behind and above his right shoulder, toward the setting sun
                    let cam = chair + Vec3::new(1.3, 1.55, 3.3);
                    let look = chair + Vec3::new(-1.9, 0.45, -8.0);
                    director::shot(w, cam, look, 1.6, 46.0);
                    w.resource_mut::<Sky>().set_rate(0.0, None);
                    set_flag(w, "seated");
                    sfx(w, "sfx.wood");
                    ns = 1;
                    nt = 0.0;
                }
                1 => {
                    // hop to the golden hour in 3.5 s, then the sunset to the dusk hold in 22 s
                    let hop = if test { 0.6 } else { 3.5 };
                    let e = ease(t / hop) as f64;
                    w.resource_mut::<Sky>().set_hour(from + (to - from) * e);
                    if t >= hop {
                        w.resource_mut::<Sky>().start_sunset(if test { 3.0 } else { 22.0 });
                        ns = 2;
                        nt = 0.0;
                    }
                }
                _ => {
                    st(w).sunset_t = t;
                    if sunset_over(w) {
                        st(w).seq = Seq::None;
                        set_flag(w, "day_over");
                        return;
                    }
                }
            }
            st(w).seq = Seq::Sit { t: nt, stage: ns, from, to };
        }
    }
}

/// Moves the can and the cloche (carried, pouring, dipping, resting).
fn place_props(w: &mut World) {
    let (can, cloche, carrying, seq, rest, held) = {
        let s = w.resource::<B612State>();
        (s.can, s.cloche, s.carrying, s.seq, s.can_rest, s.cloche_held)
    };
    let sites = w.resource::<Sites>().clone();
    let p = player_pos(w);
    let yaw = player_yaw(w);
    let ry = Quat::from_rotation_y(yaw);
    let can_tf = if carrying {
        match seq {
            Seq::Fill { stage, t, .. } if stage >= 1 => {
                let base = p + ry * Vec3::new(-0.1, 0.0, 0.42);
                let down = match stage {
                    1 => (t / 0.6).min(1.0) * 0.4,
                    2 => 0.4 + (t / 0.4).min(1.0) * 0.6,
                    _ => 1.0 - (t / 0.5).min(1.0),
                };
                let y = (p.y + 0.45).lerp(sites.water_y - 0.14, ease(down));
                Transform::from_translation(Vec3::new(base.x, y, base.z)).with_rotation(ry * Quat::from_rotation_x(0.35 * down))
            }
            Seq::Water { stage, t } if stage >= 1 => {
                let tilt = match stage {
                    1 => (t / 0.7).min(1.0) * 0.3,
                    2 => 0.3 + (t / 0.4).min(1.0) * 0.6,
                    _ => 0.9 * (1.0 - (t / 0.5).min(1.0)),
                };
                Transform::from_translation(p + ry * Vec3::new(-0.08, 0.5, 0.3)).with_rotation(ry * Quat::from_rotation_x(tilt))
            }
            _ => Transform::from_translation(p + ry * Vec3::new(-0.26, 0.24, 0.06)).with_rotation(ry * Quat::from_rotation_x(0.05)),
        }
    } else {
        Transform::from_translation(rest.0).with_rotation(Quat::from_rotation_y(rest.1.to_radians()))
    };
    if let Some(mut t) = w.get_mut::<Transform>(can) {
        t.translation = can_tf.translation;
        t.rotation = can_tf.rotation;
    }
    // cloche
    let hands = p + ry * Vec3::new(0.0, 0.8, 0.4);
    let ctf = match seq {
        Seq::Ritual { stage: 2, t } => {
            let k = ease(t / 1.2);
            let mut v = sites.cloche_home.lerp(hands, k);
            v.y += (k * PI).sin() * 0.25;
            v
        }
        Seq::Place { stage: 1, t } => {
            let k = ease(t / 1.0);
            let mut v = hands.lerp(sites.cloche_home, k);
            v.y += (k * PI).sin() * 0.2;
            v
        }
        _ if held => hands,
        _ => sites.cloche_home,
    };
    if let Some(mut t) = w.get_mut::<Transform>(cloche) {
        t.translation = ctf;
    }
}

/// B612RoseActing: idle sway, poses (tilt/lean/turn) and trembles.
fn rose_acting(w: &mut World, dt: f32) {
    if w.resource::<B612State>().rose_head.is_none() {
        let mut q = w.query::<(Entity, &PartName)>();
        let head = q.iter(w).find(|(_, n)| n.0 == "rose_head").map(|(e, _)| e);
        st(w).rose_head = head;
    }
    let (rose, head, pose, mut cur, tr, time) = {
        let s = w.resource::<B612State>();
        (s.rose, s.rose_head, s.rose_pose, s.rose_cur, s.tremble, s.time)
    };
    // (head tilt, lean, turn) in degrees
    let (goal, tau) = match pose {
        RosePose::Rest => (Vec3::ZERO, 0.6),
        RosePose::Lift => (Vec3::new(-9.0, -3.0, 0.0), 0.6),
        RosePose::Proud => (Vec3::new(-5.0, -2.0, 0.0), 0.6),
        RosePose::Droop => (Vec3::new(75.0, 12.0, 0.0), 1.6),
        RosePose::TurnAway => (Vec3::new(14.0, 5.0, 40.0), 0.6),
    };
    let a = 1.0 - (-dt * 3.0 / tau).exp();
    cur += (goal - cur) * a;
    let tremble = if tr > 0.0 {
        (time * 2.0 * PI * 11.0).sin() * 3.0 * (tr / 0.6).min(1.0)
    } else {
        0.0
    };
    {
        let mut s = st(w);
        s.rose_cur = cur;
        s.tremble = (tr - dt).max(0.0);
    }
    let sway = (time * TAU * 0.22).sin() * 1.4;
    let head_sway = (time * TAU * 0.22 + 1.1).sin() * 0.9;
    if let Some(mut t) = w.get_mut::<Transform>(rose) {
        t.rotation = Quat::from_rotation_y((ROSE_YAW + cur.z).to_radians())
            * Quat::from_rotation_x((cur.y + tremble * 0.4).to_radians())
            * Quat::from_rotation_z(sway.to_radians());
    }
    if let Some(h) = head {
        if let Some(mut t) = w.get_mut::<Transform>(h) {
            t.rotation = Quat::from_rotation_x((cur.x + head_sway + tremble).to_radians());
        }
    }
}

/// Near the chair before the last event, he tells himself he loves sunsets.
fn chair_line(w: &mut World) {
    let (phase, done) = {
        let s = w.resource::<B612State>();
        (s.phase, s.chair_line)
    };
    if done || !matches!(phase, Phase::Explore | Phase::Care) {
        return;
    }
    let chair = w.resource::<Sites>().chair;
    if near(w, chair, 2.4) {
        st(w).chair_line = true;
        w.resource_mut::<Hud>().monologue("b612.mono.chair", 4.0);
    }
}

fn soil(w: &mut World, dt: f32) {
    let (soil, wet, phase) = {
        let s = w.resource::<B612State>();
        (s.soil, s.wet, s.phase)
    };
    if phase >= Phase::Sunset {
        st(w).wet = (wet - dt / 90.0).max(0.0);
    }
    let Some(h) = w.get::<MeshMaterial3d<StandardMaterial>>(soil).map(|m| m.0.clone()) else {
        return;
    };
    if let Some(mut m) = w.resource_mut::<Assets<StandardMaterial>>().get_mut(&h) {
        let c = mix_color(hex(0x8a6f50), hex(0x4a3828), wet);
        if m.base_color != c {
            m.base_color = c;
            m.perceptual_roughness = 0.95 - 0.5 * wet;
        }
    }
}

impl PartialOrd for Phase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((*self as u8).cmp(&(*other as u8)))
    }
}

fn spawn_particle(w: &mut World, pos: Vec3, vel: Vec3, life: f32, size: f32, grow: f32, gravity: f32, kind: u8) {
    let a = w.resource::<B612State>().assets.clone();
    let mat = match kind {
        0 => a.water,
        1 => a.steam,
        _ => a.spark,
    };
    w.spawn((
        Particle {
            billboard: kind == 1,
            vel,
            life,
            max: life,
            gravity,
            size,
            grow,
        },
        Mesh3d(if kind == 1 { a.smoke } else { a.ball }),
        MeshMaterial3d(mat),
        Transform::from_translation(pos).with_scale(Vec3::splat(size)),
        NotShadowCaster,
        ChapterScoped,
    ));
}

/// Steam from the two active vents, waterfall spray, the pouring stream and
/// the bubbles while filling.
fn emitters(w: &mut World, dt: f32) {
    let sites = w.resource::<Sites>().clone();
    let (seq, timers, can, mut rng) = {
        let s = w.resource::<B612State>();
        (s.seq, s.emit_t, s.can, s.rng.clone())
    };
    let mut t = timers;
    for v in t.iter_mut() {
        *v += dt;
    }
    // steam
    if t[0] > 0.12 {
        t[0] = 0.0;
        for v in &sites.vents {
            let j = Vec3::new(rng.range(-0.08, 0.08), 0.0, rng.range(-0.08, 0.08));
            spawn_particle(w, *v + j, Vec3::new(0.12, 0.45, 0.05), 7.0, 0.13, 0.6, -0.0, 1);
        }
    }
    // waterfall spray
    if t[1] > 0.12 {
        t[1] = 0.0;
        let out = flat(sites.brook_end).normalize_or_zero();
        let p = sites.brook_end + out * 0.5 + Vec3::new(rng.range(-0.25, 0.25), -rng.range(0.0, 2.0), rng.range(-0.25, 0.25));
        spawn_particle(w, p, out * 0.4 + Vec3::Y * -1.5, 1.4, 0.05, 0.02, 6.0, 0);
    }
    // pouring stream
    if let Seq::Water { stage: 2, t: st_t } = seq {
        if st_t > 0.35 && t[2] > 0.03 {
            t[2] = 0.0;
            if let Some(ct) = w.get::<Transform>(can).copied() {
                let tip = ct.translation + ct.rotation * Vec3::new(0.0, 0.27, 0.32);
                let v = ct.rotation * Vec3::new(0.0, -0.2, 0.7) + Vec3::new(rng.range(-0.05, 0.05), 0.0, rng.range(-0.05, 0.05));
                spawn_particle(w, tip, v, 0.55, 0.018, 0.0, 9.0, 0);
            }
        }
    }
    // bubbles + sparkles while filling
    if let Seq::Fill { stage: 2, .. } = seq {
        if t[3] > 0.05 {
            t[3] = 0.0;
            if let Some(ct) = w.get::<Transform>(can).copied() {
                let p = Vec3::new(ct.translation.x, sites.water_y - 0.05, ct.translation.z) + Vec3::new(rng.range(-0.12, 0.12), 0.0, rng.range(-0.12, 0.12));
                spawn_particle(w, p, Vec3::Y * 0.5, 0.35, 0.02, 0.0, 0.0, 0);
                if rng.f() < 0.3 {
                    spawn_particle(
                        w,
                        p + Vec3::Y * 0.1,
                        Vec3::new(rng.range(-0.3, 0.3), 0.8, rng.range(-0.3, 0.3)),
                        0.6,
                        0.015,
                        0.0,
                        2.0,
                        2,
                    );
                }
            }
        }
    }
    let mut s = st(w);
    s.emit_t = t;
    s.rng = rng;
}

// ------------------------------------------------------------- plain systems

pub fn particles(mut commands: Commands, time: Res<Time>, camera:Res<CameraRig>, mut q: Query<(Entity, &mut Particle, &mut Transform)>) {
    let dt = time.delta_secs();
    for (e, mut p, mut t) in &mut q {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        p.vel.y -= p.gravity * dt;
        t.translation += p.vel * dt;
        if p.billboard {
            t.look_at(camera.last_pos,Vec3::Y);
        }
        let age = 1.0 - p.life / p.max;
        let fade = (p.life / p.max * 3.0).min(1.0);
        t.scale = Vec3::splat((p.size + p.grow * age) * fade);
    }
}

pub fn trail(
    time: Res<Time>,
    state: Option<ResMut<B612State>>,
    runner: Res<StoryRunner>,
    terrain: Option<Res<Terrain>>,
    player: Query<&Transform, (With<Player>, Without<TrailDot>)>,
    mut dots: Query<(&TrailDot, &mut Transform, &mut Visibility)>,
) {
    let Some(mut s) = state else { return };
    s.trail_t = (s.trail_t - time.delta_secs()).max(0.0);
    let show = (s.trail_persist || s.trail_t > 0.0) && runner.target_world.is_some();
    let (Ok(pt), Some(tg), Some(terrain)) = (player.single(), runner.target_world, terrain) else {
        for (_, _, mut v) in &mut dots {
            *v = Visibility::Hidden;
        }
        return;
    };
    let from = Vec2::new(pt.translation.x, pt.translation.z);
    let to = Vec2::new(tg.x, tg.z);
    let len = from.distance(to);
    let dir = (to - from).normalize_or_zero();
    let side = Vec2::new(-dir.y, dir.x);
    let fade = if s.trail_persist { 1.0 } else { (s.trail_t / 1.5).min(1.0) };
    let tt = time.elapsed_secs();
    for (d, mut t, mut v) in &mut dots {
        let dist = 1.4 + d.0 as f32 * 1.0;
        if !show || dist > len - 0.8 {
            *v = Visibility::Hidden;
            continue;
        }
        *v = Visibility::Inherited;
        let curve = (dist / len.max(0.1) * PI).sin() * 0.5;
        let q = from + dir * dist + side * curve;
        let y = terrain.height(q.x, q.y) + 0.12 + (tt * 3.0 - d.0 as f32 * 0.5).sin() * 0.03;
        t.translation = Vec3::new(q.x, y, q.y);
        let pulse = 0.5 + 0.5 * (tt * 4.0 - d.0 as f32 * 0.7).sin();
        t.scale = Vec3::splat((0.035 + 0.03 * pulse) * fade);
    }
}

pub fn butterflies(time: Res<Time>, mut q: Query<(&Butterfly, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (b, mut tf) in &mut q {
        let a = t * 0.45 + b.ph;
        let p = b.center
            + Vec3::new(
                a.sin() * b.r,
                0.55 + (t * 1.3 + b.ph).sin() * 0.22 + (a * 2.0).sin() * 0.1,
                (a * 0.8).cos() * b.r * 0.8,
            );
        let prev = tf.translation;
        tf.translation = p;
        let d = p - prev;
        if d.length_squared() > 1e-8 {
            tf.rotation = Quat::from_rotation_y(d.x.atan2(d.z));
        }
    }
}

/// Wing flapping for birds and butterflies.
pub fn wings(time: Res<Time>, mut q: Query<(&PartName, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (n, mut tf) in &mut q {
        let s = match n.0.as_str() {
            "bird_wing_l" => (t * 6.0).sin() * 0.55,
            "bird_wing_r" => -(t * 6.0).sin() * 0.55,
            "bfly_wing_l" => (t * 18.0).sin() * 0.9,
            "bfly_wing_r" => -(t * 18.0).sin() * 0.9,
            _ => continue,
        };
        tf.rotation = Quat::from_rotation_z(s);
    }
}

/// Migrating birds (B612Birds): occasional day flyovers, and the flock that
/// comes for the prince when departure opens (~3 s later).
pub fn birds(
    time: Res<Time>,
    state: Option<ResMut<B612State>>,
    runner: Res<StoryRunner>,
    player: Query<&Transform, (With<Player>, Without<Bird>)>,
    mut q: Query<(&Bird, &mut Transform, &mut Visibility)>,
) {
    let Some(mut s) = state else { return };
    let dt = time.delta_secs();
    let now = s.time;
    // flyover
    if s.fly_t < 0.0 && now > s.fly_next {
        let a = s.rng.range(0.0, TAU);
        let h = s.rng.range(20.0, 30.0);
        let d = Vec3::new(a.sin(), 0.0, a.cos());
        let side = Vec3::new(-d.z, 0.0, d.x) * s.rng.range(-12.0, 12.0);
        s.fly_from = d * 70.0 + side + Vec3::Y * h;
        s.fly_to = -d * 70.0 + side + Vec3::Y * (h + 3.0);
        s.fly_t = 0.0;
    }
    if s.fly_t >= 0.0 {
        s.fly_t += dt / 30.0;
        if s.fly_t > 1.0 {
            s.fly_t = -1.0;
            let r = s.rng.range(70.0, 150.0);
            s.fly_next = now + r;
        }
    }
    let pp = player.single().map(|t| t.translation).unwrap_or_default();
    if runner.departure_ready {
        s.depart_t += dt;
    }
    let dep = s.depart_t - 3.0;
    for (b, mut t, mut v) in &mut q {
        let i = b.idx as f32;
        let row = ((b.idx + 1) / 2) as f32;
        let sgn = if b.idx % 2 == 0 { 1.0 } else { -1.0 };
        if b.depart {
            if dep <= 0.0 {
                *v = Visibility::Hidden;
                continue;
            }
            *v = Visibility::Inherited;
            let ang = dep * 0.9 + i / 7.0 * TAU;
            let circle = pp + Vec3::new(ang.sin() * 2.6, 3.2 + (dep * 2.0 + i).sin() * 0.3, ang.cos() * 2.6);
            let far = pp + Vec3::new(-30.0 + i * 1.5, 14.0 + row, -40.0 + sgn * row * 2.0);
            let k = ease(dep / 4.0);
            let p = far.lerp(circle, k);
            let prev = t.translation;
            t.translation = p;
            let d = p - prev;
            if d.length_squared() > 1e-6 {
                t.rotation = Quat::from_rotation_y(d.x.atan2(d.z)) * Quat::from_rotation_z(-0.3 * k);
            }
            continue;
        }
        if s.fly_t < 0.0 {
            *v = Visibility::Hidden;
            continue;
        }
        *v = Visibility::Inherited;
        let dir = (s.fly_to - s.fly_from).normalize_or_zero();
        let side = Vec3::new(-dir.z, 0.0, dir.x);
        let lead = s.fly_from.lerp(s.fly_to, s.fly_t);
        t.translation = lead - dir * row * 1.6 + side * sgn * row * 1.4 + Vec3::Y * (now * 1.5 + i).sin() * 0.2;
        t.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
        t.scale = Vec3::splat(1.6);
    }
}
