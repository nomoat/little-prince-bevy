//! The brass telescope (ObservatoryTelescope + TelescopeSky of the original):
//! the prince walks to the eyepiece, bends to look, the screen fades to black
//! and opens on a round lens view of the sky. Three lenses (32°, 20°, 12°);
//! B-612 waits 42° to the right of and 36° above where the tube points. With
//! a wide lens it is only a warm star; through the strongest lens it becomes a
//! little world with three volcanoes and a rose. Holding it within 3° of the
//! centre (a quarter of the lens if wider) at ≤ 20.5° for 0.6 s finds it —
//! the chapter's hidden goal. Shooting stars point the way until then.

use super::kit::{dir_from, face_toward, quad_mesh, texture};
use crate::core::interact::InteractFocus;
use crate::core::prelude::*;
use crate::core::ui::UiFont;
use crate::core::ui::hud::Hud;

pub const LENSES: [f32; 3] = [32.0, 20.0, 12.0];
pub const TUBE_AZ: f32 = 22.5;
pub const TUBE_EL: f32 = 30.0;
/// B-612 offset from the tube's aim: 42° to the right (screen right = lower
/// game azimuth), 36° above.
pub const B612_AZ: f32 = TUBE_AZ - 42.0;
pub const B612_EL: f32 = TUBE_EL + 36.0;
pub const YAW_LIMIT: f32 = 60.0;
pub const EL_MIN: f32 = 8.0;
pub const EL_MAX: f32 = 85.0;
/// The view lens sits 13 m above the telescope base (clears the roof).
pub const LENS_HEIGHT: f32 = 13.0;
const SKY_R: f32 = 380.0;
const B612_R: f32 = 365.0;
/// Planet width when fully shown (degrees).
const B612_WIDTH: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ScopeState {
    #[default]
    Idle,
    Approach,
    Align,
    Look,
    Viewing,
    Leaving,
}

#[derive(Resource)]
pub struct Scope {
    pub state: ScopeState,
    pub t: f32,
    pub base: Vec3,
    pub stance: Vec3,
    pub eyepiece: Vec3,
    pub az: f32,
    pub el: f32,
    pub lens: usize,
    pub fov: f32,
    zoom_acc: f32,
    found_t: f32,
    pub found: bool,
    view_time: f32,
    meteor_next: f32,
    meteor: Option<(f32, Vec3, Vec3)>,
    pub fade: f32,
    fade_target: f32,
    cooldown: f32,
    hinted: bool,
    rng: Rng,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            state: ScopeState::Idle,
            t: 0.0,
            base: Vec3::ZERO,
            stance: Vec3::ZERO,
            eyepiece: Vec3::ZERO,
            az: TUBE_AZ,
            el: TUBE_EL,
            lens: 0,
            fov: LENSES[0],
            zoom_acc: 0.0,
            found_t: 0.0,
            found: false,
            view_time: 0.0,
            meteor_next: 2.5,
            meteor: None,
            fade: 0.0,
            fade_target: 0.0,
            cooldown: 0.0,
            hinted: false,
            rng: Rng::new(612),
        }
    }
}

#[derive(Component)]
pub struct ScopeSkyRoot;
#[derive(Component)]
pub struct WarmStar(f32);
#[derive(Component)]
pub struct B612Pic;
#[derive(Component)]
pub struct Meteor;
#[derive(Component)]
pub struct ScopeUi;
#[derive(Component)]
pub struct ScopeFade;
#[derive(Component)]
pub struct LensLabel;
#[derive(Component)]
pub struct ScopeHint;

#[derive(Resource, Clone)]
pub struct ScopeMats {
    warm: Vec<Handle<StandardMaterial>>,
    meteor: Handle<StandardMaterial>,
}

