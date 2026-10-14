//! Story system (the original's StoryRunner + StoryChapter + StoryEvent):
//! a chapter's story is a list of named EVENTS (each with STEPS, shown as the
//! objective) plus a SCRIPT — a flat list of [`Cmd`]s run by the core. Blocking
//! commands (say, wait for an interaction, walk, sunset …) pause the script
//! until done. Branching uses labels, flags and choices. `Cmd::Run` escapes to
//! chapter Rust code with full `&mut World` access.
//!
//! In test mode (`?test=1`) the autopilot reads the current wait and plays it
//! (walk to the interactable and press E, pick a choice, run the chapter's
//! [`TestAction`] hints) — see testing.rs.

use super::audio::AudioCmd;
use super::camera::{CameraRig, Shot};
use super::i18n::I18n;
use super::input::InputGate;
use super::interact::{Interactable, Interacted};
use super::player::{Drive, Player, Pose};
use super::sky::Sky;
use super::ui::hud::{Hud, Speaker};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub type WorldFn = Arc<dyn Fn(&mut World) + Send + Sync>;
pub type WorldPred = Arc<dyn Fn(&World) -> bool + Send + Sync>;

/// A position reference usable by scripts.
#[derive(Clone, Debug)]
pub enum Pos {
    World(Vec3),
    /// Named anchor registered by the chapter (`cx.anchor("rose", …)`), plus offset.
    Anchor(String, Vec3),
    /// Relative to the player (offset in world axes).
    Player(Vec3),
    /// The focus point of an interactable by id.
    Interactable(String),
}

impl Pos {
    pub fn anchor(name: &str) -> Pos {
        Pos::Anchor(name.to_string(), Vec3::ZERO)
    }
    pub fn resolve(&self, world: &World) -> Option<Vec3> {
        match self {
            Pos::World(v) => Some(*v),
            Pos::Anchor(n, off) => world.get_resource::<Anchors>()?.get(n).map(|a| a.0 + *off),
            Pos::Player(off) => {
                let mut best = None;
                for (e, _) in world.iter_entities().filter_map(|e| e.get::<Player>().map(|p| (e.id(), p))) {
                    best = world.get::<Transform>(e).map(|t| t.translation + *off);
                }
                best
            }
            Pos::Interactable(id) => {
                for e in world.iter_entities() {
                    if let (Some(it), Some(gt)) = (e.get::<Interactable>(), e.get::<GlobalTransform>()) {
                        if &it.id == id {
                            return Some(gt.translation() + it.focus_offset);
                        }
                    }
                }
                None
            }
        }
    }
}

/// Named points (position + yaw degrees) registered by chapters.
#[derive(Resource, Default, Debug, Clone)]
pub struct Anchors(pub HashMap<String, (Vec3, f32)>);

impl Anchors {
    pub fn get(&self, name: &str) -> Option<(Vec3, f32)> {
        self.0.get(name).copied()
    }
}

/// What the autopilot does to satisfy a wait it cannot infer by itself.
#[derive(Clone)]
pub enum TestAction {
    /// Walk to the interactable and use it (press, or hold for its hold time).
    Use(String),
    /// Walk to a position.
    GoTo(Pos),
    /// Press the jump / interact / action2 button once.
    PressJump,
    PressInteract,
    PressAction2,
    /// Wait some seconds.
    Wait(f32),
    /// Run code (e.g. set the state the condition waits for).
    Run(WorldFn),
}

/// One named story event with its steps (i18n keys).
#[derive(Clone, Debug)]
pub struct EventDef {
    pub name: &'static str,
    pub steps: Vec<&'static str>,
}

pub fn event(name: &'static str, steps: &[&'static str]) -> EventDef {
    EventDef { name, steps: steps.to_vec() }
}

/// A chapter's story.
#[derive(Clone, Default)]
pub struct Story {
    pub events: Vec<EventDef>,
    /// Optional hidden goal (i18n key of its description, revealed when found).
    pub hidden: Option<&'static str>,
    pub script: Vec<Cmd>,
}

