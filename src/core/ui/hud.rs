//! In-chapter HUD (GameUI in the original): chapter card, objective (event +
//! step), "done → next" banner, dialogue with typewriter reveal and speaker
//! badge, choices, monologue, tutorial key hints, the anchored interaction
//! prompt with hold ring, goal marker, hidden-goal row, departure hold, the
//! Tab task view, and the ink fade used for transitions.

use super::super::camera::MainCamera;
use super::super::i18n::I18n;
use super::super::input::{InputFrame, TouchControls};
use super::super::interact::InteractFocus;
use super::super::state::AppState;
use super::super::story::StoryRunner;
use super::{UiAction, UiFont, theme};
use bevy::picking::Pickable;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

/// A dialogue speaker: i18n name key + badge colour.
#[derive(Clone, Copy, Debug)]
pub struct Speaker {
    pub name: &'static str,
    pub color: Color,
}

impl Speaker {
    pub const fn new(name: &'static str, color: Color) -> Self {
        Speaker { name, color }
    }
}

/// The original's cast (chapters may define more with `Speaker::new`).
pub mod speakers {
    use super::Speaker;
    use bevy::prelude::Color;
    pub const PRINCE: Speaker = Speaker::new("core.prince", Color::srgb(0.95, 0.76, 0.3));
    pub const NARRATOR: Speaker = Speaker::new("core.narrator", Color::srgb(0.7, 0.7, 0.8));
    pub const ROSE: Speaker = Speaker::new("speaker.rose", Color::srgb(0.88, 0.32, 0.36));
    pub const KING: Speaker = Speaker::new("speaker.king", Color::srgb(0.55, 0.38, 0.75));
    pub const VAIN_MAN: Speaker = Speaker::new("speaker.vain", Color::srgb(0.85, 0.45, 0.65));
    pub const DRINKER: Speaker = Speaker::new("speaker.drinker", Color::srgb(0.45, 0.55, 0.7));
    pub const MERCHANT: Speaker = Speaker::new("speaker.merchant", Color::srgb(0.6, 0.62, 0.68));
    pub const LAMPLIGHTER: Speaker = Speaker::new("speaker.lamplighter", Color::srgb(0.96, 0.62, 0.25));
    pub const GEOGRAPHER: Speaker = Speaker::new("speaker.geographer", Color::srgb(0.42, 0.62, 0.45));
}

pub const SPEAKER_STRINGS: super::super::i18n::StringTable = &[
    ("speaker.rose", "玫瑰", "The Rose"),
    ("speaker.king", "国王", "The King"),
    ("speaker.vain", "爱虚荣的人", "The Conceited Man"),
    ("speaker.drinker", "酒鬼", "The Tippler"),
    ("speaker.merchant", "商人", "The Businessman"),
    ("speaker.lamplighter", "点灯人", "The Lamplighter"),
    ("speaker.geographer", "地理学家", "The Geographer"),
];

#[derive(Clone, Debug)]
pub struct Timed<T> {
    pub v: T,
    pub t: f32,
    pub hold: f32,
}

#[derive(Clone, Debug)]
pub struct DialogueState {
    pub speaker: Speaker,
    pub key: String,
    pub t: f32,
    pub revealed: bool,
    pub needs_key: bool,
    pub auto: Option<f32>,
    pub finished: bool,
    pub after_reveal: f32,
}