/// Spawns the telescope sky (hidden until viewing) and the lens overlay UI.
pub fn build(cx: &mut ChapterCx, base: Vec3) {
    let stance = base + Quat::from_rotation_y(TUBE_AZ.to_radians()) * Vec3::new(-0.044, 0.0, -0.62);
    let eyepiece = base + Quat::from_rotation_y(TUBE_AZ.to_radians()) * Vec3::new(0.0, 1.085, -0.451);
    cx.world.insert_resource(Scope { base, stance, eyepiece, ..default() });
    let lens = base + Vec3::Y * LENS_HEIGHT;
    let w = &mut *cx.world;
    let glow = texture(w, "tex.businessman.glow", "glow");
    let flare = texture(w, "tex.businessman.flare", "flare");
    let dot = texture(w, "tex.businessman.dot", "dot");
    let quad = w.resource_mut::<Assets<Mesh>>().add(quad_mesh());
    let add_mat = |w: &mut World, tex: Handle<Image>, c: Color| {
        w.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
            base_color: c,
            base_color_texture: Some(tex),
            unlit: true,
            fog_enabled: false,
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            ..default()
        })
    };
    let root = w
        .spawn((Name::new("ScopeSky"), ScopeSkyRoot, Transform::from_translation(lens), Visibility::Hidden, ChapterScoped))
        .id();
    // two procedural star layers (mid, far), seeded, concentrated around the tube's field
    let star_mat = add_mat(w, dot.clone(), Color::srgb(1.0, 1.0, 1.0));
    let mut rng = Rng::new(5);
    let mut mesh = MeshBuilder::new();
    let tiny = |mesh: &mut MeshBuilder, d: Vec3, r: f32, size: f32, col: Color| {
        let c = d * r;
        let (u, v) = d.any_orthonormal_pair();
        let (u, v) = (u * size, v * size);
        let n = -d;
        let i0 = mesh.vertex(c - u - v, n, col, Vec2::new(0.0, 0.0));
        let i1 = mesh.vertex(c + u - v, n, col, Vec2::new(1.0, 0.0));
        let i2 = mesh.vertex(c + u + v, n, col, Vec2::new(1.0, 1.0));
        let i3 = mesh.vertex(c - u + v, n, col, Vec2::new(0.0, 1.0));
        mesh.indices.extend([i0, i1, i2, i0, i2, i3]);
    };
    for layer in 0..2 {
        let n = if layer == 0 { 900 } else { 1700 };
        let r = SKY_R - layer as f32 * 6.0;
        for _ in 0..n {
            let az = TUBE_AZ + rng.range(-80.0, 80.0);
            let el = (rng.f().sqrt() * 90.0).max(2.0);
            let d = dir_from(az, el);
            let big = rng.f();
            let size = if layer == 0 { 0.22 + big * big * big * 0.75 } else { 0.14 + big * 0.18 };
            let warm = rng.f();
            let col = mix_color(hex(0xcfdcff), hex(0xffe2b8), warm);
            let col = scale_color(col, if layer == 0 { 1.0 } else { 0.55 + 0.35 * rng.f() });
            tiny(&mut mesh, d, r, size, col);
        }
    }
    // a faint band of the Milky Way across the field
    for _ in 0..1400 {
        let t = rng.range(-1.0, 1.0);
        let az = TUBE_AZ - 70.0 + (t + 1.0) * 70.0;
        let el = 50.0 + t * 30.0 + rng.range(-7.0, 7.0) * (1.0 - t.abs() * 0.4);
        let d = dir_from(az, el.clamp(3.0, 88.0));
        tiny(&mut mesh, d, SKY_R - 12.0, 0.12 + rng.f() * 0.14, scale_color(hex(0xb8c4ec), 0.35 + rng.f() * 0.3));
    }
    let stars = w.resource_mut::<Assets<Mesh>>().add(mesh.build());
    let vc = w.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(dot.clone()), unlit: true, fog_enabled: false, alpha_mode: AlphaMode::Add, cull_mode: None, ..default() });
    let _ = star_mat;
    let s = w.spawn((Mesh3d(stars), MeshMaterial3d(vc), Transform::default())).id();
    w.entity_mut(root).add_child(s);
    // B-612: a warm star (glow, core, four-point flare) and the picture
    let d = dir_from(B612_AZ, B612_EL);
    let warm_col = Color::srgb(1.0, 0.82, 0.56);
    let mut warm = Vec::new();
    for (tex, px, k) in [(glow, 90.0f32, 0.55f32), (dot.clone(), 16.0, 1.0), (flare, 150.0, 0.7)] {
        let m = add_mat(w, tex, warm_col.with_alpha(k));
        warm.push(m.clone());
        let p = d * (B612_R - 2.0);
        let e = w.spawn((Mesh3d(quad.clone()), MeshMaterial3d(m), Transform::from_translation(p).with_rotation(face_toward(p, Vec3::ZERO)), WarmStar(px))).id();
        w.entity_mut(root).add_child(e);
    }
    let pic_scale = B612_R * (B612_WIDTH * 0.5).to_radians().tan();
    let p = d * B612_R;
    let rot = face_toward(p, Vec3::ZERO);
    let pic = w
        .spawn((
            Transform::from_translation(p).with_rotation(rot).with_scale(Vec3::splat(pic_scale)),
            Visibility::Hidden,
            crate::core::models::ModelKey("model.businessman.b612_view".into()),
            crate::core::models::PendingModel("model.businessman.b612_view".into()),
            B612Pic,
        ))
        .id();
    w.entity_mut(root).add_child(pic);
    // meteor streak
    let meteor = add_mat(w, dot, Color::srgba(1.0, 0.95, 0.85, 0.0));
    let me = w.spawn((Mesh3d(quad), MeshMaterial3d(meteor.clone()), Transform::from_translation(d * SKY_R), Visibility::Hidden, Meteor)).id();
    w.entity_mut(root).add_child(me);
    w.insert_resource(ScopeMats { warm, meteor });

    // ---- lens overlay UI: a round mask (huge outline around a circle), brass rim, hints
    let font = w.resource::<UiFont>().clone();
    w.spawn((
        Name::new("ScopeUi"),
        ScopeUi,
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), justify_content: JustifyContent::Center, align_items: AlignItems::Center, display: Display::None, ..default() },
        GlobalZIndex(4),
        bevy::picking::Pickable::IGNORE,
        ChapterScoped,
    ))
    .with_children(|ui| {
        ui.spawn((
            Node { width: vh(94), height: vh(94), border: UiRect::all(px(3)), border_radius: BorderRadius::MAX, ..default() },
            BorderColor::all(Color::srgb(0.62, 0.48, 0.22)),
            Outline::new(px(4000), px(0), Color::srgb(0.01, 0.01, 0.02)),
            bevy::picking::Pickable::IGNORE,
        ));
        ui.spawn((
            Node { position_type: PositionType::Absolute, bottom: px(18), left: px(16), right: px(16), justify_content: JustifyContent::Center, ..default() },
            bevy::picking::Pickable::IGNORE,
        ))
        .with_children(|r| {
            r.spawn((Text::new(""), Tr::new("businessman.scope.help"), font.tf(17.0), TextColor(Color::srgba(0.95, 0.9, 0.78, 0.8)), ScopeHint));
        });
        ui.spawn((
            Node { position_type: PositionType::Absolute, top: px(18), right: px(22), ..default() },
            bevy::picking::Pickable::IGNORE,
        ))
        .with_children(|r| {
            r.spawn((Text::new("32°"), font.tf(22.0), TextColor(Color::srgb(0.93, 0.76, 0.35)), LensLabel));
        });
    });
    w.spawn((
        Name::new("ScopeFade"),
        ScopeFade,
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(6),
        bevy::picking::Pickable::IGNORE,
        ChapterScoped,
    ));
}

