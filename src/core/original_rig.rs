//! Playback of the original Humanoid curves converted onto the source Avatar.
//! Root travel stays with the controller. Bones, fingers and local body sway
//! come from the clips; Unity's proprietary IK/stretch solver is not emulated.
use super::{
    models::PartName,
    params::LaunchParams,
    player::{Player, Pose},
};
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Clip {
    start: usize,
    frames: usize,
    duration: f32,
    looped: bool,
}
#[derive(Deserialize)]
struct Index {
    bones: Vec<String>,
    clips: Vec<(String, Clip)>,
}
#[derive(Resource)]
pub(crate) struct OriginalAnimations {
    index: Index,
    samples: Vec<f32>,
    bed_offset: f32,
}
#[derive(Clone, Copy)]
struct BonePose {
    rotation: Quat,
    position: Vec3,
}

impl Default for OriginalAnimations {
    fn default() -> Self {
        let index = ron::from_str(include_str!("../../assets/original/shared/animations.ron"))
            .expect("original animation index");
        let samples = include_bytes!("../../assets/original/shared/animations.bin")
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let mut bank = Self {
            index,
            samples,
            bed_offset: 0.0,
        };
        // WakeUp was authored on the bed. Put its final stance at the same
        // height as Idle without removing the actual rising motion.
        bank.bed_offset = bank.sample(bank.clip("WakeUp"), 100.0, 0).position.y
            - bank.sample(bank.clip("Idle"), 0.0, 0).position.y;
        bank
    }
}
impl OriginalAnimations {
    fn clip(&self, name: &str) -> usize {
        self.index
            .clips
            .iter()
            .position(|(n, _)| n == name)
            .unwrap_or_else(|| {
                self.index
                    .clips
                    .iter()
                    .position(|(n, _)| n == "Idle")
                    .unwrap()
            })
    }
    fn sample(&self, clip: usize, time: f32, bone: usize) -> BonePose {
        let (name, c) = &self.index.clips[clip];
        let t = if c.looped {
            time.rem_euclid(c.duration)
        } else {
            time.clamp(0.0, c.duration)
        };
        let frame = t / c.duration * (c.frames - 1) as f32;
        let first = (frame.floor() as usize).min(c.frames - 1);
        let second = (first + 1).min(c.frames - 1);
        let stride = self.index.bones.len() * 4 + 3;
        let read = |f: usize| {
            let start = (c.start + f) * stride;
            let q = start + bone * 4;
            let p = start + stride - 3;
            BonePose {
                rotation: Quat::from_xyzw(
                    self.samples[q],
                    self.samples[q + 1],
                    self.samples[q + 2],
                    self.samples[q + 3],
                ),
                position: Vec3::new(self.samples[p], self.samples[p + 1], self.samples[p + 2]),
            }
        };
        let a = read(first);
        let b = read(second);
        let mut pose = BonePose {
            rotation: a.rotation.slerp(b.rotation, frame.fract()),
            position: a.position.lerp(b.position, frame.fract()),
        };
        if name == "Sleep" || name == "WakeUp" {
            pose.position.y -= self.bed_offset;
        }
        pose
    }
}

/// Chapter choreography can scrub a clip using its own sequence clock.
#[derive(Component, Default)]
pub(crate) struct OriginalAnimationRequest {
    pub clip: Option<&'static str>,
    pub time: Option<f32>,
    pub carry_can: bool,
}
#[derive(Component)]
pub(crate) struct OriginalBone {
    pub player: Entity,
    index: usize,
    rest: Transform,
}
#[derive(Component)]
struct OriginalScarf {
    player: Entity,
    rest: Transform,
    segment: Option<f32>,
}
#[derive(Component)]
struct ScarfAnchor {
    neck: Entity,
    offset: Mat4,
}
#[derive(Component)]
pub(crate) struct OriginalPlayback {
    clip: usize,
    time: f32,
    blend: f32,
    carry: f32,
    from: Vec<BonePose>,
    pose: Vec<BonePose>,
}

