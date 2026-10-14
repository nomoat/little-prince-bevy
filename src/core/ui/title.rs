//! Title screen: a little planet turning against a dusk sky, the title, and
//! Begin / Continue / Settings / language buttons.

use super::super::camera::{CameraRig, Shot};
use super::super::i18n::{I18n, Lang, Tr};
use super::super::meshkit::{MeshBuilder, hex};
use super::super::models::SharedMaterials;
use super::super::save::SaveData;
use super::super::sky::Sky;
use super::super::state::{AppState, Flow};
use super::{UiAction, UiActionMsg, UiFont, button, theme};
use bevy::picking::Pickable;
use bevy::prelude::*;

#[derive(Component)]
struct TitlePlanet;

fn enter_title(mut commands: Commands, font: Res<UiFont>, save: Res<SaveData>, shared: Res<SharedMaterials>, mut meshes: ResMut<Assets<Mesh>>, mut rig: ResMut<CameraRig>, mut sky: ResMut<Sky>, mut audio: MessageWriter<super::super::audio::AudioCmd>) {
    *sky = Sky::default();
    sky.set_hour(17.6);
    sky.set_rate(0.0, None);
    audio.write(super::super::audio::AudioCmd::music("music.title"));
    // backdrop planet
    let mut m = MeshBuilder::new();
    m.sphere(Vec3::ZERO, 3.0, 9, 16, hex(0x9fb36c));
    m.sphere(Vec3::new(0.0, -0.4, 0.0), 2.95, 7, 14, hex(0x7d6a58));
    for (i, (a, b)) in [(0.3f32, 0.4f32), (1.8, 0.9), (3.6, 0.2)].iter().enumerate() {
        let d = Vec3::new(a.cos() * b.cos(), b.sin(), a.sin() * b.cos()).normalize();
        m.frustum(d * 2.9, 0.45, 0.18, 0.5 + i as f32 * 0.1, 7, hex(0x7a6a62));
    }
    let rose = Vec3::new(0.2, 1.0, 0.3).normalize() * 3.0;
    m.tube(rose, rose * 1.12, 0.02, 0.02, 4, hex(0x4f8a45));
    m.sphere(rose * 1.13, 0.09, 3, 6, hex(0xd8434e));
    let mesh = meshes.add(m.build());
    commands.spawn((TitlePlanet, Mesh3d(mesh), MeshMaterial3d(shared.vertex.clone()), Transform::from_xyz(0.0, 0.0, 0.0), DespawnOnExit(AppState::Title)));
    rig.focus_override = Some(Vec3::ZERO);
    // Stand on the sun's side (rotated a little) so the planet is lit warmly.
    let (_, az) = sky.sun_position(17.6);
    let a = (az - 35.0).to_radians();
    let cam = Vec3::new(a.sin() * 9.0, 1.8, a.cos() * 9.0);
    let right = Vec3::Y.cross(-cam).normalize();
    rig.begin_shot(Shot { position: cam, look_at: -right * 2.8 + Vec3::Y * 0.5, blend: 0.0, fov: Some(42.0) });

    let f = &*font;
    commands
        .spawn((
            DespawnOnExit(AppState::Title),
            Node { width: percent(100), height: percent(100), position_type: PositionType::Absolute, flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::FlexStart, padding: UiRect::left(percent(56)), row_gap: px(14), ..default() },
            GlobalZIndex(20),
        ))
        .with_children(|p| {
            p.spawn((Text::new(""), Tr::new("core.game_title"), f.tf(84.0), TextColor(theme::WHITE), TextShadow { offset: Vec2::new(2.0, 3.0), color: theme::SHADOW }, Pickable::IGNORE));
            p.spawn((Text::new(""), Tr::new("core.game_subtitle"), f.tf(32.0), TextColor(theme::SOFT_GOLD), TextShadow::default(), Node { margin: UiRect::bottom(px(30)), ..default() }, Pickable::IGNORE));
            if save.has_started() {
                p.spawn(button(f, "core.title.continue", 24.0, UiAction::TitleContinue));
            }
            p.spawn(button(f, "core.title.begin", 24.0, UiAction::TitleBegin));
            p.spawn(button(f, "core.title.settings", 22.0, UiAction::OpenSettings));
            p.spawn(Node { column_gap: px(10), ..default() }).with_children(|r| {
                r.spawn(super::small_button(f, "中", UiAction::SetLang(Lang::Zh)));
                r.spawn(super::small_button(f, "EN", UiAction::SetLang(Lang::En)));
            });
        });
    commands.spawn((
        DespawnOnExit(AppState::Title),
        Text::new(""),
        Tr::new("core.title.foot"),
        f.tf(15.0),
        TextColor(theme::DIM),
        Node { position_type: PositionType::Absolute, bottom: px(18), left: px(24), ..default() },
        Pickable::IGNORE,
    ));
}

fn title_actions(mut msgs: MessageReader<UiActionMsg>, mut flow: MessageWriter<Flow>, mut save: ResMut<SaveData>, keys: Res<ButtonInput<KeyCode>>, menu: Res<super::menu::MenuState>) {
    for UiActionMsg(a) in msgs.read() {
        match a {
            UiAction::TitleBegin => {
                if !save.has_started() {
                    save.reveal("b612");
                }
                flow.write(Flow::ToMap);
            }
            UiAction::TitleContinue => {
                flow.write(Flow::ToMap);
            }
            _ => {}
        }
    }
    if keys.just_pressed(KeyCode::Enter) && !menu.open {
        save.reveal("b612");
        flow.write(Flow::ToMap);
    }
}

fn spin(time: Res<Time<Real>>, mut q: Query<&mut Transform, With<TitlePlanet>>) {
    for mut t in &mut q {
        t.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.08) * Quat::from_rotation_z(0.25);
    }
}

fn exit_title(mut rig: ResMut<CameraRig>) {
    rig.focus_override = None;
    rig.end_shot(0.0);
    rig.shot = None;
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Title), enter_title)
        .add_systems(OnExit(AppState::Title), exit_title)
        .add_systems(Update, (title_actions, spin).run_if(in_state(AppState::Title)));
    let _ = I18n::default;
}
