//! Test autoplay harness (`?test=1`), modelled on the original's
//! AgentInput / AgentInputDriver / PlaytestRoute: a driver that plays the
//! current story wait the way a player would — walk to the interactable,
//! face it, press or hold E; advance dialogue; pick choices; run the
//! chapter's [`TestAction`] hints for custom waits; hold E to depart.
//! Prints `[test:ok <chapter>]` (at departure) or `[test:fail <chapter>] reason`.

use super::input::AgentInput;
use super::interact::{InteractFocus, Interactable};
use super::params::LaunchParams;
use super::platform;
use super::player::Player;
use super::state::AppState;
use super::story::{StoryRunner, TestAction, Waiting};
use super::ui::hud::Hud;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Autopilot {
    pc: usize,
    wait_t: f32,
    total_t: f32,
    hint_idx: usize,
    hint_t: f32,
    pulse: u32,
    last_dist: f32,
    stuck_t: f32,
    failed: bool,
    holding_t: f32,
}

/// Per-wait timeout (s, game time) and total chapter timeout.
pub const WAIT_TIMEOUT: f32 = 60.0;
pub const TOTAL_TIMEOUT: f32 = 420.0;

/// Set once a test result was printed: 1 = ok, 2 = fail. Native test runs exit on it.
pub static TEST_RESULT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Full-journey mode (`?flow=1`): per-chapter results do not end the run.
pub static FLOW_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Set when the ending screen reached its final card (test mode).
pub static ENDING_SHOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn report_ok(id: &str) {
    let msg = format!("[test:ok {id}]");
    platform::console_log(&msg);
    platform::set_page_title(&msg);
    if id == "ending" {
        ENDING_SHOWN.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    if !FLOW_MODE.load(std::sync::atomic::Ordering::SeqCst) {
        TEST_RESULT.store(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Default)]
pub(crate) struct FlowDriver {
    state: Option<AppState>,
    t: f32,
    sent: bool,
    step: u8,
    done: bool,
    entered: Vec<String>,
}

fn flow_result(d: &mut FlowDriver, ok: bool, reason: &str) {
    if d.done {
        return;
    }
    d.done = true;
    let msg = if ok { "[flow:ok]".to_string() } else { format!("[flow:fail] {reason}") };
    platform::console_log(&msg);
    platform::set_page_title(&msg);
    TEST_RESULT.store(if ok { 1 } else { 2 }, std::sync::atomic::Ordering::SeqCst);
}

/// `?flow=1&test=1`: plays the whole journey through the real UI actions —
/// Begin on the title, the frontier planet on the map (checking the unlock
/// rules: exactly the next planet is selectable), each chapter by the
/// autopilot, the ending, then back to the title (which must offer Continue).
#[allow(clippy::too_many_arguments)]
pub(crate) fn flow_driver(
    mut d: Local<FlowDriver>,
    params: Res<LaunchParams>,
    state: Res<State<AppState>>,
    time: Res<Time<Real>>,
    tr: Res<super::state::Transition>,
    ms: Res<super::ui::map::MapState>,
    save: Res<super::save::SaveData>,
    reg: Res<super::chapter::ChapterRegistry>,
    active: Res<super::state::ActiveChapter>,
    mut ui: MessageWriter<super::ui::UiActionMsg>,
) {
    use super::ui::UiAction;
    use super::ui::map::{PlanetState, planet_state};
    if !(params.flow && params.test) || d.done {
        return;
    }
    FLOW_MODE.store(true, std::sync::atomic::Ordering::SeqCst);
    let st = state.get().clone();
    if d.state.as_ref() != Some(&st) {
        d.state = Some(st.clone());
        d.t = 0.0;
        d.sent = false;
        d.step = 0;
        platform::console_log(&format!("[flow] state {st:?}"));
        if st == AppState::Chapter {
            let id = active.id.clone().unwrap_or_default();
            d.entered.push(id);
        }
    }
    d.t += time.delta_secs();
    if TEST_RESULT.load(std::sync::atomic::Ordering::SeqCst) == 2 {
        flow_result(&mut d, false, "a chapter test failed");
        return;
    }
    if tr.busy() || d.t < 1.2 {
        return;
    }
    let ids: Vec<&str> = reg.infos().iter().map(|i| i.id).collect::<Vec<_>>();
    match st {
        AppState::Title if !d.sent => {
            d.sent = true;
            if ENDING_SHOWN.load(std::sync::atomic::Ordering::SeqCst) {
                // back on the title after the ending
                let order_ok = d.entered.iter().map(|s| s.as_str()).eq(ids.iter().copied());
                if !order_ok {
                    let why = format!("chapter order {:?}", d.entered);
                    flow_result(&mut d, false, &why);
                } else if !save.has_started() || ids.iter().any(|id| !save.is_visited(id)) {
                    flow_result(&mut d, false, "progress lost after ending");
                } else {
                    flow_result(&mut d, true, "");
                }
            } else {
                ui.write(super::ui::UiActionMsg(UiAction::TitleBegin));
            }
        }
        AppState::Map if d.step == 0 => {
            d.step = 1;
            let Some(next) = ids.iter().find(|id| !save.is_visited(id)).copied() else {
                flow_result(&mut d, false, "map shown after every planet was visited");
                return;
            };
            let ni = ids.iter().position(|x| *x == next).unwrap_or(0);
            for (i, id) in ids.iter().enumerate() {
                let ps = planet_state(&save, &reg, id);
                let want_ok = match i.cmp(&ni) {
                    std::cmp::Ordering::Less => ps == PlanetState::Visited,
                    std::cmp::Ordering::Equal => ps == PlanetState::Lit,
                    std::cmp::Ordering::Greater => matches!(ps, PlanetState::Fog | PlanetState::Teaser),
                };
                if !want_ok {
                    flow_result(&mut d, false, &format!("map: {id} is {ps:?} (next is {next})"));
                    return;
                }
            }
            if ms.selected.as_deref() != Some(next) {
                flow_result(&mut d, false, &format!("map selected {:?}, expected {next}", ms.selected));
                return;
            }
            // Try to start a locked planet (no free travel): select it twice
            // and press depart. Nothing may happen.
            if let Some(locked) = ids.get(ni + 1) {
                for _ in 0..2 {
                    ui.write(super::ui::UiActionMsg(UiAction::MapSelect(locked.to_string())));
                }
                ui.write(super::ui::UiActionMsg(UiAction::MapDepart));
            }
        }
        AppState::Map if d.step == 1 && d.t > 2.6 => {
            d.step = 2;
            d.sent = true;
            let next = ids.iter().find(|id| !save.is_visited(id)).copied().unwrap_or_default();
            if tr.busy() {
                flow_result(&mut d, false, "a locked planet could be started");
                return;
            }
            ui.write(super::ui::UiActionMsg(UiAction::MapSelect(next.to_string())));
            ui.write(super::ui::UiActionMsg(UiAction::MapDepart));
        }
        AppState::Ending if !d.sent && ENDING_SHOWN.load(std::sync::atomic::Ordering::SeqCst) => {
            d.sent = true;
            ui.write(super::ui::UiActionMsg(UiAction::EndingBack));
        }
        _ => {}
    }
    if d.t > 30.0 && matches!(st, AppState::Title | AppState::Map | AppState::Ending) && d.sent {
        flow_result(&mut d, false, &format!("stuck on {st:?}"));
    }
}

/// Native only: quit shortly after a test result (so `cargo r -- --chapter x --test` terminates).
pub(crate) fn exit_when_done(mut t: Local<f32>, time: Res<Time<Real>>, mut exit: MessageWriter<AppExit>) {
    let r = TEST_RESULT.load(std::sync::atomic::Ordering::SeqCst);
    if r == 0 || platform::is_web() {
        return;
    }
    *t += time.delta_secs();
    if *t > 1.0 {
        exit.write(if r == 1 { AppExit::Success } else { AppExit::error() });
    }
}

/// Native dev aid: `--shot 6,12,20` saves window screenshots at those seconds
/// (real time) to `$LP_SHOT_DIR` (default `tests/out/native`) as `<tag>-<sec>.png`;
/// `--shot_exit 1` quits after the last one.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn native_shots(mut commands: Commands, params: Res<LaunchParams>, time: Res<Time<Real>>, mut taken: Local<usize>, mut exit: MessageWriter<AppExit>) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let Some(list) = params.get("shot") else { return };
    let times: Vec<f32> = list.split(',').filter_map(|v| v.parse().ok()).collect();
    if *taken >= times.len() {
        if params.get("shot_exit").is_some() && time.elapsed_secs() > times.last().copied().unwrap_or(0.0) + 1.5 {
            exit.write(AppExit::Success);
        }
        return;
    }
    if time.elapsed_secs() >= times[*taken] {
        let dir = std::env::var("LP_SHOT_DIR").unwrap_or_else(|_| "tests/out/native".into());
        let _ = std::fs::create_dir_all(&dir);
        let tag = params.get("tag").or(params.chapter.as_deref()).unwrap_or("shot").to_string();
        let path = format!("{dir}/{tag}-{:03}.png", times[*taken] as i32);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        *taken += 1;
    }
}

fn fail(ap: &mut Autopilot, chapter: &str, reason: &str) {
    if !ap.failed {
        TEST_RESULT.store(2, std::sync::atomic::Ordering::SeqCst);
        ap.failed = true;
        let msg = format!("[test:fail {chapter}] {reason}");
        platform::console_log(&msg);
        platform::set_page_title(&msg);
    }
}

pub(crate) fn autopilot(world: &mut World) {
    if !world.resource::<LaunchParams>().test || *world.resource::<State<AppState>>().get() != AppState::Chapter {
        return;
    }
    let dt = world.resource::<Time>().delta_secs();
    let mut agent = AgentInput { active: true, ..default() };
    let (pc, waiting, chapter, running, ready) = {
        let r = world.resource::<StoryRunner>();
        (r.pc, r.waiting.clone(), r.chapter.clone(), r.running, r.departure_ready)
    };
    {
        let mut ap = world.resource_mut::<Autopilot>();
        if ap.failed {
            *world.resource_mut::<AgentInput>() = agent;
            return;
        }
        if ap.pc != pc {
            ap.pc = pc;
            ap.wait_t = 0.0;
            ap.hint_idx = 0;
            ap.hint_t = 0.0;
            ap.stuck_t = 0.0;
            ap.last_dist = f32::MAX;
            ap.holding_t = 0.0;
        }
        ap.wait_t += dt;
        ap.total_t += dt;
        ap.pulse += 1;
        if ap.total_t > TOTAL_TIMEOUT {
            let desc = waiting.as_ref().map(|w| w.describe()).unwrap_or_default();
            fail(&mut ap, &chapter, &format!("total timeout at pc {pc} ({desc})"));
        } else if ap.wait_t > WAIT_TIMEOUT && !matches!(waiting, Some(Waiting::Departure)) {
            let desc = waiting.as_ref().map(|w| w.describe()).unwrap_or_else(|| "no wait".into());
            fail(&mut ap, &chapter, &format!("stuck at pc {pc}: {desc}"));
        }
    }
    if !running && !ready {
        let t = world.resource::<Autopilot>().wait_t;
        if t > 5.0 {
            let mut ap = world.resource_mut::<Autopilot>();
            fail(&mut ap, &chapter, &format!("script ended without departure (pc {pc})"));
        }
    }
    let pulse = world.resource::<Autopilot>().pulse % 6 == 0;
    // dialogue / card / banner / narration: advance
    {
        let hud = world.resource::<Hud>();
        if hud.dialogue.is_some() || hud.card.is_some() || hud.narration.is_some() {
            agent.advance = pulse;
        }
    }
    match waiting {
        Some(Waiting::Choice(_, pick)) => {
            let ready = world.resource::<Autopilot>().wait_t > 0.3;
            if let Some(c) = world.resource_mut::<Hud>().choice.as_mut() {
                if ready {
                    c.1 = Some(pick.min(c.0.len().saturating_sub(1)));
                }
            }
        }
        Some(Waiting::Use(id)) => drive_use(world, &mut agent, id, dt),
        Some(Waiting::Near(p, r)) => {
            if let Some(t) = p.resolve(world) {
                drive_to(world, &mut agent, t, r * 0.6, dt);
            }
        }
        Some(Waiting::Flag(_, hints)) | Some(Waiting::Until(_, hints)) => run_hints(world, &mut agent, &hints, dt),
        Some(Waiting::Departure) => {
            let focused = world.resource::<InteractFocus>().entity.is_some();
            if focused {
                // step away from whatever is focused
                let pp = player(world).map(|p| p.0);
                if let Some(pp) = pp {
                    let away = Vec2::new(pp.x, pp.z).normalize_or_zero();
                    agent.move_world = Some(if away == Vec2::ZERO { Vec2::X } else { -away });
                }
            } else {
                agent.interact_held = true;
            }
        }
        _ => {}
    }
    *world.resource_mut::<AgentInput>() = agent;
}

fn player(world: &mut World) -> Option<(Vec3, f32)> {
    let mut q = world.query::<(&Transform, &Player)>();
    q.iter(world).next().map(|(t, p)| (t.translation, p.yaw))
}

fn face(world: &mut World, target: Vec3) {
    let mut q = world.query::<(&Transform, &mut Player)>();
    if let Some((t, mut p)) = q.iter_mut(world).next() {
        let d = target - t.translation;
        p.yaw = d.x.atan2(d.z);
    }
}

fn teleport_near(world: &mut World, target: Vec3, dist: f32) {
    let terrain_h = world.get_resource::<super::planet::Terrain>().cloned();
    let mut q = world.query::<(&mut Transform, &mut Player)>();
    if let Some((mut t, mut p)) = q.iter_mut(world).next() {
        let from = Vec2::new(t.translation.x - target.x, t.translation.z - target.z).normalize_or(Vec2::X);
        let pos2 = Vec2::new(target.x, target.z) + from * dist;
        let y = terrain_h.as_ref().map(|tr| tr.height(pos2.x, pos2.y)).unwrap_or(target.y).max(target.y - 0.2);
        t.translation = Vec3::new(pos2.x, y + 0.05, pos2.y);
        p.velocity = Vec3::ZERO;
        p.grounded = false;
        let d = target - t.translation;
        p.yaw = d.x.atan2(d.z);
    }
}

/// Walk toward `target`; returns true when within `stop`. Teleports if stuck.
fn drive_to(world: &mut World, agent: &mut AgentInput, target: Vec3, stop: f32, dt: f32) -> bool {
    let Some((pp, _)) = player(world) else { return false };
    let d = Vec2::new(target.x - pp.x, target.z - pp.z);
    let dist = d.length();
    if dist <= stop.max(0.3) {
        return true;
    }
    agent.move_world = Some(d / dist);
    agent.run = dist > 4.0;
    let mut ap = world.resource_mut::<Autopilot>();
    if dist < ap.last_dist - 0.05 {
        ap.last_dist = dist;
        ap.stuck_t = 0.0;
    } else {
        ap.stuck_t += dt;
    }
    if ap.stuck_t > 2.5 {
        ap.stuck_t = 0.0;
        ap.last_dist = f32::MAX;
        teleport_near(world, target, stop.max(0.3) * 0.8);
    }
    false
}

fn drive_use(world: &mut World, agent: &mut AgentInput, id: &str, dt: f32) {
    let mut q = world.query::<(Entity, &Interactable, &GlobalTransform)>();
    let found = q.iter(world).find(|(_, it, _)| it.id == id).map(|(e, it, gt)| (e, it.clone(), gt.translation() + it.focus_offset));
    let Some((e, it, fp)) = found else { return };
    if !it.enabled {
        return;
    }
    let focus = world.resource::<InteractFocus>().entity;
    if focus == Some(e) {
        face(world, fp);
        if it.hold_seconds > 0.0 {
            agent.interact_held = true;
        } else {
            let mut ap = world.resource_mut::<Autopilot>();
            ap.holding_t += dt;
            agent.interact = ap.pulse % 8 == 0;
        }
        return;
    }
    let near = drive_to(world, agent, fp, (it.radius * 0.55).max(0.5), dt);
    if near {
        face(world, fp);
        let mut ap = world.resource_mut::<Autopilot>();
        ap.holding_t += dt;
        if ap.holding_t > 1.5 {
            // facing / side rules may need another approach angle
            ap.holding_t = 0.0;
            teleport_near(world, fp, (it.radius * 0.5).max(0.45));
        }
    }
}

fn run_hints(world: &mut World, agent: &mut AgentInput, hints: &[TestAction], dt: f32) {
    let (idx, t) = {
        let ap = world.resource::<Autopilot>();
        (ap.hint_idx, ap.hint_t)
    };
    let Some(h) = hints.get(idx) else { return };
    world.resource_mut::<Autopilot>().hint_t += dt;
    let next = match h {
        TestAction::Use(id) => {
            drive_use(world, agent, id, dt);
            world.resource::<StoryRunner>().used_now.iter().any(|u| u == id)
        }
        TestAction::GoTo(p) => match p.resolve(world) {
            Some(tp) => drive_to(world, agent, tp, 0.6, dt),
            None => true,
        },
        TestAction::PressJump => {
            agent.jump = true;
            true
        }
        TestAction::PressInteract => {
            agent.interact = true;
            true
        }
        TestAction::PressAction2 => {
            agent.action2 = true;
            true
        }
        TestAction::Wait(s) => t >= *s,
        TestAction::Run(f) => {
            f(world);
            true
        }
    };
    if next {
        let mut ap = world.resource_mut::<Autopilot>();
        ap.hint_idx += 1;
        ap.hint_t = 0.0;
        ap.stuck_t = 0.0;
        ap.last_dist = f32::MAX;
    }
}

/// `?probe=<seconds>` (`1` = every second): prints `[probe] {json}` with the app state,
/// player position/yaw, camera orbit, menu and touch state, the language and
/// the save's progress — for browser tests that drive real input (touch,
/// keyboard) and need to see the effect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn probe(
    mut t: Local<f32>,
    params: Res<LaunchParams>,
    time: Res<Time<Real>>,
    state: Res<State<AppState>>,
    rig: Res<super::camera::CameraRig>,
    menu: Res<super::ui::menu::MenuState>,
    touch: Res<super::input::TouchControls>,
    i18n: Res<super::i18n::I18n>,
    save: Res<super::save::SaveData>,
    hud: Res<Hud>,
    active: Res<super::state::ActiveChapter>,
    player: Query<(&Transform, &Player)>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    extra: (Option<Res<StoryRunner>>, Res<InteractFocus>, Res<super::input::InputGate>),
) {
    // `?probe=1` → every second; `?probe=0.25` → every 0.25 s.
    let Some(every) = params.get("probe").map(|v| v.parse::<f32>().unwrap_or(1.0).clamp(0.05, 10.0)) else { return };
    *t += time.delta_secs();
    if *t < every {
        return;
    }
    *t = 0.0;
    let (pos, yaw, grounded) = player.single().map(|(tf, p)| (tf.translation, p.yaw, p.grounded)).unwrap_or((Vec3::ZERO, 0.0, false));
    let list = |v: &Vec<String>| v.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(",");
    let msg = format!(
        "[probe] {{\"state\":\"{:?}\",\"chapter\":\"{}\",\"pos\":[{:.3},{:.3},{:.3}],\"yaw\":{:.3},\"grounded\":{},\"cam_yaw\":{:.3},\"cam_pitch\":{:.3},\"cam_dist\":{:.3},\"menu\":{},\"touch\":{},\"lang\":\"{:?}\",\"dialogue\":{},\"revealed\":[{}],\"visited\":[{}],\"free\":{},\"win\":[{:.1},{:.1},{:.2}],\"ui_scale\":{:.3}}}",
        state.get(),
        active.id.clone().unwrap_or_default(),
        pos.x, pos.y, pos.z, yaw, grounded,
        rig.yaw, rig.pitch, rig.target_distance,
        menu.open,
        touch.enabled,
        i18n.lang,
        hud.dialogue.is_some(),
        list(&save.revealed),
        list(&save.visited),
        save.settings.free_travel,
        window.single().map(|w| w.width()).unwrap_or(0.0),
        window.single().map(|w| w.height()).unwrap_or(0.0),
        window.single().map(|w| w.scale_factor()).unwrap_or(0.0),
        ui_scale.0,
    );
    // Story state (for player-like bots): wait kind, target marker, interaction focus, objective.
    let (runner, focus, gate) = extra;
    let drive = player.single().map(|(_, p)| match p.drive { super::player::Drive::Player => "player", super::player::Drive::WalkTo { .. } => "walk", super::player::Drive::External => "ext" }).unwrap_or("");
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let (wait, target, tlabel) = runner
        .map(|r| (r.waiting.as_ref().map(|w| w.describe()).unwrap_or_default(), r.target_world, r.target_label.unwrap_or("")))
        .unwrap_or_default();
    let tgt = target.map(|t| format!("[{:.2},{:.2},{:.2}]", t.x, t.y, t.z)).unwrap_or_else(|| "null".into());
    let obj = hud.objective.as_ref().map(|o| format!("{}|{}", o.0.unwrap_or(""), o.1.unwrap_or(""))).unwrap_or_default();
    let msg = format!(
        "{},\"story\":{{\"wait\":\"{}\",\"target\":{},\"tlabel\":\"{}\",\"focus\":\"{}\",\"hold\":{:.2},\"objective\":\"{}\",\"choice\":{},\"card\":{},\"narr\":{},\"locked\":{},\"drive\":\"{}\",\"teach\":\"{}\"}}",
        &msg[..msg.len() - 1],
        esc(&wait), tgt, tlabel,
        esc(focus.id.as_deref().unwrap_or("")), focus.hold_seconds, esc(&obj),
        hud.choice.as_ref().map(|c| c.0.len()).unwrap_or(0), hud.card.is_some(), hud.narration.is_some(), gate.movement_locked, drive, esc(&hud.teach.as_ref().map(|t| format!("{}|{}", t.v.0.join(" "), t.v.1)).unwrap_or_default()),
    ) + "}";
    platform::console_log(&msg);
}