pub(crate) fn bind_original_parts(
    mut commands: Commands,
    bank: Res<OriginalAnimations>,
    added: Query<(Entity, &Name, &Transform), Added<Name>>,
    parents: Query<&ChildOf>,
    players: Query<Entity, With<Player>>,
    unbound: Query<Entity, (With<Player>, Without<OriginalPlayback>)>,
) {
    for player in &unbound {
        let clip = bank.clip("Idle");
        let pose: Vec<_> = (0..bank.index.bones.len())
            .map(|b| bank.sample(clip, 0.0, b))
            .collect();
        commands.entity(player).insert(OriginalPlayback {
            clip,
            time: 0.0,
            blend: 1.0,
            carry: 0.0,
            from: pose.clone(),
            pose,
        });
    }
    for (entity, name, rest) in &added {
        if let Some(part) = name.as_str().strip_prefix("part:") {
            commands.entity(entity).insert(PartName(part.to_string()));
        }
        let index = bank.index.bones.iter().position(|n| n == name.as_str());
        let scarf = name.as_str().starts_with("scarf_") && name.as_str() != "scarf_rig";
        if index.is_none() && !scarf {
            continue;
        }
        let mut current = entity;
        while let Ok(parent) = parents.get(current) {
            current = parent.parent();
            if players.contains(current) {
                if let Some(index) = index {
                    commands.entity(entity).insert(OriginalBone {
                        player: current,
                        index,
                        rest: *rest,
                    });
                }
                if scarf {
                    let segment = name
                        .as_str()
                        .rsplit('_')
                        .next()
                        .and_then(|n| n.parse::<f32>().ok());
                    commands.entity(entity).insert(OriginalScarf {
                        player: current,
                        rest: *rest,
                        segment,
                    });
                }
                break;
            }
        }
    }
}

pub(crate) fn advance_original_animation(
    time: Res<Time>,
    bank: Res<OriginalAnimations>,
    params: Res<LaunchParams>,
    mut players: Query<(
        &Player,
        Option<&OriginalAnimationRequest>,
        &mut OriginalPlayback,
    )>,
) {
    for (player, request, mut state) in &mut players {
        let requested = request.and_then(|r| r.clip);
        let mut name = requested.unwrap_or_else(|| match player.pose {
            Pose::Sit => "SitIdle",
            Pose::Kneel => "FillCan",
            Pose::Reach => "WaterRose",
            Pose::Clap => "Clap",
            Pose::Wave => "Shrug",
            _ if !player.grounded => "Jump",
            _ if player.planar_speed > 2.0 => "Run",
            _ if player.planar_speed > 0.08 => "Walk",
            _ => "Idle",
        });
        // Review parameters only affect explicit review URLs, never normal play.
        if params.get("review") == Some("opening") {
            name = "Idle";
        }
        if let Some(review) = params.get("animation") {
            name = review;
        }
        let clip = bank.clip(name);
        if state.clip != clip {
            state.from = state.pose.clone();
            state.clip = clip;
            state.time = 0.0;
            state.blend = 0.0;
        }
        let rate = if requested.is_none() && name == "Walk" {
            (player.planar_speed / 1.05).clamp(0.5, 1.8)
        } else if requested.is_none() && name == "Run" {
            (player.planar_speed / 3.0).clamp(0.65, 1.4)
        } else {
            1.0
        };
        state.time += time.delta_secs() * rate;
        let mut sample_time = request
            .and_then(|r| r.time)
            .filter(|_| requested == Some(name))
            .unwrap_or(state.time);
        if let Some(t) = params
            .get("animation_time")
            .and_then(|s| s.parse::<f32>().ok())
        {
            sample_time = t;
        }
        state.blend = (state.blend + time.delta_secs() / 0.22).min(1.0);
        let carry = request.is_some_and(|r| r.carry_can) || player.pose == Pose::Carry;
        let carry_target = if carry && requested.is_none() {
            1.0
        } else {
            0.0
        };
        let step = time.delta_secs() / 0.3;
        state.carry += (carry_target - state.carry).clamp(-step, step);
        let blend = state.blend * state.blend * (3.0 - 2.0 * state.blend);
        for bone in 0..bank.index.bones.len() {
            let mut target = bank.sample(clip, sample_time, bone);
            if bone == 0 && name == "Jump" {
                // The controller already supplies the ballistic arc. Preserve
                // crouching but do not add the clip's upward root travel twice.
                target.position.y = target
                    .position
                    .y
                    .min(bank.sample(bank.clip("Idle"), 0.0, 0).position.y);
            }
            let n = bank.index.bones[bone].as_str();
            // The original CarryArm layer affects only the right arm; legs
            // continue the locomotion cycle while the can is held.
            if n.ends_with(".r")
                && (n.starts_with("shoulder")
                    || n.starts_with("arm_")
                    || n.starts_with("forearm_")
                    || n.starts_with("hand")
                    || n.starts_with("c_"))
            {
                target.rotation = target.rotation.slerp(
                    bank.sample(bank.clip("CarryCan"), state.time, bone)
                        .rotation,
                    state.carry,
                );
            }
            let from = state.from[bone];
            state.pose[bone] = BonePose {
                rotation: from.rotation.slerp(target.rotation, blend),
                position: from.position.lerp(target.position, blend),
            };
        }
    }
}

