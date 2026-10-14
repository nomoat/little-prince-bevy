//! One input frame per update, merged from keyboard+mouse, touch (virtual
//! joystick, buttons, drag-look, pinch) and the test autopilot — the
//! original's PlayerInputHub / PlayerInputFrame / AgentInput design.

use super::platform;
use super::save::{SaveData, TouchMode};
use super::params::LaunchParams;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

/// Everything gameplay reads. Game code never reads devices directly.
#[derive(Resource, Default, Debug, Clone)]
pub struct InputFrame {
    /// Movement stick, camera-relative (x right, y forward), length ≤ 1.
    pub move_axis: Vec2,
    /// World-space XZ movement (autopilot); overrides `move_axis` when set.
    pub move_world: Option<Vec2>,
    pub run: bool,
    /// Look delta in degrees (x = yaw, y = pitch).
    pub look: Vec2,
    /// Zoom delta in metres (+ = farther).
    pub zoom: f32,
    pub interact: bool,
    pub interact_held: bool,
    pub jump: bool,
    /// Generic secondary action (C / touch button) — e.g. clapping in vanity.
    pub action2: bool,
    /// Esc / menu button.
    pub menu: bool,
    /// Tab: chapter task view (held).
    pub tasks_held: bool,
    /// Advance dialogue (Space / Enter / E / click / tap).
    pub advance: bool,
    /// A tap/click position this frame (logical px), for UI-free picking.
    pub tap: Option<Vec2>,
    /// Gameplay input suppressed this frame (copied from [`InputGate`]).
    pub blocked: bool,
}

/// Who may use input right now. Set by the UI / story layers every frame.
#[derive(Resource, Default, Debug, Clone)]
pub struct InputGate {
    /// A menu (pause/settings) is open: everything but `menu` is dropped.
    pub menu_open: bool,
    /// Player movement/interaction locked (dialogue, cutscene, story lock).
    pub movement_locked: bool,
}

/// Test autopilot input, written by the harness (AgentInput).
#[derive(Resource, Default, Debug, Clone)]
pub struct AgentInput {
    pub active: bool,
    pub move_world: Option<Vec2>,
    pub run: bool,
    pub interact: bool,
    pub interact_held: bool,
    pub jump: bool,
    pub action2: bool,
    pub advance: bool,
    pub look: Vec2,
}

/// On-screen touch controls state.
#[derive(Resource, Default, Debug)]
pub struct TouchControls {
    /// Controls visible (touch device detected / forced).
    pub enabled: bool,
    pub joy_touch: Option<u64>,
    pub joy_origin: Vec2,
    pub joy_pos: Vec2,
    pub look_touch: Option<u64>,
    pub look_last: Vec2,
    pub look_start: Vec2,
    pub look_start_time: f32,
    pub button_touch: [Option<u64>; 5],
    pub pinch_last: Option<f32>,
    /// Held state of buttons: interact, jump, action2, menu, tasks.
    pub held: [bool; 5],
    pub pressed: [bool; 5],
    /// Optional label shown on the interact button (prompt verb).
    pub screen: Vec2,
}

pub const BTN_INTERACT: usize = 0;
pub const BTN_JUMP: usize = 1;
pub const BTN_ACTION2: usize = 2;
pub const BTN_MENU: usize = 3;
pub const BTN_TASKS: usize = 4;

/// Joystick radius (logical px).
pub const JOY_RADIUS: f32 = 60.0;

impl TouchControls {
    /// Button centres and radii in logical px for a screen size.
    pub fn buttons(screen: Vec2) -> [(Vec2, f32); 5] {
        let (w, h) = (screen.x, screen.y);
        [
            (Vec2::new(w - 92.0, h - 120.0), 46.0), // interact
            (Vec2::new(w - 190.0, h - 70.0), 36.0), // jump
            (Vec2::new(w - 190.0, h - 170.0), 30.0), // action2
            (Vec2::new(w - 36.0, 36.0), 24.0),      // menu
            (Vec2::new(w - 92.0, 36.0), 24.0),      // tasks
        ]
    }
}

