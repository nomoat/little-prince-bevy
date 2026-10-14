//! Businessman chapter mechanics: the observatory's double door, the
//! merchant's write loop and one-shots, the interior camera zone, floating
//! rocks, dust motes, flickering lanterns, the prince's fill light and the
//! pencil-scratch sound.

use super::kit::ease_in_out;
use crate::core::camera::CameraTuning;
use crate::core::prelude::*;

// ------------------------------------------------------------------ door --

#[derive(Component)]
pub struct DoorLeaf {
    /// +1 / -1: which way the leaf swings (both swing inward, toward +Z).
    pub sign: f32,
    pub base_yaw: f32,
}

#[derive(Component)]
pub struct DoorBlocker;

#[derive(Clone, Copy, Debug, PartialEq)]
enum DoorAnim {
    /// Walking to the push stance (target).
    Walk(Vec2),
    /// The two-hand push: contact at frame 16, release 40, clip end 54,
    /// leaves rest at 90° at frame 135 (30 fps).
    Push,
    /// Plain close over 1.2 s.
    Close,
}

#[derive(Resource, Default)]
pub struct Door {
    pub open: bool,
    /// Leaf angle in degrees (0 = closed, 90 = open).
    pub angle: f32,
    anim: Option<DoorAnim>,
    t: f32,
    from: f32,
    sounds: u8,
}

const FPS: f32 = 30.0;

/// Baked push curve: 0 before contact, eased to 90° at frame 135.
fn push_curve(frame: f32) -> f32 {
    if frame <= 16.0 {
        return 0.0;
    }
    let u = ((frame - 16.0) / (135.0 - 16.0)).clamp(0.0, 1.0);
    // a firm shove while the hands are on the wood (16..40), then a long coast
    let shove = (u * 3.2).min(1.0);
    let coast = 1.0 - (1.0 - u).powf(2.4);
    90.0 * (0.35 * ease_in_out(shove) + 0.65 * coast)
}

#[allow(clippy::too_many_arguments)]
pub fn door_system(
    time: Res<Time>,
    mut door: ResMut<Door>,
    mut msgs: MessageReader<Interacted>,
    mut player: Query<(&mut Player, &Transform), Without<DoorLeaf>>,
    mut leaves: Query<(&DoorLeaf, &mut Transform)>,
    mut blocker: Query<&mut Collider, With<DoorBlocker>>,
    mut its: Query<&mut Interactable>,
    mut runner: ResMut<StoryRunner>,
    mut audio: MessageWriter<AudioCmd>,
) {
    let dt = time.delta_secs();
    for m in msgs.read() {
        if m.id != "door" || door.anim.is_some() {
            continue;
        }
        if let Ok((mut p, tf)) = player.single_mut() {
            if !door.open {
                // walk to a stance 0.32 m from the seam on the player's side
                let outside = tf.translation.z < -3.2;
                let stance = Vec2::new(0.0, if outside { -3.52 } else { -2.88 });
                p.drive = Drive::WalkTo { target: stance, speed: 0.9, stop_radius: 0.08 };
                door.anim = Some(DoorAnim::Walk(stance));
            } else {
                door.anim = Some(DoorAnim::Close);
                door.from = door.angle;
                audio.write(AudioCmd::Sfx("sfx.businessman.creak".into(), 0.6));
            }
            door.t = 0.0;
            door.sounds = 0;
        }
    }
    match door.anim {
        Some(DoorAnim::Walk(stance)) => {
            door.t += dt;
            if let Ok((mut p, tf)) = player.single_mut() {
                let arrived = p.is_arrived() || Vec2::new(tf.translation.x, tf.translation.z).distance(stance) < 0.12;
                if arrived || door.t > 6.0 {
                    p.drive = Drive::External;
                    p.face_target = Some(if tf.translation.z < -3.2 { 0.0 } else { std::f32::consts::PI });
                    p.pose = Pose::Reach;
                    door.anim = Some(DoorAnim::Push);
                    door.t = 0.0;
                }
            }
        }
        Some(DoorAnim::Push) => {
            door.t += dt;
            let frame = door.t * FPS;
            door.angle = push_curve(frame);
            if frame >= 16.0 && door.sounds == 0 {
                door.sounds = 1;
                audio.write(AudioCmd::Sfx("sfx.businessman.latch".into(), 0.7));
                audio.write(AudioCmd::Sfx("sfx.businessman.creak".into(), 0.5));
                for mut c in &mut blocker {
                    c.solid = false;
                }
            }
            if frame >= 54.0 && door.sounds == 1 {
                door.sounds = 2;
                if let Ok((mut p, _)) = player.single_mut() {
                    p.pose = Pose::None;
                    p.drive = Drive::Player;
                }
            }
            if frame >= 40.0 && !door.open {
                door.open = true;
                runner.set_flag("bm_door_open");
            }
            if frame >= 135.0 {
                door.angle = 90.0;
                door.anim = None;
                audio.write(AudioCmd::Sfx("sfx.businessman.settle".into(), 0.5));
            }
        }
        Some(DoorAnim::Close) => {
            door.t += dt;
            let k = ease_in_out(door.t / 1.2);
            door.angle = door.from * (1.0 - k);
            if k >= 1.0 {
                door.anim = None;
                door.open = false;
                door.angle = 0.0;
                for mut c in &mut blocker {
                    c.solid = true;
                }
                audio.write(AudioCmd::Sfx("sfx.businessman.settle".into(), 0.5));
            }
        }
        None => {}
    }
    for (leaf, mut t) in &mut leaves {
        t.rotation = Quat::from_rotation_y((leaf.base_yaw + leaf.sign * door.angle).to_radians());
    }
    let busy = door.anim.is_some();
    for mut it in &mut its {
        if it.id == "door" {
            it.enabled = !busy;
            it.prompt = if door.open { "businessman.verb.close".into() } else { "businessman.verb.open".into() };
        }
    }
}

