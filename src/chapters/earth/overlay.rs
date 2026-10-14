//! Earth screen overlays: the railway page (TrainRoutePage — the valley→dunes
//! loading page with a little train running along a drawn track), the
//! drawing panel (DesertDrawingPanel — a torn sketchbook page in the right
//! third), the fox's memory subtitle (GameUI.Memory), the steam veil, and the
//! end card (DesertEndCard, "Rose" style). Sketches come from the manifest
//! (`image.earth.*`) or are drawn procedurally into textures.

use crate::core::i18n::{I18n, Tr};
use crate::core::manifest::Manifest;
use crate::core::prelude::*;
use crate::core::ui::{UiFont, theme};
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::picking::Pickable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sketch {
    OldSheep,
    Box,
}

#[derive(Resource, Default)]
pub struct EarthUi {
    pub page: Option<PageAnim>,
    pub panel: Option<PanelAnim>,
    pub veil: (f32, f32, f32),
    pub memory: Option<(&'static str, f32)>,
    pub end: Option<f32>,
    pub sheep: Handle<Image>,
    pub boxed: Handle<Image>,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PageAnim {
    pub t: f32,
    /// Seconds since fully covered.
    pub covered_t: f32,
    pub loaded: bool,
    pub progress: f32,
    pub full_t: f32,
    pub uncover: f32,
    pub covered_flag: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct PanelAnim {
    pub sketch: Option<Sketch>,
    pub t: f32,
    pub closing: bool,
}

#[derive(Component)]
pub struct PanelRoot;
#[derive(Component)]
pub struct PanelImage;
#[derive(Component)]
pub struct MemoryText;
#[derive(Component)]
pub struct Veil;
#[derive(Component)]
pub struct PageRoot;
#[derive(Component)]
pub struct PageCard;
#[derive(Component)]
pub struct PageStar;
#[derive(Component)]
pub struct PageTrain;
#[derive(Component)]
pub struct PageFill;
#[derive(Component)]
pub struct EndRoot;
#[derive(Component)]
pub enum EndPart {
    Star,
    Q1,
    Q2,
    Ded,
    Title,
    Fin,
}

const INK: Color = Color::srgb(0.2, 0.17, 0.22);

fn txt(font: &UiFont, key: &str, size: f32, color: Color) -> impl Bundle {
    (Text::new(""), Tr::new(key), font.tf(size), TextColor(color), TextLayout::justify(Justify::Center), Pickable::IGNORE)
}

/// Spawns the (hidden) overlay nodes; called once per chapter entry.
pub fn spawn(world: &mut World) {
    let font = world.resource::<UiFont>().clone();
    let test = world.resource::<crate::core::params::LaunchParams>().test;
    let (sheep, boxed) = sketches(world);
    world.insert_resource(EarthUi { sheep, boxed, speed: if test { 4.0 } else { 1.0 }, ..default() });
    let f = &font;
    // drawing panel (right third, 60–66 % of the screen height, slight tilt)
    world
        .spawn((
            PanelRoot,
            ChapterScoped,
            Node {
                position_type: PositionType::Absolute,
                right: percent(4),
                top: percent(16),
                width: percent(28),
                height: percent(62),
                padding: UiRect::all(px(14)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgb(0.97, 0.94, 0.86)),
            BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.35), px(6), px(8), px(0), px(14)),
            UiTransform { rotation: Rot2::degrees(-2.5), ..UiTransform::IDENTITY },
            GlobalZIndex(9),
            Pickable::IGNORE,
        ))
        .with_children(|p| {
            p.spawn((PanelImage, ImageNode::default(), Node { width: percent(100), height: percent(100), ..default() }, Pickable::IGNORE));
        });
    // the fox's memory line
    world
        .spawn((ChapterScoped, Node { position_type: PositionType::Absolute, width: percent(100), bottom: percent(30), justify_content: JustifyContent::Center, ..default() }, GlobalZIndex(11), Pickable::IGNORE))
        .with_children(|p| {
            p.spawn((MemoryText, Text::new(""), f.tf(26.0), TextColor(theme::SOFT_GOLD.with_alpha(0.0)), TextShadow { offset: Vec2::new(1.5, 1.5), color: theme::SHADOW }, TextLayout::justify(Justify::Center), Node { max_width: px(900), ..default() }, Pickable::IGNORE));
        });
    // steam veil
    world.spawn((Veil, ChapterScoped, Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() }, BackgroundColor(Color::srgba(0.96, 0.95, 0.92, 0.0)), GlobalZIndex(14), Pickable::IGNORE));
    // railway page
    world
        .spawn((
            PageRoot,
            ChapterScoped,
            Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), justify_content: JustifyContent::Center, align_items: AlignItems::Center, display: Display::None, ..default() },
            GlobalZIndex(15),
            Pickable::IGNORE,
        ))
        .with_children(|r| {
            r.spawn((PageStar, Text::new("✦"), f.tf(54.0), TextColor(theme::GOLD.with_alpha(0.0)), Node { position_type: PositionType::Absolute, ..default() }, Pickable::IGNORE));
            r.spawn((
                PageCard,
                Node { width: percent(100), height: percent(100), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: px(14), ..default() },
                BackgroundColor(Color::srgb(0.95, 0.91, 0.82)),
                UiTransform { scale: Vec2::splat(0.05), ..UiTransform::IDENTITY },
                Pickable::IGNORE,
            ))
            .with_children(|c| {
                c.spawn(txt(f, "earth.page.chapter", 20.0, INK.with_alpha(0.7)));
                c.spawn(txt(f, "earth.page.title", 64.0, INK));
                c.spawn(txt(f, "earth.page.sub", 22.0, INK.with_alpha(0.75)));
                c.spawn((Node { height: px(36), ..default() }, Pickable::IGNORE));
                // drawn track: valley ● ———— ● dunes, a little train runs along it
                c.spawn((Node { width: percent(62), height: px(46), ..default() }, Pickable::IGNORE)).with_children(|t| {
                    t.spawn((Node { position_type: PositionType::Absolute, left: px(0), right: px(0), top: px(30), height: px(3), ..default() }, BackgroundColor(INK.with_alpha(0.35)), Pickable::IGNORE));
                    t.spawn((PageFill, Node { position_type: PositionType::Absolute, left: px(0), width: percent(0), top: px(29), height: px(5), ..default() }, BackgroundColor(Color::srgb(0.72, 0.45, 0.25)), Pickable::IGNORE));
                    for left in [true, false] {
                        let mut n = Node { position_type: PositionType::Absolute, top: px(24), width: px(15), height: px(15), border_radius: BorderRadius::all(px(8)), ..default() };
                        if left {
                            n.left = px(-7);
                        } else {
                            n.right = px(-7);
                        }
                        t.spawn((n, BackgroundColor(INK), Pickable::IGNORE));
                    }
                    t.spawn((PageTrain, Node { position_type: PositionType::Absolute, left: percent(0), top: px(0), width: px(56), height: px(30), margin: UiRect::left(px(-28)), ..default() }, Pickable::IGNORE)).with_children(|tr| {
                        let part = |x: f32, y: f32, w: f32, h: f32, c: Color| (Node { position_type: PositionType::Absolute, left: px(x), top: px(y), width: px(w), height: px(h), border_radius: BorderRadius::all(px(2)), ..default() }, BackgroundColor(c), Pickable::IGNORE);
                        tr.spawn(part(0.0, 12.0, 20.0, 14.0, Color::srgb(0.48, 0.18, 0.16)));
                        tr.spawn(part(22.0, 8.0, 26.0, 18.0, Color::srgb(0.18, 0.35, 0.27)));
                        tr.spawn(part(42.0, 1.0, 6.0, 9.0, INK));
                        tr.spawn(part(4.0, 25.0, 6.0, 6.0, INK));
                        tr.spawn(part(26.0, 25.0, 6.0, 6.0, INK));
                        tr.spawn(part(38.0, 25.0, 6.0, 6.0, INK));
                    });
                });
                c.spawn((Node { width: percent(62), justify_content: JustifyContent::SpaceBetween, ..default() }, Pickable::IGNORE)).with_children(|l| {
                    l.spawn(txt(f, "earth.page.from", 18.0, INK.with_alpha(0.75)));
                    l.spawn(txt(f, "earth.page.to", 18.0, INK.with_alpha(0.75)));
                });
                c.spawn((Node { height: px(26), ..default() }, Pickable::IGNORE));
                c.spawn((txt(f, "earth.page.line", 22.0, INK.with_alpha(0.85)), Node { max_width: px(760), ..default() }));
            });
        });
    // end card
    world
        .spawn((
            EndRoot,
            ChapterScoped,
            Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: px(18), display: Display::None, ..default() },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.14, 0.0)),
            GlobalZIndex(16),
            Pickable::IGNORE,
        ))
        .with_children(|r| {
            r.spawn((EndPart::Star, Text::new("✦"), f.tf(72.0), TextColor(theme::GOLD.with_alpha(0.0)), Pickable::IGNORE));
            r.spawn((Node { height: px(10), ..default() }, Pickable::IGNORE));
            r.spawn((EndPart::Q1, Text::new(""), f.tf(30.0), TextColor(theme::WHITE), TextLayout::justify(Justify::Center), Pickable::IGNORE));
            r.spawn((EndPart::Q2, Text::new(""), f.tf(30.0), TextColor(theme::WHITE), TextLayout::justify(Justify::Center), Pickable::IGNORE));
            r.spawn((Node { height: px(26), ..default() }, Pickable::IGNORE));
            r.spawn((EndPart::Ded, txt(f, "earth.end.ded", 18.0, theme::DIM.with_alpha(0.0))));
            r.spawn((Node { height: px(20), ..default() }, Pickable::IGNORE));
            r.spawn((EndPart::Title, txt(f, "earth.end.title", 54.0, theme::SOFT_GOLD.with_alpha(0.0))));
            r.spawn((EndPart::Fin, txt(f, "earth.end.fin", 24.0, theme::WHITE.with_alpha(0.0))));
        });
}

