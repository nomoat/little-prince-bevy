//! Pause menu + settings page (PauseView): resume, settings (language, volumes,
//! look sensitivity, invert Y, pointer lock, free travel, quality, touch
//! controls), journey map, title, fullscreen (web), clear progress.

use super::super::i18n::{I18n, Lang, Tr};
use super::super::input::InputFrame;
use super::super::save::{Quality, SaveData, TouchMode};
use super::super::state::{AppState, Flow};
use super::{UiAction, UiActionMsg, UiFont, button, small_button, theme};
use bevy::picking::Pickable;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MenuPage {
    #[default]
    Main,
    Settings,
}

#[derive(Resource, Default)]
pub struct MenuState {
    pub open: bool,
    pub page: MenuPage,
    pub reset_armed: bool,
    dirty: bool,
}

impl MenuState {
    pub fn open_settings(&mut self) {
        self.open = true;
        self.page = MenuPage::Settings;
        self.dirty = true;
    }
}

#[derive(Component)]
struct MenuRoot;

fn toggle_menu(
    input: Res<InputFrame>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut menu: ResMut<MenuState>,
    mut msgs: MessageReader<UiActionMsg>,
    mut flow: MessageWriter<Flow>,
    mut save: ResMut<SaveData>,
    mut time: ResMut<Time<Virtual>>,
) {
    let in_chapter = *state.get() == AppState::Chapter;
    let esc = input.menu || (menu.open && keys.just_pressed(KeyCode::Escape));
    if esc {
        if menu.open {
            if menu.page == MenuPage::Settings && in_chapter {
                menu.page = MenuPage::Main;
            } else {
                menu.open = false;
            }
        } else if in_chapter {
            menu.open = true;
            menu.page = MenuPage::Main;
        }
        menu.dirty = true;
        menu.reset_armed = false;
    }
    for UiActionMsg(a) in msgs.read() {
        menu.dirty = true;
        match a {
            UiAction::OpenMenu => {
                menu.open = true;
                menu.page = MenuPage::Main;
            }
            UiAction::Resume => menu.open = false,
            UiAction::OpenSettings => {
                menu.open = true;
                menu.page = MenuPage::Settings;
            }
            UiAction::CloseSettings => {
                if in_chapter {
                    menu.page = MenuPage::Main;
                } else {
                    menu.open = false;
                }
            }
            UiAction::ToMap => {
                menu.open = false;
                flow.write(Flow::ToMap);
            }
            UiAction::ToTitle => {
                menu.open = false;
                flow.write(Flow::ToTitle);
            }
            UiAction::ResetProgress => {
                if menu.reset_armed {
                    let settings = save.settings.clone();
                    let lang = save.lang;
                    *save = SaveData { settings, lang, ..default() };
                    super::super::save::clear_save_storage();
                    menu.open = false;
                    menu.reset_armed = false;
                    flow.write(Flow::ToTitle);
                } else {
                    menu.reset_armed = true;
                }
            }
            _ => {}
        }
    }
    if save.is_changed() {
        menu.dirty = true;
    }
    // pause game time while the menu is open in a chapter
    let want_pause = menu.open && in_chapter;
    if want_pause != time.is_paused() {
        if want_pause { time.pause() } else { time.unpause() }
    }
}

fn label(font: &UiFont, key: &str) -> impl Bundle {
    (Text::new(""), Tr::new(key), font.tf(20.0), TextColor(theme::WHITE), Node { min_width: px(230), ..default() }, Pickable::IGNORE)
}

fn row() -> Node {
    Node { column_gap: px(10), align_items: AlignItems::Center, ..default() }
}

fn choice_btn(font: &UiFont, key: &str, on: bool, action: UiAction) -> impl Bundle {
    if on {
        super::button_styled(font, key, 18.0, action, theme::BUTTON_PRESS, theme::BUTTON_PRESS)
    } else {
        super::button_styled(font, key, 18.0, action, theme::BUTTON, theme::BUTTON_HOVER)
    }
}

