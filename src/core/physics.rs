//! Minimal kinematic physics for a code-driven world: static colliders
//! (vertical cylinders, yaw-rotated boxes) that block the player, whose tops
//! can be stood on (steps up to `STEP_HEIGHT`), plus ray tests for the camera.
//! The terrain heightfield is the main ground (see planet.rs).

use super::planet::Terrain;
use bevy::prelude::*;

/// Maximum step the character climbs without jumping (m).
pub const STEP_HEIGHT: f32 = 0.36;

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// Vertical cylinder; `height` upward from the entity origin.
    Cylinder { radius: f32, height: f32 },
    /// Box with half extents, centred at origin + (0, half.y, 0) (sits on the origin),
    /// rotated by the entity's yaw.
    Box { half: Vec3 },
}

/// Static collider. Add to any entity with a Transform (world space is read
/// from GlobalTransform; scale is ignored).
#[derive(Component, Clone, Copy, Debug)]
pub struct Collider {
    pub shape: Shape,
    /// Blocks the player.
    pub solid: bool,
    /// Its top can be walked on.
    pub walkable_top: bool,
    /// Blocks the camera.
    pub blocks_camera: bool,
    /// Extra local offset of the shape.
    pub offset: Vec3,
}

impl Collider {
    pub fn cylinder(radius: f32, height: f32) -> Self {
        Collider { shape: Shape::Cylinder { radius, height }, solid: true, walkable_top: true, blocks_camera: true, offset: Vec3::ZERO }
    }
    pub fn cuboid(size: Vec3) -> Self {
        Collider { shape: Shape::Box { half: size * 0.5 }, solid: true, walkable_top: true, blocks_camera: true, offset: Vec3::ZERO }
    }
    pub fn with_offset(mut self, o: Vec3) -> Self {
        self.offset = o;
        self
    }
    pub fn no_camera(mut self) -> Self {
        self.blocks_camera = false;
        self
    }
    pub fn not_walkable(mut self) -> Self {
        self.walkable_top = false;
        self
    }
}

/// World-space snapshot of every collider, rebuilt each frame (cheap: tens of items).
#[derive(Resource, Default)]
pub struct ColliderWorld {
    pub items: Vec<WorldCollider>,
}

#[derive(Clone, Copy, Debug)]
pub struct WorldCollider {
    pub entity: Entity,
    pub pos: Vec3,
    pub yaw: f32,
    pub c: Collider,
}

impl WorldCollider {
    fn bottom_top(&self) -> (f32, f32) {
        match self.c.shape {
            Shape::Cylinder { height, .. } => (self.pos.y, self.pos.y + height),
            Shape::Box { half } => (self.pos.y, self.pos.y + half.y * 2.0),
        }
    }
    /// Local XZ of a world point (box frame).
    fn local(&self, p: Vec3) -> Vec2 {
        let d = Vec2::new(p.x - self.pos.x, p.z - self.pos.z);
        let (s, c) = (-self.yaw).sin_cos();
        Vec2::new(d.x * c + d.y * s, -d.x * s + d.y * c)
    }
    fn world_dir(&self, l: Vec2) -> Vec2 {
        let (s, c) = self.yaw.sin_cos();
        Vec2::new(l.x * c + l.y * s, -l.x * s + l.y * c)
    }
    /// Is the XZ point inside the shape inflated by `r`?
    pub fn contains_xz(&self, p: Vec3, r: f32) -> bool {
        match self.c.shape {
            Shape::Cylinder { radius, .. } => Vec2::new(p.x - self.pos.x, p.z - self.pos.z).length() < radius + r,
            Shape::Box { half } => {
                let l = self.local(p);
                l.x.abs() < half.x + r && l.y.abs() < half.z + r
            }
        }
    }
    /// Horizontal push-out vector for a circle of radius `r` at `p`, if overlapping.
    pub fn push_out(&self, p: Vec3, r: f32) -> Option<Vec2> {
        match self.c.shape {
            Shape::Cylinder { radius, .. } => {
                let d = Vec2::new(p.x - self.pos.x, p.z - self.pos.z);
                let len = d.length();
                let min = radius + r;
                if len < min {
                    let n = if len > 1e-4 { d / len } else { Vec2::X };
                    Some(n * (min - len))
                } else {
                    None
                }
            }
            Shape::Box { half } => {
                let l = self.local(p);
                // closest point on the box rectangle
                let q = Vec2::new(l.x.clamp(-half.x, half.x), l.y.clamp(-half.z, half.z));
                let d = l - q;
                let len = d.length();
                if len > 1e-4 {
                    if len < r {
                        return Some(self.world_dir(d / len * (r - len)));
                    }
                    return None;
                }
                // centre inside the box: push along the smallest axis
                let px = half.x + r - l.x.abs();
                let pz = half.z + r - l.y.abs();
                let v = if px < pz { Vec2::new(px * l.x.signum(), 0.0) } else { Vec2::new(0.0, pz * l.y.signum()) };
                Some(self.world_dir(v))
            }
        }
    }
}

