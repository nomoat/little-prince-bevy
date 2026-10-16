//! Synchronize source clips with the chapter's sequence clocks and held props.
use super::systems::{B612State, Phase, Seq, Sites};
use crate::core::{
    original_rig::{OriginalAnimationRequest, hand_transform},
    prelude::*,
};

pub fn requests(mut commands: Commands, state: Option<Res<B612State>>) {
    let Some(s) = state else {
        return;
    };
    let mut request = OriginalAnimationRequest {
        carry_can: s.carrying,
        ..default()
    };
    let clip = if s.phase == Phase::Opening && !s.opened {
        if s.open_t < 4.5 {
            Some(("Sleep", s.open_t))
        } else if s.open_t < 4.5 + 3.9 / 1.2 {
            Some(("WakeUp", (s.open_t - 4.5) * 1.2))
        } else {
            Some(("Idle", s.open_t - 4.5 - 3.9 / 1.2))
        }
    } else {
        match s.seq {
            Seq::Fill { t, stage, .. } if stage >= 1 => {
                let elapsed = t + match stage {
                    1 => 0.0,
                    2 => 0.6,
                    _ => 2.8,
                };
                Some(("FillCan", elapsed / 3.3 * 6.3333335))
            }
            Seq::Water { t, stage } if stage >= 1 => {
                let elapsed = t + match stage {
                    1 => 0.0,
                    2 => 0.7,
                    _ => 2.9,
                };
                Some(("WaterRose", elapsed / 3.4 * 4.4333334))
            }
            Seq::Ritual { t, stage } if (1..=3).contains(&stage) => {
                let elapsed = t + match stage {
                    1 => 0.0,
                    2 => 0.8,
                    _ => 2.0,
                };
                Some(("DomeLift", elapsed / 3.2 * 3.5))
            }
            Seq::Place { t, stage } => Some((
                "DomePlace",
                (t + if stage >= 1 { 0.9 } else { 0.0 }) / 1.9 * 3.333334,
            )),
            Seq::Sit { t, stage: 1, .. } if t < 2.666667 => Some(("SitDown", t)),
            _ if s.cloche_held => Some(("DomeHold", s.time)),
            _ => None,
        }
    };
    if let Some((name, time)) = clip {
        request.clip = Some(name);
        request.time = Some(time);
    }
    commands.entity(s.player).insert(request);
}

/// Run after bone animation and before transform propagation. Grip coordinates
/// are from WateringCan/Grip in the original scene (Unity X reflected).
pub fn attach_props(world: &mut World) {
    let Some(s) = world.get_resource::<B612State>() else {
        return;
    };
    let (player, can, cloche, carrying, held, seq) =
        (s.player, s.can, s.cloche, s.carrying, s.cloche_held, s.seq);
    let Some(right) = hand_transform(world, player, false) else {
        return;
    };
    if carrying {
        // Inverse CarryCan hand orientation × the can's authored carry pose.
        let grip_rotation =
            Quat::from_xyzw(-0.6576291, 0.12099257, 0.7036385, -0.2403701).normalize();
        let rotation = right.rotation * grip_rotation;
        let grip = Vec3::new(-0.003927203, 0.2203537, -0.09259842);
        if let Some(mut tf) = world.get_mut::<Transform>(can) {
            tf.rotation = rotation;
            tf.translation = right.translation - rotation * (grip * tf.scale);
        }
    }
    let Some(left) = hand_transform(world, player, true) else {
        return;
    };
    let hands = (left.translation + right.translation) * 0.5 - Vec3::Y * 0.05;
    let home = world.resource::<Sites>().cloche_home;
    let ease = |t: f32| {
        let x = t.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    let position = match seq {
        Seq::Ritual { stage: 2, t } => Some(home.lerp(hands, ease(t / 1.2))),
        Seq::Place { stage: 0, .. } => Some(hands),
        Seq::Place { stage: 1, t } => Some(hands.lerp(home, ease(t))),
        _ if held => Some(hands),
        _ => None,
    };
    if let Some(position) = position {
        if let Some(mut tf) = world.get_mut::<Transform>(cloche) {
            tf.translation = position;
        }
    }
}
