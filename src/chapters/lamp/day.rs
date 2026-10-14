//! The one-minute day (B612TimeOfDay in day/night-cycle mode) and the people
//! of the lamp: the lamplighter's timed state machine and the street lamp.
//!
//! Day clock: a 60 s cycle split into the original's phase weights — day 9,
//! dusk 9, night 15, dawn 8, morning 19 seconds — with the sun's elevation
//! paced by the original's curve (it races through the horizon band). Sun
//! azimuths come from the presets (day 80°, sunset 30°, night, sunrise 358°).
//! The clock overrides the core sky's sun each frame.
//!
//! Lamplighter: rests, fidgets every 3-8 s, and when the sun's upper edge is
//! about to cross the sea line (-0.3°) he looks at the sky, raises his wand,
//! lights the lamp at dusk (touch at 1.333 s of a 2.667 s clip) or puts it
//! out at dawn (touch at 1.0 s of 2.0 s), and lowers it again.

use super::super::businessman::kit::{dir_from, ease_in_out};
use crate::core::prelude::*;

pub const CYCLE: f32 = 60.0;
/// Phase seconds: day, dusk, night, dawn, morning.
pub const PHASES: [f32; 5] = [9.0, 9.0, 15.0, 8.0, 19.0];
/// The sun's upper edge on the sea line.
pub const HORIZON_EL: f32 = -0.3;

