//! Sky, sun and the day-night cycle (B612TimeOfDay / SunCycle / DayNightCycle).
//! The sun follows a real solar arc for a latitude/declination; free play
//! advances `hours_per_minute` but never passes `cap_hour`; a story-started
//! sunset sweeps the sun down to `end_elevation` over N seconds with an
//! eased-in high sweep, a slow golden phase and an eased final sink.
//! Chapters get: [`Sky::set_hour`], [`Sky::start_sunset`], [`Sky::set_rate`],
//! [`Sky::phase`], [`Sky::sunset_done`], and palettes per [`SkyPalette`].

use super::camera::MainCamera;
use super::meshkit::{Rng, hex, lin, mix_color};
use bevy::asset::RenderAssetUsages;
use bevy::light::{CascadeShadowConfigBuilder, NotShadowCaster};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::DistanceFog;
use bevy::prelude::*;

/// DayPhase of the original.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPhase {
    Day,
    Golden,
    Dusk,
    Night,
    Dawn,
    Morning,
}

/// Colours for day / golden / dusk / night; chapters may replace any of them.
#[derive(Clone, Debug)]
pub struct SkyPalette {
    pub day_zenith: Color,
    pub day_horizon: Color,
    pub golden_zenith: Color,
    pub golden_horizon: Color,
    pub dusk_zenith: Color,
    pub dusk_horizon: Color,
    pub night_zenith: Color,
    pub night_horizon: Color,
    pub sun_day: Color,
    pub sun_dusk: Color,
    /// Fog density multiplier (0 = no fog).
    pub fog: f32,
}

