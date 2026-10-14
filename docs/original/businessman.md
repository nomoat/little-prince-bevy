# Chapter 5 — The Businessman (original id `counting`, scene `StarObservatory`)

Design reference for our Bevy rebuild. Reproduce the structure, mechanics and numbers.
Do NOT copy the original art, audio, code or text. Dialogue here is paraphrased; write our own lines
(our Three.js version `little-prince-journey/src/chapters/businessman/strings.ts` is ours and may be reused).

Legend: **OBSERVED** = read from the shipped build. **INFERRED** = my reading of names, tooltips or flow; verify in play.

Sources (all OBSERVED unless marked):
- `[sk]` Cpp2IL skeletons `out_cs/DiffableCs/LittlePrince.StarObservatory/LittlePrince/StarObservatory/*.cs` (field names, tooltips, consts).
- `[ctor]` field initialisers decoded from `game.dcmp` (MerchantStory instance ctor = function labelled `MerchantStory_Turn`, func91034; TelescopeSky ctor = label `TelescopeSky_Chime`, func90843). The decompiler labels are shifted by one function; values decoded from IEEE-754 bit patterns.
- `[scene]` serialized MonoBehaviour values and transforms from the shipped addressable bundle `counting_scenes_all_*.bundle` (UnityPy, type trees present). Extracted dump: scratchpad `anB/d_counting/` (mb.json, hier.txt, ta_*.txt).
- `[loc]` `LPStrings_en.txt` (TextAsset in `data.unity3d`): the list of on-screen lines with their order.

---

## 1. Identity and journey map

| Item | Value | Src |
|---|---|---|
| Chapter number | 5 | [loc] |
| Journey map label | Asteroid 328 | [loc] |
| Card title / subtitle | "The Businessman's Planet" / a businessman too busy to look up | [loc] (paraphrase) |
| Map blurb before / after visit | he counts stars nonstop and claims them / the prince spoke of his flower and volcanoes and the man had no answer | [loc] (paraphrase) |
| Scene name | `StarObservatory` | [sk] `MerchantStory.SceneName` |
| Story controller | `MerchantStory` is created at runtime (`RuntimeInitializeOnLoadMethod AfterSceneLoad` → `OnLoaded` checks scene name). Not placed in the scene. | [sk] |
| Music | `StarObservatory_Music`, -31.1 LUFS, trim -2 dB, start delay 6 s, rest 40–80 s between plays, fade in 4 s / out 1.2 s, duck -5 dB under dialogue | [scene] ChapterMusicSettings |

## 2. World layout (metres, world space, +Y up, +Z = north/forward)

Small floating rock island at night with a round domed observatory on it. 28 floating rocks drift around the island.

| Thing | Position | Notes | Src |
|---|---|---|---|
| Player spawn | (-1.8, 0, -8.2), facing +Z | outside, south of the door | [scene] PlayerSpawn |
| Door (double, closed at start) | centre (0, 0.37, -3.2); hinges at x = ±0.85 | doorway 1.7 m wide, 2.25 m high | [scene] |
| Outside stand point | (0, 0, -6) `DoorOutside` | | [scene] |
| Room centre | (0, 0.36, 0) — floor is 0.36 m above outside ground | round room, radius ≈ 3.25 m (charts on wall at r = 3.248) | [scene] |
| Brass telescope (interactable) | (0, 0.37, 0), yaw ≈ 22.5° | room centre, on the star rug | [scene] Prop_BrassTelescope |
| Writing desk | (0, 0.36, 2.0) | north side, facing south | [scene] |
| Desk front anchor | (0, 0.36, 0.9) | | [scene] DeskFront |
| Merchant (seated on chair) | (0, 0.36, 2.58), facing -Z (toward the door) | writes in an open ledger at (-0.04, 1.14, 2.17) | [scene] |
| Prince's talk stance | (x 0, z 1.0) | face to face across the desk | [ctor] `stance` |
| Furniture ring | bookshelf NE, archive cabinet NW, side desk E, small table W, low cabinet SE, bench cabinet SW, stool, 3 armillary spheres, 5 wall charts (north, moon, constellation, zodiac, orbits), oil lamp, candles, sextant, scrolls, boxes | | [scene] |
| Glass dome | over the room; ribs + ring beam | `DomeCutaway` hides parts when the camera is inside | [scene]/[sk] |
| Lanterns | 4 exterior at r ≈ 3.9 m, h 2.1; 4+ interior at h 2.1–2.3 | flicker (`LanternFlicker`) | [scene] |

