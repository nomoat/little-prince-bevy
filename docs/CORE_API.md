# Core API for chapter authors

You own exactly two places:

- `src/chapters/<id>/` (any files you like; `mod.rs` must keep `pub struct <Name>;` implementing `Chapter`)
- `assets/chapters/<id>.ron` (your asset keys) and files under `assets/` you add for them

Do not edit `src/core/` or other chapters. If the core lacks something, write it inside your
chapter (you have full `&mut World` access through `Cmd::Run` and your own systems).

`src/chapters/b612/mod.rs` is a small but complete worked example of everything below.

```rust
use crate::core::prelude::*;   // bevy prelude + the whole core API
```

## 1. The `Chapter` trait

```rust
pub struct King;
impl Chapter for King {
    fn info(&self) -> ChapterInfo { ... }            // id, number, i18n keys, map position, music key
    fn strings(&self) -> StringTable { STRINGS }      // your zh/en table
    fn setup_app(&self, app: &mut App) { ... }        // systems, procedural models (optional)
    fn build(&self, cx: &mut ChapterCx) { ... }       // build the world (runs on chapter entry)
    fn story(&self) -> Story { ... }                  // events + script
}
```

Lifecycle: map → `Flow::Enter(id)` → ink fade → `build()` → story starts → ... →
`departure()` opens → player holds **E 1.6 s** → birds lift the prince → progress saved
(visited + next planet revealed) → map (or the ending after `earth`). Everything spawned
through `cx` is despawned on exit; anything you spawn yourself later must carry `ChapterScoped`.

`ChapterInfo` fields: `id`, `number` (1..8), `title`, `subtitle`, `line` (map narration before
visiting), `visited_line`, `loading` (fade/loading line), `map_pos` (stage px, 1672×941, origin
top-left), `map_radius`, `map_colors`, `music` (manifest key, started automatically).

## 2. Strings (i18n)