pub(crate) fn animate_original_prince(
    players: Query<&OriginalPlayback>,
    mut bones: Query<(&OriginalBone, &mut Transform)>,
) {
    for (bone, mut tf) in &mut bones {
        let Ok(state) = players.get(bone.player) else {
            continue;
        };
        tf.rotation = state.pose[bone.index].rotation;
        tf.translation = if bone.index == 0 {
            state.pose[0].position
        } else {
            bone.rest.translation
        };
    }
}

/// The scarf has its own skin in the source prefab. Follow the neck's delta
/// from bind pose, then add a small traveling wave along its four tail chains.
pub(crate) fn animate_original_scarf(world: &mut World) {
    let neck_index = world
        .resource::<OriginalAnimations>()
        .index
        .bones
        .iter()
        .position(|n| n == "neck.x")
        .unwrap();
    let roots: Vec<_> = world
        .query::<(Entity, &Name, &OriginalScarf)>()
        .iter(world)
        .filter(|(_, name, _)| name.as_str() == "scarf_root")
        .map(|(e, _, s)| (e, s.player))
        .collect();
    for (entity, player) in roots {
        if world.get::<ScarfAnchor>(entity).is_none() {
            let neck = world
                .query::<(Entity, &OriginalBone)>()
                .iter(world)
                .find(|(_, b)| b.player == player && b.index == neck_index)
                .map(|(e, _)| e);
            if let Some(neck) = neck {
                let offset =
                    bone_matrix(world, neck, true).inverse() * bone_matrix(world, entity, true);
                world
                    .entity_mut(entity)
                    .insert(ScarfAnchor { neck, offset });
            }
        }
        if let Some(anchor) = world.get::<ScarfAnchor>(entity) {
            let matrix = bone_matrix(world, anchor.neck, false) * anchor.offset;
            if let Some(parent) = world.get::<ChildOf>(entity) {
                let local = bone_matrix(world, parent.parent(), false).inverse() * matrix;
                *world.get_mut::<Transform>(entity).unwrap() = Transform::from_matrix(local);
            }
        }
    }
    let t = world.resource::<Time>().elapsed_secs();
    let segments: Vec<_> = world
        .query::<(Entity, &OriginalScarf)>()
        .iter(world)
        .filter_map(|(e, s)| s.segment.map(|i| (e, s.player, s.rest, i)))
        .collect();
    for (entity, player, rest, segment) in segments {
        let speed = world
            .get::<Player>(player)
            .map(|p| p.planar_speed)
            .unwrap_or(0.0);
        let amplitude = 0.014 + speed.min(3.5) * 0.017;
        let angle = (t * 4.5 - segment * 0.55).sin() * amplitude;
        if let Some(mut tf) = world.get_mut::<Transform>(entity) {
            tf.rotation =
                rest.rotation * Quat::from_rotation_z(angle) * Quat::from_rotation_x(angle * 0.5);
        }
    }
}