/// HUD state written by the story runner / chapters, drawn by the HUD systems.
#[derive(Resource, Default)]
pub struct Hud {
    pub card: Option<Timed<(u32, &'static str, &'static str)>>,
    pub narration: Option<(Vec<&'static str>, usize, f32, f32)>,
    pub objective: Option<(Option<&'static str>, Option<&'static str>, f32)>,
    pub banner: Option<Timed<(&'static str, Option<&'static str>)>>,
    pub dialogue: Option<DialogueState>,
    pub choice: Option<(Vec<&'static str>, Option<usize>, usize)>,
    choice_spawned: bool,
    pub mono: Option<Timed<&'static str>>,
    pub teach: Option<Timed<(Vec<&'static str>, &'static str)>>,
    pub hidden: Option<(&'static str, bool)>,
    pub hidden_flash: f32,
    pub marker_flash: f32,
    pub depart_line: bool,
    pub depart_progress: f32,
    /// Free-form toast (literal text) with seconds left.
    pub toast: Option<(String, f32)>,
    /// When true the HUD is hidden (cinematics).
    pub hide_all: bool,
}

impl Hud {
    pub fn reset(&mut self) {
        *self = Hud::default();
    }
    pub fn show_card(&mut self, number: u32, title: &'static str, subtitle: &'static str, hold: f32) {
        self.card = Some(Timed { v: (number, title, subtitle), t: 0.0, hold });
    }
    pub fn card_active(&self) -> bool {
        self.card.is_some()
    }
    pub fn show_narration(&mut self, keys: Vec<&'static str>, per: f32) {
        self.narration = Some((keys, 0, 0.0, per));
    }
    pub fn narration_active(&self) -> bool {
        self.narration.is_some()
    }
    pub fn show_objective(&mut self, event: Option<&'static str>, step: Option<&'static str>, _flash: bool) {
        self.objective = Some((event, step, 0.0));
        self.depart_line = false;
    }
    pub fn hide_objective(&mut self) {
        self.objective = None;
    }
    pub fn show_banner(&mut self, done: &'static str, next: Option<&'static str>, speed: f32) {
        // In 0.6 s, hold 2.2 s, out 0.8 s (BannerView), scaled by speed.
        self.banner = Some(Timed { v: (done, next), t: 0.0, hold: 3.6 * speed });
    }
    pub fn banner_active(&self) -> bool {
        self.banner.is_some()
    }
    pub fn say(&mut self, speaker: Speaker, key: &str, needs_key: bool, auto: Option<f32>) {
        self.dialogue = Some(DialogueState { speaker, key: key.to_string(), t: 0.0, revealed: false, needs_key, auto, finished: false, after_reveal: 0.0 });
    }
    pub fn dialogue_finished(&self) -> bool {
        self.dialogue.as_ref().map(|d| d.finished).unwrap_or(true)
    }
    pub fn close_dialogue(&mut self) {
        self.dialogue = None;
    }
    pub fn open_choice(&mut self, keys: Vec<&'static str>) {
        self.choice = Some((keys, None, 0));
        self.choice_spawned = false;
    }
    pub fn choice_result(&self) -> Option<usize> {
        self.choice.as_ref().and_then(|c| c.1)
    }
    pub fn close_choice(&mut self) {
        self.choice = None;
        self.dialogue = None;
    }
    pub fn monologue(&mut self, key: &'static str, secs: f32) {
        self.mono = Some(Timed { v: key, t: 0.0, hold: secs });
    }
    pub fn teach(&mut self, keys: Vec<&'static str>, text: &'static str, secs: f32) {
        self.teach = Some(Timed { v: (keys, text), t: 0.0, hold: secs });
    }
    pub fn hidden_found(&mut self, key: &'static str) {
        self.hidden = Some((key, true));
        self.hidden_flash = 4.0;
    }
    pub fn flash_marker(&mut self, secs: f32) {
        self.marker_flash = secs;
    }
    pub fn toast(&mut self, text: impl Into<String>, secs: f32) {
        self.toast = Some((text.into(), secs));
    }
    /// Movement is locked while a keyed dialogue, a choice or the card is up.
    pub fn blocks_movement(&self) -> bool {
        self.card.is_some() || self.choice.is_some() || self.dialogue.as_ref().map(|d| d.needs_key).unwrap_or(false)
    }
}

/// Fade overlay for transitions (alpha 0..1) with an optional line of text.
#[derive(Resource, Default)]
pub struct Fade {
    pub alpha: f32,
    pub target: f32,
    pub speed: f32,
    pub text: Option<String>,
    pub sub: Option<String>,
}

impl Fade {
    pub fn to(&mut self, target: f32, seconds: f32) {
        self.target = target;
        self.speed = 1.0 / seconds.max(0.01);
    }
    pub fn done(&self) -> bool {
        (self.alpha - self.target).abs() < 1e-3
    }
}

// ------------------------------------------------------------- markers ----
/// Which HUD text an entity shows.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum HudSlot {
    ObjEvent,
    ObjStep,
    HiddenRow,
    BannerTitle,
    BannerNext,
    CardNum,
    CardTitle,
    CardSub,
    NarrText,
    DlgName,
    DlgText,
    MonoText,
    TeachText,
    TasksText,
    ToastText,
    DepartText,
}
#[derive(Component)]
struct HudRoot;
#[derive(Component)]
struct ObjBox;
#[derive(Component)]
struct BannerBox;
#[derive(Component)]
struct CardBox;
#[derive(Component)]
struct DlgBox;
#[derive(Component)]
struct DlgBadge;
#[derive(Component)]
struct DlgMore;
#[derive(Component)]
struct ChoiceBox;
#[derive(Component)]
struct TeachBox;
#[derive(Component)]
struct PromptBox;
#[derive(Component)]
struct PromptKey;
#[derive(Component)]
struct PromptVerb;
#[derive(Component)]
struct PromptBar;
#[derive(Component)]
struct MarkerBox;
#[derive(Component)]
struct MarkerLabel;
#[derive(Component)]
struct DepartBox;
#[derive(Component)]
struct DepartBar;
#[derive(Component)]
struct FadeBox;
#[derive(Component)]
struct FadeTitle;
#[derive(Component)]
struct FadeSub;
#[derive(Component)]
struct TasksBox;

fn abs_node() -> Node {
    Node { position_type: PositionType::Absolute, ..default() }
}

fn text_bundle(font: &UiFont, size: f32, color: Color) -> impl Bundle {
    (Text::new(""), font.tf(size), TextColor(color), TextShadow { offset: Vec2::new(1.5, 1.5), color: theme::SHADOW }, Pickable::IGNORE)
}

fn spawn_hud(mut commands: Commands, font: Res<UiFont>) {
    let f = &*font;
    commands
        .spawn((HudRoot, Name::new("HUD"), Node { width: percent(100), height: percent(100), position_type: PositionType::Absolute, ..default() }, Pickable::IGNORE, GlobalZIndex(10)))
        .with_children(|root| {
            // Objective (top-left)
            // (a soft dark backdrop keeps it readable over white walls and bright sky)
            root.spawn((ObjBox, Node { left: px(22), top: px(18), flex_direction: FlexDirection::Column, row_gap: px(4), max_width: px(560), padding: UiRect::axes(px(14), px(9)), border_radius: BorderRadius::all(px(12)), ..abs_node() }, BackgroundColor(Color::srgba(0.03, 0.04, 0.12, 0.38)), Pickable::IGNORE))
                .with_children(|b| {
                    b.spawn((HudSlot::ObjEvent, text_bundle(f, 17.0, theme::SOFT_GOLD)));
                    b.spawn((HudSlot::ObjStep, text_bundle(f, 24.0, theme::WHITE)));
                    b.spawn((HudSlot::HiddenRow, text_bundle(f, 17.0, theme::DIM), Node { margin: UiRect::top(px(8)), ..default() }));
                });
            // Banner (top centre)
            root.spawn((BannerBox, Node { top: px(84), width: percent(100), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(6), ..abs_node() }, Pickable::IGNORE))
                .with_children(|b| {
                    b.spawn((HudSlot::BannerTitle, text_bundle(f, 30.0, theme::SOFT_GOLD)));
                    b.spawn((HudSlot::BannerNext, text_bundle(f, 19.0, theme::WHITE)));
                });
            // Chapter card (centre)
            root.spawn((CardBox, Node { width: percent(100), height: percent(100), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: px(12), ..abs_node() }, Pickable::IGNORE))
                .with_children(|b| {
                    b.spawn((HudSlot::CardNum, text_bundle(f, 22.0, theme::SOFT_GOLD)));
                    b.spawn((HudSlot::CardTitle, text_bundle(f, 60.0, theme::WHITE)));
                    b.spawn((HudSlot::CardSub, text_bundle(f, 24.0, theme::WHITE)));
                    b.spawn((HudSlot::NarrText, text_bundle(f, 24.0, theme::WHITE), TextLayout::justify(Justify::Center), Node { max_width: px(900), margin: UiRect::top(px(28)), ..default() }));
                });
            // Monologue
            root.spawn((Node { top: percent(60), width: percent(100), justify_content: JustifyContent::Center, ..abs_node() }, Pickable::IGNORE)).with_children(|b| {
                b.spawn((HudSlot::MonoText, text_bundle(f, 26.0, theme::WHITE), TextLayout::justify(Justify::Center), Node { max_width: px(900), ..default() }));
            });
            // Toast
            root.spawn((Node { top: percent(18), width: percent(100), justify_content: JustifyContent::Center, ..abs_node() }, Pickable::IGNORE)).with_children(|b| {
                b.spawn((HudSlot::ToastText, text_bundle(f, 22.0, theme::SOFT_GOLD), TextLayout::justify(Justify::Center)));
            });
            // Tutorial (bottom centre, above dialogue)
            root.spawn((TeachBox, Node { bottom: px(250), width: percent(100), justify_content: JustifyContent::Center, ..abs_node() }, Pickable::IGNORE)).with_children(|b| {
                b.spawn((
                    Node { padding: UiRect::axes(px(22), px(10)), border_radius: BorderRadius::all(px(18)), ..default() },
                    BackgroundColor(Color::srgba(0.05, 0.06, 0.15, 0.55)),
                    Pickable::IGNORE,
                ))
                .with_children(|c| {
                    c.spawn((HudSlot::TeachText, text_bundle(f, 21.0, theme::WHITE)));
                });
            });
            // Dialogue (bottom)
            root.spawn((DlgBox, Node { bottom: px(40), width: percent(100), justify_content: JustifyContent::Center, ..abs_node() }, Pickable::IGNORE)).with_children(|b| {
                b.spawn((
                    Node {
                        width: percent(88),
                        max_width: px(1180),
                        min_height: px(150),
                        padding: UiRect { left: px(56), right: px(56), top: px(46), bottom: px(30) },
                        flex_direction: FlexDirection::Column,
                        border_radius: BorderRadius::all(px(20)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.07, 0.16, 0.8)),
                    BorderColor::all(Color::srgba(0.98, 0.88, 0.62, 0.35)),
                    Pickable::IGNORE,
                ))
                .with_children(|p| {
                    // name plate
                    p.spawn((Node { left: px(40), top: px(-20), padding: UiRect::axes(px(18), px(6)), column_gap: px(10), align_items: AlignItems::Center, border_radius: BorderRadius::all(px(16)), ..abs_node() }, BackgroundColor(theme::PAPER_SOLID), Pickable::IGNORE))
                        .with_children(|n| {
                            n.spawn((DlgBadge, Node { width: px(14), height: px(14), border_radius: BorderRadius::MAX, ..default() }, BackgroundColor(theme::GOLD), Pickable::IGNORE));
                            n.spawn((HudSlot::DlgName, Text::new(""), f.tf(19.0), TextColor(theme::INK), Pickable::IGNORE));
                        });
                    p.spawn((HudSlot::DlgText, text_bundle(f, 26.0, theme::WHITE), Node { min_height: px(70), ..default() }));
                    p.spawn((DlgMore, Text::new("▼"), f.tf(16.0), TextColor(theme::SOFT_GOLD), Node { right: px(28), bottom: px(14), ..abs_node() }, Pickable::IGNORE));
                });
            });
            // Choices (above dialogue)
            root.spawn((ChoiceBox, Node { bottom: px(230), width: percent(100), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(10), ..abs_node() }));
            // Interaction prompt (anchored)
            root.spawn((PromptBox, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(4), ..abs_node() }, Pickable::IGNORE)).with_children(|b| {
                b.spawn((Node { column_gap: px(8), align_items: AlignItems::Center, padding: UiRect::axes(px(12), px(6)), border_radius: BorderRadius::all(px(16)), ..default() }, BackgroundColor(Color::srgba(0.05, 0.06, 0.15, 0.6)), Pickable::IGNORE))
                    .with_children(|r| {
                        r.spawn((
                            Node { min_width: px(30), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(px(7)), padding: UiRect::horizontal(px(6)), ..default() },
                            BackgroundColor(theme::PAPER_SOLID),
                            Pickable::IGNORE,
                        ))
                        .with_children(|k| {
                            k.spawn((PromptKey, Text::new("E"), f.tf(18.0), TextColor(theme::INK), Pickable::IGNORE));
                        });
                        r.spawn((PromptVerb, text_bundle(f, 21.0, theme::WHITE)));
                    });
                b.spawn((Node { width: px(90), height: px(5), border_radius: BorderRadius::all(px(3)), ..default() }, BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.2)), Pickable::IGNORE)).with_children(|t| {
                    t.spawn((PromptBar, Node { width: percent(0), height: percent(100), border_radius: BorderRadius::all(px(3)), ..default() }, BackgroundColor(theme::GOLD), Pickable::IGNORE));
                });
            });
            // Goal marker
            root.spawn((MarkerBox, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..abs_node() }, Pickable::IGNORE)).with_children(|b| {
                b.spawn((Text::new("✦"), f.tf(30.0), TextColor(theme::GOLD), TextShadow::default(), Pickable::IGNORE));
                b.spawn((MarkerLabel, text_bundle(f, 17.0, theme::SOFT_GOLD)));
            });
            // Departure hold
            root.spawn((DepartBox, Node { bottom: px(120), width: percent(100), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(8), ..abs_node() }, Pickable::IGNORE)).with_children(|b| {
                b.spawn((HudSlot::DepartText, text_bundle(f, 22.0, theme::SOFT_GOLD)));
                b.spawn((Node { width: px(200), height: px(6), border_radius: BorderRadius::all(px(3)), ..default() }, BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.2)), Pickable::IGNORE)).with_children(|t| {
                    t.spawn((DepartBar, Node { width: percent(0), height: percent(100), border_radius: BorderRadius::all(px(3)), ..default() }, BackgroundColor(theme::GOLD), Pickable::IGNORE));
                });
            });
            // Tab task view
            root.spawn((TasksBox, Node { right: px(34), top: px(80), padding: UiRect::all(px(22)), border_radius: BorderRadius::all(px(16)), max_width: px(460), ..abs_node() }, BackgroundColor(Color::srgba(0.05, 0.06, 0.15, 0.78)), Pickable::IGNORE)).with_children(|b| {
                b.spawn((HudSlot::TasksText, text_bundle(f, 20.0, theme::WHITE)));
            });
        });
    // Fade overlay (above everything)
    commands
        .spawn((
            FadeBox,
            Node { width: percent(100), height: percent(100), position_type: PositionType::Absolute, flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: px(14), ..default() },
            BackgroundColor(theme::NIGHT.with_alpha(0.0)),
            GlobalZIndex(100),
            Pickable::IGNORE,
        ))
        .with_children(|b| {
            b.spawn((FadeTitle, text_bundle(&font, 40.0, theme::WHITE)));
            b.spawn((FadeSub, text_bundle(&font, 22.0, theme::DIM), TextLayout::justify(Justify::Center), Node { max_width: px(820), ..default() }));
        });
}

