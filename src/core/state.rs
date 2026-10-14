//! Scene/chapter state machine and the journey flow (JourneyFlow +
//! JourneyProgress + JourneyDeparture in the original):
//! Boot → Title → Map → Loading → Chapter → (hold E to set off) → Map … →
//! Ending. Transitions go through an ink-wash fade.

use super::audio::AudioCmd;
use super::camera::{CameraRig, CameraTuning};
use super::chapter::{ChapterCx, ChapterRegistry};
use super::i18n::I18n;
use super::input::InputFrame;
use super::interact::InteractFocus;
use super::params::LaunchParams;
use super::planet::Terrain;
use super::player::{Drive, Player};
use super::save::SaveData;
use super::sky::Sky;
use super::story::{Anchors, StoryLock, StoryRunner};
use super::ui::hud::{Fade, Hud};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Boot,
    Title,
    Map,
    /// One-frame hop so re-entering a chapter re-runs OnExit/OnEnter.
    Loading,
    Chapter,
    Ending,
}

/// Despawned when the chapter ends.
#[derive(Component, Default, Clone, Copy)]
pub struct ChapterScoped;

#[derive(Resource, Default, Debug, Clone)]
pub struct ActiveChapter {
    pub id: Option<String>,
    /// Planet the map should animate the birds from (just left).
    pub returning_from: Option<String>,
}

#[derive(Message, Clone, Debug)]
pub enum Flow {
    ToTitle,
    ToMap,
    Enter(String),
    ToEnding,
    /// The prince sets off from the current planet (departure).
    Depart,
}

#[derive(Resource, Default)]
pub struct Transition {
    pending: Option<Flow>,
    /// A request made while the new scene was still fading in; it runs as
    /// soon as the fade ends (so an early click or key press is not lost).
    queued: Option<Flow>,
    phase: u8,
    frames: u32,
}

impl Transition {
    pub fn busy(&self) -> bool {
        self.phase != 0
    }
}

/// Departure hold progress (hold E for 1.6 s, DepartHoldSeconds).
#[derive(Resource, Default)]
pub struct DepartHold {
    pub progress: f32,
    pub departing: f32,
}

pub const DEPART_HOLD_SECONDS: f32 = 1.6;

/// JourneyProgress.Leave: mark visited and reveal the next stop.
pub fn leave_planet(save: &mut SaveData, reg: &ChapterRegistry, id: &str) {
    save.reveal(id);
    save.visit(id);
    if let Some(i) = reg.index_of(id) {
        if let Some(next) = reg.0.get(i + 1) {
            save.reveal(next.info().id);
        }
    }
}

pub(crate) fn handle_flow(
    mut msgs: MessageReader<Flow>,
    mut tr: ResMut<Transition>,
    mut fade: ResMut<Fade>,
    mut next: ResMut<NextState<AppState>>,
    mut active: ResMut<ActiveChapter>,
    state: Res<State<AppState>>,
    reg: Res<ChapterRegistry>,
    i18n: Res<I18n>,
    params: Res<LaunchParams>,
    mut save: ResMut<SaveData>,
    mut audio: MessageWriter<AudioCmd>,
) {
    let quick = params.test;
    if tr.phase == 0 {
        let incoming = msgs.read().last().cloned();
        if let Some(f) = incoming.or_else(|| tr.queued.take()) {
            let f = match f {
                Flow::Depart => {
                    // progress + where to go next
                    let id = active.id.clone().unwrap_or_default();
                    leave_planet(&mut save, &reg, &id);
                    active.returning_from = Some(id.clone());
                    audio.write(AudioCmd::sfx("sfx.depart"));
                    if params.test {
                        super::testing::report_ok(&id);
                    }
                    if id == reg.last_id() { Flow::ToEnding } else { Flow::ToMap }
                }
                f => f,
            };
            fade.text = None;
            fade.sub = None;
            if let Flow::Enter(id) = &f {
                if let Some(info) = reg.info(id) {
                    fade.text = Some(i18n.t(info.title).to_string());
                    fade.sub = Some(i18n.t(info.loading).to_string());
                }
            }
            let from_boot = *state.get() == AppState::Boot;
            if from_boot {
                fade.alpha = 1.0;
            }
            fade.to(1.0, if quick { 0.15 } else { 0.7 });
            tr.pending = Some(f);
            tr.phase = 1;
            tr.frames = 0;
        }
        return;
    }
    if tr.phase == 3 {
        if let Some(f) = msgs.read().last().cloned() {
            tr.queued = Some(f);
        }
    } else {
        msgs.clear();
    }
    match tr.phase {
        1 if fade.done() => {
            match tr.pending.clone() {
                Some(Flow::ToTitle) => next.set(AppState::Title),
                Some(Flow::ToMap) => next.set(AppState::Map),
                Some(Flow::ToEnding) => next.set(AppState::Ending),
                Some(Flow::Enter(id)) => {
                    active.id = Some(id.clone());
                    active.returning_from = None;
                    next.set(AppState::Loading);
                }
                _ => {}
            }
            tr.phase = 2;
            tr.frames = 0;
        }
        2 => {
            tr.frames += 1;
            let ready = match state.get() {
                AppState::Loading => false,
                _ => tr.frames > 3,
            };
            if ready {
                fade.to(0.0, if quick { 0.2 } else { 0.9 });
                tr.phase = 3;
            }
        }
        3 if fade.done() => {
            tr.phase = 0;
            tr.pending = None;
        }
        _ => {}
    }
}

pub(crate) fn loading_hop(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Chapter);
}

