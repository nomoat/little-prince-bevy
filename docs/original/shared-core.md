# Original game — shared systems used by every chapter (analyst A)

Source: Cpp2IL skeletons (`out_cs/DiffableCs/B612.Core`, `LittlePrince.Story`, `LittlePrince.Journey`) and
wasm-decompile bodies (`dcmp/*.dcmp`). Default values come from decoding the constructor bodies
(IL2CPP field-initialiser stores). Note: in `dcmp` the function *labels* are often shifted by one
(the body under label `X_Turn` is really `X..ctor`); values below were matched by field offset.

String-literal trick (for later analysts): a reference `d_GGG…[N]` to a string resolves to
`string_literals.txt` index `N − 32756` (verified on all chapter titles; exact for CJK strings,
unreliable in the ASCII range). Helper: `scratchpad/anaA/ann.py <dcmp> <function>`.

Legend: **OBSERVED** = read from code/data (citation given). **INFERRED** = my reading of names,
tooltips or partial bodies, not proven.

## 1. Player locomotion (`B612.PlayerController`, ctor) — OBSERVED

| Field | Value | Note |
|---|---|---|
| walkSpeed | 1.5 m/s | scripted walks only |
| jogSpeed | 2.4 m/s | **default free-control speed** |
| runSpeed | 3.5 m/s | Shift held |
| acceleration / deceleration | 7 / 8 m/s² | |
| turnSmoothTime / maxTurnSpeed | 0.085 s / 560 °/s | |
| fullSpeedAngle / turnInPlaceAngle | 35° / 110° | turns on the spot first above 110° error |
| gravity | −18 m/s² | world −Y (domed terrain, not radial gravity) |
| groundSnapDistance | 0.32 m | |
| slope slow-down | starts 14°, ×0.78 at 35° limit; slides above 36° (accel 9, max 4 m/s) | |
| jumpHeight / cooldown | 0.3 m / 0.45 s | "optional, never required" (tooltip) |
| stepSmoothTime | 0.09 s | visual smoothing of step-ups |
| Character height | ~1.2 m child, eyes ~1.05 m | ThirdPersonCamera tooltip |

Scripted walks in story beats use 0.9 m/s (`walkSpeed` fields in King/Vanity/Bar stories, door approach speeds).

## 2. Input (`KeyboardMouseInputSource.ReadInput`, `PlayerInputFrame`) — OBSERVED

`PlayerInputFrame` = { move, run, look, zoom, interact (press), interactHeld, jump, **clap** }.
- Move: WASD + arrow keys. Run: Shift. Interact: **E** (press and hold variants). Jump: Space. **Clap: C** (used by the vanity chapter).
- Mouse look: yaw 0.11 °/px, pitch 0.09 °/px; only while the cursor is locked (click to lock, Esc releases).
- Wheel zoom: notch = delta/120, clamped ±3 notches per frame, 0.35 m per notch.
- Departure prompt (all chapters): "Q = depart following the migrating birds / Esc = stay a while" (string literal 5251).
- Debug: a key toggles day/dusk ("下一时段" strings) — dev only.

## 3. Third-person camera (`ThirdPersonCamera`, ctor) — OBSERVED

pivotHeight 1.0 m; distance default 3.2 m (min 1.8, max 5.0); pitch default 12° (−30…60); FOV 50°;
near 0.08, far 2000; follow smoothing XZ 0.06 s / Y 0.14 s, max lag 0.5 m; collision radius 0.2,
recover 0.35 s; hide character when lens < 0.58 m from head (show again > 0.79 m); ground clearance 0.3 m.
Obstacle lift: when < 1.5 m room, raise pitch in 5° steps up to 80°; "swing" around walls up to `maxSwing`
at 110 °/s; comfort lift pitch 55°. Seated "view mode": enter 1.6 s, exit 0.9 s.
Story code drives it with `BeginShot(yaw,pitch,dist,enter,exit,lookAt,snap,fov)`, `MoveShot(..., seconds)`,
`CutTo`, `EndShot`.

## 4. Interactables (`Interactable`, `PlayerInteractor`) — OBSERVED

Defaults: radius 1.6 m, requireFacing with maxFacingAngle 100°, sideHalfAngle 110°, promptHeight 0.6 m.
Prompt = key label + verb (e.g. "E  坐下"); optional `holdSeconds` shows a filling ring.
Highest `priority`, then score (distance + facing×0.01) wins. While carrying a prop only
`usableWhileCarrying` interactables are offered.

`ChairSeat`: standDistance 0.42 m; seated view pivot 0.8 m, distance 2.4 m, pitch 3° (−10…24),
yaw range ±40°, side offset 0.35 m. Prompt changes after the day is over (`dayOverPrompt`); needs free hands
(`carryingPrompt`). Clip sets: child chair (theatre, seat 0.30 m) and "High" adult chair/stool (bar, ~0.46 m).