impl Default for SkyPalette {
    fn default() -> Self {
        SkyPalette {
            day_zenith: hex(0x4a86d8),
            day_horizon: hex(0xc9def2),
            golden_zenith: hex(0x7489c4),
            golden_horizon: hex(0xf6cf98),
            dusk_zenith: hex(0x34406f),
            dusk_horizon: hex(0xf08a5c),
            night_zenith: hex(0x070b22),
            night_horizon: hex(0x1b2650),
            sun_day: hex(0xfff2dc),
            sun_dusk: hex(0xff9a52),
            fog: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
struct SunsetAnim {
    from_hour: f64,
    to_hour: f64,
    t: f32,
    duration: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct Sky {
    /// Solar hour (0..24).
    pub hour: f64,
    /// Sun-hours per real minute in free play (original default 0.5).
    pub hours_per_minute: f32,
    /// Free play never passes this hour (None = no cap, the cycle wraps).
    pub cap_hour: Option<f64>,
    pub latitude: f32,
    pub declination: f32,
    /// Game azimuth (deg) of true north. Game azimuth 0 = +Z, 90 = +X.
    pub north_azimuth: f32,
    /// Elevation (deg) the sunset ends at (half-sunk).
    pub end_elevation: f32,
    pub palette: SkyPalette,
    /// Force a fixed look regardless of the hour (interiors, title): 0..1 night amount.
    pub fixed_night: Option<f32>,
    sunset: Option<SunsetAnim>,
    pub sunset_finished: bool,
    /// Derived each frame.
    pub sun_elevation: f32,
    pub sun_azimuth: f32,
    pub sun_dir: Vec3,
    pub night: f32,
    pub dusk: f32,
    pub sun_intensity_scale: f32,
}

impl Default for Sky {
    fn default() -> Self {
        Sky {
            hour: 9.0,
            hours_per_minute: 0.5,
            cap_hour: Some(16.5),
            latitude: 55.0,
            declination: 2.555,
            north_azimuth: 270.0,
            end_elevation: -1.5,
            palette: SkyPalette::default(),
            fixed_night: None,
            sunset: None,
            sunset_finished: false,
            sun_elevation: 30.0,
            sun_azimuth: 120.0,
            sun_dir: Vec3::new(0.3, 0.8, 0.4).normalize(),
            night: 0.0,
            dusk: 0.0,
            sun_intensity_scale: 1.0,
        }
    }
}

impl Sky {
    pub fn set_hour(&mut self, h: f64) {
        self.hour = h.rem_euclid(24.0);
        self.sunset = None;
        self.sunset_finished = false;
    }
    /// Sun-hours per real minute; 0 freezes time. Lamplighter: very fast.
    pub fn set_rate(&mut self, hours_per_minute: f32, cap_hour: Option<f64>) {
        self.hours_per_minute = hours_per_minute;
        self.cap_hour = cap_hour;
    }
    /// Starts the sunset: the sun sinks to `end_elevation` over `seconds`.
    pub fn start_sunset(&mut self, seconds: f32) {
        let to = self.hour_for_elevation(self.end_elevation, true);
        let from = self.hour.min(to - 0.05);
        self.sunset = Some(SunsetAnim { from_hour: from, to_hour: to, t: 0.0, duration: seconds.max(0.5) });
        self.sunset_finished = false;
    }
    pub fn sunset_running(&self) -> bool {
        self.sunset.is_some()
    }
    pub fn sunset_done(&self) -> bool {
        self.sunset_finished
    }
    pub fn phase(&self) -> DayPhase {
        let e = self.sun_elevation;
        let afternoon = self.hour >= 12.0;
        if e > 15.0 {
            if afternoon { DayPhase::Day } else { DayPhase::Morning }
        } else if e > 4.0 {
            if afternoon { DayPhase::Golden } else { DayPhase::Morning }
        } else if e > -6.0 {
            if afternoon { DayPhase::Dusk } else { DayPhase::Dawn }
        } else {
            DayPhase::Night
        }
    }

    /// Solar position for an hour: (elevation°, game azimuth°).
    pub fn sun_position(&self, hour: f64) -> (f32, f32) {
        let lat = (self.latitude as f64).to_radians();
        let dec = (self.declination as f64).to_radians();
        let ha = ((hour - 12.0) * 15.0).to_radians();
        let sin_el = lat.sin() * dec.sin() + lat.cos() * dec.cos() * ha.cos();
        let el = sin_el.clamp(-1.0, 1.0).asin();
        // azimuth from north, clockwise (east = 90)
        let cos_az = (dec.sin() - el.sin() * lat.sin()) / (el.cos() * lat.cos()).max(1e-6);
        let mut az = cos_az.clamp(-1.0, 1.0).acos().to_degrees();
        if ha > 0.0 {
            az = 360.0 - az;
        }
        (el.to_degrees() as f32, (self.north_azimuth as f64 + az).rem_euclid(360.0) as f32)
    }

    pub fn hour_for_elevation(&self, elevation: f32, afternoon: bool) -> f64 {
        // bisection on the afternoon/morning half
        let (mut lo, mut hi) = if afternoon { (12.0, 23.9) } else { (0.1, 12.0) };
        for _ in 0..40 {
            let mid = (lo + hi) * 0.5;
            let (e, _) = self.sun_position(mid);
            let above = e > elevation;
            if afternoon == above {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (lo + hi) * 0.5
    }
}

fn dir_from(el_deg: f32, az_deg: f32) -> Vec3 {
    let (el, az) = (el_deg.to_radians(), az_deg.to_radians());
    Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos())
}

#[derive(Component)]
pub struct SkyDome;
#[derive(Component)]
pub struct SunDisk;
#[derive(Component)]
pub struct StarField;
#[derive(Component)]
pub struct SunLight;

#[derive(Resource)]
pub(crate) struct SkyAssets {
    dome_mesh: Handle<Mesh>,
    star_mat: Handle<StandardMaterial>,
    sun_mat: Handle<StandardMaterial>,
}

const DOME_R: f32 = 420.0;
const DOME_RINGS: u32 = 18;
const DOME_SEGS: u32 = 36;

pub(crate) fn setup_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    // Dome: UV sphere seen from inside, vertex coloured, unlit, no fog.
    let mut pos = Vec::new();
    let mut idx = Vec::new();
    for i in 0..=DOME_RINGS {
        let th = i as f32 / DOME_RINGS as f32 * std::f32::consts::PI;
        for j in 0..=DOME_SEGS {
            let ph = j as f32 / DOME_SEGS as f32 * std::f32::consts::TAU;
            pos.push([th.sin() * ph.cos() * DOME_R, th.cos() * DOME_R, th.sin() * ph.sin() * DOME_R]);
        }
    }
    let w = DOME_SEGS + 1;
    for i in 0..DOME_RINGS {
        for j in 0..DOME_SEGS {
            let (a, b, c, d) = (i * w + j, (i + 1) * w + j, (i + 1) * w + j + 1, i * w + j + 1);
            idx.extend([a, c, b, a, d, c]);
        }
    }
    let n = pos.len();
    let dome = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; n])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.5, 0.6, 0.9, 1.0]; n])
        .with_inserted_indices(Indices::U32(idx));
    let dome_mesh = meshes.add(dome);
    let dome_mat = mats.add(StandardMaterial { base_color: Color::WHITE, unlit: true, fog_enabled: false, cull_mode: None, ..default() });
    commands.spawn((Name::new("SkyDome"), SkyDome, Mesh3d(dome_mesh.clone()), MeshMaterial3d(dome_mat), Transform::default(), NotShadowCaster));

    // Stars: small quads on a sphere.
    let mut rng = Rng::new(612);
    let (mut sp, mut sc, mut si) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..900 {
        let z = rng.range(-0.25, 1.0);
        let a = rng.range(0.0, std::f32::consts::TAU);
        let r = (1.0 - z * z).sqrt();
        let d = Vec3::new(r * a.cos(), z, r * a.sin());
        let size = rng.range(0.35, 1.0) * if rng.f() > 0.97 { 1.8 } else { 1.0 };
        let (u, v) = (d.any_orthonormal_pair().0 * size, d.any_orthonormal_pair().1 * size);
        let c = d * (DOME_R - 15.0);
        let base = sp.len() as u32;
        for p in [c - u - v, c + u - v, c + u + v, c - u + v] {
            sp.push(p.to_array());
        }
        let warm = rng.f();
        let col = lin(mix_color(hex(0xdfe8ff), hex(0xffe7c2), warm));
        sc.extend([col; 4]);
        si.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let n = sp.len();
    let stars = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, sp)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; n])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, sc)
        .with_inserted_indices(Indices::U32(si));
    let star_mat = mats.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        unlit: true,
        fog_enabled: false,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    commands.spawn((Name::new("Stars"), StarField, Mesh3d(meshes.add(stars)), MeshMaterial3d(star_mat.clone()), Transform::default(), NotShadowCaster));

    // Sun disk.
    let sun_mat = mats.add(StandardMaterial { base_color: Color::WHITE, unlit: true, fog_enabled: false, ..default() });
    commands.spawn((
        Name::new("SunDisk"),
        SunDisk,
        Mesh3d(meshes.add(Sphere::new(9.0).mesh().uv(16, 10))),
        MeshMaterial3d(sun_mat.clone()),
        Transform::default(),
        NotShadowCaster,
    ));

    commands.spawn((
        Name::new("Sun"),
        SunLight,
        DirectionalLight { illuminance: 9000.0, shadow_maps_enabled: true, ..default() },
        CascadeShadowConfigBuilder { num_cascades: 1, maximum_distance: 40.0, first_cascade_far_bound: 40.0, ..default() }.build(),
        Transform::default(),
    ));
    commands.insert_resource(SkyAssets { dome_mesh, star_mat, sun_mat });
}

