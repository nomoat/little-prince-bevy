//! Procedural low-poly mesh building. Every placeholder model in the game is
//! built here from boxes, frusta, spheres and lathes with per-vertex colours,
//! so one shared vertex-colour material renders them all.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

/// Accumulates flat- or smooth-shaded triangles with vertex colours.
#[derive(Default, Clone)]
pub struct MeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Applied to every shape added while set (see [`MeshBuilder::with`]).
    pub xf: Transform,
}

pub fn lin(c: Color) -> [f32; 4] {
    c.to_linear().to_f32_array()
}

/// sRGB hex like `0xf2c14e` → Color.
pub fn hex(rgb: u32) -> Color {
    Color::srgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

impl MeshBuilder {
    pub fn new() -> Self {
        MeshBuilder { xf: Transform::IDENTITY, ..default() }
    }

    /// Runs `f` with an extra local transform applied to the shapes it adds.
    pub fn with(&mut self, t: Transform, f: impl FnOnce(&mut MeshBuilder)) -> &mut Self {
        let saved = self.xf;
        self.xf = saved * t;
        f(self);
        self.xf = saved;
        self
    }

    fn p(&self, v: Vec3) -> Vec3 {
        self.xf.transform_point(v)
    }
    fn n(&self, v: Vec3) -> Vec3 {
        (self.xf.rotation * (v / self.xf.scale.max(Vec3::splat(1e-6)))).normalize_or_zero()
    }

    /// Adds one flat-shaded triangle (counter-clockwise = front).
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, color: Color) {
        let (a, b, c) = (self.p(a), self.p(b), self.p(c));
        let n = (b - a).cross(c - a).normalize_or_zero();
        let col = lin(color);
        let base = self.positions.len() as u32;
        for v in [a, b, c] {
            self.positions.push(v.to_array());
            self.normals.push(n.to_array());
            self.colors.push(col);
            self.uvs.push([v.x, v.z]);
        }
        self.indices.extend([base, base + 1, base + 2]);
    }

    /// Flat quad a-b-c-d (counter-clockwise).
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, color: Color) {
        self.tri(a, b, c, color);
        self.tri(a, c, d, color);
    }

    /// Adds a smooth vertex (world normal supplied); returns its index.
    pub fn vertex(&mut self, p: Vec3, n: Vec3, color: Color, uv: Vec2) -> u32 {
        let i = self.positions.len() as u32;
        self.positions.push(self.p(p).to_array());
        self.normals.push(self.n(n).to_array());
        self.colors.push(lin(color));
        self.uvs.push(uv.to_array());
        i
    }

    /// Axis-aligned box centred at `c` with full size `s`.
    pub fn cuboid(&mut self, c: Vec3, s: Vec3, color: Color) -> &mut Self {
        let h = s * 0.5;
        let v = |x: f32, y: f32, z: f32| c + Vec3::new(x * h.x, y * h.y, z * h.z);
        let (p000, p100, p110, p010) = (v(-1., -1., -1.), v(1., -1., -1.), v(1., 1., -1.), v(-1., 1., -1.));
        let (p001, p101, p111, p011) = (v(-1., -1., 1.), v(1., -1., 1.), v(1., 1., 1.), v(-1., 1., 1.));
        self.quad(p001, p101, p111, p011, color); // +z
        self.quad(p100, p000, p010, p110, color); // -z
        self.quad(p101, p100, p110, p111, color); // +x
        self.quad(p000, p001, p011, p010, color); // -x
        self.quad(p011, p111, p110, p010, color); // +y
        self.quad(p000, p100, p101, p001, color); // -y
        self
    }

    /// Frustum / cylinder / cone along +Y from `base` (r0 at bottom, r1 at top).
    pub fn frustum(&mut self, base: Vec3, r0: f32, r1: f32, h: f32, segs: u32, color: Color) -> &mut Self {
        self.frustum_caps(base, r0, r1, h, segs, color, true, true)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn frustum_caps(&mut self, base: Vec3, r0: f32, r1: f32, h: f32, segs: u32, color: Color, bottom: bool, top: bool) -> &mut Self {
        let segs = segs.max(3);
        let ring = |r: f32, y: f32, i: u32| {
            let a = i as f32 / segs as f32 * TAU;
            base + Vec3::new(a.cos() * r, y, -a.sin() * r)
        };
        for i in 0..segs {
            let (a0, a1) = (ring(r0, 0.0, i), ring(r0, 0.0, i + 1));
            let (b0, b1) = (ring(r1, h, i), ring(r1, h, i + 1));
            if r1 > 1e-5 {
                self.quad(a0, a1, b1, b0, color);
            } else {
                self.tri(a0, a1, b0, color);
            }
            if top && r1 > 1e-5 {
                self.tri(base + Vec3::Y * h, b0, b1, color);
            }
            if bottom && r0 > 1e-5 {
                self.tri(base, a1, a0, color);
            }
        }
        self
    }

    /// Cylinder between two arbitrary points.
    pub fn tube(&mut self, a: Vec3, b: Vec3, r0: f32, r1: f32, segs: u32, color: Color) -> &mut Self {
        let d = b - a;
        let len = d.length();
        if len < 1e-5 {
            return self;
        }
        let rot = Quat::from_rotation_arc(Vec3::Y, d / len);
        let t = Transform::from_translation(a).with_rotation(rot);
        self.with(t, |m| {
            m.frustum(Vec3::ZERO, r0, r1, len, segs, color);
        })
    }

    /// Low-poly UV sphere (flat shaded), squashable via `scale`.
    pub fn sphere(&mut self, c: Vec3, r: f32, rings: u32, segs: u32, color: Color) -> &mut Self {
        self.ellipsoid(c, Vec3::splat(r), rings, segs, color)
    }

    pub fn ellipsoid(&mut self, c: Vec3, r: Vec3, rings: u32, segs: u32, color: Color) -> &mut Self {
        let rings = rings.max(2);
        let segs = segs.max(3);
        let pt = |i: u32, j: u32| {
            let th = i as f32 / rings as f32 * PI;
            let ph = j as f32 / segs as f32 * TAU;
            c + Vec3::new(th.sin() * ph.cos() * r.x, th.cos() * r.y, -th.sin() * ph.sin() * r.z)
        };
        for i in 0..rings {
            for j in 0..segs {
                let (a, b, cc, d) = (pt(i, j), pt(i + 1, j), pt(i + 1, j + 1), pt(i, j + 1));
                if i == 0 {
                    self.tri(a, b, cc, color);
                } else if i == rings - 1 {
                    self.tri(a, b, d, color);
                } else {
                    self.quad(a, b, cc, d, color);
                }
            }
        }
        self
    }

    /// Lathe: revolves a profile of (radius, y) points around +Y.
    pub fn lathe(&mut self, base: Vec3, profile: &[(f32, f32)], segs: u32, color: Color) -> &mut Self {
        let segs = segs.max(3);
        for w in profile.windows(2) {
            let (r0, y0) = w[0];
            let (r1, y1) = w[1];
            for i in 0..segs {
                let a0 = i as f32 / segs as f32 * TAU;
                let a1 = (i + 1) as f32 / segs as f32 * TAU;
                let p = |r: f32, y: f32, a: f32| base + Vec3::new(a.cos() * r, y, -a.sin() * r);
                let (p00, p01, p10, p11) = (p(r0, y0, a0), p(r0, y0, a1), p(r1, y1, a0), p(r1, y1, a1));
                if r0 < 1e-5 {
                    self.tri(p00, p11, p10, color);
                } else if r1 < 1e-5 {
                    self.tri(p00, p01, p10, color);
                } else {
                    self.quad(p00, p01, p11, p10, color);
                }
            }
        }
        self
    }

    /// Irregular rock: a jittered low-poly ellipsoid (deterministic by seed).
    pub fn rock(&mut self, c: Vec3, r: Vec3, seed: u32, color: Color) -> &mut Self {
        let rings = 4;
        let segs = 7;
        let pt = |i: u32, j: u32| {
            let th = i as f32 / rings as f32 * PI;
            let ph = (j % segs) as f32 / segs as f32 * TAU;
            let k = 1.0 + (hash2(i as i32, (j % segs) as i32 + seed as i32 * 31) - 0.5) * 0.35;
            let k = if i == 0 || i == rings { 1.0 } else { k };
            c + Vec3::new(th.sin() * ph.cos() * r.x * k, th.cos() * r.y * k, -th.sin() * ph.sin() * r.z * k)
        };
        for i in 0..rings {
            for j in 0..segs {
                let (a, b, cc, d) = (pt(i, j), pt(i + 1, j), pt(i + 1, j + 1), pt(i, j + 1));
                let shade = 0.9 + 0.2 * hash2(i as i32 * 7, j as i32 + seed as i32);
                let col = scale_color(color, shade);
                if i == 0 {
                    self.tri(a, b, cc, col);
                } else if i == rings - 1 {
                    self.tri(a, b, d, col);
                } else {
                    self.quad(a, b, cc, d, col);
                }
            }
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn append(&mut self, other: &MeshBuilder) {
        let base = self.positions.len() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.colors.extend_from_slice(&other.colors);
        self.uvs.extend_from_slice(&other.uvs);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }

    pub fn build(&self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone())
            .with_inserted_indices(Indices::U32(self.indices.clone()))
    }
}

pub fn scale_color(c: Color, k: f32) -> Color {
    let s = c.to_srgba();
    Color::srgba((s.red * k).min(1.0), (s.green * k).min(1.0), (s.blue * k).min(1.0), s.alpha)
}

pub fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (a.to_linear(), b.to_linear());
    let t = t.clamp(0.0, 1.0);
    Color::LinearRgba(LinearRgba::new(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    ))
}

// ---------------------------------------------------------------- noise ----

/// Deterministic hash → [0,1).
pub fn hash2(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// Smooth value noise in [-1,1].
pub fn value_noise(x: f32, y: f32, seed: i32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (xf, yf) = (x - x.floor(), y - y.floor());
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (u, v) = (s(xf), s(yf));
    let h = |a: i32, b: i32| hash2(a + seed * 1013, b - seed * 7919);
    let a = h(xi, yi) + (h(xi + 1, yi) - h(xi, yi)) * u;
    let b = h(xi, yi + 1) + (h(xi + 1, yi + 1) - h(xi, yi + 1)) * u;
    (a + (b - a) * v) * 2.0 - 1.0
}

/// Fractal noise in roughly [-1,1].
pub fn fbm(x: f32, y: f32, octaves: u32, seed: i32) -> f32 {
    let (mut amp, mut freq, mut sum, mut norm) = (1.0, 1.0, 0.0, 0.0);
    for o in 0..octaves {
        sum += value_noise(x * freq, y * freq, seed + o as i32) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.03;
    }
    sum / norm
}

/// Tiny deterministic RNG for scattering props.
#[derive(Clone, Copy, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) | 1)
    }
    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }
    /// Uniform in [0,1).
    pub fn f(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}