`CarryProp` (watering can): putBackRadius 1.2 m, attach blend 0.15 s, fillLevel 0..1 (starts 0 = empty in the shed).
`CarryPutSpot`: put-down spot beside the rose. `FillTarget` / `WaterTarget`: see b612.md.

## 5. Story framework (`LittlePrince.Story.StoryRunner`, `StoryChapter`, `StoryEvent`) — OBSERVED

- `StoryChapter` = Number ("第N章"), Title, Subtitle, Narration[] (2 lines each in these 4 chapters), Events[]
  (each = Name + Steps[]). Built in each `<Chapter>Story` static ctor.
- `ChapterCard()` shows Number/Title/Subtitle for **3.0 s** (GameUI call with 3.0f).
- Objective panel shows events/steps (done / current / pending). Toggled by a key ("press once to show the
  chapter goal, press again to hide" — taught in B612). `MarkDone`, `StartEvent`, `SetStep`, `CompleteEvent`.
- Star marker / target: `SetTarget(transform, label)` puts a star marker + label on the goal
  ("approach the starred thing" is a taught control). **Stall help**: if the distance to the target has not
  dropped by ≥ 0.5 m for `stallSeconds` = **40 s**, the marker lights up once for `stallMarkerSeconds` = **8 s**;
  `unmetMarkerSeconds` = 5 s for refused actions (`Unmet(line)`).
- `ShowFree(step)` = free-roam objective line; `SetHidden(text, done)` = hidden-goal slot exists in the UI
  (`ChapterView.Hidden/HiddenDone`) — no chapter in A's set was found to set one (INFERRED).
- Dialogue lines (`Line()` in each story): auto-advance after `0.18 s × chars + 1.2 s`, at least `2.6 s`;
  a click advances sooner but only after `lockFor` (default 0.5 s); `maxLock` 10 s cap; some lines wait for
  an animation (`lockWhile`/`endWhen`).
- Every chapter story exposes `Phase` (string) and `ShotName` for tests. Phases seen: `card → visit/explore →
  talk/audience/... → depart`.

## 6. Chapter opening template (King, Vanity, Bar `Start()`) — OBSERVED + INFERRED

1. Phase `card`: camera placed at `openingShot` (world framing, planet in right two thirds, sky on the left
   for the card), drifts `openingPush` = 3 m toward the planet while the chapter card is up (3 s).
2. Descends over `openingSettle` = 4 s to the normal play view behind the prince.
3. Phase `visit`: `MarkDone(0)`?, `SetTarget(npc marker, npc name)`; free play.
4. Talk is offered by an `Interactable` with verb "交谈" (talk) near the NPC.
5. After the talk: phase `depart` → `CompleteChapter` → the journey departure UI ("可以启程了").

## 7. Journey goal types (`JourneyGoal`, `JourneyPlanet`) — OBSERVED

`None / B612Sunset / ReachObject / Story`; per planet `goalObject`, `goalRadius`,
`goalFallbackSeconds` ("Story goal only: open the departure anyway after this many seconds", 0 = never).
B612 uses `B612Sunset`. Planet states on the map: Fog / Lit / Visited / Teaser.

## 8. Test harness (`AgentInput`, `AgentInputDriver`, `PlaytestRoute`) — OBSERVED

- `AgentInput` static API: `Move(x,y,s,run)`, `MoveWorld`, `MoveTo(x,z,run,timeout=30,stopRadius=0.3)`,
  `Look(yaw,pitch,s)`, `LookToward`, `Interact()`, `InteractNow()`, `Jump()`, `Zoom(m)`, `Wait(s)`, `Stop()`,
  `DebugTeleport(anchorName)`, `Capture(...)`; queries `Position`, `FacingYaw`, `NearbyInteractable`,
  `IsBusy`, `IsSettled`, `CameraDistance`. Driver also has `PressClapNow()`.
- `PlaytestRoute.Run(label, captureDir, script)`: a text script, one command per step, words
  `move, moveto, look, interact, jump, wait, zoom, teleport, capture, phase, stop, run/jog`; per-step timeout
  `stepTimeout`; status JSON `{"running":..,"log":..}`; log "route '{0}' started with {1} steps", "aborted at step".
  The `phase` command (INFERRED) waits until the story's `Phase` equals a value.
- No chapter route scripts are baked into the build (they were passed in from outside) — the routes in each
  chapter doc are INFERRED from the story flow.
- Launch flags seen: `chapter`, `test`, `free`, `debug`, `noveg`, `nossao`, `notaa`, `-courtBenchmarkHall`, ...