/// Advances the clock and the sunset animation (virtual time).
pub(crate) fn tick_sky_clock(time: Res<Time>, mut sky: ResMut<Sky>) {
    let dt = time.delta_secs();
    if let Some(mut s) = sky.sunset.clone() {
        s.t = (s.t + dt / s.duration).min(1.0);
        // ease: quick start, slow golden middle, eased final sink
        let k = sunset_curve(s.t);
        sky.hour = s.from_hour + (s.to_hour - s.from_hour) * k as f64;
        if s.t >= 1.0 {
            sky.sunset = None;
            sky.sunset_finished = true;
        } else {
            sky.sunset = Some(s);
        }
    } else if sky.hours_per_minute != 0.0 && !sky.sunset_finished {
        let mut h = sky.hour + (sky.hours_per_minute as f64) * dt as f64 / 60.0;
        if let Some(cap) = sky.cap_hour {
            h = h.min(cap.max(sky.hour));
        }
        sky.hour = h.rem_euclid(24.0);
    }
    let (el, az) = sky.sun_position(sky.hour);
    sky.sun_elevation = el;
    sky.sun_azimuth = az;
    sky.sun_dir = dir_from(el, az);
    let night = match sky.fixed_night {
        Some(n) => n,
        None => 1.0 - super::planet::smoothstep(-9.0, -1.0, el),
    };
    sky.night = night;
    sky.dusk = 1.0 - super::planet::smoothstep(1.0, 16.0, el);
}