/// OnEnter(Chapter): reset core state, build the chapter, start its story.
pub(crate) fn enter_chapter(world: &mut World) {
    let id = world.resource::<ActiveChapter>().id.clone().unwrap_or_else(|| "b612".into());
    let Some(ch) = world.resource::<ChapterRegistry>().get(&id) else {
        error!("unknown chapter '{id}'");
        world.write_message(Flow::ToMap);
        return;
    };
    let info = ch.info();
    world.resource_mut::<Hud>().reset();
    world.insert_resource(Anchors::default());
    world.resource_mut::<StoryLock>().0 = false;
    *world.resource_mut::<InteractFocus>() = InteractFocus::default();
    *world.resource_mut::<DepartHold>() = DepartHold::default();
    *world.resource_mut::<Sky>() = Sky::default();
    // Every chapter starts from the default camera tuning (chapters that
    // change it in `build` do so after this).
    world.insert_resource(CameraTuning::default());
    world.insert_resource(super::audio::StepZones::default());
    let tuning = CameraTuning::default();
    world.resource_mut::<CameraRig>().reset(&tuning);
    *world.resource_mut::<super::player::PlayerTuning>() = super::player::PlayerTuning::default();
    {
        let mut save = world.resource_mut::<SaveData>();
        save.reveal(info.id);
        save.last = Some(info.id.to_string());
    }
    let test = world.resource::<LaunchParams>().test;
    world.write_message(AudioCmd::Music(Some(info.music.to_string())));
    {
        let mut cx = ChapterCx { world, id: info.id, test };
        ch.build(&mut cx);
    }
    let story = ch.story();
    world.resource_mut::<StoryRunner>().begin(info.id, story);
    super::platform::set_page_title(&format!("littleprince · {}", info.id));
    info!("chapter '{}' started", info.id);
}

pub(crate) fn exit_chapter(world: &mut World) {
    let mut q = world.query_filtered::<Entity, With<ChapterScoped>>();
    let es: Vec<Entity> = q.iter(world).collect();
    for e in es {
        if let Ok(em) = world.get_entity_mut(e) {
            em.despawn();
        }
    }
    world.remove_resource::<Terrain>();
    world.insert_resource(Anchors::default());
    world.resource_mut::<Hud>().reset();
    world.resource_mut::<StoryRunner>().running = false;
    world.write_message(AudioCmd::Ambience(None));
    world.resource_mut::<CameraRig>().shot = None;
    world.resource_mut::<super::input::TouchControls>().action2_label = None;
    world.resource_mut::<Time<Virtual>>().unpause();
}

/// Hold E (no interactable focused) for 1.6 s once departure is open.
#[allow(clippy::too_many_arguments)]
pub(crate) fn departure(
    time: Res<Time>,
    input: Res<InputFrame>,
    focus: Res<InteractFocus>,
    mut runner: ResMut<StoryRunner>,
    mut hold: ResMut<DepartHold>,
    mut flow: MessageWriter<Flow>,
    tr: Res<Transition>,
    mut player: Query<(&mut Player, &mut Transform)>,
    hud: Res<Hud>,
) {
    let dt = time.delta_secs();
    if hold.departing > 0.0 {
        // lifted away by the birds
        hold.departing += dt;
        if let Ok((mut p, mut t)) = player.single_mut() {
            p.drive = Drive::External;
            t.translation.y += dt * (0.5 + hold.departing * 2.0);
        }
        if hold.departing > 1.4 && !tr.busy() {
            flow.write(Flow::Depart);
            hold.departing = -100.0;
        }
        return;
    }
    if hold.departing < 0.0 {
        return;
    }
    if !runner.departure_ready {
        hold.progress = 0.0;
        return;
    }
    // not while the prince is busy (sitting, looking through a telescope …):
    // E belongs to that activity then
    let free = player.single().map(|(p, _)| matches!(p.drive, Drive::Player)).unwrap_or(true);
    let holding = input.interact_held && focus.entity.is_none() && hud.dialogue.is_none() && free;
    if holding {
        hold.progress = (hold.progress + dt / DEPART_HOLD_SECONDS).min(1.0);
    } else {
        hold.progress = (hold.progress - dt * 2.0).max(0.0);
    }
    if hold.progress >= 1.0 || runner.leave_requested {
        runner.leave_requested = false;
        hold.departing = 0.001;
    }
}

/// Boot: wait for the font + manifests, then route by URL params.
#[allow(clippy::too_many_arguments)]
pub(crate) fn boot(
    mut timer: Local<f32>,
    mut done: Local<bool>,
    time: Res<Time<Real>>,
    server: Res<AssetServer>,
    font: Res<super::ui::UiFont>,
    loading: Res<super::manifest::ManifestLoading>,
    params: Res<LaunchParams>,
    reg: Res<ChapterRegistry>,
    mut flow: MessageWriter<Flow>,
    mut save: ResMut<SaveData>,
) {
    if *done {
        return;
    }
    *timer += time.delta_secs();
    let font_ok = server.is_loaded_with_dependencies(&font.0) || matches!(server.load_state(&font.0), bevy::asset::LoadState::Failed(_));
    let man_ok = loading.merged;
    if !(font_ok && man_ok) && *timer < 8.0 {
        return;
    }
    *done = true;
    super::platform::notify_ready();
    if params.free {
        save.settings.free_travel = true;
    }
    if let Some(id) = &params.chapter {
        if reg.get(id).is_some() {
            flow.write(Flow::Enter(id.clone()));
            return;
        }
        super::platform::console_log(&format!("[test:fail {id}] unknown chapter"));
        super::testing::TEST_RESULT.store(2, std::sync::atomic::Ordering::SeqCst);
    }
    if params.ending {
        flow.write(Flow::ToEnding);
    } else if params.map {
        flow.write(Flow::ToMap);
    } else {
        flow.write(Flow::ToTitle);
    }
}