pub(crate) fn rebuild_collider_world(
    mut world: ResMut<ColliderWorld>,
    q: Query<(Entity, &Collider, &GlobalTransform)>,
) {
    world.items.clear();
    for (e, c, gt) in &q {
        let (_, rot, pos) = gt.to_scale_rotation_translation();
        let fwd = rot * Vec3::Z;
        let yaw = fwd.x.atan2(fwd.z);
        let off = rot * c.offset;
        world.items.push(WorldCollider { entity: e, pos: pos + off, yaw, c: *c });
    }
}

impl ColliderWorld {
    /// Highest walkable collider top under `p` (radius r) that is not above `max_y`.
    pub fn support_top(&self, p: Vec3, r: f32, max_y: f32) -> Option<f32> {
        let mut best: Option<f32> = None;
        for it in &self.items {
            if !it.c.walkable_top {
                continue;
            }
            let (_, top) = it.bottom_top();
            if top <= max_y && it.contains_xz(p, r * 0.6) {
                best = Some(best.map_or(top, |b: f32| b.max(top)));
            }
        }
        best
    }

    /// Resolves horizontal penetration for a vertical capsule [feet, feet+height].
    pub fn resolve(&self, mut p: Vec3, r: f32, height: f32) -> Vec3 {
        for _ in 0..3 {
            let mut moved = false;
            for it in &self.items {
                if !it.c.solid {
                    continue;
                }
                let (bottom, top) = it.bottom_top();
                // standing on it / above it, or entirely below it
                if p.y >= top - STEP_HEIGHT || p.y + height <= bottom {
                    continue;
                }
                if let Some(push) = it.push_out(p, r) {
                    p.x += push.x;
                    p.z += push.y;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        p
    }

    /// Distance along `dir` (unit) from `from` to the first camera-blocking
    /// collider within `max`, sampled (good enough for a follow camera).
    pub fn ray_camera(&self, from: Vec3, dir: Vec3, max: f32, pad: f32) -> Option<f32> {
        let steps = (max / 0.12).ceil() as i32;
        for i in 1..=steps {
            let t = i as f32 * max / steps as f32;
            let p = from + dir * t;
            for it in &self.items {
                if !it.c.blocks_camera {
                    continue;
                }
                let (bottom, top) = it.bottom_top();
                if p.y > bottom - pad && p.y < top + pad && it.contains_xz(p, pad) {
                    return Some((t - 0.12).max(0.0));
                }
            }
        }
        None
    }
}

/// Ground height under `p` considering terrain and walkable collider tops.
pub fn ground_height(terrain: Option<&Terrain>, cols: &ColliderWorld, p: Vec3, r: f32) -> f32 {
    let t = terrain.map(|t| t.height(p.x, p.z)).unwrap_or(0.0);
    match cols.support_top(p, r, p.y + STEP_HEIGHT) {
        Some(top) => top.max(t),
        None => t,
    }
}