fn sunset_curve(t: f32) -> f32 {
    // Hermite S-curve sweep for the first 30%, a slow golden middle, eased sink.
    let a = 0.3;
    if t < a {
        let u = t / a;
        0.45 * (u * u * (3.0 - 2.0 * u))
    } else {
        let u = (t - a) / (1.0 - a);
        0.45 + 0.55 * (1.0 - (1.0 - u).powf(1.6))
    }
}

/// Applies the sky state to dome colours, stars, sun disk, light, ambient and fog.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_sky(
    sky: Res<Sky>,
    assets: Res<SkyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
    cam: Query<&Transform, (With<MainCamera>, Without<SkyDome>, Without<StarField>, Without<SunDisk>)>,
    mut dome: Query<&mut Transform, (With<SkyDome>, Without<StarField>, Without<SunDisk>)>,
    mut stars: Query<&mut Transform, (With<StarField>, Without<SkyDome>, Without<SunDisk>)>,
    mut sun_disk: Query<&mut Transform, (With<SunDisk>, Without<SkyDome>, Without<StarField>)>,
    mut light: Query<(&mut DirectionalLight, &mut Transform), (With<SunLight>, Without<MainCamera>, Without<SkyDome>, Without<StarField>, Without<SunDisk>)>,
    mut fog: Query<&mut DistanceFog>,
    time: Res<Time>,
) {
    let p = &sky.palette;
    let el = sky.sun_elevation;
    let night = sky.night;
    // zenith/horizon by elevation
    let (z, h) = if el > 15.0 {
        (p.day_zenith, p.day_horizon)
    } else if el > 4.0 {
        let k = (15.0 - el) / 11.0;
        (mix_color(p.day_zenith, p.golden_zenith, k), mix_color(p.day_horizon, p.golden_horizon, k))
    } else if el > -4.0 {
        let k = (4.0 - el) / 8.0;
        (mix_color(p.golden_zenith, p.dusk_zenith, k), mix_color(p.golden_horizon, p.dusk_horizon, k))
    } else {
        let k = ((-4.0 - el) / 6.0).clamp(0.0, 1.0);
        (mix_color(p.dusk_zenith, p.night_zenith, k), mix_color(p.dusk_horizon, p.night_horizon, k))
    };
    let (z, h) = match sky.fixed_night {
        Some(n) => (mix_color(z, p.night_zenith, n), mix_color(h, p.night_horizon, n)),
        None => (z, h),
    };
    let sun_dir = sky.sun_dir;
    let glow = mix_color(p.sun_day, p.sun_dusk, sky.dusk);
    let glow_amt = (0.15 + 0.6 * sky.dusk) * (1.0 - night);
    if let Some(mut mesh) = meshes.get_mut(&assets.dome_mesh) {
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION).cloned() {
            let cols: Vec<[f32; 4]> = pos
                .iter()
                .map(|v| {
                    let d = Vec3::from(*v) / DOME_R;
                    let up = d.y;
                    let t = (1.0 - up.max(0.0)).powf(2.6);
                    let mut c = mix_color(z, h, t);
                    if up < 0.0 {
                        c = mix_color(h, mix_color(h, z, 0.5), (-up * 3.0).min(1.0));
                    }
                    let hd = Vec2::new(d.x, d.z).normalize_or_zero();
                    let sd = Vec2::new(sun_dir.x, sun_dir.z).normalize_or_zero();
                    let toward = (hd.dot(sd) * 0.5 + 0.5).powf(6.0);
                    let low = (1.0 - (up - sun_dir.y).abs() * 2.2).max(0.0);
                    c = mix_color(c, glow, toward * low * glow_amt);
                    lin(c)
                })
                .collect();
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, cols);
        }
    }
    let cam_pos = cam.single().map(|t| t.translation).unwrap_or(Vec3::ZERO);
    if let Ok(mut t) = dome.single_mut() {
        t.translation = cam_pos;
    }
    if let Ok(mut t) = stars.single_mut() {
        t.translation = cam_pos;
        t.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.004) * Quat::from_rotation_x(0.3);
    }
    if let Some(mut m) = mats.get_mut(&assets.star_mat) {
        m.base_color = Color::srgba(1.0, 1.0, 1.0, (night * 1.1 - 0.1).clamp(0.0, 1.0));
    }
    if let Ok(mut t) = sun_disk.single_mut() {
        t.translation = cam_pos + sun_dir * (DOME_R - 30.0);
        let s = 1.0 + sky.dusk * 0.6;
        t.scale = Vec3::splat(s);
    }
    if let Some(mut m) = mats.get_mut(&assets.sun_mat) {
        let c = mix_color(hex(0xfffbea), hex(0xffa860), sky.dusk);
        m.base_color = if el < -3.0 { Color::NONE } else { c };
        m.alpha_mode = if el < -3.0 { AlphaMode::Blend } else { AlphaMode::Opaque };
    }
    // Sun light: colour/intensity by elevation; dims into a moonlight at night.
    if let Ok((mut l, mut lt)) = light.single_mut() {
        let sun_up = super::planet::smoothstep(-4.0, 6.0, el);
        let col = mix_color(p.sun_day, p.sun_dusk, sky.dusk);
        let moon = hex(0x8fa4d8);
        l.color = mix_color(moon, col, sun_up);
        // a moon bright enough to read the prince by at night (was 600 lx: he
        // vanished against the bar's dark hill)
        l.illuminance = (600.0 + 900.0 * night * (1.0 - sun_up) + 9400.0 * sun_up * (0.5 + 0.5 * (1.0 - sky.dusk))) * sky.sun_intensity_scale;
        let dir = if sun_up > 0.02 { sun_dir } else { Vec3::new(-sun_dir.x, 0.6, -sun_dir.z).normalize() };
        let dir = Vec3::new(dir.x, dir.y.max(0.12), dir.z).normalize();
        *lt = Transform::from_translation(dir * 50.0).looking_at(Vec3::ZERO, Vec3::Y);
    }
    // night ambient keeps a cool moon tint instead of the near-black sky colour
    ambient.color = mix_color(mix_color(h, z, 0.5), hex(0x7080b8), night * 0.65);
    ambient.brightness = 220.0 + 380.0 * (1.0 - night) + 120.0 * night;
    clear.0 = h;
    for mut f in &mut fog {
        f.color = mix_color(h, z, 0.15).with_alpha(1.0);
        f.directional_light_color = glow.with_alpha(0.35 * (1.0 - night));
        let d = p.fog.max(0.0);
        f.falloff = if d <= 0.0 {
            bevy::pbr::FogFalloff::Linear { start: 5000.0, end: 6000.0 }
        } else {
            bevy::pbr::FogFalloff::Linear { start: 30.0 / d, end: 190.0 / d }
        };
    }
}