Atmosphere (OBSERVED [scene] StarObservatorySceneSettings): night sky skybox + moon light; trilight ambient sky (0.066,0.066,0.092), equator (0.068,0.058,0.072), ground (0.048,0.038,0.034); exponential fog colour (0.061,0.10,0.21), density 0.0065. Interior has its own warmer ambient (sky 0.085,0.06,0.042) applied to interior roots (`InteriorAmbient`). Dust motes in a 5.4×2.6×5.4 m box, 130 max, rate 7/s, life 14–22 s, size 6–16 mm, warm colour alpha 0.45.

Floating rocks (OBSERVED [scene] FloatingRock ×28): bob amplitude 0.08–0.14 m, bob period ≈ 7–7.6 s, slow spin ≈ 2.4–2.5 °/s about a random axis, seeded.

## 3. Camera and player settings (this chapter)

- Third-person rig: pivot 1.0 m, default distance 3.2 (1.8–5.0), default pitch 12° (-30..60), FOV 50. [scene]
- Start framing: pitch 3°, distance 4.2, held 3 frames. [scene] CameraStartFraming
- `InteriorCameraZone` (box over the room): inside, distance 2.3 (1.5–2.8), pitch 22° (-12..55), FOV 56, max lift pitch 55, blend 1.0 s. Exit margin 0.3 m. [scene]
- `CameraOccluderFade`: objects between camera and prince fade (radius 0.18, hold 0.25 s). Desk, chair, cabinets have `CameraOccluder`. [scene]
- A warm fill light follows the prince (height 1.35, distance 1.2, side 0.35), affects only his rendering layer. [scene] CharacterFillLight
- Footsteps by zone: rug (vol 0.35), wood (0.5), stone outside (0.22); jitter 0.12, pitch 0.94–1.06. [scene]

Player controller (shared, OBSERVED [scene]): walk 1.5, jog 2.4, run 3.5 m/s, accel 7, decel 8, gravity -18, jump height 0.3 m, jump cooldown 0.45 s, slope slow from 14°, slide at 36°.

## 4. Beats in order

1. **Opening** (OBSERVED fields [ctor], flow INFERRED): chapter card over a landing shot — camera (8, 8.5, -14) looking at (0.93, 3.71, -5.56), FOV 45; slow push 3 m while the card is up; then a 4 s descent to the play view behind the prince.
2. **Free roam outside.** Objective: "Visit the man who counts the stars" (single event). [loc]
3. **Door.** Double door, closed. Prompt "Open the door"/"Close the door" (`ObservatoryDoor`, radius 1.8, must face it within 110°). With `ObservatoryDoorPush` the prince walks to a stance 0.32 m from the seam and plays a two-hand push clip; leaves follow a baked 135-frame angle curve to 90°; push contact at frame 16, release 40, clip end 54, door rests at frame 135 (30 fps). Plain open: 100° over 1.2 s, eased. Door blocks the player while opening if he is in the sweep. Sounds: latch, creak (fades 0.4 s), settle. [scene]+[sk]
4. **Approach the merchant.** He loops `Merchant_Write` (240 frames @30 fps; legal entry frames 38, 85, 130, 163, 223; canonical 163). One-shots: `Merchant_Breath` 84 f, `Merchant_Stumped` 141 f. Pencil-scratch sound tied to the write loop (vol 0.3, fade 0.3 s). [scene] ta_Merchant_Timing
5. **Talk** — prompt "Talk" offered within **2.1 m of the desk front**, front side only. On E the prince walks (0.9 m/s) to the stance and turns to face the merchant. Four camera beats, all from the east (armillary) side of the line: [ctor]+[sk]
   - Beat 1 two-shot (cam (1.5,1.75,0.6) → look (-0.1,1.2,1.7), FOV 44), pushes to `twoShotIn` (cam (1.32,1.72,0.82), FOV 42) over the greeting.
   - Lines 1–6 (merchant shot cam (0.3,1.62,1.25) look (0,1.25,2.4) FOV 35):
     L1 merchant finishes a sum aloud and announces a huge total (≈ 501,622,731);
     L2 prince: "five hundred million what?"; L3 merchant: "stars"; L4 prince: what do you do with them;
     L5 merchant: nothing, I own them; L6 prince: then what do you do with them.
   - Beat 2 ledger shot (cam (0.45,1.75,1.7) look (-0.05,1.12,2.2) FOV 30) on L7: he writes the number on a slip and locks it in a drawer. L8 prince: "that's all?" L9 merchant: "that's enough!"
   - Beat 3 wide shot (cam (1.9,1.35,-0.15) look (-0.2,2.0,1.8) FOV 52): **the prince looks up at the glass dome** (head pitch 34°, ease in 0.8 s, hold 1.2 s, out 0.6 s). Prince shot (cam (0.6,1.78,2.85) look (0,1.25,1.0) FOV 33) for L10 (he waters his flower daily, sweeps three volcanoes weekly) and L11 (his owning is useful to them; the merchant is of no use to the stars).
   - Beat 4 stumped shot over the prince's right shoulder (cam (0.36,1.5,0.3) look (-0.05,1.3,2.45) FOV 40): merchant plays `Merchant_Stumped`; between clip frames 40 and 90 he holds his head up at the prince with an empty "……" box. Then he goes back to writing.
   - Line pacing: auto-advance after `0.18 s × chars + 1.2 s`, min 2.6 s; a click advances sooner (after a 0.5 s lock).