fn rebuild_menu(mut commands: Commands, mut menu: ResMut<MenuState>, font: Res<UiFont>, save: Res<SaveData>, i18n: Res<I18n>, state: Res<State<AppState>>, roots: Query<Entity, With<MenuRoot>>) {
    if !menu.dirty && !i18n.is_changed() {
        return;
    }
    menu.dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    if !menu.open {
        return;
    }
    let f = &*font;
    let s = &save.settings;
    let in_chapter = *state.get() == AppState::Chapter;
    commands
        .spawn((
            MenuRoot,
            Node { width: percent(100), height: percent(100), position_type: PositionType::Absolute, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.1, 0.62)),
            GlobalZIndex(50),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(12),
                    padding: UiRect::axes(px(44), px(30)),
                    border_radius: BorderRadius::all(px(24)),
                    border: UiRect::all(px(1)),
                    max_width: percent(94),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.07, 0.08, 0.2, 0.92)),
                BorderColor::all(Color::srgba(0.98, 0.88, 0.62, 0.35)),
            ))
            .with_children(|p| match menu.page {
                MenuPage::Main => {
                    p.spawn((Text::new(""), Tr::new("core.menu.paused"), f.tf(36.0), TextColor(theme::SOFT_GOLD), Node { margin: UiRect::bottom(px(10)), ..default() }));
                    p.spawn(button(f, "core.menu.resume", 22.0, UiAction::Resume));
                    p.spawn(button(f, "core.menu.settings", 22.0, UiAction::OpenSettings));
                    p.spawn(button(f, "core.menu.to_map", 22.0, UiAction::ToMap));
                    p.spawn(button(f, "core.menu.to_title", 22.0, UiAction::ToTitle));
                    if super::super::platform::is_web() {
                        p.spawn(button(f, "core.menu.fullscreen", 22.0, UiAction::Fullscreen));
                    }
                    p.spawn(button(f, if menu.reset_armed { "core.menu.reset_confirm" } else { "core.menu.reset" }, 18.0, UiAction::ResetProgress));
                    p.spawn((Text::new(""), Tr::new("core.menu.keys_text"), f.tf(15.0), TextColor(theme::DIM), TextLayout::justify(Justify::Center), Node { max_width: px(560), margin: UiRect::top(px(10)), ..default() }));
                }
                MenuPage::Settings => {
                    p.spawn((Text::new(""), Tr::new("core.menu.settings"), f.tf(32.0), TextColor(theme::SOFT_GOLD), Node { margin: UiRect::bottom(px(6)), ..default() }));
                    p.spawn(row()).with_children(|r| {
                        r.spawn(label(f, "core.menu.language"));
                        r.spawn(choice_btn(f, "lang.zh", save.lang == Lang::Zh, UiAction::SetLang(Lang::Zh)));
                        r.spawn(choice_btn(f, "lang.en", save.lang == Lang::En, UiAction::SetLang(Lang::En)));
                    });
                    for (i, key, v) in [(0u8, "core.menu.master", s.master), (1, "core.menu.music", s.music), (2, "core.menu.sfx", s.sfx), (3, "core.menu.sensitivity", s.look)] {
                        p.spawn(row()).with_children(|r| {
                            r.spawn(label(f, key));
                            let step = if i == 3 { 0.1 } else { 0.1 };
                            r.spawn(small_button(f, "−", UiAction::Adjust(i, -step)));
                            r.spawn((Text::new(format!("{:>3}%", (v * 100.0).round() as i32)), f.tf(20.0), TextColor(theme::WHITE), Node { min_width: px(70), justify_content: JustifyContent::Center, ..default() }));
                            r.spawn(small_button(f, "+", UiAction::Adjust(i, step)));
                        });
                    }
                    let onoff = |b: bool| if b { "core.menu.on" } else { "core.menu.off" };
                    for (key, v, a) in [
                        ("core.menu.invert_y", s.invert_y, UiAction::ToggleInvertY),
                        ("core.menu.pointer_lock", s.pointer_lock, UiAction::TogglePointerLock),
                        ("core.menu.free_travel", s.free_travel, UiAction::ToggleFreeTravel),
                    ] {
                        p.spawn(row()).with_children(|r| {
                            r.spawn(label(f, key));
                            r.spawn(choice_btn(f, onoff(v), v, a));
                        });
                    }
                    p.spawn(row()).with_children(|r| {
                        r.spawn(label(f, "core.menu.quality"));
                        for (k, q) in [("core.quality.low", Quality::Low), ("core.quality.medium", Quality::Medium), ("core.quality.high", Quality::High)] {
                            r.spawn(choice_btn(f, k, s.quality == q, UiAction::SetQuality(q)));
                        }
                    });
                    p.spawn(row()).with_children(|r| {
                        r.spawn(label(f, "core.menu.touch"));
                        for (k, t) in [("core.menu.auto", TouchMode::Auto), ("core.menu.on", TouchMode::On), ("core.menu.off", TouchMode::Off)] {
                            r.spawn(choice_btn(f, k, s.touch == t, UiAction::SetTouch(t)));
                        }
                    });
                    p.spawn(button(f, "core.menu.back", 22.0, UiAction::CloseSettings)).insert(Node {
                        margin: UiRect::top(px(10)),
                        padding: UiRect::axes(px(26), px(10)),
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(px(22)),
                        min_width: px(180),
                        ..default()
                    });
                    let _ = in_chapter;
                }
            });
        });
}

pub const MENU_STRINGS: super::super::i18n::StringTable = &[("lang.zh", "中文", "中文"), ("lang.en", "English", "English")];

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (toggle_menu, rebuild_menu).chain());
}
