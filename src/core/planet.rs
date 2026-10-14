//! The planet: a DOMED TERRAIN, exactly like the original (B612PlanetParams):
//! a heightfield shaped as a sphere cap ("drop 4-5 m at the walkable edge"),
//! with world-Y gravity — the planet is not a true sphere. Past the walkable
//! disc the ground rounds over a lip into a rocky rim and an underside, so the
//! world reads as a little floating planet. A soft boundary pushes the player
//! back toward the centre between `soft_radius` and `hard_radius`.

use super::meshkit::{MeshBuilder, fbm, hash2, hex, lin, mix_color};
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Shape parameters (names follow the original's B612PlanetParams).
#[derive(Clone, Debug)]
pub struct PlanetParams {
    /// Planet centre in world XZ.
    pub center: Vec2,
    /// World y of the ground at the centre (top of the cap).
    pub top_y: f32,
    /// Sphere-cap radius (m). Original PRD: 50-60, drop 4-5 m at the walkable edge.
    pub cap_radius: f32,
    /// 0 = pure sphere cap; > 0 mixes in a quartic that flattens the middle.
    pub cap_flatten: f32,
    /// Walkable disc radius (m).
    pub walk_radius: f32,
    /// Soft push toward the centre starts here …
    pub soft_radius: f32,
    /// … and saturates here.
    pub hard_radius: f32,
    /// Absolute clamp (never passable).
    pub failsafe_radius: f32,
    /// The ground rounds over into the rim between `lip_start` and `hole_radius`.
    pub lip_start: f32,
    pub hole_radius: f32,
    /// Extra drop at `hole_radius`.
    pub lip_depth: f32,
    /// Gentle ground undulation (m) and its wavelength.
    pub noise_amp: f32,
    pub noise_wavelength: f32,
    /// Depth of the underside below the lip (m).
    pub underside_depth: f32,
    /// Rock displacement on the rim / underside (m).
    pub rim_noise: f32,
    pub rim_segments: u32,
    /// Push speed at `hard_radius` (the jog is 2.4, the Shift run 3.5).
    pub max_push: f32,
    pub min_push: f32,
    /// Ground colours: base, variation, path/dry, rim rock.
    pub ground: Color,
    pub ground2: Color,
    pub dry: Color,
    pub rock: Color,
    /// Optional manifest texture key for the walkable ground (tiled in XZ).
    pub ground_texture: Option<String>,
    pub texture_tiling: f32,
    pub seed: i32,
    /// Local shape edits (mounds, flattened pads, pits).
    pub mods: Vec<TerrainMod>,
}

impl Default for PlanetParams {
    fn default() -> Self {
        PlanetParams {
            center: Vec2::ZERO,
            top_y: 0.0,
            cap_radius: 55.0,
            cap_flatten: 0.15,
            walk_radius: 22.0,
            soft_radius: 20.0,
            hard_radius: 22.5,
            failsafe_radius: 23.5,
            lip_start: 22.5,
            hole_radius: 26.0,
            lip_depth: 1.6,
            noise_amp: 0.18,
            noise_wavelength: 7.0,
            underside_depth: 14.0,
            rim_noise: 0.8,
            rim_segments: 120,
            max_push: 4.2,
            min_push: 0.4,
            ground: hex(0x9fb36c),
            ground2: hex(0x86a35a),
            dry: hex(0xc9b98a),
            rock: hex(0x8d7f70),
            ground_texture: None,
            texture_tiling: 0.25,
            seed: 7,
            mods: Vec::new(),
        }
    }
}

impl PlanetParams {
    /// Convenience: a planet of walkable radius `r` with proportional rings.
    pub fn sized(walk_radius: f32) -> Self {
        let r = walk_radius;
        let cap = (r * r / (2.0 * 4.5)).max(r * 1.6);
        PlanetParams {
            cap_radius: cap,
            walk_radius: r,
            soft_radius: r - 2.0,
            hard_radius: r + 0.5,
            failsafe_radius: r + 1.5,
            lip_start: r + 0.5,
            hole_radius: r + 4.0,
            underside_depth: (r * 0.6).max(8.0),
            rim_segments: ((r * 5.5) as u32).clamp(64, 220),
            ..default()
        }
    }
}

/// Local edits to the heightfield.
#[derive(Clone, Debug)]
pub enum TerrainMod {
    /// Smooth bump of `height` (negative = dip) over `radius`.
    Mound { center: Vec2, radius: f32, height: f32 },
    /// Levels the ground to `height` (or to the height at `center` when None)
    /// inside `radius`, blending out over `blend` metres.
    Flatten { center: Vec2, radius: f32, blend: f32, height: Option<f32> },
    /// Ground colour override (paths, sand patches); `color` blended over `blend`.
    Paint { center: Vec2, radius: f32, blend: f32, color: Color },
}

/// The live heightfield (resource while a chapter with a planet runs).
#[derive(Resource, Clone, Debug)]
pub struct Terrain {
    pub params: PlanetParams,
    /// Resolved flatten heights (Flatten.height None → sampled).
    flat_heights: Vec<f32>,
}

impl Terrain {
    pub fn new(params: PlanetParams) -> Self {
        let mut t = Terrain { params, flat_heights: Vec::new() };
        // Each auto-height flatten samples the terrain with the earlier
        // flattens applied, so heights must be stored as they are found.
        let flats: Vec<(Vec2, Option<f32>)> = t
            .params
            .mods
            .iter()
            .filter_map(|m| match m {
                TerrainMod::Flatten { center, height, .. } => Some((*center, *height)),
                _ => None,
            })
            .collect();
        for (center, height) in flats {
            let n = t.flat_heights.len();
            let h = height.unwrap_or_else(|| t.base_height(center.x, center.y, n));
            t.flat_heights.push(h);
        }
        t
    }

    /// Distance from the planet centre in XZ.
    pub fn radial(&self, x: f32, z: f32) -> f32 {
        Vec2::new(x, z).distance(self.params.center)
    }

    /// Height of the sphere cap + lip + noise + mounds (without flattens after `upto`).
    fn base_height(&self, x: f32, z: f32, upto_flat: usize) -> f32 {
        let p = &self.params;
        let r = self.radial(x, z);
        let rr = r.min(p.hole_radius + 6.0);
        let cap = p.cap_radius;
        let sphere = cap - (cap * cap - (rr * rr).min(cap * cap * 0.98)).sqrt();
        // Quartic flatten with the same drop at walk_radius.
        let w = p.walk_radius.max(1.0);
        let drop_w = cap - (cap * cap - (w * w).min(cap * cap * 0.98)).sqrt();
        let quartic = drop_w * (rr / w).powi(4);
        let mut y = p.top_y - (sphere * (1.0 - p.cap_flatten) + quartic * p.cap_flatten);
        // Lip: round over between lip_start and hole_radius.
        if rr > p.lip_start {
            let t = ((rr - p.lip_start) / (p.hole_radius - p.lip_start).max(0.1)).min(1.5);
            y -= p.lip_depth * t * t;
        }
        // Gentle undulation, fading out toward the centre spawn and the lip.
        if p.noise_amp > 0.0 {
            let f = 1.0 / p.noise_wavelength.max(0.5);
            y += fbm(x * f, z * f, 3, p.seed) * p.noise_amp;
        }
        let mut fi = 0;
        for m in &p.mods {
            match m {
                TerrainMod::Mound { center, radius, height } => {
                    let d = Vec2::new(x, z).distance(*center) / radius.max(0.01);
                    if d < 1.0 {
                        let k = (1.0 - d * d).powi(2);
                        y += height * k;
                    }
                }
                TerrainMod::Flatten { center, radius, blend, .. } => {
                    if fi < upto_flat {
                        let d = Vec2::new(x, z).distance(*center);
                        let k = 1.0 - smoothstep(*radius, radius + blend.max(0.01), d);
                        y = y + (self.flat_heights[fi] - y) * k;
                    }
                    fi += 1;
                }
                TerrainMod::Paint { .. } => {}
            }
        }
        y
    }

    /// Ground height at world (x, z).
    pub fn height(&self, x: f32, z: f32) -> f32 {
        self.base_height(x, z, usize::MAX)
    }

    /// Point on the ground.
    pub fn ground(&self, x: f32, z: f32) -> Vec3 {
        Vec3::new(x, self.height(x, z), z)
    }

    /// Unit ground normal (finite differences).
    pub fn normal(&self, x: f32, z: f32) -> Vec3 {
        let e = 0.15;
        let hx = self.height(x + e, z) - self.height(x - e, z);
        let hz = self.height(x, z + e) - self.height(x, z - e);
        Vec3::new(-hx, 2.0 * e, -hz).normalize()
    }

    /// Slope angle in degrees.
    pub fn slope_deg(&self, x: f32, z: f32) -> f32 {
        self.normal(x, z).angle_between(Vec3::Y).to_degrees()
    }

    /// Ground colour at a point (base/variation/paint).
    pub fn color(&self, x: f32, z: f32) -> Color {
        let p = &self.params;
        let n = fbm(x * 0.21, z * 0.21, 2, p.seed + 11) * 0.5 + 0.5;
        let mut c = mix_color(p.ground, p.ground2, n);
        for m in &p.mods {
            if let TerrainMod::Paint { center, radius, blend, color } = m {
                let d = Vec2::new(x, z).distance(*center);
                let k = 1.0 - smoothstep(*radius, radius + blend.max(0.01), d);
                if k > 0.0 {
                    c = mix_color(c, *color, k);
                }
            }
        }
        let r = self.radial(x, z);
        if r > p.walk_radius {
            let k = ((r - p.walk_radius) / (p.hole_radius - p.walk_radius).max(0.1)).clamp(0.0, 1.0);
            c = mix_color(c, p.dry, k * 0.6);
        }
        c
    }

    /// Boundary push velocity (m/s, XZ) for a player at `pos` — B612PlanetBoundary.
    pub fn boundary_push(&self, pos: Vec3) -> Vec2 {
        let p = &self.params;
        let d = Vec2::new(pos.x, pos.z) - p.center;
        let r = d.length();
        if r <= p.soft_radius || r < 1e-3 {
            return Vec2::ZERO;
        }
        let t = ((r - p.soft_radius) / (p.hard_radius - p.soft_radius).max(0.01)).clamp(0.0, 1.0);
        let speed = p.min_push + (p.max_push - p.min_push) * t * t;
        -d / r * speed
    }

    /// Clamps a position inside the failsafe ring.
    pub fn clamp_inside(&self, pos: Vec3) -> Vec3 {
        let p = &self.params;
        let d = Vec2::new(pos.x, pos.z) - p.center;
        let r = d.length();
        if r > p.failsafe_radius {
            let q = p.center + d / r * p.failsafe_radius;
            Vec3::new(q.x, pos.y, q.y)
        } else {
            pos
        }
    }

    /// Polar → world helper: `angle_deg` measured like the original's yaw
    /// (0 = +Z, 90 = +X), `r` metres from the centre; returns the ground point.
    pub fn polar(&self, angle_deg: f32, r: f32) -> Vec3 {
        let a = angle_deg.to_radians();
        let c = self.params.center;
        self.ground(c.x + a.sin() * r, c.y + a.cos() * r)
    }

    /// Builds the top surface + rim + underside meshes.
    pub fn build_meshes(&self) -> (Mesh, Mesh) {
        let p = &self.params;
        let c = p.center;
        // --- Top: polar grid out to hole_radius.
        let mut top = MeshBuilder::new();
        let ring_step = 0.55f32;
        let rings = (p.hole_radius / ring_step).ceil() as u32;
        let segs = p.rim_segments.max(48);
        let tiling = p.texture_tiling;
        let mut idx = Vec::with_capacity(((rings + 1) * segs) as usize);
        // centre vertex
        let centre = top.vertex(
            self.ground(c.x, c.y),
            self.normal(c.x, c.y),
            self.color(c.x, c.y),
            Vec2::new(c.x, c.y) * tiling,
        );
        for i in 1..=rings {
            let r = (i as f32 * ring_step).min(p.hole_radius);
            for j in 0..segs {
                let a = j as f32 / segs as f32 * TAU;
                let (x, z) = (c.x + a.cos() * r, c.y - a.sin() * r);
                let v = top.vertex(self.ground(x, z), self.normal(x, z), self.color(x, z), Vec2::new(x, z) * tiling);
                idx.push(v);
            }
        }
        let at = |i: u32, j: u32| idx[((i - 1) * segs + (j % segs)) as usize];
        for j in 0..segs {
            top.indices.extend([centre, at(1, j), at(1, j + 1)]);
        }
        for i in 1..rings {
            for j in 0..segs {
                let (a, b, cc, d) = (at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
                top.indices.extend([a, b, cc, a, cc, d]);
            }
        }
        // --- Rim + underside: flat-shaded rocky cone below the lip.
        let mut rim = MeshBuilder::new();
        let edge_r = p.hole_radius;
        let layers = 5u32;
        let ring_pts = |layer: u32, j: u32| -> Vec3 {
            let a = (j % segs) as f32 / segs as f32 * TAU;
            let t = layer as f32 / layers as f32;
            let (x0, z0) = (c.x + a.cos() * edge_r, c.y - a.sin() * edge_r);
            let lip_y = self.height(x0, z0);
            if layer == 0 {
                return Vec3::new(x0, lip_y, z0);
            }
            // radius shrinks with depth like a rounded cone (floating-island underside)
            let rr = edge_r * (1.0 - t.powf(1.4) * 0.92) + 0.4;
            let n = (hash2(j as i32 % segs as i32, layer as i32 * 17 + p.seed) - 0.5) * p.rim_noise * (1.0 - t * 0.5);
            let y = lip_y - 0.6 - p.underside_depth * t.powf(0.85) + n * 0.6;
            Vec3::new(c.x + a.cos() * (rr + n), y, c.y - a.sin() * (rr + n))
        };
        let tip = Vec3::new(c.x, p.top_y - p.underside_depth - 2.5, c.y);
        for l in 0..layers {
            for j in 0..segs {
                let shade = 0.82 + 0.25 * hash2(j as i32, l as i32 + 99);
                let col = mix_color(p.rock, hex(0x5a4f48), l as f32 / layers as f32 * 0.6);
                let col = super::meshkit::scale_color(col, shade);
                let (a, b, cc, d) = (ring_pts(l, j), ring_pts(l + 1, j), ring_pts(l + 1, j + 1), ring_pts(l, j + 1));
                // Outward-facing (a top, b below).
                rim.quad(a, b, cc, d, col);
            }
        }
        for j in 0..segs {
            let col = super::meshkit::scale_color(hex(0x4e4540), 0.85 + 0.2 * hash2(j as i32, 5));
            rim.tri(ring_pts(layers, j), tip, ring_pts(layers, j + 1), col);
        }
        let _ = lin;
        (top.build(), rim.build())
    }
}

pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Marker on the planet ground mesh entities.
#[derive(Component)]
pub struct PlanetGround;