6. **Return view**: play camera returns from 50° to the prince's right-rear (straight behind, the telescope fills the view). [ctor] `returnYaw`
7. **Chapter complete** banner after the talk (INFERRED: the only required event). Map text switches to the "after" blurb.

## 5. Hidden goal — "Find B-612 among the stars" (OBSERVED [sk] `HiddenName`, [loc])

Shown as "???" (`HiddenUnknown`) in the goal list until revealed (INFERRED: revealed when the prince first looks through the telescope, or after the talk). Completing it does not gate the chapter.

Telescope (`ObservatoryTelescope`, an Interactable) [scene]+[sk]:
- Prompt "Look at the stars" / "Leave", radius 1.5, must face it (100°), must be on its eyepiece side (±75°). Priority 2 (door 3).
- States: Idle → Approaching → Raising → Viewing → Lowering.
- Approach at 0.9 m/s to the stance from TelescopeLook.json: stance (-0.044, 0, -0.620) in the telescope frame, eyepiece (0, 1.085, -0.451), tube elevation 30°; align 0.25 s; look clip 1.6 s with the eye at the eyepiece at 1.4 s, then hold clip. New move input > 0.6 during the walk cancels.
- On entering: fade to black 0.3 s, round vignette mask (radius 0.47 × screen height), view lens 13 m above the telescope base (clears the roof). View FOV starts at **32°**; yaw limit ±60° from the tube's aim, elevation 8°–85°; look speed 0.4× normal.

Sky in the view (`TelescopeSky`, created at runtime) [ctor]:
- Lenses (mouse wheel / zoom steps): **32°, 20°, 12°**, zoom ease 0.45 s.
- B-612 is placed **42° right of** and **36° above** where the tube aims, 800 m away (sky sphere 880 m). Planet width 10° when fully shown (12° lens: ≈ 0.4 screen).
- Above lens 24° only a warm star shows; below 14° only the planet picture; cross-fade between. Warm star colour (1, 0.82, 0.56); glow 90 px, core 16 px, four-point flare 150 px (at 1080 lines).
- Shooting stars toward B-612 until found: first after 2.5 s, then every 15 s, each 0.75 s long, 20° arc.
- **Found** when B-612 stays within **3°** of the view centre (or within a quarter of the lens if wider) at lens **≤ 20.5°** for **0.6 s**. Plays a chime (vol 0.35), sets `B612Found`, raises `Found`. Story: `OnFound` → `ShowFound(inView)`; if found during the talk it waits (`_foundPending`) (INFERRED).
- Two procedurally built star layers (mid, far) with a seeded RNG; dot and flare textures are generated in code.

## 6. NPC behaviour (merchant)

- Always seated at the desk writing; never walks. `MerchantDriver` blends one-shots onto the write loop only at legal entry frames (fade 0.25 s), so the hand never jumps. [scene]
- Occasional `Merchant_Breath` (INFERRED idle variety). `Merchant_Stumped` only in beat 4.
- No fail state. No timer.

## 7. Win / fail

- Win: finish the talk. No fail. Hidden goal optional.

## 8. Test route (for `?test=1`)

1. Spawn (-1.8, 0, -8.2). Walk to (0, 0, -4.0), face +Z, press E at the door (or open it instantly).
2. Walk to (0, 0.36, 0.6) (south of the desk, east of the telescope), face +Z, press E "Talk".
3. Advance lines (click / wait). Expect phase sequence: two-shot → merchant → ledger → wide/look-up → prince → stumped → return.
4. Assert chapter complete.
5. Optional hidden: walk to the telescope stance (≈ (0.24, 0.37, -0.57) after its 22.5° yaw), press E, call `DebugPointView(42°, 36°+30°)` equivalent (aim at B-612), set lens index 1 (20°), wait ≥ 0.6 s, assert found. Print `[test:ok businessman]`.

## 9. Notes for the rebuild

- Our planet model: this chapter is a small flat island with a building, not a domed planet. Use a flat disc with a rim and the round room.
- The telescope view is a separate camera with a circular mask and a procedural star sphere; B-612 is a billboard texture behind a manifest key.
- Keep numbers: lenses 32/20/12, offset 42°/36°, found 3°/≤20.5°/0.6 s, meteors 2.5/15/0.75 s.
