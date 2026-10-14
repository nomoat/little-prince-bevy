//! King chapter mechanics: CourtDoor (walk to the stance, two-handed push,
//! leaf opens to 100°), CourtArrivalHint, KingIdleDriver (fidgets every
//! 2–5 s, notices the prince within 7 m, re-arms past 10 m for 3 s),
//! KingHeadLook, the audience gestures (Talk A/B, Consult, lean-out call),
//! CourtCandleFlicker, and the audience camera shots in King space.

use super::world::{KING_YAW, Z0};
use crate::chapters::b612::director::{self, local_to_world};
use crate::core::interact::Interacted;
use crate::core::meshkit::Rng;
use crate::core::player::PlayerTuning;
use crate::core::prelude::*;
use crate::core::story::{Anchors, set_flag};
use bevy::ecs::message::{MessageCursor, Messages};

#[derive(Resource, Clone)]
pub struct KingSites {
    pub floor_y: f32,
    /// King space origin (dais top under his seat); +x = his right, +z = in front.
    pub king_origin: Vec3,
    pub door_stance: Vec3,
    pub hall_inside_z: f32,
}

impl KingSites {
    pub fn k(&self, v: Vec3) -> Vec3 {
        local_to_world(self.king_origin, KING_YAW, v)
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Gesture {
    #[default]
    None,
    Notice,
    TalkA,
    TalkB,
    Consult,
    LeanOut,
    Fidget(u8),
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum DoorState {
    #[default]
    Closed,
    Approaching,
    Reaching,
    Pushing,
    Releasing,
    Open,
}

#[derive(Resource)]
pub struct KingState {
    pub test: bool,
    pub player: Entity,
    pub king: Entity,
    pub hinge: Entity,
    pub leaf: Entity,
    pub lights: Vec<(Entity, f32)>,
    door: DoorState,
    door_t: f32,
    door_angle: f32,
    pub hint_shown: bool,
    pub gesture: Gesture,
    gest_t: f32,
    gest_w: f32,
    next_fidget: f32,
    noticed: bool,
    far_t: f32,
    head: Vec2,
    look_w: f32,
    parts: Option<KingParts>,
    rng: Rng,
    time: f32,
    pub opening_t: f32,
    pub leave_from: Option<Vec3>,
    pub audience: bool,
}

#[derive(Clone, Copy)]
struct KingParts {
    body: Entity,
    head: Entity,
    arm_r: Entity,
    arm_l: Entity,
}

pub fn init_state(world: &mut World, test: bool, player: Entity, king: Entity, hinge: Entity, leaf: Entity, lights: Vec<Entity>) {
    let lights = lights
        .into_iter()
        .map(|e| (e, world.get::<PointLight>(e).map(|l| l.intensity).unwrap_or(1.0)))
        .collect();
    world.insert_resource(KingState {
        test,
        player,
        king,
        hinge,
        leaf,
        lights,
        door: DoorState::Closed,
        door_t: 0.0,
        door_angle: 0.0,
        hint_shown: false,
        gesture: Gesture::None,
        gest_t: 0.0,
        gest_w: 0.0,
        next_fidget: 3.0,
        noticed: false,
        far_t: 0.0,
        head: Vec2::ZERO,
        look_w: 1.0,
        parts: None,
        rng: Rng::new(2),
        time: 0.0,
        opening_t: -1.0,
        leave_from: None,
        audience: false,
    });
}

fn ks(w: &mut World) -> Mut<'_, KingState> {
    w.resource_mut::<KingState>()
}

pub fn player_pos(w: &World) -> Vec3 {
    let p = w.resource::<KingState>().player;
    w.get::<Transform>(p).map(|t| t.translation).unwrap_or_default()
}

fn with_player(w: &mut World, f: impl FnOnce(&mut Player, &mut Transform)) {
    let e = w.resource::<KingState>().player;
    let Some(mut t) = w.get::<Transform>(e).copied() else { return };
    {
        let Some(mut pl) = w.get_mut::<Player>(e) else { return };
        f(&mut pl, &mut t);
    }
    if let Some(mut slot) = w.get_mut::<Transform>(e) {
        *slot = t;
    }
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn gesture(w: &mut World, g: Gesture) {
    let mut s = ks(w);
    s.gesture = g;
    s.gest_t = 0.0;
}

/// Gesture durations (s); LeanOut holds until replaced.
fn gesture_len(g: Gesture) -> f32 {
    match g {
        Gesture::None => 0.0,
        Gesture::Notice => 1.6,
        Gesture::TalkA => 2.6,
        Gesture::TalkB => 2.4,
        Gesture::Consult => 6.4,
        Gesture::LeanOut => 1e9,
        Gesture::Fidget(_) => 1.6,
    }
}

// ------------------------------------------------------------- story hooks

/// Opening shot over the whole planet from above-front-right, pushing in 3 m
/// during the card.
pub fn opening_shot(w: &mut World) {
    ks(w).opening_t = 0.0;
    let (cam, look) = opening_cam(0.0);
    director::shot(w, cam, look, 0.0, 50.0);
}

fn opening_cam(k: f32) -> (Vec3, Vec3) {
    let look = Vec3::new(-0.8, 2.6, 1.5);
    let from = Vec3::new(17.0, 10.5, -21.0);
    let dir = (look - from).normalize();
    (from + dir * 3.0 * ease(k), look)
}

/// Settle: the camera descends to the play view over 4 s.
pub fn opening_settle(w: &mut World) {
    ks(w).opening_t = -1.0;
    w.resource_mut::<CameraRig>().snap_behind(0.0);
    director::release(w, 4.0);
}

/// Audience route (AudienceRoute): out to the foot of the steps first, then
/// up to the stance (0.7, 0, 1.25) off the carpet line on his right.
pub fn audience_begin(w: &mut World) {
    ks(w).audience = true;
    w.resource_mut::<PlayerTuning>().walk_speed = 0.9;
    let s = w.resource::<KingSites>().clone();
    let foot = s.k(Vec3::new(0.7, 0.0, 2.95));
    let foot = Vec3::new(foot.x, s.floor_y, foot.z);
    let stance = s.k(Vec3::new(0.7, 0.0, 1.25));
    let p = player_pos(w);
    // already on the dais front (beside him)? go straight to the stance
    let local = Quat::from_rotation_y(-KING_YAW.to_radians()) * (p - s.king_origin);
    let on_dais = p.y > s.floor_y + 0.3 && local.z.abs() < 2.0;
    let a = if on_dais { stance } else { foot };
    w.resource_mut::<Anchors>().0.insert("aud_a".into(), (a, 0.0));
    w.resource_mut::<Anchors>().0.insert("aud_b".into(), (stance, 0.0));
    w.resource_mut::<Hud>().hide_objective();
    w.resource_mut::<Hud>().toast = None;
    let mut r = w.resource_mut::<StoryRunner>();
    r.target = None;
    r.target_label = None;
}

fn kshot(w: &mut World, cam: Vec3, look: Vec3, blend: f32, fov: f32) {
    let s = w.resource::<KingSites>().clone();
    director::shot(w, s.k(cam), s.k(look), blend, fov);
}

pub fn shot_two(w: &mut World) {
    kshot(w, Vec3::new(-1.75, 1.2, 2.75), Vec3::new(0.25, 0.82, 0.7), 1.0, 40.0);
}
/// The slow push-in (14 s) during the greeting.
pub fn shot_push(w: &mut World) {
    let blend = if w.resource::<KingState>().test { 2.0 } else { 14.0 };
    kshot(w, Vec3::new(-1.5, 1.2, 2.5), Vec3::new(0.2, 0.9, 0.6), blend, 39.0);
}
pub fn shot_king(w: &mut World) {
    kshot(w, Vec3::new(-0.95, 1.2, 2.05), Vec3::new(0.0, 1.15, 0.15), 2.0, 36.0);
}
pub fn shot_calendar(w: &mut World) {
    kshot(w, Vec3::new(-0.62, 1.38, 1.05), Vec3::new(-0.15, 0.94, 0.3), 1.2, 32.0);
}
pub fn shot_prince(w: &mut World) {
    kshot(w, Vec3::new(-0.95, 1.1, 0.8), Vec3::new(0.7, 0.95, 1.35), 2.0, 42.0);
}
/// From the hall (floor level): he walks toward us, the King behind him.
pub fn shot_hall(w: &mut World) {
    let s = w.resource::<KingSites>().clone();
    let cam = s.k(Vec3::new(1.35, 0.0, 5.3));
    let look = s.k(Vec3::new(0.5, 0.0, 1.6));
    let cam = Vec3::new(cam.x, s.floor_y + 1.15, cam.z);
    let look = Vec3::new(look.x, s.floor_y + 1.05, look.z);
    director::shot(w, cam, look, 2.0, 40.0);
}

/// "I must go": he walks away toward the leave point (does not block).
pub fn leave_walk(w: &mut World) {
    let s = w.resource::<KingSites>().clone();
    let lp = s.k(Vec3::new(1.6, 0.0, 2.9));
    let speed = if w.resource::<KingState>().test { 1.8 } else { 0.9 };
    let p = player_pos(w);
    ks(w).leave_from = Some(p);
    with_player(w, |pl, _| {
        pl.drive = Drive::WalkTo {
            target: Vec2::new(lp.x, lp.z),
            speed,
            stop_radius: 0.2,
        }
    });
}

pub fn walked(w: &World, d: f32) -> bool {
    let s = w.resource::<KingState>();
    match s.leave_from {
        Some(f) => {
            let p = player_pos(w);
            Vec2::new(p.x - f.x, p.z - f.z).length() >= d || w.get::<Player>(s.player).map(|p| p.is_arrived()).unwrap_or(true)
        }
        None => true,
    }
}

pub fn arrived(w: &World) -> bool {
    let s = w.resource::<KingState>();
    w.get::<Player>(s.player).map(|p| p.is_arrived()).unwrap_or(true)
}

pub fn audience_end(w: &mut World) {
    director::release(w, 1.6);
    w.resource_mut::<PlayerTuning>().walk_speed = 1.0;
    ks(w).audience = false;
    gesture(w, Gesture::None);
}

// ------------------------------------------------------------- per frame

pub fn tick(world: &mut World, mut cursor: Local<MessageCursor<Interacted>>) {
    if !world.contains_resource::<KingState>() || !world.contains_resource::<KingSites>() {
        return;
    }
    if world.resource::<Time<Virtual>>().is_paused() {
        return;
    }
    let dt = world.resource::<Time>().delta_secs();
    ks(world).time += dt;
    let used: Vec<String> = {
        let msgs = world.resource::<Messages<Interacted>>();
        cursor.read(msgs).map(|m| m.id.clone()).collect()
    };
    for id in used {
        if id == "door" && world.resource::<KingState>().door == DoorState::Closed {
            let mut s = ks(world);
            s.door = DoorState::Approaching;
            s.door_t = 0.0;
        }
    }
    opening(world, dt);
    door(world, dt);
    arrival_hint(world);
    king_idle(world, dt);
    animate_king(world, dt);
    candles(world);
}

fn opening(w: &mut World, dt: f32) {
    let t = w.resource::<KingState>().opening_t;
    if t < 0.0 {
        return;
    }
    let t = t + dt;
    ks(w).opening_t = t;
    let (cam, look) = opening_cam(t / 3.2);
    director::shot(w, cam, look, 0.0, 50.0);
}

/// CourtDoor: Closed → Approaching (0.9 m/s) → Reaching (align 0.25 s) →
/// Pushing (two hands, leaf to 100°) → Releasing → Open. Usable from outside only.
fn door(w: &mut World, dt: f32) {
    let (state, t0, test) = {
        let s = w.resource::<KingState>();
        (s.door, s.door_t, s.test)
    };
    let k = if test { 2.0 } else { 1.0 };
    let t = t0 + dt * k;
    let sites = w.resource::<KingSites>().clone();
    let p = player_pos(w);
    // offered only from outside
    if state == DoorState::Closed {
        let outside = p.z < Z0 - 0.1;
        let mut q = w.query::<&mut Interactable>();
        for mut it in q.iter_mut(w) {
            if it.id == "door" && it.enabled != outside {
                it.enabled = outside;
            }
        }
    }
    let mut next = state;
    let mut nt = t;
    match state {
        DoorState::Closed | DoorState::Open => {}
        DoorState::Approaching => {
            if t0 == 0.0 {
                let st = sites.door_stance;
                with_player(w, |pl, _| {
                    pl.drive = Drive::WalkTo {
                        target: Vec2::new(st.x, st.z),
                        speed: 0.9 * k,
                        stop_radius: 0.1,
                    }
                });
            }
            let arrived = arrived(w);
            if arrived || t > 3.0 {
                with_player(w, |pl, _| {
                    pl.drive = Drive::External;
                    pl.face_target = Some(0.0);
                });
                next = DoorState::Reaching;
                nt = 0.0;
            }
        }
        DoorState::Reaching => {
            if t > 0.25 {
                with_player(w, |pl, _| pl.pose = Pose::Reach);
                w.write_message(AudioCmd::sfx("sfx.king.latch"));
                next = DoorState::Pushing;
                nt = 0.0;
            }
        }
        DoorState::Pushing => {
            if t > 0.15 && t - dt * k <= 0.15 {
                w.write_message(AudioCmd::sfx("sfx.king.creak"));
            }
            let a = ease(((t - 0.15) / 1.6).max(0.0)) * 100.0;
            ks(w).door_angle = a;
            if t > 1.0 {
                with_player(w, |pl, _| pl.pose = Pose::None);
            }
            if t > 1.8 {
                next = DoorState::Releasing;
                nt = 0.0;
                w.write_message(AudioCmd::sfx("sfx.wood"));
            }
        }
        DoorState::Releasing => {
            if t > 0.3 {
                with_player(w, |pl, _| {
                    pl.pose = Pose::None;
                    pl.drive = Drive::Player;
                });
                next = DoorState::Open;
                set_flag(w, "door_open");
                let mut q = w.query::<&mut Interactable>();
                for mut it in q.iter_mut(w) {
                    if it.id == "door" {
                        it.enabled = false;
                    }
                }
            }
        }
    }
    let (hinge, angle) = {
        let mut s = ks(w);
        s.door = next;
        s.door_t = nt;
        (s.hinge, s.door_angle)
    };
    if let Some(mut tf) = w.get_mut::<Transform>(hinge) {
        tf.rotation = Quat::from_rotation_y(-angle.to_radians());
    }
}

/// CourtArrivalHint: a short line on entering the hall (12 s, fades 3 s).
fn arrival_hint(w: &mut World) {
    if w.resource::<KingState>().hint_shown {
        return;
    }
    let s = w.resource::<KingSites>().clone();
    let p = player_pos(w);
    if p.z > s.hall_inside_z && p.y > s.floor_y - 0.3 {
        ks(w).hint_shown = true;
        let text = w.resource::<I18n>().t("king.hint.arrive").to_string();
        w.resource_mut::<Hud>().toast(text, 15.0);
    }
}

/// KingIdleDriver: random fidgets every 2–5 s; notices the prince within 7 m
/// (re-arms after he has been farther than 10 m for 3 s).
fn king_idle(w: &mut World, dt: f32) {
    let sites = w.resource::<KingSites>().clone();
    let p = player_pos(w);
    let d = Vec2::new(p.x - sites.king_origin.x, p.z - sites.king_origin.z).length();
    let mut s = ks(w);
    s.gest_t += dt;
    let len = gesture_len(s.gesture);
    if s.gesture != Gesture::None && s.gest_t > len {
        s.gesture = Gesture::None;
        s.gest_t = 0.0;
    }
    if d > 10.0 {
        s.far_t += dt;
        if s.far_t > 3.0 {
            s.noticed = false;
        }
    } else {
        s.far_t = 0.0;
    }
    if s.audience {
        return;
    }
    if !s.noticed && d < 7.0 {
        s.noticed = true;
        s.gesture = Gesture::Notice;
        s.gest_t = 0.0;
        return;
    }
    if s.gesture == Gesture::None {
        s.next_fidget -= dt;
        if s.next_fidget <= 0.0 {
            let r = s.rng.range(2.0, 5.0);
            s.next_fidget = r;
            let which = (s.rng.f() * 3.0) as u8;
            s.gesture = Gesture::Fidget(which);
            s.gest_t = 0.0;
        }
    }
}

/// Poses the king's parts: rest pose + gesture (cross-fade 0.3 s) + head look
/// (KingHeadLook: range 9 m, yaw ±30°, pitch −18°/+5°, weight fades 0.8 s around clips).
fn animate_king(w: &mut World, dt: f32) {
    if w.resource::<KingState>().parts.is_none() {
        let mut q = w.query::<(Entity, &PartName)>();
        let find = |n: &str, q: &mut QueryState<(Entity, &PartName)>, w: &World| q.iter(w).find(|(_, p)| p.0 == n).map(|(e, _)| e);
        let (b, h, r, l) = (
            find("king_body", &mut q, w),
            find("king_head", &mut q, w),
            find("king_arm_r", &mut q, w),
            find("king_arm_l", &mut q, w),
        );
        if let (Some(body), Some(head), Some(arm_r), Some(arm_l)) = (b, h, r, l) {
            ks(w).parts = Some(KingParts { body, head, arm_r, arm_l });
        } else {
            return;
        }
    }
    let sites = w.resource::<KingSites>().clone();
    let p = player_pos(w);
    let (parts, g, gt, mut gw, mut head, mut look_w, time) = {
        let s = w.resource::<KingState>();
        (s.parts.unwrap(), s.gesture, s.gest_t, s.gest_w, s.head, s.look_w, s.time)
    };
    let len = gesture_len(g);
    let target_w = if g == Gesture::None {
        0.0
    } else if gt > len - 0.3 {
        ((len - gt) / 0.3).clamp(0.0, 1.0)
    } else {
        1.0
    };
    gw += (target_w - gw) * (1.0 - (-dt / 0.1).exp());
    // head look target in the king's local frame
    let head_world = sites.king_origin + Vec3::Y * 1.15;
    let to = (p + Vec3::Y * 1.0) - head_world;
    let local = Quat::from_rotation_y(-KING_YAW.to_radians()) * to;
    let in_range = to.length() < 9.0;
    let yaw_t = if in_range {
        local.x.atan2(local.z).clamp(-30f32.to_radians(), 30f32.to_radians())
    } else {
        0.0
    };
    let pitch_t = if in_range {
        (-local.y)
            .atan2(Vec2::new(local.x, local.z).length())
            .clamp(-5f32.to_radians(), 18f32.to_radians())
    } else {
        0.0
    };
    let gesture_busy = matches!(g, Gesture::Consult | Gesture::LeanOut);
    let lw_t = if gesture_busy { 0.0 } else { 1.0 };
    look_w += (lw_t - look_w) * (1.0 - (-dt / 0.8 * 3.0).exp());
    let a = 1.0 - (-dt * 2.5).exp();
    head += (Vec2::new(yaw_t, pitch_t) - head) * a;
    {
        let mut s = ks(w);
        s.gest_w = gw;
        s.head = head;
        s.look_w = look_w;
    }
    // rest pose
    let breathe = (time * 1.3).sin() * 0.012;
    let mut body = Quat::from_rotation_x(breathe);
    let mut head_r = Quat::from_rotation_y(head.x * look_w) * Quat::from_rotation_x(head.y * look_w);
    let mut arm_r = Quat::from_rotation_x(-0.15);
    let mut arm_l = Quat::from_rotation_x(0.05) * Quat::from_rotation_z(-0.12);
    let wv = gw;
    let s = |amp: f32, f: f32| (gt * f).sin() * amp;
    match g {
        Gesture::None => {}
        Gesture::Notice => {
            body = body.slerp(Quat::from_rotation_x(-0.08), wv);
            head_r = head_r * Quat::from_rotation_x(-0.15 * wv);
        }
        Gesture::TalkA => {
            arm_r = arm_r.slerp(Quat::from_rotation_x(-0.75 + s(0.18, 5.0)) * Quat::from_rotation_z(-0.25), wv);
            body = body.slerp(Quat::from_rotation_x(-0.05), wv);
        }
        Gesture::TalkB => {
            arm_l = arm_l.slerp(Quat::from_rotation_x(-0.6) * Quat::from_rotation_z(0.6 + s(0.15, 4.0)), wv);
        }
        Gesture::Consult => {
            // head bows to the calendar on his left thigh; a finger runs down the page; at 5.2 s he nods
            let nod = if gt > 5.2 { ((gt - 5.2) * 9.0).sin().max(0.0) * 0.15 } else { 0.0 };
            head_r = head_r.slerp(Quat::from_rotation_y(0.35) * Quat::from_rotation_x(0.42 + nod), wv);
            arm_l = arm_l.slerp(Quat::from_rotation_x(0.35 + (gt * 1.1).sin() * 0.06) * Quat::from_rotation_z(-0.05), wv);
            body = body.slerp(Quat::from_rotation_x(0.1), wv);
        }
        Gesture::LeanOut => {
            body = body.slerp(Quat::from_rotation_x(0.22) * Quat::from_rotation_y(0.15), wv);
            arm_r = arm_r.slerp(Quat::from_rotation_x(-1.25) * Quat::from_rotation_z(0.15), wv);
            head_r = head_r.slerp(Quat::from_rotation_x(-0.12), wv);
        }
        Gesture::Fidget(0) => {
            arm_l = arm_l.slerp(Quat::from_rotation_x(-1.1 + s(0.08, 6.0)) * Quat::from_rotation_z(0.25), wv);
        }
        Gesture::Fidget(1) => {
            arm_r = arm_r.slerp(Quat::from_rotation_x(-0.35 + (gt * 7.0).sin().abs() * 0.12), wv);
        }
        Gesture::Fidget(_) => {
            body = body.slerp(Quat::from_rotation_z(0.06) * Quat::from_rotation_x(-0.03), wv);
            head_r = head_r * Quat::from_rotation_z(-0.05 * wv);
        }
    }
    let hip = Vec3::new(0.0, 0.5, 0.0);
    let set = |w: &mut World, e: Entity, pivot: Vec3, rot: Quat, follow_body: bool| {
        if let Some(mut tf) = w.get_mut::<Transform>(e) {
            if follow_body {
                tf.translation = hip + body * (pivot - hip);
                tf.rotation = body * rot;
            } else {
                tf.translation = pivot;
                tf.rotation = rot;
            }
        }
    };
    set(w, parts.body, hip, body, false);
    set(w, parts.head, Vec3::new(0.0, 1.06, 0.0), head_r, true);
    set(w, parts.arm_r, Vec3::new(-0.25, 0.98, 0.0), arm_r, true);
    set(w, parts.arm_l, Vec3::new(0.25, 0.98, 0.0), arm_l, true);
}

/// CourtCandleFlicker.
fn candles(w: &mut World) {
    let (lights, t) = {
        let s = w.resource::<KingState>();
        (s.lights.clone(), s.time)
    };
    for (i, (e, base)) in lights.iter().enumerate() {
        let f = i as f32 * 1.7;
        let k = 1.0 + (t * 9.0 + f).sin() * 0.06 + (t * 23.0 + f * 2.0).sin() * 0.04 + (t * 3.1 + f).sin() * 0.05;
        if let Some(mut l) = w.get_mut::<PointLight>(*e) {
            l.intensity = base * k;
        }
    }
}