fn angle_between(a: Vec3, b: Vec3) -> f32 {
    a.angle_between(b).to_degrees()
}

/// Begins the approach when the telescope is used.
pub fn on_use(mut msgs: MessageReader<Interacted>, mut scope: ResMut<Scope>, mut player: Query<&mut Player>, mut its: Query<&mut Interactable>) {
    for m in msgs.read() {
        if m.id != "telescope" || scope.state != ScopeState::Idle {
            continue;
        }
        scope.state = ScopeState::Approach;
        scope.t = 0.0;
        if let Ok(mut p) = player.single_mut() {
            p.drive = Drive::WalkTo { target: Vec2::new(scope.stance.x, scope.stance.z), speed: 0.9, stop_radius: 0.08 };
        }
        for mut it in &mut its {
            if it.id == "telescope" {
                it.enabled = false;
            }
        }
    }
}

/// The telescope state machine + the view.
#[allow(clippy::too_many_arguments)]
pub fn update(
    time: Res<Time>,
    input: Res<InputFrame>,
    mut scope: ResMut<Scope>,
    mut rig: ResMut<CameraRig>,
    mut hud: ResMut<Hud>,
    mut focus: ResMut<InteractFocus>,
    mut audio: MessageWriter<AudioCmd>,
    mut commands: Commands,
    mut player: Query<(&mut Player, &Transform)>,
    mut its: Query<&mut Interactable>,
    i18n: Res<I18n>,
) {
    let dt = time.delta_secs();
    let s = &mut *scope;
    s.t += dt;
    s.cooldown = (s.cooldown - dt).max(0.0);
    // fade toward its target (0.3 s)
    let k = dt / 0.3;
    s.fade = if s.fade < s.fade_target { (s.fade + k).min(s.fade_target) } else { (s.fade - k).max(s.fade_target) };
    let Ok((mut pl, ptf)) = player.single_mut() else { return };
    match s.state {
        ScopeState::Idle => {
            if s.cooldown <= 0.0 {
                for mut it in &mut its {
                    if it.id == "telescope" && !it.enabled {
                        it.enabled = true;
                    }
                }
            }
        }
        ScopeState::Approach => {
            if input.move_axis.length() > 0.6 {
                // new move input cancels
                pl.drive = Drive::Player;
                s.state = ScopeState::Idle;
                s.cooldown = 0.4;
            } else if pl.is_arrived() || s.t > 8.0 {
                pl.drive = Drive::External;
                let d = s.eyepiece - ptf.translation;
                pl.face_target = Some(d.x.atan2(d.z));
                s.state = ScopeState::Align;
                s.t = 0.0;
            }
        }
        ScopeState::Align => {
            if s.t >= 0.25 {
                pl.pose = Pose::Reach;
                s.state = ScopeState::Look;
                s.t = 0.0;
            }
        }
        ScopeState::Look => {
            // look clip 1.6 s: the eye reaches the eyepiece at 1.4 s
            if s.t >= 1.4 {
                s.fade_target = 1.0;
            }
            if s.t >= 1.4 && s.fade >= 0.999 {
                s.state = ScopeState::Viewing;
                s.t = 0.0;
                s.view_time = 0.0;
                s.az = TUBE_AZ;
                s.el = TUBE_EL;
                s.lens = 0;
                s.fov = LENSES[0];
                s.zoom_acc = 0.0;
                s.found_t = 0.0;
                s.fade_target = 0.0;
                hud.hide_all = true;
                focus.suppressed = true;
                audio.write(AudioCmd::Sfx("sfx.businessman.lens".into(), 0.5));
                let lens = s.base + Vec3::Y * LENS_HEIGHT;
                rig.begin_shot(Shot { position: lens, look_at: lens + dir_from(s.az, s.el), blend: 0.0, fov: Some(s.fov) });
            }
        }
        ScopeState::Viewing => {
            s.view_time += dt;
            // aim: drag / stick at 0.4× the normal look speed
            let slow = 0.4 * (s.fov / LENSES[0]).max(0.35);
            s.az -= input.look.x * slow;
            s.el -= input.look.y * slow;
            s.az -= input.move_axis.x * 22.0 * dt * (s.fov / LENSES[0]);
            s.el += input.move_axis.y * 22.0 * dt * (s.fov / LENSES[0]);
            s.az = s.az.clamp(TUBE_AZ - YAW_LIMIT, TUBE_AZ + YAW_LIMIT);
            s.el = s.el.clamp(EL_MIN, EL_MAX);
            // lenses: wheel / pinch steps, C (action2) cycles stronger
            s.zoom_acc += input.zoom;
            if s.zoom_acc > 0.5 {
                s.zoom_acc = 0.0;
                if s.lens > 0 {
                    s.lens -= 1;
                    audio.write(AudioCmd::Sfx("sfx.businessman.click".into(), 0.5));
                }
            } else if s.zoom_acc < -0.5 {
                s.zoom_acc = 0.0;
                if s.lens < LENSES.len() - 1 {
                    s.lens += 1;
                    audio.write(AudioCmd::Sfx("sfx.businessman.click".into(), 0.5));
                }
            }
            if input.action2 {
                s.lens = (s.lens + 1) % LENSES.len();
                audio.write(AudioCmd::Sfx("sfx.businessman.click".into(), 0.5));
            }
            // zoom ease 0.45 s
            let target = LENSES[s.lens];
            s.fov += (target - s.fov) * (1.0 - (-dt / 0.15).exp());
            let lens = s.base + Vec3::Y * LENS_HEIGHT;
            let dir = dir_from(s.az, s.el);
            if let Some(shot) = rig.shot.as_mut() {
                shot.position = lens;
                shot.look_at = lens + dir * 10.0;
                shot.fov = Some(s.fov);
            }
            // found?
            let b = dir_from(B612_AZ, B612_EL);
            let off = angle_between(dir, b);
            if !s.found {
                if off <= (s.fov * 0.25).max(3.0) && s.fov <= 20.5 {
                    s.found_t += dt;
                    if s.found_t >= 0.6 {
                        s.found = true;
                        audio.write(AudioCmd::Sfx("sfx.businessman.chime".into(), 0.35));
                        hud.toast(i18n.t("businessman.scope.found").to_string(), 4.5);
                        commands.queue(|w: &mut World| {
                            w.resource_mut::<StoryRunner>().set_flag("bm_b612_found");
                            found_hidden(w);
                        });
                    }
                } else {
                    s.found_t = 0.0;
                }
                if !s.hinted && s.fov > 24.0 && off < 7.0 {
                    s.hinted = true;
                    hud.toast(i18n.t("businessman.scope.warm").to_string(), 4.0);
                }
                // shooting stars toward B-612: after 2.5 s, then every 15 s, 0.75 s each, 20° arc
                if s.meteor.is_none() && s.view_time >= s.meteor_next {
                    s.meteor_next = s.view_time + 15.0;
                    let a = s.rng.range(0.0, std::f32::consts::TAU);
                    let side = Vec3::new(a.cos(), a.sin() * 0.6, 0.0);
                    let (u, v) = b.any_orthonormal_pair();
                    let off_dir = (u * side.x + v * side.y).normalize_or(u);
                    let end = (b + off_dir * (3.0f32).to_radians().tan()).normalize();
                    let start = (b + off_dir * (23.0f32).to_radians().tan()).normalize();
                    s.meteor = Some((0.0, start, end));
                }
            }
            // E leaves; Esc is the pause menu
            if input.interact && s.t > 0.5 {
                begin_leave(s);
            }
            // the HUD toast still shows: keep only it (dialogue etc. hidden)
            hud.hide_all = hud.toast.is_none();
        }
        ScopeState::Leaving => {
            if s.fade >= 0.999 {
                rig.end_shot(0.0);
                hud.hide_all = false;
                focus.suppressed = false;
                pl.pose = Pose::None;
                pl.drive = Drive::Player;
                s.state = ScopeState::Idle;
                s.cooldown = 0.6;
                s.fade_target = 0.0;
            }
        }
    }
}

