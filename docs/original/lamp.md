# Chapter 6 — The Lamplighter (original id `lamp`, scene `Lighter_Planet`)

Design reference for our Bevy rebuild. Reproduce structure, mechanics and numbers; never copy the
original art, audio, code or text. Dialogue below is paraphrased.

Legend: **OBSERVED** = read from the shipped build. **INFERRED** = my reading; verify in play.

Sources:
- `[sk]` skeletons `out_cs/DiffableCs/Assembly-CSharp/LittlePrince/Lighter/*.cs` (LighterStory, Lamplighter, LighterLamp, LamplighterLook, LighterIslandShape, LighterSurf …).
- `[ctor]` LighterStory field initialisers decoded from `game.dcmp` func87147 (label `f_vwxd`, the function before `LighterStory_ctor`).
- `[scene]` serialized values / transforms from `lamp_scenes_all_*.bundle` (scratchpad `anB/d_lamp/`).
- `[loc]` LPStrings_en.txt.

---

## 1. Identity

| Item | Value | Src |
|---|---|---|
| Chapter | 6, Asteroid 329 | [loc] |
| Card | "The Lamplighter's Planet" / a planet that turns once a minute | [loc] paraphrase |
| Map blurb before / after | the smallest planet, one lamp and one man who never rests / he wanted to stay, but the planet was too small for two | [loc] paraphrase |
| Event | one: "Go and see the lamp" | [loc] |
| Controller | `LighterStory`, created at runtime on scene load (`Init` → `OnLoaded`) | [sk] |
| Music | `Lamplighter_Music`, -20.2 LUFS, trim -4 dB, delay 6 s, rest 40–80 s, fades 4 / 1.2 s, duck -5 dB | [scene] |

## 2. World layout

A tiny rocky island in a sea at sunset (sea + cloud sea to the horizon). Walkable rock is about **7 m wide (x -3.4..3.6) and 15.5 m long (z -9.1..6.3)**, rising from the shore (south, y≈0.7) to a crown (north, y≈4.5). An invisible ring of 128 box colliders (`Unseen_Walk_Boundary/Edge_*`) keeps the prince on the rock. Fall recovery: below y = -18 the player respawns (`LighterSceneState.recoveryHeight`). [scene]

| Thing | Position | Src |
|---|---|---|
| Player spawn / landing stone | (0.6, 0.67, -8.55), facing +Z | [scene] Prince_Spawn |
| Main lamp (street lamp) | (0.2, 4.50, 4.6); lantern ≈ 2.9 m above its foot | [scene] Main Lamp |
| Lamplighter | (1.23, 4.50, 4.55), facing -X (yaw -90°): stands beside the lamp | [scene] |
| Conversation approach | (0.2, 4.45, 2.95) | [scene] Lamp_ConversationApproach |
| Prince's talk stance | (x 0.35, z 3.2): lamplighter's front-left, by the post, clear of the wand | [ctor] |
| Wick target (ignition point) | (0.27, 7.42, 4.61) | [scene] |
| Sunset seat (rock ledge) | seat top (2.52, 2.21, -2.29), yaw 40°; mesh `Sunset_Ledge_Seat` at (2.22, 1.83, -1.24) | [ctor]/[scene] |
| Tide pool | water at (2.55, 2.36, 0.45) with boulder rim | [scene] |
| Path stones / ledge steps | stepping path from landing up the west/centre to the crown | [scene] |
| "First star" | far glow at (-55, 35, 318), wakes when the lamp is lit | [scene] |
| Surf | breaking-wave spots along the shore (spray/drops/mist) | [scene] LighterSurf |
| Island cover | procedural vegetation by density map | [scene] |

Static helper `LighterIslandShape` (OBSERVED names): height function `Height(x,z)` built from `BaseHeight` + `Crest(z)` + noise; named points `Lamp`, `Landing`, `Ledge`, `TidePool`; polylines `Path`, `Branch`; `HalfWidthAt(z, side)`. Our version can use a similar analytic heightfield.

## 3. Day–night: the one-minute day (core mechanic)

`B612TimeOfDay` with `mode = 1` (day/night cycle). OBSERVED [scene]:

- **cycleSeconds = 60**. Phase weights (seconds of the 60): **day 9, dusk 9, night 15, dawn 8, morning 19**.
- Elevation key points: night -13°, twilight -5°, sunrise 1°, sunrise key 2°, morning→day 16°, golden 16°.
- Pace curve (sun speed multiplier by elevation): -90°:0.5, -14°:0.8, -9°:1.4, -5°:2.0, -1°:2.4, 2°:2.7, 5°:2.7, 9°:2.4, 13°:2.0, 17°:1.5, 22°:1.15, 28°+:1.0 — the sun races through the horizon band.
- Start at dusk (`startPreset = dusk`, start hour 16.66, latitude 63°). First sunset at 11 s in the scene clock (`firstSunsetSeconds`).
- Presets (sun azimuth / elevation / intensity): Day 80° / 43° / 3.0; Sunset 30° / 3.5° / 3.4; Twilight 190° / -5° / 0; Night 90° / -30° / 0.32 (stars 1.2, Milky Way 1.4, moon 1.0); Sunrise 358° / 2° / 2.2.
- Stars/Milky Way fade in with night factor; clouds drift (wind 7, cloud relight 0.5).