/// Sun speed multiplier by elevation (the original's pace curve).
fn pace(el: f32) -> f32 {
    const K: [(f32, f32); 12] = [
        (-90.0, 0.5), (-14.0, 0.8), (-9.0, 1.4), (-5.0, 2.0), (-1.0, 2.4), (2.0, 2.7),
        (5.0, 2.7), (9.0, 2.4), (13.0, 2.0), (17.0, 1.5), (22.0, 1.15), (28.0, 1.0),
    ];
    if el <= K[0].0 {
        return K[0].1;
    }
    for w in K.windows(2) {
        if el <= w[1].0 {
            let t = (el - w[0].0) / (w[1].0 - w[0].0);
            return w[0].1 + (w[1].1 - w[0].1) * t;
        }
    }
    1.0
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let d = ((b - a + 540.0) % 360.0) - 180.0;
    (a + d * t).rem_euclid(360.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClockMode {
    Free,
    /// Rushes from `from` to `to` (cycle seconds, `to` may exceed one cycle) over `dur`.
    Rush { from: f64, to: f64, t: f32, dur: f32 },
    /// Runs at 1× until `at`, then holds.
    HoldAt(f64),
    Hold,
}

#[derive(Resource)]
pub struct DayClock {
    /// Unbounded clock seconds (cycle position = s mod 60).
    pub s: f64,
    pub mode: ClockMode,
    /// The prince is sitting on the sunset seat: the day runs faster
    /// (3.5×, 2.0× while the sun is within -8°..12°).
    pub seated: bool,
    /// Frozen during the opening card.
    pub frozen: bool,
    /// (cycle seconds, elevation, azimuth) samples.
    table: Vec<(f32, f32, f32)>,
    pub dusk_cross: f32,
    pub dawn_cross: f32,
    pub el: f32,
    pub az: f32,
    /// Clock seconds advanced per real second this frame.
    pub rate: f32,
    /// Crossings seen this frame: +1 dusk, -1 dawn.
    pub crossed: i8,
}

impl Default for DayClock {
    fn default() -> Self {
        let mut c = DayClock { s: 0.0, mode: ClockMode::Free, seated: false, frozen: false, table: Vec::new(), dusk_cross: 0.0, dawn_cross: 0.0, el: 20.0, az: 80.0, rate: 1.0, crossed: 0 };
        c.build_table();
        c
    }
}

impl DayClock {
    fn build_table(&mut self) {
        // (elevation(u), azimuth from, azimuth to) per phase
        let phase_el = |p: usize, u: f32| -> f32 {
            match p {
                0 => 16.0 + 27.0 * (u * std::f32::consts::PI).sin(), // day: up to the 43° preset
                1 => 16.0 - 21.0 * u,                                   // dusk: 16° → -5°
                2 => -5.0 - 25.0 * (u * std::f32::consts::PI).sin(),  // night: down to -30°
                3 => -5.0 + 7.0 * u,                                    // dawn: -5° → 2°
                _ => 2.0 + 14.0 * u,                                    // morning: 2° → 16°
            }
        };
        let az = [(80.0f32, 48.0f32), (48.0, 12.0), (12.0, 340.0), (340.0, 2.0), (2.0, 80.0)];
        let n = 160;
        let mut table = Vec::new();
        let mut start = 0.0f32;
        for (p, secs) in PHASES.iter().enumerate() {
            // seconds per unit u ∝ 1 / pace(el)
            let mut cost = vec![0.0f32; n + 1];
            for i in 1..=n {
                let u = (i as f32 - 0.5) / n as f32;
                cost[i] = cost[i - 1] + 1.0 / pace(phase_el(p, u));
            }
            let total = cost[n];
            for i in 0..n {
                let u = i as f32 / n as f32;
                let s = start + cost[i] / total * secs;
                table.push((s, phase_el(p, u), lerp_angle(az[p].0, az[p].1, u)));
            }
            start += secs;
        }
        table.push((CYCLE, table[0].1, table[0].2));
        self.table = table;
        // crossings of the horizon line
        let mut dusk = 0.0;
        let mut dawn = 0.0;
        for w in self.table.windows(2) {
            if w[0].1 > HORIZON_EL && w[1].1 <= HORIZON_EL {
                dusk = w[0].0 + (w[1].0 - w[0].0) * (w[0].1 - HORIZON_EL) / (w[0].1 - w[1].1);
            }
            if w[0].1 < HORIZON_EL && w[1].1 >= HORIZON_EL {
                dawn = w[0].0 + (w[1].0 - w[0].0) * (HORIZON_EL - w[0].1) / (w[1].1 - w[0].1);
            }
        }
        self.dusk_cross = dusk;
        self.dawn_cross = dawn;
    }

    pub fn eval(&self, s: f64) -> (f32, f32) {
        let c = (s.rem_euclid(CYCLE as f64)) as f32;
        let i = self.table.partition_point(|e| e.0 <= c).clamp(1, self.table.len() - 1);
        let (a, b) = (self.table[i - 1], self.table[i]);
        let t = ((c - a.0) / (b.0 - a.0).max(1e-4)).clamp(0.0, 1.0);
        (a.1 + (b.1 - a.1) * t, lerp_angle(a.2, b.2, t))
    }

    pub fn cycle(&self) -> f32 {
        (self.s.rem_euclid(CYCLE as f64)) as f32
    }

    /// Next clock time (unbounded) at which the cycle position equals `c`.
    pub fn next_at(&self, c: f32) -> f64 {
        let base = self.s - self.s.rem_euclid(CYCLE as f64);
        let mut t = base + c as f64;
        if t <= self.s + 0.01 {
            t += CYCLE as f64;
        }
        t
    }

    pub fn rush_to(&mut self, c: f32, dur: f32) {
        let to = self.next_at(c.rem_euclid(CYCLE));
        self.mode = ClockMode::Rush { from: self.s, to, t: 0.0, dur: dur.max(0.05) };
    }

    pub fn hold_at(&mut self, c: f32) {
        let to = self.next_at(c.rem_euclid(CYCLE));
        self.mode = ClockMode::HoldAt(to);
    }

    pub fn held(&self) -> bool {
        matches!(self.mode, ClockMode::Hold)
    }

    pub fn is_night(&self) -> bool {
        self.el < HORIZON_EL
    }

    /// Real seconds until the next crossing: (seconds, true = dusk).
    pub fn until_cross(&self) -> (f32, bool) {
        let c = self.cycle();
        let d = (self.dusk_cross - c).rem_euclid(CYCLE);
        let n = (self.dawn_cross - c).rem_euclid(CYCLE);
        let rate = self.rate.max(0.05);
        if d < n { (d / rate, true) } else { (n / rate, false) }
    }
}

/// Advances the clock (Gameplay set).
pub fn tick_clock(time: Res<Time>, mut clock: ResMut<DayClock>) {
    let dt = time.delta_secs();
    let before = clock.s;
    if clock.frozen {
        clock.rate = 0.0;
    } else {
        match clock.mode {
            ClockMode::Free | ClockMode::HoldAt(_) => {
                let mut rate = 1.0;
                if clock.seated && matches!(clock.mode, ClockMode::Free) {
                    rate = if (-8.0..=12.0).contains(&clock.el) { 2.0 } else { 3.5 };
                }
                clock.rate = rate;
                clock.s += (rate * dt) as f64;
                if let ClockMode::HoldAt(at) = clock.mode {
                    if clock.s >= at {
                        clock.s = at;
                        clock.mode = ClockMode::Hold;
                    }
                }
            }
            ClockMode::Rush { from, to, t, dur } => {
                let nt = (t + dt / dur).min(1.0);
                let k = ease_in_out(nt) as f64;
                clock.s = from + (to - from) * k;
                clock.rate = ((to - from) as f32 / dur).max(1.0);
                clock.mode = if nt >= 1.0 { ClockMode::Hold } else { ClockMode::Rush { from, to, t: nt, dur } };
            }
            ClockMode::Hold => clock.rate = 0.0,
        }
    }
    let (el, az) = clock.eval(clock.s);
    clock.el = el;
    clock.az = az;
    // crossings passed this frame (for rushes)
    clock.crossed = 0;
    if clock.s > before {
        let (e0, _) = clock.eval(before);
        if e0 > HORIZON_EL && el <= HORIZON_EL {
            clock.crossed = 1;
        } else if e0 < HORIZON_EL && el >= HORIZON_EL {
            clock.crossed = -1;
        }
    }
}

/// Writes the clock's sun into the core sky (after the core clock tick,
/// before the sky is applied).
pub fn drive_sky(clock: Res<DayClock>, mut sky: ResMut<Sky>) {
    let (el, az) = (clock.el, clock.az);
    sky.hours_per_minute = 0.0;
    sky.sun_elevation = el;
    sky.sun_azimuth = az;
    sky.sun_dir = dir_from(az, el);
    sky.night = 1.0 - crate::core::planet::smoothstep(-9.0, -1.0, el);
    sky.dusk = 1.0 - crate::core::planet::smoothstep(1.0, 16.0, el);
    // afternoon half (day, dusk, first half of the night) vs morning half
    let c = clock.cycle();
    let afternoon = c < PHASES[0] + PHASES[1] + PHASES[2] * 0.5;
    sky.hour = if afternoon { 15.0 } else { 8.0 };
}

// ------------------------------------------------------------------- lamp --

#[derive(Resource)]
pub struct LampState {
    /// 0..1 glow.
    pub lit: f32,
    pub on: bool,
}

impl Default for LampState {
    fn default() -> Self {
        LampState { lit: 0.0, on: false }
    }
}

#[derive(Component)]
pub struct LampRoot;
#[derive(Component)]
pub struct LampLight;
#[derive(Component)]
pub struct FirstStar;

/// Lamp glow: light 1.1 s, put-out 0.6 s; at full night ×2.2 intensity and ×1.35 reach.
#[allow(clippy::too_many_arguments)]
pub fn lamp_glow(
    time: Res<Time>,
    mut lamp: ResMut<LampState>,
    sky: Res<Sky>,
    root: Query<&Children, With<LampRoot>>,
    mut parts: Query<(&PartName, &mut Transform, Option<&MeshMaterial3d<StandardMaterial>>)>,
    mut lights: Query<&mut PointLight, With<LampLight>>,
    mut star: Query<(&mut Visibility, &MeshMaterial3d<StandardMaterial>), With<FirstStar>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let rate = if lamp.on { dt / 1.1 } else { -dt / 0.6 };
    lamp.lit = (lamp.lit + rate).clamp(0.0, 1.0);
    let lit = lamp.lit;
    let night = sky.night;
    let t = time.elapsed_secs();
    let waver = 1.0 + 0.07 * (t * 11.0).sin() * (t * 7.3).cos();
    for mut l in &mut lights {
        l.intensity = 60000.0 * lit * (1.0 + 1.2 * night) * waver;
        l.range = 9.0 * (1.0 + 0.35 * night);
    }
    if let Ok(children) = root.single() {
        for c in children.iter() {
            if let Ok((name, mut tf, mat)) = parts.get_mut(c) {
                match name.0.as_str() {
                    "flame" => {
                        tf.scale = Vec3::new(1.0, waver, 1.0) * lit.max(0.001);
                    }
                    "halo" => {
                        tf.scale = Vec3::splat((0.2 + 0.8 * lit) * (1.0 + 0.3 * night)).max(Vec3::splat(0.001)) * if lit < 0.01 { 0.0 } else { 1.0 };
                    }
                    "glass" => {
                        if let Some(m) = mat.and_then(|m| mats.get_mut(&m.0)) {
                            let mut m = m;
                            // panes keep 22 % of the glow
                            let k = 0.22 + 0.78 * lit;
                            m.emissive = LinearRgba::rgb(4.0, 2.6, 1.0) * k * lit;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    // the first star wakes when the lamp is lit
    for (mut v, m) in &mut star {
        let a = lit * night;
        *v = if a > 0.01 { Visibility::Inherited } else { Visibility::Hidden };
        if let Some(mut mm) = mats.get_mut(&m.0) {
            mm.base_color = Color::srgba(1.0, 0.9, 0.75, a);
        }
    }
}

// ----------------------------------------------------------- lamplighter --

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Light,
    PutOut,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fidget {
    LookSky,
    Shift,
    Wipe,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LlState {
    Rest,
    Fidget(Fidget),
    /// Looks at the sky just before raising (SkyBeforeRaise).
    SkyLook(Act),
    Raise(Act),
    Act(Act),
    Lower,
    Ready,
}

#[derive(Component)]
pub struct LamplighterRoot;

#[derive(Resource)]
pub struct Lamplighter {
    pub state: LlState,
    pub t: f32,
    pub next_fidget: f32,
    /// The story drives him (the talk): no automatic actions.
    pub scripted: bool,
    touched: bool,
    rng: Rng,
    /// Smoothed arm angle (deg about X; raised = 0).
    arm: f32,
    head_pitch: f32,
    head_yaw: f32,
    pub look_up: bool,
}

impl Default for Lamplighter {
    fn default() -> Self {
        Lamplighter { state: LlState::Rest, t: 0.0, next_fidget: 4.0, scripted: false, touched: false, rng: Rng::new(329), arm: -46.0, head_pitch: 0.0, head_yaw: 0.0, look_up: false }
    }
}

pub const SKY_LOOK: f32 = 1.0;
pub const RAISE: f32 = 1.667;
pub const LOWER: f32 = 1.0;
pub fn act_len(a: Act) -> (f32, f32) {
    match a {
        Act::Light => (2.667, 1.333),
        Act::PutOut => (2.0, 1.0),
    }
}

/// Starts an action now (used by the story during the talk).
pub fn start_act(w: &mut World, a: Act, sky_look: bool) {
    let mut l = w.resource_mut::<Lamplighter>();
    l.state = if sky_look { LlState::SkyLook(a) } else { LlState::Raise(a) };
    l.t = 0.0;
    l.touched = false;
}

#[allow(clippy::too_many_arguments)]
pub fn lamplighter(
    time: Res<Time>,
    clock: Res<DayClock>,
    mut ll: ResMut<Lamplighter>,
    mut lamp: ResMut<LampState>,
    root: Query<(&Children, &GlobalTransform), With<LamplighterRoot>>,
    mut parts: Query<(&PartName, &mut Transform)>,
    player: Query<&Transform, (With<Player>, Without<PartName>)>,
    mut audio: MessageWriter<AudioCmd>,
) {
    let dt = time.delta_secs();
    ll.t += dt;
    // keep the lamp right when a rush sweeps past a crossing
    if clock.crossed == 1 && !lamp.on && !matches!(ll.state, LlState::Act(Act::Light) | LlState::Raise(Act::Light)) && ll.scripted {
        lamp.on = true;
    }
    if clock.crossed == -1 && lamp.on && !matches!(ll.state, LlState::Act(Act::PutOut) | LlState::Raise(Act::PutOut)) && ll.scripted {
        lamp.on = false;
    }
    let state = ll.state;
    match state {
        LlState::Rest | LlState::Fidget(_) | LlState::Ready => {
            let fidget_len = match state {
                LlState::Fidget(Fidget::LookSky) => 3.5,
                LlState::Fidget(Fidget::Shift) => 3.33,
                LlState::Fidget(Fidget::Wipe) => 3.2,
                LlState::Ready => 1.0,
                _ => f32::MAX,
            };
            if ll.t >= fidget_len {
                ll.state = LlState::Rest;
                ll.t = 0.0;
            }
            if !ll.scripted && clock.rate > 0.0 {
                // when the sun is about to cross the sea line, act
                let (secs, dusk) = clock.until_cross();
                let want = if dusk { Act::Light } else { Act::PutOut };
                let needed = (dusk && !lamp.on) || (!dusk && lamp.on);
                let lead = SKY_LOOK + RAISE + act_len(want).1;
                // wrong state for the time of day (after a rush): act now
                let wrong = (clock.is_night() && !lamp.on) || (!clock.is_night() && lamp.on);
                if (needed && secs <= lead) || (wrong && secs > lead + 2.0) {
                    let a = if wrong { if clock.is_night() { Act::Light } else { Act::PutOut } } else { want };
                    ll.state = LlState::SkyLook(a);
                    ll.t = 0.0;
                    ll.touched = false;
                } else if ll.state == LlState::Rest {
                    ll.next_fidget -= dt;
                    if ll.next_fidget <= 0.0 {
                        let r = ll.rng.f();
                        ll.next_fidget = ll.rng.range(3.0, 8.0);
                        ll.state = LlState::Fidget(if r < 0.4 { Fidget::LookSky } else if r < 0.7 { Fidget::Shift } else { Fidget::Wipe });
                        ll.t = 0.0;
                    }
                }
            }
        }
        LlState::SkyLook(a) => {
            if ll.t >= SKY_LOOK {
                ll.state = LlState::Raise(a);
                ll.t = 0.0;
            }
        }
        LlState::Raise(a) => {
            if ll.t >= RAISE {
                ll.state = LlState::Act(a);
                ll.t = 0.0;
                ll.touched = false;
            }
        }
        LlState::Act(a) => {
            let (len, touch) = act_len(a);
            if !ll.touched && ll.t >= touch {
                ll.touched = true;
                lamp.on = a == Act::Light;
                audio.write(AudioCmd::Sfx(if a == Act::Light { "sfx.lamp.light".into() } else { "sfx.lamp.out".into() }, if a == Act::Light { 0.9 } else { 0.8 }));
            }
            if ll.t >= len {
                ll.state = LlState::Lower;
                ll.t = 0.0;
            }
        }
        LlState::Lower => {
            if ll.t >= LOWER {
                ll.state = LlState::Ready;
                ll.t = 0.0;
            }
        }
    }
    // ---- pose: arm angle (raised = 0°, rest = -46° leaning back, ready = -20°)
    let t = ll.t;
    let (arm_goal, mut pitch_goal) = match ll.state {
        LlState::Rest => (-46.0, 0.0),
        LlState::Ready => (-20.0, 0.0),
        LlState::Fidget(Fidget::LookSky) => (-46.0, -0.45 * (t / 3.5 * std::f32::consts::PI).sin()),
        LlState::Fidget(_) => (-46.0, 0.05),
        LlState::SkyLook(_) => (-40.0, -0.5),
        LlState::Raise(_) => (-46.0 + 46.0 * ease_in_out(t / RAISE), -0.35 * (1.0 - t / RAISE)),
        LlState::Act(a) => {
            let (len, touch) = act_len(a);
            // a small jab to the wick and back
            let j = (1.0 - ((t - touch) / (len * 0.35)).abs()).max(0.0);
            (2.5 * j, -0.25)
        }
        LlState::Lower => (-46.0 * ease_in_out(t / LOWER), -0.1),
    };
    if ll.look_up {
        pitch_goal = -0.55;
    }
    let k = (dt * 12.0).min(1.0);
    ll.arm += (arm_goal - ll.arm) * k;
    // glance at the prince when near (yaw limited)
    let mut yaw_goal = 0.0;
    if let (Ok((_, gt)), Ok(pt)) = (root.single(), player.single()) {
        let local = gt.affine().inverse().transform_point3(pt.translation);
        let d = Vec2::new(local.x, local.z).length();
        if d < 4.0 && matches!(ll.state, LlState::Rest | LlState::Ready) && !ll.look_up {
            yaw_goal = local.x.atan2(local.z).clamp(-0.9, 0.9);
        }
    }
    ll.head_pitch += (pitch_goal - ll.head_pitch) * (dt * 5.0).min(1.0);
    ll.head_yaw += (yaw_goal - ll.head_yaw) * (dt * 4.0).min(1.0);
    let wipe = if let LlState::Fidget(Fidget::Wipe) = ll.state { (t / 3.2 * std::f32::consts::PI).sin() } else { 0.0 };
    let shift = if let LlState::Fidget(Fidget::Shift) = ll.state { (t / 3.33 * std::f32::consts::TAU).sin() * 0.04 } else { 0.0 };
    if let Ok((children, _)) = root.single() {
        for c in children.iter() {
            if let Ok((name, mut tf)) = parts.get_mut(c) {
                match name.0.as_str() {
                    "arm" | "tip" => tf.rotation = Quat::from_rotation_x(ll.arm.to_radians()),
                    "head" => tf.rotation = Quat::from_rotation_y(ll.head_yaw) * Quat::from_rotation_x(ll.head_pitch),
                    "arm_l" => tf.rotation = Quat::from_rotation_x(-2.2 * wipe) * Quat::from_rotation_z(-0.3 * wipe),
                    "body" => tf.translation.x = shift,
                    _ => {}
                }
            }
        }
    }
}