pub fn begin_leave(s: &mut Scope) {
    if s.state == ScopeState::Viewing {
        s.state = ScopeState::Leaving;
        s.t = 0.0;
        s.fade_target = 1.0;
        s.meteor = None;
    }
}

/// Visuals of the view: sky visibility, B-612 star/picture cross-fade, meteor, overlay.
#[allow(clippy::too_many_arguments)]
pub fn visuals(
    time: Res<Time>,
    mut scope: ResMut<Scope>,
    mats: Option<Res<ScopeMats>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut root: Query<&mut Visibility, (With<ScopeSkyRoot>, Without<B612Pic>, Without<Meteor>, Without<ScopeUi>)>,
    mut warm: Query<(&WarmStar, &mut Transform), (Without<B612Pic>, Without<Meteor>)>,
    mut pic: Query<(&mut Transform, &mut Visibility), (With<B612Pic>, Without<WarmStar>, Without<Meteor>, Without<ScopeSkyRoot>, Without<ScopeUi>)>,
    mut meteor: Query<(&mut Transform, &mut Visibility), (With<Meteor>, Without<WarmStar>, Without<B612Pic>, Without<ScopeSkyRoot>, Without<ScopeUi>)>,
    mut ui: Query<&mut Node, (With<ScopeUi>, Without<ScopeFade>)>,
    mut fade: Query<&mut BackgroundColor, With<ScopeFade>>,
    mut label: Query<&mut Text, With<LensLabel>>,
    window: Query<&Window>,
) {
    let viewing = scope.state == ScopeState::Viewing || (scope.state == ScopeState::Leaving && scope.fade < 0.999);
    for mut v in &mut root {
        *v = if viewing { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut n in &mut ui {
        n.display = if viewing { Display::Flex } else { Display::None };
    }
    for mut b in &mut fade {
        b.0 = Color::srgba(0.0, 0.0, 0.0, scope.fade);
    }
    if !viewing {
        return;
    }
    let fov = scope.fov;
    for mut t in &mut label {
        t.0 = format!("{:.0}°", fov);
    }
    // planet weight: only the star above 24°, only the picture below 14°
    let wp = crate::core::planet::smoothstep(24.0, 14.0, fov);
    let lines = window.single().map(|w| w.height()).unwrap_or(1080.0).max(200.0);
    let _ = lines;
    for (ws, mut t) in &mut warm {
        // pixel sizes are given at 1080 lines: degrees = px / 1080 * fov
        let deg = ws.0 / 1080.0 * fov;
        let size = 2.0 * B612_R * (deg * 0.5).to_radians().tan();
        t.scale = Vec3::splat(size * (1.0 - wp).max(0.001));
    }
    if let Some(m) = mats.as_ref() {
        for (i, h) in m.warm.iter().enumerate() {
            if let Some(mut mat) = materials.get_mut(h) {
                let k = [0.55, 1.0, 0.7][i] * (1.0 - wp);
                mat.base_color = Color::srgba(1.0, 0.82, 0.56, k);
            }
        }
    }
    let pic_scale = B612_R * (B612_WIDTH * 0.5).to_radians().tan();
    for (mut t, mut v) in &mut pic {
        *v = if wp > 0.02 { Visibility::Inherited } else { Visibility::Hidden };
        t.scale = Vec3::splat(pic_scale * (0.25 + 0.75 * wp));
        let base = t.rotation;
        let _ = base;
    }
    // meteor
    let dt = time.delta_secs();
    let mut done = false;
    if let Some((t, a, b)) = scope.meteor.as_mut() {
        *t += dt / 0.75;
        let k = *t;
        if k >= 1.0 {
            done = true;
        } else {
            let p = a.lerp(*b, k).normalize() * (SKY_R - 20.0);
            let motion = (*b - *a).normalize_or(Vec3::X);
            for (mut tf, mut v) in &mut meteor {
                *v = Visibility::Inherited;
                let fwd = (-p).normalize();
                *tf = Transform::from_translation(p).looking_to(-fwd, motion);
                tf.scale = Vec3::new(0.9, 9.0 * (1.0 - (k - 0.5).abs()), 1.0);
            }
            if let Some(mut m) = mats.as_ref().and_then(|m| materials.get_mut(&m.meteor)) {
                m.base_color = Color::srgba(1.0, 0.95, 0.85, (1.0 - (2.0 * k - 1.0).abs()).powf(0.6));
            }
        }
    }
    if done || scope.meteor.is_none() {
        scope.meteor = None;
        for (_, mut v) in &mut meteor {
            *v = Visibility::Hidden;
        }
    }
}

/// Test route helper: aim the view at B-612 through the 20° lens.
pub fn test_aim(w: &mut World) {
    let mut s = w.resource_mut::<Scope>();
    if s.state == ScopeState::Viewing {
        s.az = B612_AZ;
        s.el = B612_EL;
        s.lens = 1;
    }
}

/// Leaves the view at once (script / test).
pub fn force_leave(w: &mut World) {
    let mut s = w.resource_mut::<Scope>();
    begin_leave(&mut s);
}

/// Test route helper (the original AgentInput's DebugTeleport): put the
/// prince on the eyepiece side, facing the eyepiece.
pub fn test_to_stance(w: &mut World) {
    let (stance, eye) = {
        let s = w.resource::<Scope>();
        (s.stance, s.eyepiece)
    };
    let mut q = w.query::<(&mut Transform, &mut Player)>();
    if let Some((mut t, mut p)) = q.iter_mut(w).next() {
        t.translation = stance + Vec3::Y * 0.02;
        let d = eye - stance;
        p.yaw = d.x.atan2(d.z);
        p.velocity = Vec3::ZERO;
    }
}
