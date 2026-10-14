# Chapter 7 — The Geographer (original id `geo`, scene `MapLibrary`)

Design reference for our Bevy rebuild. Reproduce structure, mechanics and numbers; never copy the
original art, audio, code or text. Dialogue is paraphrased.

Legend: **OBSERVED** = read from the shipped build. **INFERRED** = my reading; verify in play.

Sources:
- `[sk]` skeletons `out_cs/DiffableCs/Geography.Runtime/LittlePrince/Geography/*.cs`.
- `[ctor]` GeographyStory initialisers from `game.dcmp` func90637 (label `GeographyStory_Turn`, the function before `GeographyStory_ctor`).
- `[scene]` serialized values / transforms from `geo_scenes_all_*.bundle` (scratchpad `anB/d_geo/`).
- `[loc]` LPStrings_en.txt.

---

## 1. Identity

| Item | Value | Src |
|---|---|---|
| Chapter | 7, Asteroid 330 | [loc] |
| Card | "The Geographer's Planet" / a geographer who records mountains and seas he has never seen | [loc] paraphrase |
| Map blurb before / after | he never goes out yet records far places / for the first time he worried about the flower left at home, then went to Earth as the geographer advised | [loc] paraphrase |
| Event | one: "Look around the map library" | [loc] |
| Controller | `GeographyStory` (runtime-created) + `GeographerConversation` (Interactable placed in scene) | [sk]/[scene] |
| Music | `Geography_Music`, -28.2 LUFS, delay 6 s, rest 40–80 s, fades 4/1.2 s, duck -5 dB | [scene] |
| Light | golden late afternoon; sun 30° elevation; skylight sunbeam with dust inside the hall | [scene] |

## 2. World layout

A Mediterranean-style map library (hall + tower) on a small island above a sea of clouds (cloud sea plane at y = -55; 3121 cumulus sprites; drifting cloud shadows). Garden beds and cypresses/olives around. [scene]

| Thing | Position / size | Src |
|---|---|---|
| Player spawn | (0, -0.7, -8.0), facing +Z, in the front garden | [scene] Player |
| Entrance steps | ramp at z ≈ -1.27 (rising to the floor y = 0) | [scene] |
| Front doors | double leaves at x = ±1.3, z 0.36, fixed **open 100° inward** | [scene] |
| Hall (map room) | x -3.1..3.1, z 1.7..10.9 (centre (0, 1, 6.3), half extents 4 × 2 × 5.5 used for camera) | [scene] |
| Back opening → balcony | back threshold z 12.2; balcony floor centre z 14.1; far rail at **z = 15.3**, usable length ±3.4 m | [scene]/[ctor] |
| Tower | east side (x ≈ 7, z ≈ 11.6) | [scene] |
| Map table | (-1.95, 0, 6.5), map on top (y 0.81), armillary sphere, book stacks, inkwell, papers, pencil stand (-1.77, 0.81, 6.03) | [scene] |
| Geographer | (-2.17, 0, 5.55), facing +Z (toward the table), seated/standing at the table reading a huge book | [scene] |
| Conversation trigger | (-2.15, 0, 7.6), facing -Z: the prince's place across the table | [scene] |
| Table corner waypoint | (x -0.72, z 7.6): when coming from the room side he walks round the table end through it | [ctor] |
| Shelves with globes and books | along both side walls (x ±3.75) | [scene] |
| Lanterns / candles | facade lanterns at the doors front and back, hall lanterns, 3 candles (flicker) | [scene] |
| Garden | raised beds south of the building (z -15..-9), shrubs, cypresses at x ±7 | [scene] |

## 3. Camera settings

- Rig: pivot 1.0, distance 3.6 (1.2–5.0), **default pitch -8°** (looks up at the facade on landing), pitch range -25..60, FOV 55, near 0.05. [scene]
- `GeoCameraPitchAssist`: inside the hall box, pitch eases to **10°** over 1.2 s if below 4°; the landing look-up is released after he walks 0.5 m. [scene]
- `GeoCameraSoftLimit`: in the balcony zone (centre (0,1,14), ext (4.9,2,2)) and a west side zone, side probes ±1 m, lead 0.65 s — the camera shortens instead of clipping walls. [scene]
- Interior colour grade volume while the camera is inside. [scene]

## 4. Beats in order