/// Pointer state for mouse drag-look.
#[derive(Resource, Default)]
pub(crate) struct MouseState {
    dragging: bool,
    drag_dist: f32,
    any_touch_seen: bool,
}

/// Set by the UI layer when the pointer is over an interactive UI element.
#[derive(Resource, Default)]
pub struct PointerOverUi(pub bool);

#[allow(clippy::too_many_arguments)]
pub(crate) fn gather_input(
    mut frame: ResMut<InputFrame>,
    gate: Res<InputGate>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    touches: Res<Touches>,
    mut touch: ResMut<TouchControls>,
    mut ms: ResMut<MouseState>,
    agent: Res<AgentInput>,
    save: Res<SaveData>,
    params: Res<LaunchParams>,
    over_ui: Res<PointerOverUi>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
    time: Res<Time<Real>>,
) {
    let blocked = gate.menu_open;
    let mut f = InputFrame { blocked: blocked || gate.movement_locked, ..default() };
    let s = &save.settings;
    let look_k = 0.16 * s.look;
    let inv = if s.invert_y { -1.0 } else { 1.0 };

    // ---------------------------------------------------- keyboard + mouse
    let mut mv = Vec2::ZERO;
    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        mv.y += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        mv.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        mv.x += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        mv.x -= 1.0;
    }
    f.move_axis = mv.clamp_length_max(1.0);
    f.run = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    f.jump = keys.just_pressed(KeyCode::Space);
    f.interact = keys.just_pressed(KeyCode::KeyE);
    f.interact_held = keys.pressed(KeyCode::KeyE);
    f.action2 = keys.just_pressed(KeyCode::KeyC) || keys.just_pressed(KeyCode::KeyF);
    f.menu = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP);
    f.tasks_held = keys.pressed(KeyCode::Tab);
    f.advance = keys.any_just_pressed([KeyCode::Space, KeyCode::Enter, KeyCode::KeyE, KeyCode::NumpadEnter]);

    let locked = cursor.single().map(|c| c.grab_mode != CursorGrabMode::None).unwrap_or(false);
    if mouse.just_pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Right) {
        ms.dragging = !over_ui.0;
        ms.drag_dist = 0.0;
    }
    let held = mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right);
    if !held {
        ms.dragging = false;
    }
    if (ms.dragging && held) || locked {
        ms.drag_dist += motion.delta.length();
        f.look = Vec2::new(motion.delta.x, motion.delta.y * inv) * look_k;
    }
    if mouse.just_released(MouseButton::Left) && ms.drag_dist < 6.0 && !over_ui.0 {
        f.advance = true;
        if let Ok(w) = window.single() {
            f.tap = w.cursor_position();
        }
    }
    // pointer lock option: click locks, Esc releases
    if s.pointer_lock && !blocked && mouse.just_pressed(MouseButton::Left) && !over_ui.0 && !touch.enabled {
        if let Ok(mut c) = cursor.single_mut() {
            c.grab_mode = CursorGrabMode::Locked;
            c.visible = false;
        }
    }
    if (blocked || f.menu || !s.pointer_lock) && locked {
        if let Ok(mut c) = cursor.single_mut() {
            c.grab_mode = CursorGrabMode::None;
            c.visible = true;
        }
    }
    let notch = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 60.0,
    };
    f.zoom = -notch * 0.6;

    // ---------------------------------------------------------------- touch
    let screen = window.single().map(|w| Vec2::new(w.width(), w.height())).unwrap_or(Vec2::new(1280.0, 720.0));
    touch.screen = screen;
    if touches.iter().next().is_some() || touches.iter_just_pressed().next().is_some() {
        ms.any_touch_seen = true;
    }
    touch.enabled = match (params.touch, s.touch) {
        (Some(v), _) => v,
        (None, TouchMode::On) => true,
        (None, TouchMode::Off) => false,
        (None, TouchMode::Auto) => ms.any_touch_seen || platform::is_touch_device(),
    };
    touch.pressed = [false; 5];
    let buttons = TouchControls::buttons(screen);
    for t in touches.iter_just_pressed() {
        let p = t.position();
        let mut hit = false;
        if touch.enabled {
            for (i, (c, r)) in buttons.iter().enumerate() {
                if p.distance(*c) < r + 10.0 && touch.button_touch[i].is_none() {
                    touch.button_touch[i] = Some(t.id());
                    touch.pressed[i] = true;
                    hit = true;
                    break;
                }
            }
        }
        if hit || over_ui.0 {
            continue;
        }
        if touch.enabled && p.x < screen.x * 0.45 && touch.joy_touch.is_none() {
            touch.joy_touch = Some(t.id());
            touch.joy_origin = p;
            touch.joy_pos = p;
        } else if touch.look_touch.is_none() {
            touch.look_touch = Some(t.id());
            touch.look_last = p;
            touch.look_start = p;
            touch.look_start_time = time.elapsed_secs();
        }
    }
    for t in touches.iter_just_released().chain(touches.iter_just_canceled()) {
        let id = t.id();
        if touch.joy_touch == Some(id) {
            touch.joy_touch = None;
        }
        if touch.look_touch == Some(id) {
            touch.look_touch = None;
            // quick tap without movement = advance / tap
            if t.position().distance(touch.look_start) < 12.0 && time.elapsed_secs() - touch.look_start_time < 0.35 {
                f.advance = true;
                f.tap = Some(t.position());
            }
        }
        for i in 0..5 {
            if touch.button_touch[i] == Some(id) {
                touch.button_touch[i] = None;
            }
        }
    }
    for i in 0..5 {
        touch.held[i] = touch.button_touch[i].is_some();
    }
    if let Some(id) = touch.joy_touch {
        if let Some(t) = touches.get_pressed(id) {
            touch.joy_pos = t.position();
            let mut d = t.position() - touch.joy_origin;
            // drag the base along when pulled past the rim
            if d.length() > JOY_RADIUS * 1.4 {
                let n = d.normalize();
                touch.joy_origin = t.position() - n * JOY_RADIUS * 1.4;
                d = n * JOY_RADIUS * 1.4;
            }
            let v = Vec2::new(d.x, -d.y) / JOY_RADIUS;
            if v.length() > 0.12 {
                f.move_axis = v.clamp_length_max(1.0);
                f.run = f.run || v.length() > 1.15;
            }
        }
    }
    let pinch: Vec<Vec2> = touches.iter().filter(|t| Some(t.id()) != touch.joy_touch && !touch.button_touch.contains(&Some(t.id()))).map(|t| t.position()).collect();
    if pinch.len() >= 2 {
        let d = pinch[0].distance(pinch[1]);
        if let Some(last) = touch.pinch_last {
            f.zoom += (last - d) * 0.02;
        }
        touch.pinch_last = Some(d);
    } else {
        touch.pinch_last = None;
        if let Some(id) = touch.look_touch {
            if let Some(t) = touches.get_pressed(id) {
                let d = t.position() - touch.look_last;
                touch.look_last = t.position();
                f.look += Vec2::new(d.x, d.y * inv) * 0.28 * s.look;
            }
        }
    }
    if touch.enabled {
        f.interact |= touch.pressed[BTN_INTERACT];
        f.interact_held |= touch.held[BTN_INTERACT];
        f.advance |= touch.pressed[BTN_INTERACT];
        f.jump |= touch.pressed[BTN_JUMP];
        f.action2 |= touch.pressed[BTN_ACTION2];
        f.menu |= touch.pressed[BTN_MENU];
        f.tasks_held |= touch.held[BTN_TASKS];
    }

    // ---------------------------------------------------------------- agent
    if agent.active {
        f.move_world = agent.move_world;
        f.run |= agent.run;
        f.interact |= agent.interact;
        f.interact_held |= agent.interact_held;
        f.jump |= agent.jump;
        f.action2 |= agent.action2;
        f.advance |= agent.advance;
        f.look += agent.look;
    }

    if blocked || gate.movement_locked {
        f.move_axis = Vec2::ZERO;
        f.move_world = None;
        f.jump = false;
        f.interact = false;
        f.interact_held = false;
        f.action2 = false;
    }
    if blocked {
        f.look = Vec2::ZERO;
        f.zoom = 0.0;
        f.advance = false;
        f.tasks_held = false;
    }
    *frame = f;
}
