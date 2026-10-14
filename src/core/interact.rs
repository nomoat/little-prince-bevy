//! Interactables (the original's Interactable + PlayerInteractor): the best
//! one in range (distance, facing, side, priority) is focused, its prompt
//! ("E  坐下") is drawn above it, and pressing — or holding for
//! `hold_seconds` — uses it, emitting an [`Interacted`] message.

use super::input::InputFrame;
use super::player::Player;
use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Interactable {
    /// Stable id used by story scripts (`WaitUse("rose")`) and messages.
    pub id: String,
    /// i18n key of the verb shown after the key, e.g. `"b612.verb.water"`.
    pub prompt: String,
    /// Distance (m) from the player to the focus point within which it is offered.
    pub radius: f32,
    /// Focus point offset from the entity origin (world-axis aligned).
    pub focus_offset: Vec3,
    /// The player must roughly face the focus point.
    pub require_facing: bool,
    pub max_facing_angle: f32,
    /// Optional: the player must stand on the side the entity faces (+Z), within this half-angle.
    pub side_half_angle: Option<f32>,
    /// Higher wins when several are in range.
    pub priority: i32,
    pub enabled: bool,
    /// Seconds E must be held (0 = a press). The prompt shows a filling ring.
    pub hold_seconds: f32,
    /// Prompt height above the focus point (m).
    pub prompt_height: f32,
    /// Disable automatically after one use.
    pub once: bool,
}

impl Interactable {
    pub fn new(id: &str, prompt_key: &str) -> Self {
        Interactable {
            id: id.to_string(),
            prompt: prompt_key.to_string(),
            radius: 1.8,
            focus_offset: Vec3::ZERO,
            require_facing: true,
            max_facing_angle: 75.0,
            side_half_angle: None,
            priority: 0,
            enabled: true,
            hold_seconds: 0.0,
            prompt_height: 1.2,
            once: false,
        }
    }
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }
    pub fn hold(mut self, secs: f32) -> Self {
        self.hold_seconds = secs;
        self
    }
    pub fn focus(mut self, offset: Vec3) -> Self {
        self.focus_offset = offset;
        self
    }
    pub fn prompt_height(mut self, h: f32) -> Self {
        self.prompt_height = h;
        self
    }
    pub fn priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
    pub fn once(mut self) -> Self {
        self.once = true;
        self
    }
    pub fn any_facing(mut self) -> Self {
        self.require_facing = false;
        self
    }
}

/// Emitted when an interactable is used.
#[derive(Message, Clone, Debug)]
pub struct Interacted {
    pub id: String,
    pub entity: Entity,
}

/// Current focus (read by the prompt UI and the autopilot).
#[derive(Resource, Default, Debug)]
pub struct InteractFocus {
    pub entity: Option<Entity>,
    pub id: Option<String>,
    pub prompt: Option<String>,
    pub world_pos: Vec3,
    pub hold_seconds: f32,
    pub hold_progress: f32,
    held: f32,
    wait_release: bool,
    /// Story/UI can suppress interaction entirely (e.g. while a dialogue is open).
    pub suppressed: bool,
}

pub(crate) fn update_interaction(
    time: Res<Time>,
    input: Res<InputFrame>,
    mut focus: ResMut<InteractFocus>,
    player: Query<(&Transform, &Player)>,
    mut q: Query<(Entity, &mut Interactable, &GlobalTransform)>,
    mut out: MessageWriter<Interacted>,
) {
    let Ok((ptf, pl)) = player.single() else {
        focus.entity = None;
        return;
    };
    let ppos = ptf.translation;
    let fwd = pl.forward();
    let mut best: Option<(f32, Entity)> = None;
    if !focus.suppressed && !input.blocked {
        for (e, it, gt) in &q {
            if !it.enabled {
                continue;
            }
            let fp = gt.translation() + it.focus_offset;
            let d = Vec2::new(fp.x - ppos.x, fp.z - ppos.z);
            let dist = d.length();
            if dist > it.radius || (fp.y - ppos.y).abs() > 2.5 {
                continue;
            }
            let mut angle = 0.0;
            if it.require_facing && dist > 0.35 {
                let dir = d / dist;
                angle = Vec2::new(fwd.x, fwd.z).angle_to(dir).abs().to_degrees();
                if angle > it.max_facing_angle {
                    continue;
                }
            }
            if let Some(half) = it.side_half_angle {
                let f = gt.forward();
                let side = Vec2::new(-f.x, -f.z); // entity +Z
                let to_player = -d.normalize_or_zero();
                if side.angle_to(to_player).abs().to_degrees() > half {
                    continue;
                }
            }
            let score = dist + angle / 90.0 - it.priority as f32 * 10.0;
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, e));
            }
        }
    }
    let new_focus = best.map(|(_, e)| e);
    if new_focus != focus.entity {
        focus.held = 0.0;
        focus.hold_progress = 0.0;
    }
    focus.entity = new_focus;
    let Some(e) = new_focus else {
        focus.id = None;
        focus.prompt = None;
        focus.wait_release = false;
        return;
    };
    let Ok((_, mut it, gt)) = q.get_mut(e) else { return };
    focus.id = Some(it.id.clone());
    focus.prompt = Some(it.prompt.clone());
    focus.world_pos = gt.translation() + it.focus_offset + Vec3::Y * it.prompt_height;
    focus.hold_seconds = it.hold_seconds;
    let mut used = false;
    if it.hold_seconds > 0.0 {
        if input.interact_held && !focus.wait_release {
            focus.held += time.delta_secs();
            focus.hold_progress = (focus.held / it.hold_seconds).min(1.0);
            if focus.held >= it.hold_seconds {
                used = true;
                focus.wait_release = true;
            }
        } else {
            focus.held = (focus.held - time.delta_secs() * 2.0).max(0.0);
            focus.hold_progress = (focus.held / it.hold_seconds).min(1.0);
        }
        if !input.interact_held {
            focus.wait_release = false;
        }
    } else if input.interact {
        used = true;
    }
    if used {
        focus.held = 0.0;
        focus.hold_progress = 0.0;
        if it.once {
            it.enabled = false;
        }
        out.write(Interacted { id: it.id.clone(), entity: e });
    }
}