// ---------------------------------------------------------- script API ----

pub fn page_start(world: &mut World) {
    world.resource_mut::<EarthUi>().page = Some(PageAnim { t: 0.0, covered_t: 0.0, loaded: false, progress: 0.0, full_t: 0.0, uncover: 0.0, covered_flag: false });
    world.resource_mut::<Hud>().hide_all = true;
}
pub fn page_loaded(world: &mut World) {
    if let Some(p) = world.resource_mut::<EarthUi>().page.as_mut() {
        p.loaded = true;
    }
}
pub fn panel_show(world: &mut World, sketch: Option<Sketch>) {
    let mut ui = world.resource_mut::<EarthUi>();
    match ui.panel.as_mut() {
        Some(p) if !p.closing => p.sketch = sketch,
        _ => ui.panel = Some(PanelAnim { sketch, t: 0.0, closing: false }),
    }
}
pub fn panel_hide(world: &mut World) {
    if let Some(p) = world.resource_mut::<EarthUi>().panel.as_mut() {
        p.closing = true;
        p.t = p.t.min(0.4);
    }
}
pub fn memory(world: &mut World, key: &'static str) {
    world.resource_mut::<EarthUi>().memory = Some((key, 0.0));
}
/// Veil: (alpha, target, seconds per unit).
pub fn veil(world: &mut World, target: f32, secs: f32) {
    let mut ui = world.resource_mut::<EarthUi>();
    let a = ui.veil.0;
    ui.veil = (a, target, secs.max(0.01));
}
pub fn end_start(world: &mut World) {
    world.resource_mut::<EarthUi>().end = Some(0.0);
    world.resource_mut::<Hud>().hide_all = true;
}