```rust
pub const STRINGS: StringTable = &[
    ("king.title", "国王的星球", "The King's Planet"),
    ("king.l.hello", "啊！来了一个臣民。", "Ah! Here is a subject."),
];
```
Keys are `<id>.*`. Every visible string goes through a key. Use our own wording (never the
original's text). `I18n::t(key)`, `I18n::tf(key, &[("n","3")])` for `{n}` placeholders.
A `Text` entity with `Tr::new("key")` re-translates itself on language switch.
After adding Chinese text, the UI font subset must be regenerated: `scripts/subset_font.sh`
(build_web.sh does it automatically).

## 3. Building the world (`ChapterCx`)

```rust
fn build(&self, cx: &mut ChapterCx) {
    let mut p = PlanetParams::sized(12.0);          // walkable radius (m); rim/underside/boundary scale with it
    p.ground = hex(0xe9e2ec); p.ground2 = hex(0xd9cfe0);
    p.mods = vec![
        TerrainMod::Flatten { center: Vec2::new(0., 5.), radius: 2.0, blend: 1.5, height: None }, // pad
        TerrainMod::Mound { center: Vec2::new(3., -2.), radius: 2.5, height: 0.4 },              // knoll
        TerrainMod::Paint { center: Vec2::ZERO, radius: 1.5, blend: 1.0, color: hex(0x7c6a52) },  // path
    ];
    cx.planet(p);                                    // domed terrain + rim + Terrain resource
    cx.sky(10.5, 0.5, Some(16.5), SkyPalette::default()); // hour, sun-hours/min, free-play cap
    cx.ambience("amb.wind");

    let at = cx.ground(0.0, 5.0);                    // ground point; cx.polar(angle_deg, r) too
    let king = cx.model("model.king.king", at, 180.0, 1.0);   // manifest key, pos, yaw°, scale
    cx.collider(king, Collider::cylinder(0.4, 1.6));
    cx.interactable(king, Interactable::new("king", "king.verb.greet")
        .radius(2.2).focus(Vec3::Y).prompt_height(0.8));     // .hold(1.2) .disabled() .once() .priority(n) .any_facing()
    cx.anchor("king", at, 180.0);                    // named point for scripts: Pos::anchor("king")
    cx.point_light(at + Vec3::Y * 2.0, hex(0xffd08a), 20000.0, 8.0);
    cx.scatter("model.rock", 8, 4.0, 11.0, 1, (0.6, 1.2), &[(Vec2::new(0., 5.), 2.0)]);
    cx.player(cx.ground(0.0, -2.0), 0.0);            // spawn + camera snaps behind (yaw 0 = +Z)
}
```

- **World model** (same as the original): heightfield sphere-cap planet, world-Y gravity,
  soft boundary push between `soft_radius` and `hard_radius`. `Terrain` resource:
  `height(x,z)`, `ground(x,z)`, `normal(x,z)`, `polar(a,r)`. All `PlanetParams` fields are
  documented in `core/planet.rs` (names follow B612PlanetParams).
- **Colliders**: `Collider::cylinder(r, h)` / `Collider::cuboid(size)` (+`.with_offset()`,
  `.no_camera()`, `.not_walkable()`). Tops are walkable (steps ≤ 0.36 m) — stairs and floors are
  just boxes.
- **Raw world**: `cx.world` (`&mut World`), `cx.spawn(bundle)` (scoped), `cx.empty(pos, yaw)`.
- **Player tuning** (`PlayerTuning` resource: jog 2.4, run 3.5, walk 1.0, jump …) and
  **camera tuning** (`CameraTuning`) can be changed per chapter in `build`; the core resets it
  to `CameraTuning::default()` before every chapter's `build`.

## 4. Models (always via the manifest)

`cx.model("model.<id>.thing", pos, yaw, scale)` or `commands.spawn_model(key, transform)`.
The key resolves in `assets/chapters/<id>.ron`:

```ron
"model.king.throne": (path: "models/king/throne.glb", procedural: "king.throne", scale: 1.0),
```
No `path` (or a failed load) → the procedural generator. Register generators in `setup_app`:

```rust
app.register_model("king.throne", throne);
fn throne(p: &mut ModelParts) {
    let mut m = MeshBuilder::new();
    m.cuboid(Vec3::Y * 0.4, Vec3::new(0.8, 0.8, 0.7), hex(0x8a3a5a));   // also frustum, sphere,
    m.lathe(Vec3::ZERO, &[(0.3, 0.0), (0.2, 1.0), (0.0, 1.1)], 8, hex(0xd8b04a)); // lathe, tube, rock, with()
    p.add(&m);                                                          // lit, vertex coloured
    p.add_part(Some("glow"), &m2, Transform::IDENTITY, PartMat::Emissive(hex(0xffc864), 4.0));
}
```
Named parts get a `PartName` component (animate them by querying `(&PartName, &mut Transform)`).
Materials: `PartMat::{Vertex, Unlit, Emissive(c,k), Glass(c), Custom(StandardMaterial)}`.
Core keys (`assets/manifest.ron`): `model.rock`, `model.tree`, `model.bush`, `model.flower*`,
`model.grass_tuft`, `model.chair`, `model.lamp_post`, `model.npc`, Kenney Nature Kit props …

## 5. Story: events, steps, script

```rust
fn story(&self) -> Story {
    Story {
        events: vec![event("king.ev.audience", &["king.st.door", "king.st.talk"])],
        hidden: Some("king.hidden"),           // optional hidden goal description key
        script: vec![
            card(),                                    // chapter card (waits)
            narrate(&["king.n1", "king.n2"]),          // centred narration lines (waits)
            start_event(0),                            // objective = event name + step 0
            target(Pos::anchor("door"), "king.t.door"),// marker flashes after 14 s without progress
            wait_use("door"),                          // enables + waits for interactable "door"
            step(1),
            wait_near(Pos::anchor("king"), 2.5),
            face(Pos::anchor("king")),
            say(speakers::KING, "king.l.hello"),        // waits for continue; say_for(who,key,secs) auto
            choice_test(Some((speakers::KING, "king.l.q")),
                        &[("king.o.yes", "yes"), ("king.o.no", "no")], 0), // (option key, label); autopilot picks 0
            label("yes"), say(speakers::PRINCE, "king.l.yes"), goto("after"),
            label("no"),  say(speakers::PRINCE, "king.l.no"),
            label("after"),
            wait_flag("bowed", vec![t_use("cushion")]),  // set by your code; hints drive the autopilot
            run(|w: &mut World| { /* any code */ }),
            Cmd::HiddenPending,                          // show "◇ hidden goal: ???"
            complete_event(0),                           // banner "done → next" (waits)
            departure(),                                 // "可以启程了": hold E 1.6 s to set off
        ],
    }
}
```

Full command list (`core/story.rs`, helpers in `story::cmd`): `card, narrate, start_event,
step, complete_event, say, say_for, mono (inner voice), choice / choice_test, label, goto,
if_flag, if_not_flag, flag, unflag, wait, wait_use, wait_flag, wait_near, wait_until(pred,
hints), target / clear_target, Cmd::HiddenPending / Cmd::HiddenFound, enable, disable, teach
(key hints), lock / unlock (player input), walk_to, face, teleport, pose (Pose::Sit/Kneel/
Wave/Reach/Clap/LookUp/Carry/None), shot(from, look, blend) / end_shot, set_hour, sunset(secs),
wait_sunset, Cmd::TimeRate(h_per_min, cap), music / ambience / sfx, run, test_only,
departure, leave, end`.

`Pos`: `Pos::anchor("name")`, `Pos::World(v)`, `Pos::Player(offset)`, `Pos::Interactable(id)`.

Speakers: `speakers::{PRINCE, ROSE, KING, VAIN_MAN, DRINKER, MERCHANT, LAMPLIGHTER,
GEOGRAPHER, NARRATOR}`; make your own with `pub const FOX: Speaker = Speaker::new("earth.fox",
Color::srgb(..))` (the name is an i18n key).

## 6. Chapter systems (mechanics)

```rust
fn setup_app(&self, app: &mut App) {
    app.add_systems(Update, my_mechanic.in_set(CoreSet::Gameplay).run_if(in_chapter("king")));
}
fn my_mechanic(mut msgs: MessageReader<Interacted>, mut runner: ResMut<StoryRunner>,
               input: Res<InputFrame>, mut audio: MessageWriter<AudioCmd>, mut hud: ResMut<Hud>) {
    for m in msgs.read() { if m.id == "cushion" { runner.set_flag("bowed"); } }
    if input.action2 { audio.write(AudioCmd::sfx("sfx.clap")); }    // C/F key or touch button
    // The touch button for action2 is hidden until a chapter names it:
    //   touch_controls.action2_label = Some("vanity.teach.clap");   // i18n key; None hides it
    // (cleared by the core when the chapter ends).
    // hud.toast("…", 2.0); hud.monologue("king.m1", 3.0); hud.teach(vec!["C"], "vanity.t.clap", 5.0)
}
```
Useful resources: `StoryRunner` (flags, `waiting`, `pc`, events), `InputFrame` (move_axis,
run, look, zoom, interact, interact_held, jump, action2, advance — never read devices directly),
`Hud`, `Sky`, `CameraRig` (`begin_shot`, `end_shot`, `snap_behind`), `Terrain`,
`ColliderWorld`, `SaveData` (`chapter_data` map for persistent values), `I18n`, `Anchors`.
Player: query `(&mut Player, &mut Transform)`; `player.drive = Drive::External` to move him
yourself (sitting, carried), `Drive::WalkTo{..}` to walk, `player.pose = Pose::Sit`.
Hidden goal found from code: `commands.queue(|w: &mut World| found_hidden(w))`.

## 7. Sky, sun, time

`cx.sky(hour, hours_per_minute, cap_hour, palette)`; at runtime `ResMut<Sky>`:
`set_hour(h)`, `set_rate(h_per_min, cap)` (lamplighter: `set_rate(24.0, None)` = one day per
minute), `start_sunset(secs)` (quick start, slow golden middle, eased sink to `end_elevation`),
`sunset_done()`, `phase()` → `DayPhase::{Day, Golden, Dusk, Night, Dawn, Morning}`,
`sun_elevation`, `sun_dir`, `night` (0..1), `fixed_night = Some(1.0)` for interiors/night
scenes, `palette: SkyPalette` (zenith/horizon colours per phase, sun colours, fog amount).
Solar arc parameters (`latitude`, `declination`, `north_azimuth`) match the original.

## 8. Audio

`AudioCmd::sfx("sfx.chime")`, `AudioCmd::Sfx(key, volume)`, `AudioCmd::music("music.king")`,
`AudioCmd::Music(None)`, `AudioCmd::Ambience(Some("amb.night".into()))`. Keys resolve through
the manifest: `path`/`variants` (ogg/wav) or `procedural` synth ids (`chime, chime_low, bell,
click, tick, step, step_soft, whoosh, pop, clap, sparkle, depart, found, banner, water, creak,
wind, night, pad_title, pad_home, pad_wonder, pad_melancholy, pad_playful, pad_night,
pad_desert, pad_ending`). Footsteps are automatic (`sfx.step`).

## 9. Test autoplay (`?test=1`)

The autopilot plays the *current wait* like a player: walks to the interactable (teleports
if stuck > 2.5 s), faces it, presses or holds E; advances dialogue; picks `choice_test`'s
option; holds E at departure. For `wait_flag`/`wait_until` it runs your hints in order:
`t_use(id)`, `t_goto(Pos)`, `t_wait(s)`, `t_run(|w| ...)`, `TestAction::PressJump /
PressInteract / PressAction2`. `test_only(|w| ...)` runs only in tests (e.g. to skip a long
timing minigame). Waits are shortened in test mode (`wait` ≤ 0.4 s, sunsets ≤ 2 s).
Each wait times out after 60 s → `[test:fail <id>] stuck at pc N: <wait>`; success prints
`[test:ok <id>]` when the prince departs.

```
node tests/run_web.mjs king            # web (after scripts/build_web.sh)
cargo r -- --chapter king --test       # native, exits with the result
FLOW=1 node tests/run_web.mjs          # whole journey: ?flow=1&test=1 → [flow:ok]
cargo r -- --flow --test               # same, native
node tests/ui_web.mjs                  # real input: touch (phone emulation), keyboard, save, language, free travel
```

## 10. Conventions

- Units: metres; the prince is ~1.2 m; yaw degrees with 0 = +Z, 90 = +X.
- Keep your interactable ids, anchors, flags and asset keys prefixed or chapter-local.
- Use your own `CARGO_TARGET_DIR=target-<id>`; `cargo c`, `cargo cw` (wasm check) aliases add
  `--ignore-rust-version` (required with the installed 1.95 toolchain).