// -------------------------------------------------------------- merchant --

#[derive(Component)]
pub struct MerchantRoot;
#[derive(Component)]
pub struct DeskRoot;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum MerchantMode {
    #[default]
    Write,
    /// Merchant_Breath (84 frames).
    Breath,
    /// Writes the number on a slip and locks it in a drawer.
    Lock,
    /// Merchant_Stumped (141 frames): head up at the prince between frames 40 and 90.
    Stumped,
}

#[derive(Resource, Default)]
pub struct Merchant {
    pub mode: MerchantMode,
    pub t: f32,
    /// Seconds of the write loop (240 frames = 8 s).
    pub loop_t: f32,
    pub next_breath: f32,
    scratch_t: f32,
}

pub fn set_mode(w: &mut World, mode: MerchantMode) {
    let mut m = w.resource_mut::<Merchant>();
    m.mode = mode;
    m.t = 0.0;
}

#[allow(clippy::too_many_arguments)]
pub fn merchant_anim(
    time: Res<Time>,
    mut st: ResMut<Merchant>,
    merchant: Query<(&Children, &GlobalTransform), With<MerchantRoot>>,
    desk: Query<&Children, With<DeskRoot>>,
    mut parts: Query<(&PartName, &mut Transform)>,
    player: Query<&Transform, (With<Player>, Without<PartName>)>,
    mut audio: MessageWriter<AudioCmd>,
) {
    let dt = time.delta_secs();
    st.t += dt;
    st.loop_t = (st.loop_t + dt) % 8.0;
    if st.next_breath <= 0.0 {
        st.next_breath = 11.0;
    }
    // one-shots blend in only at the write loop's legal entry frames (38, 85, 130, 163, 223)
    if st.mode == MerchantMode::Write {
        st.next_breath -= dt;
        if st.next_breath <= 0.0 {
            let f = st.loop_t * FPS;
            if [38.0f32, 85.0, 130.0, 163.0, 223.0].iter().any(|e| (f - e).abs() < 1.5) {
                st.mode = MerchantMode::Breath;
                st.t = 0.0;
                st.next_breath = 9.0 + (st.loop_t * 7.3) % 6.0;
            } else {
                st.next_breath = 0.01;
            }
        }
    }
    let (mode, t) = (st.mode, st.t);
    let end = match mode {
        MerchantMode::Breath => 84.0 / FPS,
        MerchantMode::Stumped => 141.0 / FPS,
        MerchantMode::Lock => 4.2,
        MerchantMode::Write => f32::MAX,
    };
    if t >= end {
        st.mode = MerchantMode::Write;
        st.t = 0.0;
    }
    let lt = st.loop_t;
    // write loop: small zig-zag strokes, a pause, a carriage return
    let stroke = (lt * 9.0).sin() * 0.06 * if (lt % 2.7) < 2.1 { 1.0 } else { 0.2 };
    let carry = ((lt % 2.7) / 2.7) * 0.18 - 0.09;
    let (mut head_pitch, mut head_yaw, mut arm_x, mut arm_y, mut body_lift) = (0.32f32, 0.0f32, stroke * 0.6, carry + stroke, 0.0f32);
    match mode {
        MerchantMode::Breath => {
            let k = (t / (84.0 / FPS) * std::f32::consts::PI).sin();
            body_lift = 0.02 * k;
            head_pitch = 0.32 - 0.22 * k;
        }
        MerchantMode::Lock => {
            // reaches to the drawer, slides the slip in, closes it
            let k = (t / 4.2).clamp(0.0, 1.0);
            let reach = (k * std::f32::consts::PI).sin();
            arm_x = -0.25 * reach;
            arm_y = 0.45 * reach;
            head_pitch = 0.45;
        }
        MerchantMode::Stumped => {
            let f = t * FPS;
            let up = if f < 40.0 {
                ease_in_out(f / 40.0)
            } else if f < 90.0 {
                1.0
            } else {
                1.0 - ease_in_out((f - 90.0) / 51.0)
            };
            head_pitch = 0.32 - 0.62 * up;
            arm_x = 0.25 * up;
            arm_y = 0.0;
            // look at the prince
            if let (Ok((_, gt)), Ok(pt)) = (merchant.single(), player.single()) {
                let local = gt.affine().inverse().transform_point3(pt.translation);
                head_yaw = local.x.atan2(local.z).clamp(-0.6, 0.6) * up;
            }
        }
        MerchantMode::Write => {}
    }
    if let Ok((children, _)) = merchant.single() {
        for c in children.iter() {
            if let Ok((name, mut tf)) = parts.get_mut(c) {
                match name.0.as_str() {
                    "head" => {
                        tf.rotation = Quat::from_rotation_y(head_yaw) * Quat::from_rotation_x(head_pitch);
                        tf.translation.y = 0.93 + body_lift;
                    }
                    "arm" => tf.rotation = Quat::from_rotation_x(arm_x) * Quat::from_rotation_y(arm_y),
                    "body" => tf.translation.y = body_lift,
                    _ => {}
                }
            }
        }
    }
    // drawer + slip during the Lock beat
    let (drawer, slip) = if mode == MerchantMode::Lock {
        let k = (t / 4.2).clamp(0.0, 1.0);
        let open = if k < 0.3 { ease_in_out(k / 0.3) } else if k < 0.7 { 1.0 } else { 1.0 - ease_in_out((k - 0.7) / 0.3) };
        let slip_in = ease_in_out(((k - 0.25) / 0.35).clamp(0.0, 1.0));
        (open * 0.28, slip_in)
    } else {
        (0.0, 0.0)
    };
    if let Ok(children) = desk.single() {
        for c in children.iter() {
            if let Ok((name, mut tf)) = parts.get_mut(c) {
                match name.0.as_str() {
                    "drawer" => tf.translation.z = drawer,
                    "slip" => {
                        let a = Vec3::new(0.3, 0.805, 0.22);
                        let b = Vec3::new(0.3, 0.68, 0.28);
                        tf.translation = a.lerp(b, slip);
                        tf.scale = Vec3::splat(if slip >= 0.99 && mode == MerchantMode::Lock { 0.0 } else { 1.0 });
                    }
                    _ => {}
                }
            }
        }
    }
    // pencil scratch tied to the write loop (vol 0.3), heard near the desk
    if mode == MerchantMode::Write {
        st.scratch_t -= dt;
        if st.scratch_t <= 0.0 && (lt % 2.7) < 2.1 {
            st.scratch_t = 0.55;
            if let (Ok((_, gt)), Ok(pt)) = (merchant.single(), player.single()) {
                let d = gt.translation().distance(pt.translation);
                if d < 7.0 {
                    audio.write(AudioCmd::Sfx("sfx.businessman.scratch".into(), 0.3 * (1.0 - d / 7.0)));
                }
            }
        }
    }
}