fn set_text(t: &mut Text, s: &str) {
    if t.0 != s {
        t.0 = s.to_string();
    }
}

fn alpha_in_out(t: f32, hold: f32, fade_in: f32, fade_out: f32) -> f32 {
    if t < fade_in {
        t / fade_in
    } else if t < fade_in + hold {
        1.0
    } else {
        (1.0 - (t - fade_in - hold) / fade_out).max(0.0)
    }
}

/// Ticks HUD timers and handles dialogue/choice input (virtual time).
fn tick_hud(time: Res<Time>, mut hud: ResMut<Hud>, input: Res<InputFrame>, keys: Res<ButtonInput<KeyCode>>, i18n: Res<I18n>, mut msgs: MessageReader<super::UiActionMsg>) {
    let dt = time.delta_secs();
    if let Some(c) = &mut hud.card {
        c.t += dt;
        if c.t > c.hold + 1.6 || (input.advance && c.t > 0.8) {
            hud.card = None;
        }
    }
    if let Some((keys_, idx, t, per)) = &mut hud.narration {
        *t += dt;
        if *t > *per || (input.advance && *t > 0.6) {
            *t = 0.0;
            *idx += 1;
            if *idx >= keys_.len() {
                hud.narration = None;
            }
        }
    }
    if let Some(b) = &mut hud.banner {
        b.t += dt;
        if b.t > b.hold {
            hud.banner = None;
        }
    }
    if let Some(m) = &mut hud.mono {
        m.t += dt;
        if m.t > m.hold + 1.2 {
            hud.mono = None;
        }
    }
    if let Some(m) = &mut hud.teach {
        m.t += dt;
        if m.t > m.hold + 1.0 {
            hud.teach = None;
        }
    }
    if let Some((_, s)) = &mut hud.toast {
        *s -= dt;
        if *s <= 0.0 {
            hud.toast = None;
        }
    }
    hud.hidden_flash = (hud.hidden_flash - dt).max(0.0);
    hud.marker_flash = (hud.marker_flash - dt).max(0.0);
    if let Some(o) = &mut hud.objective {
        o.2 += dt;
    }
    let choice_open = hud.choice.is_some();
    if let Some(d) = &mut hud.dialogue {
        d.t += dt;
        let len = i18n.t(&d.key).chars().count() as f32;
        let glyph = if i18n.is_zh() { 0.04 } else { 0.016 };
        if !d.revealed && d.t * 1.0 / glyph >= len {
            d.revealed = true;
        }
        if d.revealed {
            d.after_reveal += dt;
        }
        if !choice_open {
            if input.advance && d.t > 0.12 {
                if !d.revealed {
                    d.revealed = true;
                    d.t = len * glyph;
                } else {
                    d.finished = true;
                }
            }
            if let Some(a) = d.auto {
                if d.revealed && d.after_reveal > a {
                    d.finished = true;
                }
            }
        }
    }
    // choices: keys 1-4 or clicks
    if let Some((opts, pick, sel)) = &mut hud.choice {
        let n = opts.len();
        for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
            if keys.just_pressed(*k) && i < n {
                *pick = Some(i);
            }
        }
        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
            *sel = (*sel + 1) % n.max(1);
        }
        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
            *sel = (*sel + n.max(1) - 1) % n.max(1);
        }
        if keys.just_pressed(KeyCode::Enter) {
            *pick = Some(*sel);
        }
        for super::UiActionMsg(a) in msgs.read() {
            if let UiAction::Choice(i) = a {
                *pick = Some(*i);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn draw_hud(
    hud: Res<Hud>,
    i18n: Res<I18n>,
    runner: Res<StoryRunner>,
    state: Res<State<AppState>>,
    input: Res<InputFrame>,
    mut texts: Query<(&HudSlot, &mut Text, &mut TextColor)>,
    mut nodes: ParamSet<(
        Query<&mut Node, With<ObjBox>>,
        Query<&mut Node, With<BannerBox>>,
        Query<&mut Node, With<DlgBox>>,
        Query<&mut Node, With<TeachBox>>,
        Query<&mut Node, With<TasksBox>>,
        Query<&mut Node, With<HudRoot>>,
        Query<&mut Visibility, With<DlgMore>>,
        Query<&mut BackgroundColor, With<DlgBadge>>,
    )>,
    time: Res<Time<Real>>,
) {
    let in_chapter = *state.get() == AppState::Chapter;
    if let Ok(mut n) = nodes.p5().single_mut() {
        n.display = if in_chapter && !hud.hide_all { Display::Flex } else { Display::None };
    }
    if !in_chapter {
        return;
    }
    let disp = |b: bool| if b { Display::Flex } else { Display::None };
    let obj_vis = hud.objective.is_some() || hud.depart_line;
    if let Ok(mut n) = nodes.p0().single_mut() {
        n.display = disp(obj_vis || hud.hidden.is_some());
    }
    if let Ok(mut n) = nodes.p1().single_mut() {
        n.display = disp(hud.banner.is_some());
    }
    if let Ok(mut n) = nodes.p2().single_mut() {
        n.display = disp(hud.dialogue.is_some());
    }
    if let Ok(mut n) = nodes.p3().single_mut() {
        n.display = disp(hud.teach.is_some() && hud.dialogue.is_none());
    }
    if let Ok(mut n) = nodes.p4().single_mut() {
        n.display = disp(input.tasks_held);
    }
    if let Some(d) = &hud.dialogue {
        if let Ok(mut c) = nodes.p7().single_mut() {
            c.0 = d.speaker.color;
        }
        if let Ok(mut v) = nodes.p6().single_mut() {
            let blink = (time.elapsed_secs() * 3.0).sin() > -0.3;
            *v = if d.revealed && d.needs_key && blink && hud.choice.is_none() { Visibility::Inherited } else { Visibility::Hidden };
        }
    }
    // --- compute every slot's text (+ alpha)
    let (ev, st) = match (&hud.objective, hud.depart_line) {
        (_, true) => (i18n.t("core.objective").to_string(), i18n.t("core.depart_line").to_string()),
        (Some((e, s, _)), _) => (e.map(|k| i18n.t(k).to_string()).unwrap_or_default(), s.map(|k| i18n.t(k).to_string()).unwrap_or_default()),
        _ => (String::new(), String::new()),
    };
    let hidden = match hud.hidden {
        Some((k, true)) => format!("◆ {}：{}", i18n.t("core.hidden_found"), i18n.t(k)),
        Some((_, false)) => format!("◇ {}：{}", i18n.t("core.hidden"), i18n.t("core.hidden_unknown")),
        None => String::new(),
    };
    let (banner_t, banner_n) = match &hud.banner {
        Some(b) => (
            format!("✦  {}  ·  {}  ✦", i18n.t(b.v.0), i18n.t("core.banner.done")),
            b.v.1.map(|n| format!("{}：{}", i18n.t("core.banner.next"), i18n.t(n))).unwrap_or_default(),
        ),
        None => (String::new(), String::new()),
    };
    let (card_a, num, title, sub) = match &hud.card {
        Some(c) => (alpha_in_out(c.t, c.hold, 0.8, 0.8), i18n.tf("core.chapter_n", &[("n", &c.v.0.to_string())]), i18n.t(c.v.1).to_string(), i18n.t(c.v.2).to_string()),
        None => (0.0, String::new(), String::new(), String::new()),
    };
    let (narr, narr_a) = match &hud.narration {
        Some((keys, i, tt, per)) => (i18n.t(keys.get(*i).copied().unwrap_or("")).to_string(), alpha_in_out(*tt, (per - 1.2).max(0.1), 0.6, 0.6)),
        None => (String::new(), 0.0),
    };
    let (dname, dtext) = match &hud.dialogue {
        Some(d) => {
            let name = i18n.t(d.speaker.name);
            let full = i18n.t(&d.key);
            let glyph = if i18n.is_zh() { 0.04 } else { 0.016 };
            let shown: String = if d.revealed { full.to_string() } else { full.chars().take((d.t / glyph) as usize).collect() };
            (if name.is_empty() { "…".to_string() } else { name.to_string() }, shown)
        }
        None => (String::new(), String::new()),
    };
    let (mono, mono_a) = match &hud.mono {
        Some(m) => (i18n.t(m.v).to_string(), alpha_in_out(m.t, m.hold, 0.5, 1.0)),
        None => (String::new(), 0.0),
    };
    let teach = match &hud.teach {
        Some(tch) => {
            let keys: Vec<String> = tch.v.0.iter().map(|k| format!("[{}]", i18n.t(k))).collect();
            format!("{}  {}", keys.join(" "), i18n.t(tch.v.1))
        }
        None => String::new(),
    };
    let mut tasks = String::new();
    if input.tasks_held {
        tasks = i18n.t("core.events").to_string();
        tasks.push('\n');
        for (i, e) in runner.events.iter().enumerate() {
            let done = runner.done.get(i).copied().unwrap_or(false);
            let mark = if done { "✓" } else if runner.event_index == Some(i) { "▶" } else { "·" };
            tasks.push_str(&format!("\n{mark} {}", i18n.t(e.name)));
            if runner.event_index == Some(i) && !done {
                if let Some(k) = runner.current_step_key() {
                    tasks.push_str(&format!("\n    {}", i18n.t(k)));
                }
            }
        }
        if let Some(h) = runner.hidden_key {
            if runner.hidden_shown || runner.hidden_done {
                let txt = if runner.hidden_done { i18n.t(h) } else { i18n.t("core.hidden_unknown") };
                tasks.push_str(&format!("\n\n{}：{}", i18n.t("core.hidden"), txt));
            }
        }
    }
    let (toast, toast_a) = match &hud.toast {
        Some((s, left)) => (s.clone(), left.min(1.0)),
        None => (String::new(), 0.0),
    };
    let depart = if hud.depart_line && runner.departure_ready { format!("{} E {}", i18n.t("core.depart_hold"), i18n.t("core.depart_verb")) } else { String::new() };
    for (slot, mut t, mut c) in &mut texts {
        let (s, col): (&str, Option<Color>) = match slot {
            HudSlot::ObjEvent => (&ev, None),
            HudSlot::ObjStep => (&st, None),
            HudSlot::HiddenRow => (&hidden, None),
            HudSlot::BannerTitle => (&banner_t, None),
            HudSlot::BannerNext => (&banner_n, None),
            HudSlot::CardNum => (&num, Some(theme::SOFT_GOLD.with_alpha(card_a))),
            HudSlot::CardTitle => (&title, Some(theme::WHITE.with_alpha(card_a))),
            HudSlot::CardSub => (&sub, Some(theme::WHITE.with_alpha(card_a * 0.85))),
            HudSlot::NarrText => (&narr, Some(theme::WHITE.with_alpha(narr_a))),
            HudSlot::DlgName => (&dname, None),
            HudSlot::DlgText => (&dtext, None),
            HudSlot::MonoText => (&mono, Some(theme::WHITE.with_alpha(mono_a))),
            HudSlot::TeachText => (&teach, None),
            HudSlot::TasksText => (&tasks, None),
            HudSlot::ToastText => (&toast, Some(theme::SOFT_GOLD.with_alpha(toast_a))),
            HudSlot::DepartText => (&depart, None),
        };
        set_text(&mut t, s);
        if let Some(col) = col {
            if c.0 != col {
                c.0 = col;
            }
        }
    }
}

/// Spawns/clears the choice buttons.
fn draw_choices(mut commands: Commands, mut hud: ResMut<Hud>, font: Res<UiFont>, q: Query<Entity, With<ChoiceBox>>, children: Query<&Children>) {
    let Ok(boxe) = q.single() else { return };
    let want = hud.choice.is_some();
    if want && !hud.choice_spawned {
        hud.choice_spawned = true;
        let opts = hud.choice.as_ref().unwrap().0.clone();
        commands.entity(boxe).with_children(|b| {
            for (i, k) in opts.iter().enumerate() {
                b.spawn(super::button(&font, k, 22.0, UiAction::Choice(i))).insert(Node {
                    padding: UiRect::axes(px(30), px(12)),
                    min_width: px(420),
                    max_width: px(900),
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(22)),
                    border: UiRect::all(px(1)),
                    ..default()
                });
            }
        });
    } else if !want {
        if let Ok(ch) = children.get(boxe) {
            for c in ch.iter() {
                commands.entity(c).despawn();
            }
        }
        hud.choice_spawned = false;
    }
}

/// Places the anchored interaction prompt, the marker and the departure bar.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn draw_prompt(
    focus: Res<InteractFocus>,
    hud: Res<Hud>,
    i18n: Res<I18n>,
    runner: Res<StoryRunner>,
    touch: Res<TouchControls>,
    scale: Res<UiScale>,
    cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q: ParamSet<(
        Query<&mut Node, With<PromptBox>>,
        Query<&mut Text, With<PromptVerb>>,
        Query<&mut Text, With<PromptKey>>,
        Query<&mut Node, With<PromptBar>>,
        Query<&mut Node, With<MarkerBox>>,
        Query<&mut Text, With<MarkerLabel>>,
        Query<&mut Node, With<DepartBox>>,
        Query<&mut Node, With<DepartBar>>,
    )>,
    world_q: Query<&GlobalTransform>,
    flow: Res<super::super::state::DepartHold>,
) {
    let Ok((camera, cgt)) = cam.single() else { return };
    let s = scale.0.max(0.01);
    let show_prompt = focus.entity.is_some() && hud.dialogue.is_none() && hud.choice.is_none() && hud.card.is_none();
    let vp = if show_prompt { camera.world_to_viewport(cgt, focus.world_pos).ok() } else { None };
    if let Ok(mut n) = q.p0().single_mut() {
        match vp {
            Some(p) => {
                n.display = Display::Flex;
                n.left = px(p.x / s - 70.0);
                n.top = px(p.y / s - 30.0);
            }
            None => n.display = Display::None,
        }
    }
    if let Some(k) = &focus.prompt {
        if let Ok(mut t) = q.p1().single_mut() {
            set_text(&mut t, i18n.t(k));
        }
    }
    if let Ok(mut t) = q.p2().single_mut() {
        set_text(&mut t, if touch.enabled { "●" } else { "E" });
    }
    if let Ok(mut n) = q.p3().single_mut() {
        n.width = percent(focus.hold_progress * 100.0);
    }
    // marker: flashes on stall (or while Tab is held)
    let target = runner.target.clone();
    let mut mpos = None;
    if hud.marker_flash > 0.0 {
        if let Some(t) = target {
            let wp = match &t {
                super::super::story::Pos::World(v) => Some(*v),
                _ => None,
            };
            let _ = &world_q;
            let wp = wp.or(runner.target_world);
            if let Some(w) = wp {
                if let Ok(p) = camera.world_to_viewport(cgt, w + Vec3::Y * 1.6) {
                    mpos = Some(p);
                }
            }
        }
    }
    if let Ok(mut n) = q.p4().single_mut() {
        match mpos {
            Some(p) => {
                n.display = Display::Flex;
                n.left = px(p.x / s - 40.0);
                n.top = px(p.y / s - 30.0);
            }
            None => n.display = Display::None,
        }
    }
    if let Ok(mut t) = q.p5().single_mut() {
        set_text(&mut t, runner.target_label.map(|k| i18n.t(k)).unwrap_or(""));
    }
    if let Ok(mut n) = q.p6().single_mut() {
        n.display = if runner.departure_ready && focus.entity.is_none() && hud.dialogue.is_none() { Display::Flex } else { Display::None };
    }
    if let Ok(mut n) = q.p7().single_mut() {
        n.width = percent(flow.progress * 100.0);
    }
}

