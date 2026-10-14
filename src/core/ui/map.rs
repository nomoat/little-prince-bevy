//! The journey map (JourneyMapScreen): eight planets on a night-blue stage
//! (1672 × 941 stage px, scaled to fit), joined by a dotted route. Planet
//! states follow the original: Fog (not yet revealed), Lit (revealed, not yet
//! visited), Visited, Teaser (the next fogged planet beyond the frontier).
//! Free travel makes every planet selectable. Returning from a planet, the
//! prince (✦) flies along the route to the newly revealed stop.

use super::super::chapter::{ChapterInfo, ChapterRegistry};
use super::super::i18n::{I18n, Tr};
use super::super::meshkit::{Rng, mix_color};
use super::super::save::SaveData;
use super::super::state::{ActiveChapter, AppState, Flow};
use super::{UiAction, UiActionMsg, UiFont, button, theme};
use bevy::picking::Pickable;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use bevy::window::PrimaryWindow;

pub const STAGE: Vec2 = Vec2::new(1672.0, 941.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanetState {
    Fog,
    Lit,
    Visited,
    Teaser,
}

pub fn planet_state(save: &SaveData, reg: &ChapterRegistry, id: &str) -> PlanetState {
    if save.is_visited(id) {
        return PlanetState::Visited;
    }
    if save.is_revealed(id) {
        return PlanetState::Lit;
    }
    // Teaser: the first fogged planet after the furthest revealed one.
    let infos = reg.infos();
    let furthest = infos.iter().rposition(|i| save.is_revealed(i.id) || save.is_visited(i.id));
    let first_fog = match furthest {
        Some(f) => f + 1,
        None => 0,
    };
    if infos.get(first_fog).map(|i| i.id == id).unwrap_or(false) && first_fog > 0 {
        return PlanetState::Teaser;
    }
    PlanetState::Fog
}

#[derive(Resource, Default)]
pub struct MapState {
    pub selected: Option<String>,
    built_for: Vec2,
    dirty: bool,
    /// Bird flight from → to, progress 0..1.
    flight: Option<(Vec2, Vec2, f32)>,
}

#[derive(Component)]
struct MapRoot;
#[derive(Component)]
#[allow(dead_code)]
struct MapPlanet(String);
#[derive(Component)]
struct MapPanelTitle;
#[derive(Component)]
struct MapPanelSub;
#[derive(Component)]
struct MapPanelLine;
#[derive(Component)]
struct MapDepartBtn;
#[derive(Component)]
struct MapBird;
#[derive(Component)]
struct MapProgress;

fn selectable(save: &SaveData, st: PlanetState) -> bool {
    save.settings.free_travel || matches!(st, PlanetState::Lit | PlanetState::Visited)
}

fn enter_map(mut ms: ResMut<MapState>, mut save: ResMut<SaveData>, reg: Res<ChapterRegistry>, active: Res<ActiveChapter>, mut audio: MessageWriter<super::super::audio::AudioCmd>, params: Res<super::super::params::LaunchParams>) {
    ms.dirty = true;
    // The journey always starts with the first planet revealed.
    let first = reg.first_id();
    if !save.is_revealed(first) {
        save.reveal(first);
    }
    audio.write(super::super::audio::AudioCmd::music("music.map"));
    // default selection: the frontier (first lit, not visited) else last
    let infos = reg.infos();
    let frontier = infos.iter().find(|i| planet_state(&save, &reg, i.id) == PlanetState::Lit).map(|i| i.id.to_string());
    ms.selected = frontier.clone().or_else(|| infos.last().map(|i| i.id.to_string()));
    ms.flight = None;
    if let (Some(from), Some(to)) = (active.returning_from.as_ref(), frontier.as_ref()) {
        if let (Some(a), Some(b)) = (reg.info(from), reg.info(to)) {
            if !params.test {
                ms.flight = Some((a.map_pos, b.map_pos, 0.0));
            }
        }
    }
}

fn stage_scale(window: &Window, ui_scale: f32) -> (f32, Vec2) {
    let w = window.width() / ui_scale;
    let h = window.height() / ui_scale;
    let k = (w / STAGE.x).min(h / STAGE.y);
    let off = Vec2::new((w - STAGE.x * k) * 0.5, (h - STAGE.y * k) * 0.5);
    (k, off)
}

#[allow(clippy::too_many_arguments)]
fn build_map(
    mut commands: Commands,
    mut ms: ResMut<MapState>,
    font: Res<UiFont>,
    save: Res<SaveData>,
    reg: Res<ChapterRegistry>,
    i18n: Res<I18n>,
    window: Query<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    roots: Query<Entity, With<MapRoot>>,
) {
    let Ok(win) = window.single() else { return };
    let size = Vec2::new(win.width(), win.height());
    if !ms.dirty && ms.built_for == size && !save.is_changed() && !i18n.is_changed() {
        return;
    }
    ms.dirty = false;
    ms.built_for = size;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let (k, off) = stage_scale(win, scale.0);
    let f = &*font;
    let infos = reg.infos();
    let stage = |p: Vec2| off + p * k;
    let visited = infos.iter().filter(|i| save.is_visited(i.id)).count();
    commands
        .spawn((
            MapRoot,
            DespawnOnExit(AppState::Map),
            Node { width: percent(100), height: percent(100), position_type: PositionType::Absolute, ..default() },
            BackgroundColor(Color::srgb(0.04, 0.06, 0.16)),
            GlobalZIndex(20),
        ))
        .with_children(|root| {
            // stars
            let mut rng = Rng::new(1672);
            for _ in 0..160 {
                let p = Vec2::new(rng.f() * size.x / scale.0, rng.f() * size.y / scale.0);
                let s = rng.range(1.0, 3.2);
                let a = rng.range(0.2, 0.85);
                root.spawn((Node { position_type: PositionType::Absolute, left: px(p.x), top: px(p.y), width: px(s), height: px(s), border_radius: BorderRadius::MAX, ..default() }, BackgroundColor(Color::srgba(1.0, 0.97, 0.88, a)), Pickable::IGNORE));
            }
            // route dots
            for w in infos.windows(2) {
                let (a, b) = (w[0].map_pos, w[1].map_pos);
                let lit = save.is_revealed(w[1].id) || save.settings.free_travel;
                let dist = a.distance(b);
                let n = (dist / 26.0) as i32;
                for i in 1..n {
                    let t = i as f32 / n as f32;
                    // gentle arc
                    let mid = (a + b) * 0.5 + Vec2::new(-(b - a).y, (b - a).x).normalize_or_zero() * 40.0;
                    let p = a.lerp(mid, t).lerp(mid.lerp(b, t), t);
                    let ra = w[0].map_radius * 1.15;
                    let rb = w[1].map_radius * 1.15;
                    if p.distance(a) < ra || p.distance(b) < rb {
                        continue;
                    }
                    let q = stage(p);
                    let c = if lit { theme::SOFT_GOLD.with_alpha(0.8) } else { Color::srgba(0.8, 0.85, 1.0, 0.22) };
                    root.spawn((Node { position_type: PositionType::Absolute, left: px(q.x - 2.5), top: px(q.y - 2.5), width: px(5), height: px(5), border_radius: BorderRadius::MAX, ..default() }, BackgroundColor(c), Pickable::IGNORE));
                }
            }
            // planets
            for info in &infos {
                let st = planet_state(&save, &reg, info.id);
                let c = stage(info.map_pos);
                let r = info.map_radius * k;
                let (body, accent) = info.map_colors;
                let (bg, border, alpha) = match st {
                    PlanetState::Fog => (mix_color(body, Color::srgb(0.5, 0.52, 0.62), 0.85), Color::srgba(1.0, 1.0, 1.0, 0.1), 0.35),
                    PlanetState::Teaser => (mix_color(body, Color::srgb(0.6, 0.62, 0.72), 0.6), Color::srgba(1.0, 1.0, 1.0, 0.25), 0.6),
                    PlanetState::Lit => (body, accent, 1.0),
                    PlanetState::Visited => (body, theme::GOLD, 1.0),
                };
                let can = selectable(&save, st);
                let sel = ms.selected.as_deref() == Some(info.id);
                let mut e = root.spawn((
                    MapPlanet(info.id.to_string()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(c.x - r),
                        top: px(c.y - r),
                        width: px(r * 2.0),
                        height: px(r * 2.0),
                        border_radius: BorderRadius::MAX,
                        border: UiRect::all(px(if sel { 4.0 } else { 2.0 })),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(bg.with_alpha(alpha)),
                    BorderColor::all(if sel { theme::WHITE } else { border }),
                    BoxShadow::new(accent.with_alpha(if st == PlanetState::Lit { 0.55 } else { 0.0 }), Val::Px(0.0), Val::Px(0.0), Val::Px(r * 0.2), Val::Px(r * 0.6)),
                ));
                if can {
                    e.insert((Button, Hovered::default(), UiAction::MapSelect(info.id.to_string())));
                }
                e.with_children(|pl| {
                    let label = match st {
                        PlanetState::Visited => "✓".to_string(),
                        PlanetState::Fog => "?".to_string(),
                        _ => format!("{}", info.number),
                    };
                    pl.spawn((Text::new(label), f.tf((r * 0.55).clamp(12.0, 40.0)), TextColor(theme::WHITE.with_alpha(alpha)), Pickable::IGNORE));
                });
                // name under the planet
                let name = if matches!(st, PlanetState::Fog) && !save.settings.free_travel { String::new() } else { i18n.t(info.title).to_string() };
                root.spawn((
                    Text::new(name),
                    f.tf(16.0),
                    TextColor(theme::WHITE.with_alpha(alpha * 0.9)),
                    TextLayout::justify(Justify::Center),
                    Node { position_type: PositionType::Absolute, left: px(c.x - 120.0), width: px(240.0), top: px(c.y + r + 6.0), justify_content: JustifyContent::Center, ..default() },
                    Pickable::IGNORE,
                ));
            }
            // header
            root.spawn((Node { position_type: PositionType::Absolute, left: px(34), top: px(22), flex_direction: FlexDirection::Column, row_gap: px(4), ..default() }, Pickable::IGNORE)).with_children(|h| {
                h.spawn((Text::new(""), Tr::new("map.title"), f.tf(36.0), TextColor(theme::SOFT_GOLD), Pickable::IGNORE));
                h.spawn((MapProgress, Text::new(i18n.tf("map.progress", &[("n", &visited.to_string()), ("total", &infos.len().to_string())])), f.tf(17.0), TextColor(theme::DIM), Pickable::IGNORE));
                if save.settings.free_travel {
                    h.spawn((Text::new(""), Tr::new("map.free"), f.tf(15.0), TextColor(theme::GOLD), Pickable::IGNORE));
                }
            });
            // top-right buttons
            root.spawn(Node { position_type: PositionType::Absolute, right: px(24), top: px(22), column_gap: px(10), ..default() }).with_children(|r| {
                r.spawn(button(f, "core.title.settings", 18.0, UiAction::OpenSettings));
                r.spawn(button(f, "map.back", 18.0, UiAction::ToTitle));
            });
            // bottom panel (selected planet)
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(20),
                    left: percent(50),
                    margin: UiRect::left(px(-380)),
                    width: px(760),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(6),
                    padding: UiRect::axes(px(28), px(16)),
                    border_radius: BorderRadius::all(px(20)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.07, 0.08, 0.2, 0.82)),
            ))
            .with_children(|p| {
                p.spawn((MapPanelTitle, Text::new(""), f.tf(28.0), TextColor(theme::WHITE), Pickable::IGNORE));
                p.spawn((MapPanelSub, Text::new(""), f.tf(18.0), TextColor(theme::SOFT_GOLD), Pickable::IGNORE));
                p.spawn((MapPanelLine, Text::new(""), f.tf(17.0), TextColor(theme::DIM), TextLayout::justify(Justify::Center), Pickable::IGNORE));
                p.spawn((MapDepartBtn, button(f, "map.depart", 22.0, UiAction::MapDepart)));
                p.spawn((Text::new(""), Tr::new("map.hint"), f.tf(14.0), TextColor(theme::DIM), Pickable::IGNORE));
            });
            // the prince flying with the birds
            root.spawn((MapBird, Text::new("✦"), f.tf(30.0), TextColor(theme::GOLD), Node { position_type: PositionType::Absolute, display: Display::None, ..default() }, Pickable::IGNORE));
        });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn map_update(
    mut ms: ResMut<MapState>,
    save: Res<SaveData>,
    reg: Res<ChapterRegistry>,
    i18n: Res<I18n>,
    keys: Res<ButtonInput<KeyCode>>,
    mut msgs: MessageReader<UiActionMsg>,
    mut flow: MessageWriter<Flow>,
    menu: Res<super::menu::MenuState>,
    time: Res<Time<Real>>,
    window: Query<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    mut texts: ParamSet<(Query<&mut Text, With<MapPanelTitle>>, Query<&mut Text, With<MapPanelSub>>, Query<&mut Text, With<MapPanelLine>>)>,
    mut btn: Query<&mut Node, (With<MapDepartBtn>, Without<MapBird>)>,
    mut bird: Query<&mut Node, (With<MapBird>, Without<MapDepartBtn>)>,
) {
    let infos: Vec<ChapterInfo> = reg.infos();
    let selectable_ids: Vec<&str> = infos.iter().filter(|i| selectable(&save, planet_state(&save, &reg, i.id))).map(|i| i.id).collect();
    for UiActionMsg(a) in msgs.read() {
        match a {
            UiAction::MapSelect(id) => {
                if ms.selected.as_deref() == Some(id.as_str()) {
                    // A second click starts the planet, but only if it is open.
                    if selectable_ids.contains(&id.as_str()) {
                        flow.write(Flow::Enter(id.clone()));
                    }
                } else {
                    ms.selected = Some(id.clone());
                    ms.dirty = true;
                }
            }
            UiAction::MapDepart => {
                if let Some(id) = ms.selected.clone() {
                    if selectable_ids.contains(&id.as_str()) {
                        flow.write(Flow::Enter(id));
                    }
                }
            }
            _ => {}
        }
    }
    if !menu.open {
        let cur = ms.selected.as_deref().and_then(|s| selectable_ids.iter().position(|x| *x == s));
        if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
            if let Some(i) = cur.map(|i| (i + 1) % selectable_ids.len().max(1)).or(Some(0)) {
                ms.selected = selectable_ids.get(i).map(|s| s.to_string());
                ms.dirty = true;
            }
        }
        if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
            let n = selectable_ids.len().max(1);
            if let Some(i) = cur.map(|i| (i + n - 1) % n).or(Some(0)) {
                ms.selected = selectable_ids.get(i).map(|s| s.to_string());
                ms.dirty = true;
            }
        }
        if keys.just_pressed(KeyCode::Enter) {
            if let Some(id) = ms.selected.clone() {
                if selectable_ids.contains(&id.as_str()) {
                    flow.write(Flow::Enter(id));
                }
            }
        }
    }
    // panel text
    let sel = ms.selected.clone().and_then(|id| reg.info(&id));
    let (title, sub, line, can) = match &sel {
        Some(info) => {
            let st = planet_state(&save, &reg, info.id);
            let can = selectable(&save, st);
            let line = match st {
                PlanetState::Visited => i18n.t(info.visited_line).to_string(),
                PlanetState::Fog if !save.settings.free_travel => i18n.t("map.fog").to_string(),
                PlanetState::Teaser if !save.settings.free_travel => i18n.t("map.teaser").to_string(),
                _ => i18n.t(info.line).to_string(),
            };
            (
                format!("{} · {}", i18n.tf("core.chapter_n", &[("n", &info.number.to_string())]), i18n.t(info.title)),
                i18n.t(info.subtitle).to_string(),
                line,
                can,
            )
        }
        None => (String::new(), String::new(), String::new(), false),
    };
    if let Ok(mut t) = texts.p0().single_mut() {
        if t.0 != title {
            t.0 = title;
        }
    }
    if let Ok(mut t) = texts.p1().single_mut() {
        if t.0 != sub {
            t.0 = sub;
        }
    }
    if let Ok(mut t) = texts.p2().single_mut() {
        if t.0 != line {
            t.0 = line;
        }
    }
    if let Ok(mut n) = btn.single_mut() {
        n.display = if can { Display::Flex } else { Display::None };
    }
    // bird flight
    if let (Some((a, b, t)), Ok(win)) = (ms.flight, window.single()) {
        let t2 = (t + time.delta_secs() / 2.6).min(1.0);
        ms.flight = if t2 >= 1.0 { None } else { Some((a, b, t2)) };
        let (k, off) = stage_scale(win, scale.0);
        let mid = (a + b) * 0.5 + Vec2::new(-(b - a).y, (b - a).x).normalize_or_zero() * 40.0;
        let e = t2 * t2 * (3.0 - 2.0 * t2);
        let p = off + a.lerp(mid, e).lerp(mid.lerp(b, e), e) * k;
        if let Ok(mut n) = bird.single_mut() {
            n.display = if ms.flight.is_some() { Display::Flex } else { Display::None };
            n.left = px(p.x - 12.0);
            n.top = px(p.y - 34.0 - (t2 * std::f32::consts::PI).sin() * 30.0);
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<MapState>()
        .add_systems(OnEnter(AppState::Map), enter_map)
        .add_systems(Update, (build_map, map_update).chain().run_if(in_state(AppState::Map)));
}
