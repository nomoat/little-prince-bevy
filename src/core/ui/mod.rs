//! UI: theme, fonts, buttons and the screens (title, HUD, pause/settings,
//! journey map, ending, touch controls). Everything is Bevy UI built in code.

pub mod ending;
pub mod hud;
pub mod map;
pub mod menu;
pub mod title;
pub mod touch;

use super::i18n::{I18n, Lang, Tr};
use super::input::PointerOverUi;
use super::save::{Quality, TouchMode};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};
use bevy::window::PrimaryWindow;

pub mod theme {
    use bevy::prelude::Color;
    pub const INK: Color = Color::srgb(0.17, 0.15, 0.2);
    pub const PAPER: Color = Color::srgba(0.98, 0.95, 0.88, 0.94);
    pub const PAPER_SOLID: Color = Color::srgb(0.98, 0.95, 0.88);
    pub const GOLD: Color = Color::srgb(0.93, 0.76, 0.35);
    pub const SOFT_GOLD: Color = Color::srgb(0.98, 0.88, 0.62);
    pub const NIGHT: Color = Color::srgb(0.05, 0.07, 0.17);
    pub const WHITE: Color = Color::srgb(0.99, 0.98, 0.95);
    pub const DIM: Color = Color::srgba(0.99, 0.98, 0.95, 0.65);
    pub const SHADOW: Color = Color::srgba(0.0, 0.0, 0.05, 0.55);
    pub const BUTTON: Color = Color::srgba(0.12, 0.13, 0.25, 0.72);
    pub const BUTTON_HOVER: Color = Color::srgba(0.22, 0.22, 0.4, 0.86);
    pub const BUTTON_PRESS: Color = Color::srgba(0.42, 0.34, 0.2, 0.92);
}

/// The UI font (CJK-capable; manifest key `font.ui`).
#[derive(Resource, Clone, Default)]
pub struct UiFont(pub Handle<Font>);

impl UiFont {
    pub fn tf(&self, size: f32) -> TextFont {
        TextFont { font: self.0.clone().into(), font_size: FontSize::Px(size), ..default() }
    }
}

/// Everything a UI button can do; handled centrally.
#[derive(Component, Clone, Debug, PartialEq)]
pub enum UiAction {
    Resume,
    OpenSettings,
    CloseSettings,
    ToMap,
    ToTitle,
    Fullscreen,
    ResetProgress,
    SetLang(Lang),
    ToggleInvertY,
    TogglePointerLock,
    ToggleFreeTravel,
    SetQuality(Quality),
    SetTouch(TouchMode),
    /// 0 = master, 1 = music, 2 = sfx, 3 = look sensitivity; delta.
    Adjust(u8, f32),
    TitleBegin,
    TitleContinue,
    MapSelect(String),
    MapDepart,
    EndingBack,
    Choice(usize),
    OpenMenu,
}

/// Fired when a [`UiAction`] button is activated.
#[derive(Message, Clone, Debug)]
pub struct UiActionMsg(pub UiAction);

fn on_activate(ev: On<Activate>, q: Query<&UiAction>, mut out: MessageWriter<UiActionMsg>, mut audio: MessageWriter<super::audio::AudioCmd>) {
    if let Ok(a) = q.get(ev.entity) {
        out.write(UiActionMsg(a.clone()));
        audio.write(super::audio::AudioCmd::Sfx("sfx.click".into(), 0.6));
    }
}

/// Marks a button whose look follows hover/press.
#[derive(Component)]
pub struct StyledButton {
    pub normal: Color,
    pub hover: Color,
}

/// A text button bundle (spawn with `children![…]` or `with_children`).
pub fn button(font: &UiFont, key: &str, size: f32, action: UiAction) -> impl Bundle {
    button_styled(font, key, size, action, theme::BUTTON, theme::BUTTON_HOVER)
}

/// [`button`] with explicit normal / hover colours.
pub fn button_styled(font: &UiFont, key: &str, size: f32, action: UiAction, normal: Color, hover: Color) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        action,
        StyledButton { normal, hover },
        Node {
            padding: UiRect::axes(px(26), px(10)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(22)),
            min_width: px(180),
            ..default()
        },
        BorderColor::all(Color::srgba(0.98, 0.88, 0.62, 0.5)),
        BackgroundColor(normal),
        children![(Text::new(""), Tr::new(key), font.tf(size), TextColor(theme::WHITE), bevy::picking::Pickable::IGNORE)],
    )
}

/// A small square-ish button with literal text (− / +).
pub fn small_button(font: &UiFont, label: &str, action: UiAction) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        action,
        StyledButton { normal: theme::BUTTON, hover: theme::BUTTON_HOVER },
        Node {
            width: px(40),
            height: px(36),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(theme::BUTTON),
        children![(Text::new(label), font.tf(20.0), TextColor(theme::WHITE), bevy::picking::Pickable::IGNORE)],
    )
}

fn style_buttons(mut q: Query<(&Hovered, Has<bevy::ui::Pressed>, &StyledButton, &mut BackgroundColor)>) {
    for (h, pressed, s, mut bg) in &mut q {
        bg.0 = if pressed {
            theme::BUTTON_PRESS
        } else if h.get() {
            s.hover
        } else {
            s.normal
        };
    }
}

fn pointer_over_ui(q: Query<&Hovered, With<Button>>, mut over: ResMut<PointerOverUi>) {
    over.0 = q.iter().any(|h| h.get());
}

/// Scales the whole UI to the window (1280×720 reference), phone friendly.
fn ui_scale(window: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    if let Ok(w) = window.single() {
        let s = (w.width() / 1280.0).min(w.height() / 720.0).clamp(0.55, 1.6);
        if (scale.0 - s).abs() > 0.01 {
            scale.0 = s;
        }
    }
}

/// Applies settings/menu actions that are not screen-specific.
fn apply_common_actions(
    mut msgs: MessageReader<UiActionMsg>,
    mut save: ResMut<super::save::SaveData>,
    mut i18n: ResMut<I18n>,
) {
    for UiActionMsg(a) in msgs.read() {
        let s = &mut save.settings;
        match a {
            UiAction::SetLang(l) => {
                save.lang = *l;
                i18n.lang = *l;
            }
            UiAction::ToggleInvertY => s.invert_y = !s.invert_y,
            UiAction::TogglePointerLock => s.pointer_lock = !s.pointer_lock,
            UiAction::ToggleFreeTravel => s.free_travel = !s.free_travel,
            UiAction::SetQuality(q) => s.quality = *q,
            UiAction::SetTouch(t) => s.touch = *t,
            UiAction::Adjust(which, d) => {
                let v = match which {
                    0 => &mut s.master,
                    1 => &mut s.music,
                    2 => &mut s.sfx,
                    _ => &mut s.look,
                };
                let max = if *which == 3 { 3.0 } else { 1.0 };
                let min = if *which == 3 { 0.2 } else { 0.0 };
                *v = ((*v + d) * 100.0).round() / 100.0;
                *v = v.clamp(min, max);
            }
            UiAction::Fullscreen => super::platform::toggle_fullscreen(),
            _ => {}
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<UiActionMsg>()
        .init_resource::<UiFont>()
        .init_resource::<hud::Hud>()
        .init_resource::<menu::MenuState>()
        .add_observer(on_activate)
        .add_systems(Update, (ui_scale, style_buttons, pointer_over_ui, apply_common_actions))
        .add_plugins((hud::plugin, menu::plugin, title::plugin, map::plugin, ending::plugin, touch::plugin));
}
