//! Ending: epilogue lines over the night sky, then "The End", the planets of
//! the journey, and a button back to the title.

use super::super::camera::{CameraRig, Shot};
use super::super::chapter::ChapterRegistry;
use super::super::i18n::{I18n, Tr};
use super::super::sky::Sky;
use super::super::state::{AppState, Flow};
use super::{UiAction, UiActionMsg, UiFont, button, theme};
use bevy::picking::Pickable;
use bevy::prelude::*;

const LINES: [&str; 4] = ["ending.1", "ending.2", "ending.3", "ending.4"];
const PER_LINE: f32 = 4.5;

#[derive(Resource, Default)]
struct EndingState {
    t: f32,
    shown_final: bool,
}

#[derive(Component)]
struct EndingLine;
#[derive(Component)]
struct EndingFinal;

fn enter(mut commands: Commands, font: Res<UiFont>, mut sky: ResMut<Sky>, mut rig: ResMut<CameraRig>, mut audio: MessageWriter<super::super::audio::AudioCmd>, reg: Res<ChapterRegistry>, i18n: Res<I18n>) {
    commands.insert_resource(EndingState::default());
    *sky = Sky::default();
    sky.set_hour(22.0);
    sky.set_rate(0.0, None);
    sky.fixed_night = Some(1.0);
    audio.write(super::super::audio::AudioCmd::music("music.ending"));
    rig.focus_override = Some(Vec3::ZERO);
    rig.begin_shot(Shot { position: Vec3::ZERO, look_at: Vec3::new(0.0, 0.8, -1.0), blend: 0.0, fov: Some(60.0) });
    let f = &*font;
    commands
        .spawn((
            DespawnOnExit(AppState::Ending),
            Node { width: percent(100), height: percent(100), position_type: PositionType::Absolute, flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: px(16), ..default() },
            GlobalZIndex(20),
        ))
        .with_children(|p| {
            p.spawn((EndingLine, Text::new(""), f.tf(28.0), TextColor(theme::WHITE.with_alpha(0.0)), TextLayout::justify(Justify::Center), Node { max_width: px(900), ..default() }, Pickable::IGNORE));
            p.spawn((EndingFinal, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(14), display: Display::None, ..default() }))
                .with_children(|c| {
                    c.spawn((Text::new(""), Tr::new("ending.fin"), f.tf(72.0), TextColor(theme::SOFT_GOLD), Pickable::IGNORE));
                    c.spawn((Text::new(""), Tr::new("ending.thanks"), f.tf(22.0), TextColor(theme::WHITE), Pickable::IGNORE));
                    c.spawn((Text::new(""), Tr::new("ending.route"), f.tf(17.0), TextColor(theme::DIM), Node { margin: UiRect::top(px(16)), ..default() }, Pickable::IGNORE));
                    let route: Vec<String> = reg.infos().iter().map(|i| i18n.t(i.title).to_string()).collect();
                    c.spawn((Text::new(route.join("  ·  ")), f.tf(17.0), TextColor(theme::DIM), TextLayout::justify(Justify::Center), Node { max_width: px(1000), ..default() }, Pickable::IGNORE));
                    c.spawn(button(f, "ending.back", 22.0, UiAction::EndingBack)).insert(Node {
                        margin: UiRect::top(px(24)),
                        padding: UiRect::axes(px(26), px(10)),
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(px(22)),
                        min_width: px(180),
                        ..default()
                    });
                });
        });
}

fn update(
    time: Res<Time<Real>>,
    mut st: ResMut<EndingState>,
    i18n: Res<I18n>,
    input: Res<super::super::input::InputFrame>,
    mut line: Query<(&mut Text, &mut TextColor), With<EndingLine>>,
    mut fin: Query<&mut Node, With<EndingFinal>>,
    mut msgs: MessageReader<UiActionMsg>,
    mut flow: MessageWriter<Flow>,
    params: Res<super::super::params::LaunchParams>,
) {
    let speed = if params.test { 6.0 } else { 1.0 };
    st.t += time.delta_secs() * speed;
    if input.advance && !st.shown_final {
        st.t = (st.t / PER_LINE).floor() * PER_LINE + PER_LINE - 0.6;
    }
    let idx = (st.t / PER_LINE) as usize;
    let local = st.t - idx as f32 * PER_LINE;
    if let Ok((mut t, mut c)) = line.single_mut() {
        if idx < LINES.len() {
            let s = i18n.t(LINES[idx]);
            if t.0 != s {
                t.0 = s.to_string();
            }
            let a = (local / 0.8).min(1.0).min(((PER_LINE - local) / 0.8).max(0.0));
            c.0 = theme::WHITE.with_alpha(a);
        } else {
            c.0 = theme::WHITE.with_alpha(0.0);
            if !st.shown_final {
                st.shown_final = true;
                if let Ok(mut n) = fin.single_mut() {
                    n.display = Display::Flex;
                }
                if params.test {
                    super::super::testing::report_ok("ending");
                }
            }
        }
    }
    for UiActionMsg(a) in msgs.read() {
        if *a == UiAction::EndingBack {
            flow.write(Flow::ToTitle);
        }
    }
}

fn exit(mut rig: ResMut<CameraRig>, mut sky: ResMut<Sky>) {
    rig.focus_override = None;
    rig.shot = None;
    sky.fixed_night = None;
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Ending), enter)
        .add_systems(OnExit(AppState::Ending), exit)
        .add_systems(Update, update.run_if(in_state(AppState::Ending)));
}