Lamp (`LighterLamp`, OBSERVED [scene]): `followSun = 0` — the lamp does not switch by itself; the lamplighter's animation lights and puts it out. Light 1.1 s, put-out 0.6 s. Warm light intensity 5.79, range 9; at full night ×2.2 intensity and ×1.35 reach. Halo, 4 glass panes (keep 22 % glow), flame card (waver 7 %). Sea reflection 1.2. Sounds: light (0.9), put out (0.8).

Lamplighter driver (`Lamplighter`, OBSERVED [scene]):
- `FollowSun = 1`: he raises the wand by himself when the day is about to turn — horizon elevation **-0.3°** (sun's upper edge on the sea line). Lights at dusk, puts out at dawn.
- Clip timing: Light 2.667 s, wick touches at 1.333 s, leaves at 1.933 s. PutOut 2.0 s, touch at 1.0 s. Raise 1.667 s, lower 1.0 s. Rest loop 8 s, Ready loop 4 s. Blend 0.15 s, settle 0.4 s.
- Fidgets while resting, random gap **3–8 s**: LookSky (3.5 s), Shift (3.33 s), Wipe (3.2 s); chosen 0.8 s before the rest loop's neutral pose.
- He looks at the sky just before raising (`SkyBeforeRaise`), glances at the prince when near (`LamplighterLook`: notice/near/rearm distances, glance intervals; limits on yaw/pitch). (values not decoded; INFERRED small defaults.)

## 4. Beats in order

1. **Opening** [ctor]: chapter card over a landing shot from the sea south-west: cam (-16, 8.5, -18) → look (-11.92, 5.95, -9.23), FOV 55; push 3 m; 4 s descent; the play view is handed over at pitch 3°, distance 3.8 m behind him on the landing stone with the lamp on the crown in view. The sun meets the sea **0.4 s after control** (`sunsetAfterControl`), so the first thing the player sees is the lamplighter lighting the lamp at sunset (INFERRED intent).
2. **Free roam.** The day keeps turning every minute; he lights at each dusk and puts out at each dawn on his own.
3. **Talk** — prompt "Talk" within **3.2 m** of the lamplighter. On E: prince walks (0.9 m/s) round the lamp (`RoundTheLamp`, keeps 1 m clearance) to the stance and faces him. If it is **night when E is pressed**, the sky rushes to the first dawn in **2.2 s** (`FirstDawn`) so the talk starts at a put-out. [ctor]+[sk]
   - Beat 1 dawn two-shot (cam (-3.45,5.79,1.91) look (0.79,5.95,3.88) FOV 48): lamp goes out → L1 prince: hello, why did you just put out your lamp? → L2 lamplighter: orders; good morning. The sky holds **4 s after sunrise** ("morning").
   - Beat 2 dusk lamp shot (cam (-2.55,6.22,1.88) look (0.79,6.5,3.88) FOV 55): the sky **rushes 2.6 s** to the next dusk (`RushTo`), he lights the lamp → L3 prince: why light it again? → L4: nothing to understand, orders are orders; good evening.
   - Beat 3 night face shot (cam (-1.58,5.9,3.24) look (0.98,5.8,4.6) FOV 45): the sky settles **8 s after sunset** (night) → L5, L6, L7 lamplighter: it made sense once (out in the morning, lit at night); the planet turns faster each year but orders stayed; now one turn a minute, not a second's rest.
   - Beat 4 dawn shot (cam (-2.22,5.79,2.36) look (0.75,5.79,3.81) FOV 52): dawn again; the prince faces the sunrise, the lamplighter behind him looks up → L8 prince to himself ("aside"): he is the only one who doesn't seem ridiculous, maybe because he cares about something besides himself.
   - Pacing: `0.18 s × chars + 1.2 s`, min 2.6 s; click after 0.5 s lock. Shots move only when someone acts.
4. **Chapter complete** after L8 (INFERRED: single event).

## 5. Side activity — the sunset seat (OBSERVED [ctor] "Side quest: the sunset seat")

- A rock ledge seat at (2.52, 2.21, -2.29) facing yaw 40° (toward the sunset). Prompt "Sit down" / "Stand up" (`ChairSeat` + `PlayerSitController`).
- Seated view: yaw -24° from the seat facing (the lamp up the hill enters the left of the frame), pitch -1°, distance 3.2, height 1.1, FOV 55.
- "E Stand up" prompt shows for 4 s after sitting, reappears on any input.
- While seated the day runs faster near the horizon (`seatedSpeed 3.5`, near-horizon 2.0, band -8°..12°) [scene] B612TimeOfDay — you can watch many sunsets (book reference). No reward; not required.

## 6. Win / fail

- Win: finish the talk. No fail. No hidden goal in code (none found).

## 7. Test route

1. Spawn (0.6, 0.67, -8.55). Walk the path north to (0.35, ≈4.45, 3.2) (via (0.2, 4.45, 2.95)).
2. Press E at the lamplighter. Force sky rushes (or skip waits). Expect: put-out → L1–L2 → rush to dusk → light → L3–L4 → night → L5–L7 → dawn → L8.
3. Assert chapter complete; print `[test:ok lamp]`.
4. Optional: walk to the seat (2.52, 2.21, -2.29), E sit, wait one sunset, E stand.

## 8. Notes for the rebuild

- Implement the 60-second day as a phase clock with the weights above and a sun elevation curve; drive sky colours by blending 5 presets.
- The lamplighter is a timed state machine (Rest/Fidget/Raise/Act/Lower/Ready) keyed to "seconds until the sun crosses -0.3°".
- The talk *controls* the sky (rush to dawn/dusk) — implement `rush_to(sunset: bool, seconds)` on the day clock.