1. **Opening** [ctor]: landing shot cam (27, 13.5, -43) → look (15.68, 10, -33.8), FOV 45; push 1.5 m; 4 s descent. The island sits right of the card's subtitle.
2. **Free roam**: walk up the garden path and steps into the hall.
3. **Talk** — prompt "Talk" within **2.0 m of the map table's centre** (across the table or from the room side). `GeographerConversation` Interactable radius 1.3, priority 1, must face it (100°) and be on its side (±75°). On E the prince walks (0.9 m/s) to the spot across the table, via the table corner if needed. [ctor]/[scene]
   - The geographer's clips (30 fps): `GEO_ReadIdle` loop 300 f (rest windows 1–12, 96–108, 192–204, 290–300), `GEO_SpotExplorer` 90 f, `GEO_PleaseTell` 90 f, `GEO_PickWrite` 146 f (pencil in hand from f65), `GEO_PencilStops` 75 f, `GEO_TalkCalm` loop 120 f (rest 1–50, 93–120), `GEO_BackToBook` 151 f (pencil back in stand from f83). A clip starts only from a loop's rest pose (wait ≤ 0.6 s; blend 0.1 s from rest, 0.25 s otherwise). [scene]
   - Ten lines (speaker order OBSERVED [scene] `lines[]`):
     1 geographer (spots an explorer) — clip SpotExplorer; two-shot (cam (-1.05,1.45,9.5) look (-2.16,1.05,6.4) FOV 45) pushing in (`twoShotIn` FOV 43).
     2 geographer (you come from far — tell me about your planet) — PleaseTell.
     3 prince (it's small, three volcanoes) — **PickWrite starts 0.5 s into the line**: he takes up the pencil and writes; write shot of the book and hand.
     4 prince (I also have a flower) — **PencilStops**: the pencil stops; at least **0.9 s** with no box after the line ("pen-stop beat"); stop shot up to his face as he lifts his head.
     5 geographer (we don't record flowers) — TalkCalm loop; calm shot from the prince's side.
     6 prince (why? flowers are the loveliest).
     7 geographer (because flowers are ephemeral).
     8 prince (what does ephemeral mean?).
     9 geographer (in danger of disappearing soon).
     — **stunned beat 1.2 s** (the prince freezes); head lowers 12°; side shot glides ~145° from calm shot to the prince's stunned face, then prince shot over the geographer's shoulder (balcony bright behind).
     10 prince to himself (his flower is ephemeral, has only four thorns, and he left her alone) — **BackToBook 1.0 s into the line** (crossfade 0.25 s): the geographer returns to his book.
   - Wide shot after line 10 (room from front-right, high; FOV 42) holds **1.8 s**, then the banner. `completeDelay` 1.0 s after the last subtitle fades → CompletePlanet. [ctor]/[scene]
   - Pacing (story): `0.18 s × chars + 1.2 s`, min 2.6 s. (Conversation component's own: min 2.5 s, 0.35 s/char, gap 0.3 s.)
   - A warm fill light (intensity 1, range 2.5, 3600 K) on the prince's face only, 0.9 m toward the camera, 0.25 m up, faded in for the talk (the bright back opening behind him made his face dark). [ctor]
   - Pencil sounds: writing windows PickWrite f108–146, PencilStops f1–13, BackToBook f8–39 (vol 0.55); pick 0.3, put 0.5. [scene]
4. **Return view** after the talk: along the wide shot's heading turned **-50°** (ends on the room side, prince in profile facing the geographer), pitch 10°, over 2.4 s. [ctor]
5. **Chapter complete.** Not replayable (`replayable = 0`).

## 5. Side activity — "Gaze out" at the balcony rail (OBSERVED [ctor] "Side quest: 眺望")

- Prompt "Gaze out" when standing at z ≈ 15.0 within **1.7 m**, anywhere along the far rail (|x| ≤ 3.4).
- Opens a look-out view: camera offset from him (right 0.75, up 1.75, back -1.7), initial yaw -14° from straight out, pitch 9° (down); free look yaw -60..45, pitch -12..30, FOV 50; enter 1.4 s, exit 1.2 s. "E Leave" hint fades after 4 s idle.
- Purpose: view of the sea of clouds (and, INFERRED, the direction of Earth). Not required, no reward found.

## 6. NPC behaviour

- Geographer is fixed at the table; reads in a loop; only the talk drives clips. His sleeves and pencil are physically attached by baked per-frame data (`GeographerSleeves`, `GeographerPencil` socket `pencil_socket.r`).
- No fail state.

## 7. Win / fail

- Win: finish the 10-line conversation. No fail, no hidden goal.

## 8. Test route

1. Spawn (0, -0.7, -8). Walk to (0, 0, 2.5) (inside), then to (-0.72, 0, 7.6), then (-2.15, 0, 7.6), face -Z.
2. Press E "Talk". Advance 10 lines; expect clip sequence SpotExplorer, PleaseTell, PickWrite, PencilStops, TalkCalm…, BackToBook.
3. Assert chapter complete; print `[test:ok geo]`.
4. Optional: walk to (0, 0, 15.0), E "Gaze out", E "Leave".

## 9. Notes for the rebuild

- Building: rectangular hall 6.2 × 9.2 m with a skylight, front doors open, back opening to a balcony with balustrade; small tower.
- Keep the emotional beat timing: pen stops (0.9 s silence), stunned beat (1.2 s), head down 12°, end hold 1.8 s.