// ------------------------------------------------------------ camera zone --

/// InteriorCameraZone: inside the room the camera comes closer, looks down
/// more and widens the lens (distance 2.3 [1.5, 2.8], pitch 22°, fov 56),
/// blended over 1 s; exit margin 0.3 m.
#[derive(Resource, Default)]
pub struct CamZone {
    pub inside: bool,
    pub k: f32,
    /// Seconds left of easing the orbit toward the zone's defaults.
    pub ease: f32,
}

pub fn camera_zone(
    time: Res<Time>,
    mut zone: ResMut<CamZone>,
    mut tuning: ResMut<CameraTuning>,
    mut rig: ResMut<CameraRig>,
    player: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_secs();
    let Ok(p) = player.single() else { return };
    let r = Vec2::new(p.translation.x, p.translation.z).length();
    let inside = if zone.inside { r < 3.25 + 0.3 } else { r < 3.25 && p.translation.y > 0.2 };
    if inside != zone.inside {
        zone.inside = inside;
        zone.ease = 1.0;
    }
    let goal = if inside { 1.0 } else { 0.0 };
    zone.k += (goal - zone.k).clamp(-dt, dt);
    let k = zone.k;
    tuning.fov = 50.0 + 6.0 * k;
    tuning.min_distance = 1.8 - 0.3 * k;
    tuning.max_distance = 5.0 - 2.2 * k;
    tuning.min_pitch = -30.0 + 18.0 * k;
    tuning.max_pitch = 60.0 - 5.0 * k;
    if zone.ease > 0.0 && rig.shot.is_none() {
        zone.ease -= dt;
        let a = (dt * 3.0).min(1.0);
        let (d, pitch) = if inside { (2.3, 22.0f32) } else { (3.2, 12.0f32) };
        rig.target_distance += (d - rig.target_distance) * a;
        rig.pitch += (pitch.to_radians() - rig.pitch) * a;
    }
    rig.target_distance = rig.target_distance.clamp(tuning.min_distance, tuning.max_distance);
}