// ------------------------------------------------------------- systems ----

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update(
    time: Res<Time>,
    mut ui: ResMut<EarthUi>,
    mut runner: ResMut<StoryRunner>,
    mut hud: ResMut<Hud>,
    i18n: Res<I18n>,
    mut q: ParamSet<(
        Query<(&mut Node, &mut UiTransform, &mut BackgroundColor), With<PanelRoot>>,
        Query<&mut ImageNode, With<PanelImage>>,
        Query<(&mut Text, &mut TextColor), With<MemoryText>>,
        Query<&mut BackgroundColor, With<Veil>>,
        Query<&mut Node, With<PageRoot>>,
        Query<(&mut UiTransform, &mut BackgroundColor), With<PageCard>>,
        Query<&mut TextColor, With<PageStar>>,
        Query<&mut Node, With<PageTrain>>,
    )>,
    mut fill: Query<&mut Node, (With<PageFill>, Without<PageTrain>, Without<PageRoot>, Without<PanelRoot>, Without<EndRoot>)>,
    mut end_root: Query<(&mut Node, &mut BackgroundColor), (With<EndRoot>, Without<PageRoot>, Without<PanelRoot>, Without<Veil>, Without<PageTrain>, Without<PageCard>)>,
    mut end_parts: Query<(&EndPart, &mut Text, &mut TextColor), (Without<MemoryText>, Without<PageStar>)>,
) {
    let dt = time.delta_secs() * ui.speed;
    // ---- drawing panel: slide/fade in 0.4 s
    let (sheep, boxed) = (ui.sheep.clone(), ui.boxed.clone());
    let mut panel_done = false;
    if let Some(p) = ui.panel.as_mut() {
        p.t += dt;
        let k = if p.closing { 1.0 - (p.t / 0.4).min(1.0) } else { (p.t / 0.4).min(1.0) };
        if p.closing && p.t >= 0.4 {
            panel_done = true;
        }
        let e = k * k * (3.0 - 2.0 * k);
        if let Ok((mut n, mut tf, mut bg)) = q.p0().single_mut() {
            n.display = Display::Flex;
            tf.translation = Val2::new(px((1.0 - e) * 80.0), px(0.0));
            bg.0 = Color::srgba(0.97, 0.94, 0.86, e);
        }
        let img = match p.sketch {
            Some(Sketch::OldSheep) => Some(sheep),
            Some(Sketch::Box) => Some(boxed),
            None => None,
        };
        if let Ok(mut im) = q.p1().single_mut() {
            match img {
                Some(h) => {
                    if im.image != h {
                        im.image = h;
                    }
                    im.color = Color::WHITE.with_alpha(e);
                }
                None => im.color = Color::WHITE.with_alpha(0.0),
            }
        }
    }
    if panel_done {
        ui.panel = None;
        if let Ok((mut n, _, _)) = q.p0().single_mut() {
            n.display = Display::None;
        }
    }
    // ---- memory subtitle (alpha 0.75)
    let mut mem_done = false;
    if let Some((key, t)) = ui.memory.as_mut() {
        *t += dt;
        let a = (*t / 0.8).min(1.0).min(((4.5 - *t) / 0.8).max(0.0)) * 0.75;
        if let Ok((mut tx, mut c)) = q.p2().single_mut() {
            let s = i18n.t(key);
            if tx.0 != s {
                tx.0 = s.to_string();
            }
            c.0 = theme::SOFT_GOLD.with_alpha(a);
        }
        mem_done = *t > 4.5;
    }
    if mem_done {
        ui.memory = None;
        if let Ok((mut tx, _)) = q.p2().single_mut() {
            tx.0.clear();
        }
    }
    // ---- veil
    {
        let (a, target, secs) = ui.veil;
        let step = dt / secs;
        let na = if (target - a).abs() <= step { target } else { a + step * (target - a).signum() };
        ui.veil.0 = na;
        if let Ok(mut bg) = q.p3().single_mut() {
            bg.0 = Color::srgba(0.96, 0.95, 0.92, na);
        }
    }
    // ---- railway page: star 0.4 s, cover 0.8 s, progress ≥ 3.2 s (≤ 96 % until loaded), hold 0.35 s, uncover 0.8 s
    let mut page_end = false;
    if let Some(p) = ui.page.as_mut() {
        p.t += dt;
        let star_a = (p.t / 0.4).min(1.0) * (1.0 - ((p.t - 0.6) / 0.6).clamp(0.0, 1.0));
        let cover = ((p.t - 0.4) / 0.8).clamp(0.0, 1.0);
        let scale = cover;
        if cover >= 1.0 {
            if !p.covered_flag {
                p.covered_flag = true;
                runner.set_flag("page_covered");
            }
            p.covered_t += dt;
            let target = if p.loaded { 1.0 } else { 0.96 };
            let lin = (p.covered_t / 3.2).min(1.0);
            p.progress = (lin * lin * (3.0 - 2.0 * lin)).min(target).max(p.progress);
            if p.progress >= 1.0 && p.loaded {
                p.full_t += dt;
                if p.full_t >= 0.35 {
                    p.uncover += dt / 0.8;
                    if p.uncover >= 1.0 {
                        page_end = true;
                    }
                }
            }
        }
        let ease = |x: f32| x * x * (3.0 - 2.0 * x);
        let shown = if p.uncover > 0.0 { 1.0 - ease(p.uncover.min(1.0)) } else { ease(scale) };
        if let Ok(mut n) = q.p4().single_mut() {
            n.display = Display::Flex;
        }
        if let Ok((mut tf, mut bg)) = q.p5().single_mut() {
            tf.scale = Vec2::splat(0.03 + 0.97 * shown);
            bg.0 = Color::srgba(0.95, 0.91, 0.82, ease(cover));
        }
        if let Ok(mut c) = q.p6().single_mut() {
            c.0 = theme::GOLD.with_alpha(star_a);
        }
        if let Ok(mut n) = q.p7().single_mut() {
            n.left = percent(p.progress * 100.0);
        }
        if let Ok(mut n) = fill.single_mut() {
            n.width = percent(p.progress * 100.0);
        }
    }
    if page_end {
        ui.page = None;
        runner.set_flag("page_done");
        hud.hide_all = false;
        if let Ok(mut n) = q.p4().single_mut() {
            n.display = Display::None;
        }
    }
    // ---- end card: night 2.4 s (2.5 s fade from the picture); star at 1.3 s over 1.6 s;
    //      quote lines flow in from 3.6 s (1.2 s each, 0.35 s gap); hold 3 s; dedication 1.1 s; title 1.4 s; "The End" 0.6 s later
    if let Some(t) = ui.end.as_mut() {
        *t += dt;
        let t = *t;
        if let Ok((mut n, mut bg)) = end_root.single_mut() {
            n.display = Display::Flex;
            bg.0 = Color::srgba(0.03, 0.05, 0.14, (t / 2.5).min(1.0));
        }
        let q1s = 3.6;
        let q2s = q1s + 1.2 + 0.35;
        let ded = q2s + 1.2 + 3.0;
        let title = ded + 1.1;
        let fin = title + 1.4 + 0.6;
        for (part, mut tx, mut c) in &mut end_parts {
            match part {
                EndPart::Star => c.0 = theme::GOLD.with_alpha(((t - 1.3) / 1.6).clamp(0.0, 1.0)),
                EndPart::Q1 | EndPart::Q2 => {
                    let (key, start) = if matches!(part, EndPart::Q1) { ("earth.end.q1", q1s) } else { ("earth.end.q2", q2s) };
                    let full = i18n.t(key);
                    let n = full.chars().count();
                    let k = ((t - start) / 1.2).clamp(0.0, 1.0);
                    let shown: String = full.chars().take((k * n as f32).ceil() as usize).collect();
                    if tx.0 != shown {
                        tx.0 = shown;
                    }
                    c.0 = theme::WHITE;
                }
                EndPart::Ded => c.0 = theme::DIM.with_alpha(((t - ded) / 1.1).clamp(0.0, 1.0) * 0.8),
                EndPart::Title => c.0 = theme::SOFT_GOLD.with_alpha(((t - title) / 1.4).clamp(0.0, 1.0)),
                EndPart::Fin => c.0 = theme::WHITE.with_alpha(((t - fin) / 0.8).clamp(0.0, 1.0)),
            }
        }
        if t > fin + 2.5 {
            runner.set_flag("endcard_done");
        }
    }
}