fn bone_matrix(world: &World, mut entity: Entity, rest: bool) -> Mat4 {
    let mut matrix = Mat4::IDENTITY;
    loop {
        let tf = if rest {
            world
                .get::<OriginalBone>(entity)
                .map(|b| &b.rest)
                .or_else(|| world.get::<OriginalScarf>(entity).map(|s| &s.rest))
        } else {
            None
        }
        .or_else(|| world.get::<Transform>(entity));
        if let Some(tf) = tf {
            matrix = tf.to_matrix() * matrix;
        }
        let Some(parent) = world.get::<ChildOf>(entity) else {
            break;
        };
        entity = parent.parent();
    }
    matrix
}

/// Current-frame hand transform, including animated ancestors. Called before
/// transform propagation so carried props do not lag one frame behind the hand.
pub(crate) fn hand_transform(world: &mut World, player: Entity, left: bool) -> Option<Transform> {
    let index = world
        .resource::<OriginalAnimations>()
        .index
        .bones
        .iter()
        .position(|n| n == if left { "hand.l" } else { "hand.r" })?;
    let entity = world
        .query::<(Entity, &OriginalBone)>()
        .iter(world)
        .find(|(_, b)| b.player == player && b.index == index)?
        .0;
    let mut current = entity;
    let mut matrix = Mat4::IDENTITY;
    loop {
        matrix = world.get::<Transform>(current)?.to_matrix() * matrix;
        let Some(parent) = world.get::<ChildOf>(current) else {
            break;
        };
        current = parent.parent();
    }
    Some(Transform::from_matrix(matrix))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_clips_are_complete_and_normalized() {
        let bank = OriginalAnimations::default();
        assert_eq!(bank.index.bones.len(), 52);
        assert_eq!(bank.index.clips.len(), 35);
        let stride = 52 * 4 + 3;
        let mut end = 0;
        for (_, c) in &bank.index.clips {
            assert_eq!(c.start, end);
            end += c.frames;
            assert!(c.duration > 0.0 && c.frames >= 2);
            for frame in c.start..end {
                for bone in 0..52 {
                    let p = frame * stride + bone * 4;
                    let q = &bank.samples[p..p + 4];
                    assert!((q.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 0.0001);
                }
            }
        }
        assert_eq!(bank.samples.len(), end * stride);
        assert!(bank.samples.iter().all(|v| v.is_finite()));
    }
    #[test]
    fn loops_clamp_and_wake_height_are_continuous() {
        let bank = OriginalAnimations::default();
        let walk = bank.clip("Walk");
        let duration = bank.index.clips[walk].1.duration;
        let a = bank.sample(walk, 0.17, 7);
        let b = bank.sample(walk, duration + 0.17, 7);
        assert!(a.rotation.dot(b.rotation).abs() > 0.99999);
        let wake = bank.clip("WakeUp");
        let end = bank.sample(wake, bank.index.clips[wake].1.duration, 0);
        assert!((end.position - bank.sample(wake, 999.0, 0).position).length() < 0.00001);
        assert!(
            (end.position.y - bank.sample(bank.clip("Idle"), 0.0, 0).position.y).abs() < 0.00001
        );
        assert!(
            (bank.sample(wake, 0.0, 0).position - bank.sample(bank.clip("Sleep"), 0.0, 0).position)
                .length()
                < 0.01
        );
    }
}
