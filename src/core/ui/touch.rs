//! On-screen touch controls (shown on touch devices / `?touch=1`): a floating
//! joystick on the left half, look by dragging on the right, pinch to zoom,
//! and buttons for interact (with the prompt verb), jump, the secondary
//! action, the task view and the menu. Hit-testing is done in input.rs with
//! the same geometry ([`TouchControls::buttons`]).

use super::super::i18n::I18n;
use super::super::input::{JOY_RADIUS, TouchControls};
use super::super::interact::InteractFocus;
use super::super::state::AppState;
use super::{UiFont, theme};
use bevy::picking::Pickable;
use bevy::prelude::*;

#[derive(Component)]
struct TouchRoot;
#[derive(Component)]
struct JoyBase;
#[derive(Component)]
struct JoyKnob;
#[derive(Component)]
struct TouchBtn(usize);
#[derive(Component)]
struct TouchBtnLabel(usize);

/// Literal labels (the interact / jump / tasks buttons are translated below).
const LABELS: [&str; 5] = ["●", "↑", "C", "≡", "…"];

fn spawn(mut commands: Commands, font: Res<UiFont>) {
    commands
        .spawn((TouchRoot, Node { width: percent(100), height: percent(100), position_type: PositionType::Absolute, display: Display::None, ..default() }, Pickable::IGNORE, GlobalZIndex(30)))
        .with_children(|r| {
            r.spawn((JoyBase, Node { position_type: PositionType::Absolute, width: px(JOY_RADIUS * 2.0), height: px(JOY_RADIUS * 2.0), border_radius: BorderRadius::MAX, border: UiRect::all(px(2)), ..default() }, BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)), BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.35)), Pickable::IGNORE));
            r.spawn((JoyKnob, Node { position_type: PositionType::Absolute, width: px(52), height: px(52), border_radius: BorderRadius::MAX, ..default() }, BackgroundColor(Color::srgba(1.0, 0.95, 0.8, 0.45)), Pickable::IGNORE));
            for i in 0..5 {
                r.spawn((
                    TouchBtn(i),
                    Node { position_type: PositionType::Absolute, border_radius: BorderRadius::MAX, border: UiRect::all(px(2)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, ..default() },
                    BackgroundColor(Color::srgba(0.06, 0.07, 0.18, 0.45)),
                    BorderColor::all(Color::srgba(1.0, 0.95, 0.8, 0.45)),
                    Pickable::IGNORE,
                ))
                .with_children(|b| {
                    b.spawn((TouchBtnLabel(i), Text::new(LABELS[i]), font.tf(if i == 0 { 20.0 } else { 16.0 }), TextColor(theme::WHITE), Pickable::IGNORE));
                });
            }
        });
}

#[allow(clippy::type_complexity)]
fn update(
    touch: Res<TouchControls>,
    state: Res<State<AppState>>,
    scale: Res<UiScale>,
    focus: Res<InteractFocus>,
    i18n: Res<I18n>,
    mut root: Query<&mut Node, (With<TouchRoot>, Without<JoyBase>, Without<JoyKnob>, Without<TouchBtn>)>,
    mut base: Query<&mut Node, (With<JoyBase>, Without<TouchRoot>, Without<JoyKnob>, Without<TouchBtn>)>,
    mut knob: Query<&mut Node, (With<JoyKnob>, Without<TouchRoot>, Without<JoyBase>, Without<TouchBtn>)>,
    mut btns: Query<(&TouchBtn, &mut Node, &mut BackgroundColor), (Without<TouchRoot>, Without<JoyBase>, Without<JoyKnob>)>,
    mut labels: Query<(&TouchBtnLabel, &mut Text)>,
) {
    let show = touch.enabled && *state.get() == AppState::Chapter;
    if let Ok(mut n) = root.single_mut() {
        n.display = if show { Display::Flex } else { Display::None };
    }
    if !show {
        return;
    }
    let s = scale.0.max(0.01);
    let joy_on = touch.joy_touch.is_some();
    let origin = if joy_on { touch.joy_origin } else { Vec2::new(130.0 * s, touch.screen.y - 130.0 * s) };
    let kpos = if joy_on { touch.joy_pos } else { origin };
    if let Ok(mut n) = base.single_mut() {
        n.left = px(origin.x / s - JOY_RADIUS);
        n.top = px(origin.y / s - JOY_RADIUS);
    }
    if let Ok(mut n) = knob.single_mut() {
        let d = (kpos - origin).clamp_length_max(JOY_RADIUS * s * 1.4);
        let p = origin + d;
        n.left = px(p.x / s - 26.0);
        n.top = px(p.y / s - 26.0);
    }
    let layout = TouchControls::buttons(touch.screen);
    for (b, mut n, mut bg) in &mut btns {
        let (c, r) = layout[b.0];
        n.left = px((c.x - r) / s);
        n.top = px((c.y - r) / s);
        n.width = px(r * 2.0 / s);
        n.height = px(r * 2.0 / s);
        let active = b.0 == 0 && focus.entity.is_some();
        bg.0 = if touch.held[b.0] {
            Color::srgba(0.95, 0.76, 0.35, 0.6)
        } else if active {
            Color::srgba(0.95, 0.76, 0.35, 0.35)
        } else {
            Color::srgba(0.06, 0.07, 0.18, 0.45)
        };
    }
    for (l, mut t) in &mut labels {
        let s = match l.0 {
            0 => focus.prompt.as_ref().map(|k| i18n.t(k).to_string()).unwrap_or_else(|| i18n.t("core.teach.interact").to_string()),
            1 => i18n.t("core.teach.jump").to_string(),
            4 => i18n.t("core.tab_hint").to_string(),
            _ => continue,
        };
        if t.0 != s {
            t.0 = s;
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn.after(super::super::boot_fonts)).add_systems(Update, update);
}