// ------------------------------------------------------------ sketches ----

/// Manifest image if a file path is set, else a procedural ink drawing.
fn sketches(world: &mut World) -> (Handle<Image>, Handle<Image>) {
    let man = world.resource::<Manifest>();
    let p1 = man.path("image.earth.sketch_old_sheep").map(|s| s.to_string());
    let p2 = man.path("image.earth.sketch_box").map(|s| s.to_string());
    let server = world.resource::<AssetServer>().clone();
    let mut images = world.resource_mut::<Assets<Image>>();
    let a = match p1 {
        Some(p) => server.load(p),
        None => images.add(draw(Sketch::OldSheep)),
    };
    let b = match p2 {
        Some(p) => server.load(p),
        None => images.add(draw(Sketch::Box)),
    };
    (a, b)
}

const S: usize = 384;

struct Canvas {
    px: Vec<f32>,
}

impl Canvas {
    fn new() -> Self {
        Canvas { px: vec![0.0; S * S] }
    }
    fn wobble(p: Vec2, k: f32) -> Vec2 {
        p + Vec2::new((p.y * 0.05 + k).sin() * 1.2, (p.x * 0.047 + k * 1.7).cos() * 1.2)
    }
    /// Ink stroke along a polyline.
    fn line(&mut self, pts: &[Vec2], w: f32) {
        for seg in pts.windows(2) {
            let (a, b) = (Self::wobble(seg[0], 0.3), Self::wobble(seg[1], 0.3));
            let (x0, x1) = (a.x.min(b.x) - w - 2.0, a.x.max(b.x) + w + 2.0);
            let (y0, y1) = (a.y.min(b.y) - w - 2.0, a.y.max(b.y) + w + 2.0);
            for y in (y0.max(0.0) as usize)..(y1.min(S as f32 - 1.0) as usize) {
                for x in (x0.max(0.0) as usize)..(x1.min(S as f32 - 1.0) as usize) {
                    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    let ab = b - a;
                    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-4)).clamp(0.0, 1.0);
                    let d = p.distance(a + ab * t);
                    let v = (w * 0.5 + 0.7 - d).clamp(0.0, 1.0);
                    let i = y * S + x;
                    self.px[i] = self.px[i].max(v);
                }
            }
        }
    }
    fn ellipse(&mut self, c: Vec2, r: Vec2, w: f32, a0: f32, a1: f32) {
        let n = 48;
        let pts: Vec<Vec2> = (0..=n).map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            c + Vec2::new(a.cos() * r.x, a.sin() * r.y)
        }).collect();
        self.line(&pts, w);
    }
    /// Outline of a union of circles (a woolly cloud).
    fn cloud(&mut self, circles: &[(Vec2, f32)], w: f32) {
        for y in 0..S {
            for x in 0..S {
                let p = Canvas::wobble(Vec2::new(x as f32 + 0.5, y as f32 + 0.5), 1.1);
                let mut d = f32::MAX;
                for (c, r) in circles {
                    d = d.min(p.distance(*c) - r);
                }
                let v = (w * 0.5 + 0.7 - d.abs()).clamp(0.0, 1.0);
                if v > 0.0 {
                    let i = y * S + x;
                    self.px[i] = self.px[i].max(v);
                }
            }
        }
    }
    fn dot(&mut self, c: Vec2, r: f32) {
        for y in (c.y - r - 2.0).max(0.0) as usize..(c.y + r + 2.0).min(S as f32) as usize {
            for x in (c.x - r - 2.0).max(0.0) as usize..(c.x + r + 2.0).min(S as f32) as usize {
                let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5).distance(c);
                let v = (r + 0.7 - d).clamp(0.0, 1.0);
                let i = y * S + x;
                self.px[i] = self.px[i].max(v);
            }
        }
    }
    fn image(&self) -> Image {
        let mut data = Vec::with_capacity(S * S * 4);
        for v in &self.px {
            data.extend_from_slice(&[52, 44, 58, (v * 235.0) as u8]);
        }
        Image::new(Extent3d { width: S as u32, height: S as u32, depth_or_array_layers: 1 }, TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD)
    }
}