/// Script commands. Use the lowercase helper functions in [`cmd`] to build them.
#[derive(Clone)]
pub enum Cmd {
    /// Chapter title card (number / title / subtitle) — waits ~3 s.
    Card,
    /// Centered narration lines (one after another).
    Narrate(Vec<&'static str>),
    StartEvent(usize),
    SetStep(usize),
    /// Marks the event done and plays the "done → next" banner (waits).
    CompleteEvent(usize),
    /// Dialogue line (waits for the player to continue).
    Say(Speaker, &'static str),
    /// Dialogue line that auto-continues after N seconds.
    SayFor(Speaker, &'static str, f32),
    /// The prince's inner voice (centre screen, auto-hides).
    Mono(&'static str, f32),
    /// Question with options: (option text key, label to jump to).
    Choice(Option<(Speaker, &'static str)>, Vec<(&'static str, &'static str)>, usize),
    Label(&'static str),
    Goto(&'static str),
    IfFlag(&'static str, &'static str),
    IfNotFlag(&'static str, &'static str),
    Flag(&'static str),
    Unflag(&'static str),
    Wait(f32),
    /// Wait until the interactable `id` is used.
    WaitUse(&'static str),
    /// Wait until a flag is set (by chapter code); hints tell the autopilot how.
    WaitFlag(&'static str, Vec<TestAction>),
    /// Wait until the player is within radius of a position.
    WaitNear(Pos, f32),
    /// Wait for an arbitrary condition.
    WaitUntil(WorldPred, Vec<TestAction>),
    /// Objective marker target (+ label key), or clear with None.
    Target(Option<Pos>, Option<&'static str>),
    /// Show the hidden goal as pending / found.
    HiddenPending,
    HiddenFound,
    Enable(&'static str),
    Disable(&'static str),
    /// Tutorial key hint: keys + text key, seconds.
    Teach(Vec<&'static str>, &'static str, f32),
    Lock,
    Unlock,
    /// Walk the player to a position at walking speed (waits).
    WalkTo(Pos),
    /// Turn the player toward a position.
    Face(Pos),
    Teleport(Pos, f32),
    Pose(Pose),
    /// Camera shot: position, look-at, blend seconds.
    Shot(Pos, Pos, f32),
    EndShot(f32),
    SetHour(f64),
    /// Starts the sunset over N seconds (does not wait).
    Sunset(f32),
    WaitSunset,
    TimeRate(f32, Option<f64>),
    Music(Option<&'static str>),
    Ambience(Option<&'static str>),
    Sfx(&'static str),
    Run(WorldFn),
    /// Only in test mode.
    TestOnly(WorldFn),
    /// The planet's story goal is met: departure opens (hold E to set off).
    Departure,
    /// Leave immediately (as if departed).
    Leave,
    End,
}

/// Builders for [`Cmd`] — `use crate::core::story::cmd::*;`
pub mod cmd {
    use super::*;
    pub fn card() -> Cmd { Cmd::Card }
    pub fn narrate(keys: &[&'static str]) -> Cmd { Cmd::Narrate(keys.to_vec()) }
    pub fn start_event(i: usize) -> Cmd { Cmd::StartEvent(i) }
    pub fn step(i: usize) -> Cmd { Cmd::SetStep(i) }
    pub fn complete_event(i: usize) -> Cmd { Cmd::CompleteEvent(i) }
    pub fn say(who: Speaker, key: &'static str) -> Cmd { Cmd::Say(who, key) }
    pub fn say_for(who: Speaker, key: &'static str, secs: f32) -> Cmd { Cmd::SayFor(who, key, secs) }
    pub fn mono(key: &'static str) -> Cmd { Cmd::Mono(key, 3.5) }
    pub fn choice(question: Option<(Speaker, &'static str)>, options: &[(&'static str, &'static str)]) -> Cmd { Cmd::Choice(question, options.to_vec(), 0) }
    /// Choice whose autopilot pick is `test_pick`.
    pub fn choice_test(question: Option<(Speaker, &'static str)>, options: &[(&'static str, &'static str)], test_pick: usize) -> Cmd { Cmd::Choice(question, options.to_vec(), test_pick) }
    pub fn label(name: &'static str) -> Cmd { Cmd::Label(name) }
    pub fn goto(name: &'static str) -> Cmd { Cmd::Goto(name) }
    pub fn if_flag(flag: &'static str, label: &'static str) -> Cmd { Cmd::IfFlag(flag, label) }
    pub fn if_not_flag(flag: &'static str, label: &'static str) -> Cmd { Cmd::IfNotFlag(flag, label) }
    pub fn flag(name: &'static str) -> Cmd { Cmd::Flag(name) }
    pub fn unflag(name: &'static str) -> Cmd { Cmd::Unflag(name) }
    pub fn wait(secs: f32) -> Cmd { Cmd::Wait(secs) }
    pub fn wait_use(id: &'static str) -> Cmd { Cmd::WaitUse(id) }
    pub fn wait_flag(flag: &'static str, test: Vec<TestAction>) -> Cmd { Cmd::WaitFlag(flag, test) }
    pub fn wait_near(p: Pos, radius: f32) -> Cmd { Cmd::WaitNear(p, radius) }
    pub fn wait_until(f: impl Fn(&World) -> bool + Send + Sync + 'static, test: Vec<TestAction>) -> Cmd { Cmd::WaitUntil(Arc::new(f), test) }
    pub fn target(p: Pos, label: &'static str) -> Cmd { Cmd::Target(Some(p), Some(label)) }
    pub fn clear_target() -> Cmd { Cmd::Target(None, None) }
    pub fn enable(id: &'static str) -> Cmd { Cmd::Enable(id) }
    pub fn disable(id: &'static str) -> Cmd { Cmd::Disable(id) }
    pub fn teach(keys: &[&'static str], text: &'static str) -> Cmd { Cmd::Teach(keys.to_vec(), text, 6.0) }
    pub fn lock() -> Cmd { Cmd::Lock }
    pub fn unlock() -> Cmd { Cmd::Unlock }
    pub fn walk_to(p: Pos) -> Cmd { Cmd::WalkTo(p) }
    pub fn face(p: Pos) -> Cmd { Cmd::Face(p) }
    pub fn teleport(p: Pos, yaw_deg: f32) -> Cmd { Cmd::Teleport(p, yaw_deg) }
    pub fn pose(p: Pose) -> Cmd { Cmd::Pose(p) }
    pub fn shot(from: Pos, look: Pos, blend: f32) -> Cmd { Cmd::Shot(from, look, blend) }
    pub fn end_shot() -> Cmd { Cmd::EndShot(1.0) }
    pub fn set_hour(h: f64) -> Cmd { Cmd::SetHour(h) }
    pub fn sunset(secs: f32) -> Cmd { Cmd::Sunset(secs) }
    pub fn wait_sunset() -> Cmd { Cmd::WaitSunset }
    pub fn music(key: &'static str) -> Cmd { Cmd::Music(Some(key)) }
    pub fn ambience(key: &'static str) -> Cmd { Cmd::Ambience(Some(key)) }
    pub fn sfx(key: &'static str) -> Cmd { Cmd::Sfx(key) }
    pub fn run(f: impl Fn(&mut World) + Send + Sync + 'static) -> Cmd { Cmd::Run(Arc::new(f)) }
    pub fn test_only(f: impl Fn(&mut World) + Send + Sync + 'static) -> Cmd { Cmd::TestOnly(Arc::new(f)) }
    pub fn departure() -> Cmd { Cmd::Departure }
    pub fn leave() -> Cmd { Cmd::Leave }
    pub fn end() -> Cmd { Cmd::End }
    // test action helpers
    pub fn t_use(id: &str) -> TestAction { TestAction::Use(id.to_string()) }
    pub fn t_goto(p: Pos) -> TestAction { TestAction::GoTo(p) }
    pub fn t_wait(s: f32) -> TestAction { TestAction::Wait(s) }
    pub fn t_run(f: impl Fn(&mut World) + Send + Sync + 'static) -> TestAction { TestAction::Run(Arc::new(f)) }
}

/// What the script is blocked on.
#[derive(Clone)]
pub enum Waiting {
    Seconds(f32),
    Card,
    Banner,
    Narration,
    Dialogue,
    Choice(Vec<&'static str>, usize),
    Use(&'static str),
    Flag(&'static str, Vec<TestAction>),
    Near(Pos, f32),
    Until(WorldPred, Vec<TestAction>),
    Walk,
    Sunset,
    Departure,
}

impl Waiting {
    pub fn describe(&self) -> String {
        match self {
            Waiting::Seconds(s) => format!("wait {s:.1}s"),
            Waiting::Card => "card".into(),
            Waiting::Banner => "banner".into(),
            Waiting::Narration => "narration".into(),
            Waiting::Dialogue => "dialogue".into(),
            Waiting::Choice(..) => "choice".into(),
            Waiting::Use(id) => format!("use '{id}'"),
            Waiting::Flag(f, _) => format!("flag '{f}'"),
            Waiting::Near(p, r) => format!("near {p:?} r={r}"),
            Waiting::Until(..) => "condition".into(),
            Waiting::Walk => "walk".into(),
            Waiting::Sunset => "sunset".into(),
            Waiting::Departure => "departure".into(),
        }
    }
}

#[derive(Resource, Default)]
pub struct StoryRunner {
    pub script: Arc<Vec<Cmd>>,
    pub events: Vec<EventDef>,
    pub hidden_key: Option<&'static str>,
    pub pc: usize,
    labels: HashMap<&'static str, usize>,
    pub waiting: Option<Waiting>,
    /// Seconds spent in the current wait.
    pub wait_time: f32,
    pub event_index: Option<usize>,
    pub step_index: usize,
    pub done: Vec<bool>,
    pub flags: HashSet<String>,
    /// Interactable ids used this frame.
    pub used_now: Vec<String>,
    pub target: Option<Pos>,
    pub target_label: Option<&'static str>,
    /// The target resolved to a world position (updated each frame).
    pub target_world: Option<Vec3>,
    pub hidden_shown: bool,
    pub hidden_done: bool,
    /// Departure is open (story goal met).
    pub departure_ready: bool,
    /// Departure requested (by holding E or `Leave`).
    pub leave_requested: bool,
    pub running: bool,
    /// The chapter id the story belongs to.
    pub chapter: String,
    /// Closest distance to the target so far (stall detection, original's `_best`).
    pub stall_best: f32,
    pub stall_time: f32,
}

impl StoryRunner {
    pub fn begin(&mut self, chapter: &str, story: Story) {
        let mut labels = HashMap::new();
        for (i, c) in story.script.iter().enumerate() {
            if let Cmd::Label(l) = c {
                labels.insert(*l, i);
            }
        }
        *self = StoryRunner {
            done: vec![false; story.events.len()],
            events: story.events,
            hidden_key: story.hidden,
            script: Arc::new(story.script),
            labels,
            running: true,
            chapter: chapter.to_string(),
            stall_best: f32::MAX,
            ..default()
        };
    }
    pub fn set_flag(&mut self, f: &str) {
        self.flags.insert(f.to_string());
    }
    pub fn has_flag(&self, f: &str) -> bool {
        self.flags.contains(f)
    }
    pub fn clear_flag(&mut self, f: &str) {
        self.flags.remove(f);
    }
    pub fn current_step_key(&self) -> Option<&'static str> {
        let e = self.events.get(self.event_index?)?;
        e.steps.get(self.step_index).copied()
    }
}

/// Collects interaction messages for `WaitUse`.
pub(crate) fn collect_interactions(mut runner: ResMut<StoryRunner>, mut msgs: MessageReader<Interacted>) {
    runner.used_now.clear();
    for m in msgs.read() {
        runner.used_now.push(m.id.clone());
    }
}

fn test_mode(world: &World) -> bool {
    world.get_resource::<super::params::LaunchParams>().map(|p| p.test).unwrap_or(false)
}

fn set_interactable(world: &mut World, id: &str, on: bool) {
    let mut q = world.query::<&mut Interactable>();
    for mut it in q.iter_mut(world) {
        if it.id == id {
            it.enabled = on;
        }
    }
}

fn player_pos(world: &mut World) -> Option<Vec3> {
    let mut q = world.query_filtered::<&Transform, With<Player>>();
    q.iter(world).next().map(|t| t.translation)
}

/// The story interpreter (exclusive system; runs while a chapter is active).
pub(crate) fn run_story(world: &mut World) {
    let dt = world.resource::<Time>().delta_secs();
    let paused = world.resource::<Time<Virtual>>().is_paused();
    if paused || !world.resource::<StoryRunner>().running {
        return;
    }
    // ---- check the current wait
    let waiting = world.resource::<StoryRunner>().waiting.clone();
    if let Some(w) = waiting {
        world.resource_mut::<StoryRunner>().wait_time += dt;
        let done = match &w {
            Waiting::Seconds(s) => world.resource::<StoryRunner>().wait_time >= *s,
            Waiting::Card => !world.resource::<Hud>().card_active(),
            Waiting::Banner => !world.resource::<Hud>().banner_active(),
            Waiting::Narration => !world.resource::<Hud>().narration_active(),
            Waiting::Dialogue => world.resource::<Hud>().dialogue_finished(),
            Waiting::Choice(_, _) => world.resource::<Hud>().choice_result().is_some(),
            Waiting::Use(id) => world.resource::<StoryRunner>().used_now.iter().any(|u| u == id),
            Waiting::Flag(f, _) => world.resource::<StoryRunner>().has_flag(f),
            Waiting::Near(p, r) => {
                let tp = p.resolve(world);
                let pp = player_pos(world);
                matches!((tp, pp), (Some(a), Some(b)) if Vec2::new(a.x - b.x, a.z - b.z).length() <= *r)
            }
            Waiting::Until(f, _) => f(world),
            Waiting::Walk => {
                let mut q = world.query::<&Player>();
                q.iter(world).next().map(|p| p.is_arrived()).unwrap_or(true)
            }
            Waiting::Sunset => world.resource::<Sky>().sunset_done(),
            Waiting::Departure => false,
        };
        if !done {
            return;
        }
        if let Waiting::Choice(_, _) = &w {
            // jump to the chosen label
            let pick = world.resource::<Hud>().choice_result().unwrap_or(0);
            let script = world.resource::<StoryRunner>().script.clone();
            if let Some(Cmd::Choice(_, opts, _)) = script.get(world.resource::<StoryRunner>().pc) {
                let label = opts.get(pick).map(|o| o.1);
                world.resource_mut::<Hud>().close_choice();
                if let Some(l) = label {
                    let target = world.resource::<StoryRunner>().labels.get(l).copied();
                    if let Some(t) = target {
                        let mut r = world.resource_mut::<StoryRunner>();
                        r.pc = t;
                        r.waiting = None;
                        r.wait_time = 0.0;
                    }
                }
            }
        }
        if matches!(w, Waiting::Dialogue) {
            world.resource_mut::<Hud>().close_dialogue();
        }
        let mut r = world.resource_mut::<StoryRunner>();
        if !matches!(w, Waiting::Choice(..)) {
            r.pc += 1;
        }
        r.waiting = None;
        r.wait_time = 0.0;
    }
    // ---- execute until something blocks
    for _ in 0..256 {
        let (script, pc) = {
            let r = world.resource::<StoryRunner>();
            (r.script.clone(), r.pc)
        };
        let Some(c) = script.get(pc) else {
            world.resource_mut::<StoryRunner>().running = false;
            return;
        };
        let block = exec(world, c);
        if let Some(w) = block {
            let mut r = world.resource_mut::<StoryRunner>();
            r.waiting = Some(w);
            r.wait_time = 0.0;
            return;
        }
        let mut r = world.resource_mut::<StoryRunner>();
        if r.pc == pc {
            r.pc += 1;
        }
        if !r.running {
            return;
        }
    }
}

/// Executes one command; returns a wait if it blocks.
fn exec(world: &mut World, c: &Cmd) -> Option<Waiting> {
    let test = test_mode(world);
    match c {
        Cmd::Card => {
            let ch = world.resource::<StoryRunner>().chapter.clone();
            let info = world.resource::<super::chapter::ChapterRegistry>().info(&ch);
            if let Some(info) = info {
                world.resource_mut::<Hud>().show_card(info.number, info.title, info.subtitle, if test { 0.3 } else { 3.2 });
                return Some(Waiting::Card);
            }
            None
        }
        Cmd::Narrate(keys) => {
            world.resource_mut::<Hud>().show_narration(keys.clone(), if test { 0.2 } else { 3.6 });
            Some(Waiting::Narration)
        }
        Cmd::StartEvent(i) => {
            let name = world.resource::<StoryRunner>().events.get(*i).map(|e| (e.name, e.steps.first().copied()));
            let mut r = world.resource_mut::<StoryRunner>();
            r.event_index = Some(*i);
            r.step_index = 0;
            r.stall_best = f32::MAX;
            r.stall_time = 0.0;
            if let Some((n, s)) = name {
                world.resource_mut::<Hud>().show_objective(Some(n), s, true);
            }
            None
        }
        Cmd::SetStep(s) => {
            let mut r = world.resource_mut::<StoryRunner>();
            r.step_index = *s;
            r.stall_best = f32::MAX;
            r.stall_time = 0.0;
            let key = r.current_step_key();
            let ev = r.event_index.and_then(|i| r.events.get(i)).map(|e| e.name);
            world.resource_mut::<Hud>().show_objective(ev, key, false);
            None
        }
        Cmd::CompleteEvent(i) => {
            let (name, next) = {
                let mut r = world.resource_mut::<StoryRunner>();
                if let Some(d) = r.done.get_mut(*i) {
                    *d = true;
                }
                let name = r.events.get(*i).map(|e| e.name);
                let next = r.events.get(*i + 1).map(|e| e.name);
                r.target = None;
                r.target_label = None;
                (name, next)
            };
            world.write_message(AudioCmd::sfx("sfx.banner"));
            if let Some(n) = name {
                world.resource_mut::<Hud>().show_banner(n, next, if test { 0.25 } else { 1.0 });
                world.resource_mut::<Hud>().hide_objective();
                return Some(Waiting::Banner);
            }
            None
        }
        Cmd::Say(who, key) => {
            world.resource_mut::<Hud>().say(*who, key, true, None);
            Some(Waiting::Dialogue)
        }
        Cmd::SayFor(who, key, secs) => {
            world.resource_mut::<Hud>().say(*who, key, false, Some(*secs));
            Some(Waiting::Dialogue)
        }
        Cmd::Mono(key, secs) => {
            world.resource_mut::<Hud>().monologue(key, *secs);
            None
        }
        Cmd::Choice(q, opts, test_pick) => {
            if let Some((who, key)) = q {
                world.resource_mut::<Hud>().say(*who, key, false, None);
            }
            let keys: Vec<&'static str> = opts.iter().map(|o| o.0).collect();
            world.resource_mut::<Hud>().open_choice(keys.clone());
            Some(Waiting::Choice(keys, *test_pick))
        }
        Cmd::Label(_) => None,
        Cmd::Goto(l) => {
            jump(world, l);
            None
        }
        Cmd::IfFlag(f, l) => {
            if world.resource::<StoryRunner>().has_flag(f) {
                jump(world, l);
            }
            None
        }
        Cmd::IfNotFlag(f, l) => {
            if !world.resource::<StoryRunner>().has_flag(f) {
                jump(world, l);
            }
            None
        }
        Cmd::Flag(f) => {
            world.resource_mut::<StoryRunner>().set_flag(f);
            None
        }
        Cmd::Unflag(f) => {
            world.resource_mut::<StoryRunner>().clear_flag(f);
            None
        }
        Cmd::Wait(s) => Some(Waiting::Seconds(if test { s.min(0.4) } else { *s })),
        Cmd::WaitUse(id) => {
            set_interactable(world, id, true);
            Some(Waiting::Use(id))
        }
        Cmd::WaitFlag(f, hints) => Some(Waiting::Flag(f, hints.clone())),
        Cmd::WaitNear(p, r) => Some(Waiting::Near(p.clone(), *r)),
        Cmd::WaitUntil(f, hints) => Some(Waiting::Until(f.clone(), hints.clone())),
        Cmd::Target(p, l) => {
            let mut r = world.resource_mut::<StoryRunner>();
            r.target = p.clone();
            r.target_label = *l;
            r.stall_best = f32::MAX;
            r.stall_time = 0.0;
            None
        }
        Cmd::HiddenPending => {
            world.resource_mut::<StoryRunner>().hidden_shown = true;
            let k = world.resource::<StoryRunner>().hidden_key;
            world.resource_mut::<Hud>().hidden = k.map(|k| (k, false));
            None
        }
        Cmd::HiddenFound => {
            let (ch, key) = {
                let mut r = world.resource_mut::<StoryRunner>();
                r.hidden_done = true;
                (r.chapter.clone(), r.hidden_key)
            };
            if let Some(k) = key {
                world.resource_mut::<Hud>().hidden_found(k);
                let id = format!("{ch}:{k}");
                let mut save = world.resource_mut::<super::save::SaveData>();
                if !save.hidden_found.contains(&id) {
                    save.hidden_found.push(id);
                }
                world.write_message(AudioCmd::sfx("sfx.found"));
            }
            None
        }
        Cmd::Enable(id) => {
            set_interactable(world, id, true);
            None
        }
        Cmd::Disable(id) => {
            set_interactable(world, id, false);
            None
        }
        Cmd::Teach(keys, text, secs) => {
            world.resource_mut::<Hud>().teach(keys.clone(), text, *secs);
            None
        }
        Cmd::Lock => {
            world.resource_mut::<StoryLock>().0 = true;
            None
        }
        Cmd::Unlock => {
            world.resource_mut::<StoryLock>().0 = false;
            None
        }
        Cmd::WalkTo(p) => {
            let t = p.resolve(world)?;
            let walk = world.resource::<super::player::PlayerTuning>().walk_speed;
            let mut q = world.query::<&mut Player>();
            if let Some(mut pl) = q.iter_mut(world).next() {
                pl.drive = Drive::WalkTo { target: Vec2::new(t.x, t.z), speed: if test { walk * 2.0 } else { walk }, stop_radius: 0.25 };
                return Some(Waiting::Walk);
            }
            None
        }
        Cmd::Face(p) => {
            let t = p.resolve(world)?;
            let mut q = world.query::<(&mut Player, &Transform)>();
            if let Some((mut pl, tf)) = q.iter_mut(world).next() {
                let d = t - tf.translation;
                pl.face_target = Some(d.x.atan2(d.z));
            }
            None
        }
        Cmd::Teleport(p, yaw) => {
            let t = p.resolve(world)?;
            let mut q = world.query::<(&mut Player, &mut Transform)>();
            if let Some((mut pl, mut tf)) = q.iter_mut(world).next() {
                tf.translation = t;
                pl.yaw = yaw.to_radians();
                pl.velocity = Vec3::ZERO;
            }
            world.resource_mut::<CameraRig>().snap_behind(yaw.to_radians());
            None
        }
        Cmd::Pose(pose) => {
            let mut q = world.query::<&mut Player>();
            if let Some(mut pl) = q.iter_mut(world).next() {
                pl.pose = *pose;
                pl.drive = if matches!(pose, Pose::Sit | Pose::Kneel) { Drive::External } else { Drive::Player };
            }
            None
        }
        Cmd::Shot(from, look, blend) => {
            if let (Some(a), Some(b)) = (from.resolve(world), look.resolve(world)) {
                world.resource_mut::<CameraRig>().begin_shot(Shot { position: a, look_at: b, blend: *blend, fov: None });
            }
            None
        }
        Cmd::EndShot(blend) => {
            world.resource_mut::<CameraRig>().end_shot(*blend);
            None
        }
        Cmd::SetHour(h) => {
            world.resource_mut::<Sky>().set_hour(*h);
            None
        }
        Cmd::Sunset(s) => {
            world.resource_mut::<Sky>().start_sunset(if test { s.min(2.0) } else { *s });
            None
        }
        Cmd::WaitSunset => Some(Waiting::Sunset),
        Cmd::TimeRate(r, cap) => {
            world.resource_mut::<Sky>().set_rate(*r, *cap);
            None
        }
        Cmd::Music(k) => {
            world.write_message(AudioCmd::Music(k.map(|s| s.to_string())));
            None
        }
        Cmd::Ambience(k) => {
            world.write_message(AudioCmd::Ambience(k.map(|s| s.to_string())));
            None
        }
        Cmd::Sfx(k) => {
            world.write_message(AudioCmd::sfx(k));
            None
        }
        Cmd::Run(f) => {
            f(world);
            None
        }
        Cmd::TestOnly(f) => {
            if test {
                f(world);
            }
            None
        }
        Cmd::Departure => {
            world.resource_mut::<StoryRunner>().departure_ready = true;
            world.resource_mut::<Hud>().depart_line = true;
            world.resource_mut::<Hud>().hide_objective();
            world.write_message(AudioCmd::sfx("sfx.found"));
            Some(Waiting::Departure)
        }
        Cmd::Leave => {
            let mut r = world.resource_mut::<StoryRunner>();
            r.departure_ready = true;
            r.leave_requested = true;
            Some(Waiting::Departure)
        }
        Cmd::End => {
            world.resource_mut::<StoryRunner>().running = false;
            None
        }
    }
}

/// Marks the chapter's hidden goal as found (call from chapter systems via
/// `commands.queue(|w: &mut World| found_hidden(w))`).
pub fn found_hidden(world: &mut World) {
    if !world.resource::<StoryRunner>().hidden_done {
        exec(world, &Cmd::HiddenFound);
    }
}

/// Sets a story flag from code holding `&mut World`.
pub fn set_flag(world: &mut World, flag: &str) {
    world.resource_mut::<StoryRunner>().set_flag(flag);
}

fn jump(world: &mut World, label: &str) {
    let t = world.resource::<StoryRunner>().labels.get(label).copied();
    match t {
        Some(t) => world.resource_mut::<StoryRunner>().pc = t,
        None => error!("story: unknown label '{label}'"),
    }
}

/// Story-requested player lock.
#[derive(Resource, Default)]
pub struct StoryLock(pub bool);

/// Stall detection: no progress toward the target for a while → flash the marker.
pub(crate) fn stall_marker(world: &mut World) {
    let dt = world.resource::<Time>().delta_secs();
    let target = world.resource::<StoryRunner>().target.clone();
    let resolved = target.and_then(|p| p.resolve(world));
    world.resource_mut::<StoryRunner>().target_world = resolved;
    let Some(t) = resolved else { return };
    let Some(pp) = player_pos(world) else { return };
    let d = Vec2::new(t.x - pp.x, t.z - pp.z).length();
    let mut r = world.resource_mut::<StoryRunner>();
    if d < r.stall_best - 0.5 {
        r.stall_best = d;
        r.stall_time = 0.0;
    } else {
        r.stall_time += dt;
    }
    if r.stall_time > 14.0 {
        r.stall_time = 0.0;
        r.stall_best = d;
        world.resource_mut::<Hud>().flash_marker(6.0);
    }
}

/// Gate gameplay input from story/UI state each frame.
pub(crate) fn update_input_gate(
    mut gate: ResMut<InputGate>,
    hud: Res<Hud>,
    lock: Res<StoryLock>,
    menu: Res<super::ui::menu::MenuState>,
    runner: Res<StoryRunner>,
) {
    gate.menu_open = menu.open;
    gate.movement_locked = lock.0 || hud.blocks_movement() || matches!(runner.waiting, Some(Waiting::Walk));
}

/// i18n helper re-export for chapter code.
pub fn tr<'a>(i18n: &'a I18n, key: &'a str) -> &'a str {
    i18n.t(key)
}