// --------------------------------------------------------- set dressing --

#[derive(Component)]
pub struct FloatingRock {
    pub base: Vec3,
    pub amp: f32,
    pub period: f32,
    pub phase: f32,
    pub axis: Vec3,
    pub spin: f32,
}

pub fn floating_rocks(time: Res<Time>, mut q: Query<(&FloatingRock, &mut Transform)>) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    for (r, mut tf) in &mut q {
        tf.translation = r.base + Vec3::Y * (r.amp * (t / r.period * std::f32::consts::TAU + r.phase).sin());
        tf.rotate(Quat::from_axis_angle(r.axis, (r.spin * dt).to_radians()));
    }
}

#[derive(Component)]
pub struct Mote {
    pub vel: Vec3,
    pub life: f32,
    pub age: f32,
}

/// Dust motes drifting in a 5.4 × 2.6 × 5.4 m box in the lamplight.
pub fn motes(time: Res<Time>, mut q: Query<(&mut Mote, &mut Transform)>) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    for (mut m, mut tf) in &mut q {
        m.age += dt;
        if m.age > m.life {
            m.age = 0.0;
            let seed = (tf.translation.x * 91.7 + t * 13.1).sin() * 43758.5;
            let f = seed - seed.floor();
            tf.translation = Vec3::new((f - 0.5) * 5.4, 0.5 + f * 2.4, ((f * 7.3) % 1.0 - 0.5) * 5.4);
        }
        let wob = Vec3::new((t * 0.3 + tf.translation.z).sin(), (t * 0.21 + tf.translation.x).cos() * 0.4, (t * 0.27 + tf.translation.y).cos()) * 0.03;
        tf.translation += (m.vel + wob) * dt;
        let fade = (m.age / 2.0).min(1.0) * ((m.life - m.age) / 2.0).clamp(0.0, 1.0);
        tf.scale = Vec3::splat(fade.max(0.01));
    }
}

#[derive(Component)]
pub struct Flicker {
    pub base: f32,
    pub seed: f32,
}

pub fn lantern_flicker(time: Res<Time>, mut q: Query<(&Flicker, &mut PointLight)>) {
    let t = time.elapsed_secs();
    for (f, mut l) in &mut q {
        let n = (t * 7.1 + f.seed).sin() * 0.5 + (t * 12.7 + f.seed * 3.0).sin() * 0.3 + (t * 2.3 + f.seed).sin() * 0.2;
        l.intensity = f.base * (0.92 + 0.08 * n);
    }
}

/// A warm fill light that follows the prince (height 1.35, distance 1.2, side 0.35).
#[derive(Component)]
pub struct FillLight;

pub fn fill_light(player: Query<(&Transform, &Player), Without<FillLight>>, mut q: Query<&mut Transform, With<FillLight>>) {
    let Ok((pt, pl)) = player.single() else { return };
    let fwd = pl.forward();
    let right = Vec3::new(-fwd.z, 0.0, fwd.x);
    for mut t in &mut q {
        t.translation = pt.translation + fwd * 1.2 + right * 0.35 + Vec3::Y * 1.35;
    }
}

