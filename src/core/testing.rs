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

pub fn report_ok(id: &str) {
    let msg = format!("[test:ok {id}]");
    platform::console_log(&msg);
    platform::set_page_title(&msg);
    TEST_RESULT.store(1, std::sync::atomic::Ordering::SeqCst);
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