fn draw(which: Sketch) -> Image {
    let mut c = Canvas::new();
    let v = Vec2::new;
    match which {
        Sketch::OldSheep => {
            // woolly body (a cloud of bumps), drooping head, thin bent legs, a beard
            let mut circles = Vec::new();
            let center = v(205.0, 190.0);
            for i in 0..14 {
                let a = i as f32 / 14.0 * std::f32::consts::TAU;
                circles.push((center + v(a.cos() * 82.0, a.sin() * 48.0), 26.0 + (i % 3) as f32 * 3.0));
            }
            circles.push((center, 70.0));
            c.cloud(&circles, 3.2);
            // head hanging low on the left
            c.ellipse(v(92.0, 222.0), v(30.0, 22.0), 3.2, -2.3, 2.6);
            c.line(&[v(110.0, 200.0), v(128.0, 186.0)], 3.0);
            c.line(&[v(80.0, 206.0), v(64.0, 228.0), v(74.0, 236.0)], 3.0); // drooping ear
            c.dot(v(86.0, 218.0), 3.2);
            c.line(&[v(80.0, 240.0), v(78.0, 258.0)], 2.2);
            c.line(&[v(88.0, 242.0), v(88.0, 262.0)], 2.2);
            c.line(&[v(96.0, 241.0), v(99.0, 257.0)], 2.2); // a straggly beard
            // wrinkle lines on the brow
            c.line(&[v(86.0, 206.0), v(98.0, 204.0)], 1.6);
            c.line(&[v(84.0, 211.0), v(97.0, 209.0)], 1.6);
            // legs: thin and a little bent
            for (x, bend) in [(150.0f32, 6.0f32), (178.0, -4.0), (236.0, 5.0), (262.0, -6.0)] {
                c.line(&[v(x, 228.0), v(x + bend, 262.0), v(x, 296.0)], 3.0);
                c.line(&[v(x - 3.0, 296.0), v(x + 7.0, 296.0)], 3.0);
            }
            c.line(&[v(286.0, 176.0), v(300.0, 166.0), v(304.0, 178.0)], 2.6); // tail tuft
            // a walking stick, for an old sheep
            c.line(&[v(52.0, 300.0), v(64.0, 190.0)], 2.6);
            c.ellipse(v(72.0, 190.0), v(8.0, 8.0), 2.6, std::f32::consts::PI, std::f32::consts::TAU);
        }
        Sketch::Box => {
            // a box in perspective with three air holes
            let (a, b, cc, d) = (v(96.0, 170.0), v(262.0, 170.0), v(262.0, 300.0), v(96.0, 300.0));
            c.line(&[a, b, cc, d, a], 3.4);
            let o = v(48.0, -42.0);
            c.line(&[a, a + o, b + o, b], 3.4);
            c.line(&[b + o, cc + o, cc], 3.4);
            for (x, y) in [(140.0f32, 222.0f32), (180.0, 246.0), (220.0, 222.0)] {
                c.ellipse(v(x, y), v(8.0, 7.0), 2.4, 0.0, std::f32::consts::TAU);
                c.dot(v(x, y), 3.0);
            }
            // lid seam
            c.line(&[v(96.0, 186.0), v(262.0, 186.0)], 1.6);
        }
    }
    c.image()
}