fn draw_fade(fade: Res<Fade>, hud: Res<Hud>, mut q: Query<&mut BackgroundColor, With<FadeBox>>, mut t1: Query<(&mut Text, &mut TextColor), (With<FadeTitle>, Without<FadeSub>)>, mut t2: Query<(&mut Text, &mut TextColor), (With<FadeSub>, Without<FadeTitle>)>) {
    if let Ok(mut bg) = q.single_mut() {
        bg.0 = theme::NIGHT.with_alpha(fade.alpha);
    }
    // the loading title gives way to the chapter card (they share the centre)
    let ta = if hud.card.is_some() { 0.0 } else { ((fade.alpha - 0.6) / 0.4).clamp(0.0, 1.0) };
    let show = ta > 0.02;
    if let Ok((mut t, mut c)) = t1.single_mut() {
        set_text(&mut t, if show { fade.text.as_deref().unwrap_or("") } else { "" });
        c.0 = theme::WHITE.with_alpha(ta);
    }
    if let Ok((mut t, mut c)) = t2.single_mut() {
        set_text(&mut t, if show { fade.sub.as_deref().unwrap_or("") } else { "" });
        c.0 = theme::DIM.with_alpha(ta * 0.8);
    }
}

fn tick_fade(time: Res<Time<Real>>, mut fade: ResMut<Fade>) {
    let dt = time.delta_secs().min(0.1);
    let d = fade.target - fade.alpha;
    let step = fade.speed.max(0.5) * dt;
    fade.alpha = if d.abs() <= step { fade.target } else { fade.alpha + step * d.signum() };
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Fade>()
        .add_systems(Startup, spawn_hud.after(super::super::boot_fonts))
        .add_systems(Update, (tick_hud, draw_hud, draw_choices, draw_prompt).chain().run_if(resource_exists::<StoryRunner>))
        .add_systems(Update, (tick_fade, draw_fade).chain());
    let _ = (Hovered::default(), Button);
}
