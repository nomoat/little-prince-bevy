# Chapter 8 — Earth (original id `earth`, scenes `Desert_Valley` → `Desert_Dunes`)

Design reference for our Bevy rebuild. Reproduce structure, mechanics and numbers; never copy the
original art, audio, code or text. Dialogue is paraphrased (line ids S/R/F/L are the original's
`const` names, kept so we can map beats; write our own words).

Legend: **OBSERVED** = read from the shipped build. **INFERRED** = my reading; verify in play.

Sources:
- `[sk]` skeletons `out_cs/DiffableCs/Desert.Runtime/LittlePrince/Desert/*.cs` (DesertStory, DesertPilotStory 4.8k lines, DesertTrain, TrainRoutePage, DesertSnakeBehaviour, DesertFoxWander, DesertWellRig, DesertEnding, DesertEndCard, DesertTitleCard, DesertDrawingPanel, DesertFootprints, DesertLayout …).
- `[ctor]` DesertStory initialisers from `game.dcmp` func90242 (label `DesertStory_SkipTo`, the function before `DesertStory_ctor`); `_PrivateImplementationDetails_` blobs of Desert.Runtime.
- `[valley]` / `[dunes]` serialized values / transforms from `earth_scenes_desert_valley_*.bundle` and `earth_scenes_desert_dunes_*.bundle` (scratchpad `anB/d_valley/`, `anB/d_dunes/`).
- `[loc]` LPStrings_en.txt (sections "Chapter Eight", "railway page", "aviator and the way home", "end card").

---

## 0. Big picture

| Item | Value | Src |
|---|---|---|
| Chapter | 8, "Earth" — "the seventh planet" on the map; card subtitle: a very large planet and nobody in sight | [loc] |
| Map blurb | the geographer said go see Earth / "He is here" / a desert with one especially bright star overhead | [loc] paraphrase |
| Two scenes | **Valley** (snake, flower, echo, rose garden, fox, station) then **Dunes** (pilot, well, sacred way, doorway, farewell, ending). A train ride links them. | [sk] `ValleyScene="Desert_Valley"`, train `nextScene="Desert_Dunes"` |
| Controllers | `DesertStory` (valley; runtime-created), `DesertPilotStory` + `DesertTitleCard` (dunes; placed), `DesertEnding` (dunes) | [sk]/[dunes] |
| Story stages (valley) | `Snake → Roses → Fox → Train → Boarded` | [sk] enum `DesertStory.Stage` |
| Chapter events (goal list) | 0 "Find out where you are" (step: talk to the snake on the rock) · 1 "Visit the rose garden" · 2 "Find the fox under the apple tree" · 3 "Set out for home" (train) · 4 "Wake the aviator" · 5 "Return to where you fell" (later step: follow the stone path to the gateway) | [loc] + [sk] `EvTrain=3, EvWake=4, EvHome=5` |
| Terrain | large heightfield terrain (not a domed planet): valley playable area ≈ x -280..-180, z 215..360; dunes ≈ x -15..110, z 35..80. Invisible walls bound both. | [valley]/[dunes] |
| Music | Valley -33.6 LUFS; Dunes -24 LUFS; both delay 6 s, rest 40–80 s, fades 4/1.2 s | [valley]/[dunes] |
| Camera | valley: distance 3.6 (1.8–5.5), pitch 10°, FOV 50, far clip 6000 | [valley] |
| Line pacing | `0.18 s × chars + 1.2 s`, min 2.6 s; click after 0.5 s lock (both scenes) | [ctor]/[dunes] |

The switchman, the pill merchant and the well-in-the-valley scenes of the book are **not** implemented: the valley has `Anchor_Switchman` (-244.8, 7.63, 355) and `Anchor_Merchant` (-192.8, 7.85, 348.3) but no NPC, no prompt and no lines (OBSERVED absence in [valley] + [loc]). INFERRED: cut for scope. Our rebuild may skip them (strict) or add them as optional flavour.

---

## 1. VALLEY scene

### 1.1 Layout (OBSERVED [valley] anchors; south→north is the route)

| Place | Position | Notes |
|---|---|---|
| Spawn (landing) | (-271.0, 8.29, 220.0), facing +Z | |
| Snake on its rock | (-272.9, 8.59, 220.3) — right beside the spawn | rock top 0.30 m above sand; an invisible post 1.6 m high, 0.05 m wider than the rock stops the prince stepping onto it ([ctor] `rockPostHeight/Pad`) |
| Small three-petal flower | (-272.3, 11.22, 242.4) | on the climb |
| Summit (echo point) | (-272.2, 13.0, 266.4) | ridge pass, looks down on the garden |
| Rose garden (talk anchor) | (-250.7, 7.43, 294.6) | walled garden: hedges, walls with rose caps, iron double gate (open), back arch, fountain (-247.4, 8.0, 294.9), rose beds, fallen petals, urns, topiaries |
| Fox / apple tree | fox centre (-227.0, 8.16, 316.2); trunk (-226.2, 8.16, 319.8); fox faces the wheat field (yaw 79.3°) | short-grass clearing, wheat/field beyond, footbridge over a stream, waterfall, stone cottages, vegetable plots |
| Station / board point | (-209.3, 7.85, 349.4) | platform, cabin, benches, lamps, signal, notice board, telegraph poles; rail along z = 352 |
| Way signs + cairns | signposts with "place · N m" text (rose garden / apple tree / station) and stone cairns along the route | [sk] `WaySigns`, `WayCairns`, glyphs 玫瑰园苹果树车站米 |

Guidance ([ctor]): `guideSigns = true` — the goal marker ("Rose garden · 40 m") lights up by itself when each leg starts and at its signs: halfway up the climb, the ridge pass, the garden gate, the garden's back arch, the bridge, the station stairs. It stays lit **4 s**; triggers within **6 m** of a sign.

### 1.2 Beat V1 — the snake (stage Snake, event 0)

- Snake starts coiled; head-look follows the player within 4 m (coiled yaw ≤25°, pitch ≤10°; raised yaw ≤75°, pitch ≤30°). States: Coiled, Raising, Raised, Settling, Leaving, Gone. [valley] DesertSnakeBehaviour
- Prompt "Talk to the snake" (INFERRED radius ≈ 2.2 m like the dunes prompt). On E the snake raises (BeginDialogue), plays talk / lean-in one-shots.
- Lines (speaker OBSERVED by const names / book order; text paraphrased):
  S1 prince: which planet have I fallen on? — S2 snake: Earth, in Africa — **S3 prince looks up at his star** (a star placed 40 m above him; the look may pitch up to 55°): it is right above us… but how far — S4 snake: it is beautiful; why are you here? — S5 prince: trouble with a flower — S6 prince: where are the people? it's lonely in the desert — S7 snake: it is lonely among people too — S8 snake: whoever I touch I send back to the earth he came from — S9 snake: you are weak on this granite Earth; I pity you; one day, if you miss your planet too much, I can help.
- After: `EndTalk(Roses)`; snake settles (INFERRED: stays on its rock; the valley snake has `crawlOn = 0`, fade-away 2.3–3.1 s exists but use is unclear).

### 1.3 Optional V-a — the little flower (on the climb) [sk] `FlowerTalk`, `_flowerDone`

- Prompt "Talk" at (-272.3, 11.2, 242.4). The prince's look pitches down up to 50° to the flower.
- L10 prince: where are the people? — L11 flower: people? the wind blows them about; they have no roots, that makes it hard.
- Not stage-gating (INFERRED; counted in `ValleyEventsDone`).

### 1.4 Optional V-b — the echo at the summit [sk] `EchoShout`, `_shouted`

- Prompt "Call out" near the summit. He walks to `Anchor_Summit` at **1.3 m/s**, turns to the rose garden, and shouts.
- Shot behind him: 4.5 m back, 1.4 m aside, 1.6 m up; looks at a point 10 m on toward the garden (him in the near third, garden below). [ctor] `echoShot`, `echoLookAhead`
- L12 prince: hello! be my friend, I'm lonely — L13 echo: hello… I'm lonely… — L15 echo: I'm lonely… (fading repeats).

### 1.5 Beat V2 — the rose garden (stage Roses, event 1)

- Prompt at the rose bed anchor. R1 prince: who are you? — R2 the roses (speaker "The Roses"): we are roses.
- He **stands still 2.5 s** (`roseStand`), head goes down (looks at the ground 1.1 m ahead, at most 40° down) and "cries"; R3 to himself (mono): he thought his flower was unique; it's an ordinary rose. [ctor]
- Then `EndTalk(Fox)`; signs guide over the bridge to the apple tree.

### 1.6 Beat V3 — the fox (stage Fox, event 2)

Fox wander AI (OBSERVED [valley] DesertFoxWander):
- Wanders in radius **2.5 m** round the clearing centre; keeps 0.7 m from the trunk, 1 m from the road, 0.6 m from tall (>0.3 m) vegetation; picks hops of 1–3 m, idles 2–5 s, sniffs every 3rd stop (post-sniff idle 0.5–1.5 s); walk 0.349 m/s, turn 90°/s; holds still if the player is < 0.9 m ahead, gives up after 1.5 s.
- **Alert** when the player comes within **6 m** (leaves alert at 8 m after 3 s); on talk it approaches and **sits 2.5 m** from the player.
- States: Wander, Alert, Approach, SitDown, Seated, StandUp, WatchWheat.

Talk lines:
F1 prince: come and play with me, I'm sad — F2 fox: I can't, I'm not tamed — F3 prince: what does "tame" mean? — F4 fox: to establish ties; to me you're one boy among a hundred thousand — F5 fox: if you tame me, you'll be unique to me — F6 prince: I'm beginning to understand; there's a flower… she has tamed me — F7 fox: go look at the roses again; you'll see yours is unique — **F8 prince: the roses are beautiful but empty; she matters more because she's the one I watered** (during F8 he glances back toward the rose garden for **2.2 s**, head + upper body up to 95°, head 60°, feet still, then back to the fox) — F9 fox: the secret: one sees clearly only with the heart; the essential is invisible to the eye — F10 fox: the time you spent on your rose makes her important — F11 fox: you are responsible forever for what you've tamed — F12 prince: I am responsible for my rose — then his head comes up to his star over **2 s** and holds **2 s** before the banner. [ctor]

(In the book he revisits the roses; here F8 is said to the fox with the glance — a deliberate simplification. OBSERVED via tooltip.)

### 1.7 Beat V4 — the train (stage Train → Boarded, event 3)

OBSERVED [valley] DesertTrain + [sk] TrainRoutePage:
- Steam locomotive + 2 carriages (lengths 12.69, 12.99, 12.99 m) stand at the station; rail is a straight line along z = 352.
- Prompt "Board the train", radius **3.2 m**, priority 5, at the station.
- Railway page sequence (`routePage = 1`, `cutsAndInk = 1`): E → whistle; he steps toward the carriage (0.6 s at 0.9 m/s); a ✦ appears beside him and blooms into a parchment "railway page" (star 0.4 s, cover 0.8 s) showing Valley → Dunes with a little train running along a drawn track as the **loading progress** (min 3.2 s on screen, progress capped at 96 % until loaded, hold 0.35 s at full), then uncovers (0.8 s) onto the arrival. Page texts: chapter "Chapter Eight · Earth", title "The Dunes", subtitle "Earth", one line about the last stretch with his rose on the star overhead (paraphrase).
- Departure (when not paged): cruise 9 m/s, acceleration 2.6 m/s², run 2.3 s; side-tracking ride camera across the rail (22→18 m to the side, 6.5 m up, FOV 42, 2.5 s push from carriages to locomotive); line-side poles within 16 m of the camera hide.
- Next scene prefetch starts 4 s after landing in the valley (web).

---

## 2. DUNES scene

### 2.1 Layout (OBSERVED [dunes])

| Place | Position | Notes |
|---|---|---|
| Train arrival / prince step-off | (-13.19, 7.53, 67.2); rail along z = 70 | stops at arc 1598.5 m |
| Plane wreck | (0, 5.81, 50.0), yaw 40° | repair set: crates |
| Sleeping pilot (bedroll) | (-4.74, 6.27, 49.41); wake stop (-4.63, 6.27, 51.20) | `startState = Sleep` |
| Seat crate (8 days later) | `Pilot_SeatCrate` (5.28, 6.30, 56.69); `CrateSeat` (-0.12, 6.76, 55.59) | wrench lies on the lid |
| Drawing crate (unused path) | (-0.06, 6.24, 45.54) | `noDrawing = 1` |
| Big dune top | (24.6, 14.8, 79.4) | |
| Well | (51.0, 6.72, 66.4); stone seats for both | bucket, pulley, rope |
| Sacred way start | (72.6, 1.24, 60.8) → doorway (92.33, 2.82, 49.47) | stone path with ruins (columns, blocks, sacred wall) |
| Doorway snake | `Snake_Doorway` (92.6, 3.06, 49.3), **starts raised** | prince's snake-talk spot (93.78, 2.82, 48.43) |
| Pilot comes back from | (82.81, 1.30, 54.96) | |
| Pilot farewell stand | (85.79, 1.30, 53.94), yaw 120° (on the sacred-way axis at the foot of the steps, facing the doorway) | |
| Horizon / last crest | (107.5, 6.25, 40.5) | evening star at azimuth/elev given as (120.4, 26.7) deg, intensity 30 |

Footprints (OBSERVED [dunes] DesertFootprints): the prince leaves prints on sand (0.10 × 0.20 m, ≥ 0.22 m apart, 4000 max); the pilot too (≥ 0.30 m apart). Line L49 refers to footprints.

### 2.2 Beat D1 — arrival (event 4 starts)

- Train arrives (approach 9 m over 2 s, filmed from the far side: 10 m aside, 5 m back, 3 m up, FOV 38); pause 0.2 s; a steam veil comes up (0.5 s), he is cut onto the sand 1.8 m from the rail facing the plane, veil clears (0.8 s). [dunes] DesertTrain role Arrival
- Goal "Wake the aviator". A **stardust trail** (GameUI.ShowTrail) runs from the prince to the sleeping pilot until E. [dunes] `wakeTrail`

### 2.3 Beat D2 — wake and draw (the plane talk)

OBSERVED flags [dunes]: `homeStory = 1` (the approved 2026-10-09 "way home" flow), `planeStand = 1`, `noDrawing = 1`.
- Prompt "Wake him", radius 2.6, 0.35 m over his chest. The prince stands at -70° from the pilot's seated facing, 1.35 m (beside his knees).
- Pilot sits up (sit-up stirs at 1.33 s) and **stands up**; the rest of the plane talk is standing; prince 1.3 m in front.
- Drawing is not animated: a **drawing panel** (torn sketchbook page, right third of the screen, 60–66 % screen height, slight tilt, soft shadow, slide/fade in) shows the drawing.
  L36 prince: please… draw me a sheep! — silent beat **2.0 s** → page shows an **old sheep** — L73 prince: too old; I want one that lives a long time — silent beat **1.5 s** → page shows a **box** — L37 pilot: this is the box; your sheep is inside — L38 prince: that's exactly what I wanted! — L65 prince: look! he's fallen asleep…
- Both look at each other's faces (eye points in head bones; pilot head pitch-up ≤35°).
- Plane two-shot keeps both characters between screen x 0.10 and 0.56 (page in the right third).

### 2.4 Beat D3 — "Eight days later" card, crate talk (event 5 starts)

- Title card (`DesertTitleCard`): fade out 0.6 s, big text "Eight days later" hold 1.5 s, small line "the little prince shared his journey" fades in 0.5 s after (0.18 s/char), fade in 0.6 s; card art holds 3.5 s with a slow push. [dunes]+[loc]
- Afternoon light afterwards (twilight share 0.12).
- Pilot sits on the seat crate, wrench on the lid beside him (static prop). Prince stands at 60° from his facing, 1.35 m, toward the big dune so he can walk off with the pilot behind him.
- L39 pilot: your memories are lovely, but my plane isn't fixed — L71 prince (looks up at the sky, pitch up to 45°): tonight it's a year; my star will be right above where I fell — L74 pilot: you're going back to where you fell? — L75 prince: mm… — L49 prince: you'll see where my footprints begin in the sand; wait for me there; I'll be there tonight (INFERRED position of L49 here).
- He turns and walks toward the big dune. Goal "Return to where you fell". INFERRED: the goal marker / signs lead over the dune top to the well.

### 2.5 Beat D4 — the well (alone)

OBSERVED [dunes] DesertPilotStory/DesertWellRig: `wellMechanism = 1`, `wellInstant = 1`, `princeWellActions = 0`.
- Prompt "Turn the winch" (shown when passing within **3 m**; base Interactable "Draw water" radius 2). On E: one windlass creak (vol 0.8) and the bucket is already on the rim; the lines start at once. (Non-instant variant: bucket rises 3 s, onto the rim 0.6 s, silence 1.5 s; rope lowers 2.0 s / raises 2.5 s.)
- L69 prince: what makes the desert beautiful is that it hides a well somewhere — L79 **the fox's memory** as a faint subtitle (alpha 0.75, `GameUI.Memory`): the essential is invisible to the eye. (Speaker assignment INFERRED from the tooltip "the fox's memory subtitle".)
- Step text changes to "Follow the stone path to the gateway".

### 2.6 Beat D5 — the sacred way, the doorway snake

- The light dusks with progress along the way: twilight share 0.6 at the doorway (band ±60 m), star fades in from share 0.12, full at 0.5. [dunes] `duskDoorShare`, `duskCorridor`, `duskStarFrom/Full`
- The snake waits **raised** in the doorway. Prompt "Talk to the snake" (radius 2.2, 0.3 m over its head). Prince stands on a flat slab 1.05 m from its head. On E: dusk eases to share **0.8 over 4 s**.
- L76 snake: one day, if you miss your planet too much, I can help you.
- The snake **crawls away** toward the far dune under the star: crawl-on loop, 2× speed, fades out after 1.4 m (fade 0.5 s), gone ≈ 2.5 s after start; the camera watches **2.7 s** before the prince turns. [dunes] DesertSnakeBehaviour (`startRaised = 1`, `crawlOn = 1`)

### 2.7 Beat D6 — reunion and farewell

- The pilot comes back up the sacred way from behind and stops on the farewell slab (85.79, 1.30, 53.94). Seen over the pilot's shoulder from a raised shot (back 4.0, right 0.9, up 3.0, FOV 42).
- L72 pilot: ah! you're here… — prompt "Talk" at the pilot (radius 4.5, widened to reach the prince at the top of the steps; prompt over his head).
- On E the prince walks out across the top landing (eases to 0.85 m/s, the walk down the steps is cut), meets **1.8 m** in front of the pilot; side two-shot (≥ 3.2 m out, 1 m aside).
- L50 prince: glad you found what your plane was missing; you can go home — L77 pilot: how did you know? — L51 prince: I'm going home today too… much farther… much harder — L52 pilot: little man, I want to hear you laugh again — L53 prince: at night, I'll be laughing on one of the stars; for you all the stars will laugh — L57 prince: my flower… I'm responsible for her; she's so weak, four thorns — L58 prince: there… that's all…
- Pilot head-look pitch limits 20° up / 35° down.

### 2.8 Beat D7 — walking out, the star, the end card

- He turns (165°/s) and walks up the steps through the doorway at **1.5 m/s**. From **5 m** before the doorway the camera sinks from the raised shot to (3.2 back, 0.9 right, 0.9 up from the pilot) and tilts up the sacred way over 4 s (FOV 50) until the evening star above the last dune is in frame. **3 m past the doorway** the picture fades to the end card over **2.5 s**. [dunes] `endStarShot`, `endFadePastDoor`, `endFadeSeconds`
- No death scene is shown; the snake's promise + the walk toward the star imply it (OBSERVED absence; consistent with all-ages tone).
- End card (`DesertEndCard`, `cardStyle = 1` = Rose; `StarTrail = 0` alternative): night sky fades in 2.4 s; gold star lights at 1.3 s over 1.6 s; quote line 1 (the time spent on your rose…) flows in left→right at 3.6 s (1.2 s per line, 0.35 s gap); quote holds 3 s; dedication fades 1.1 s; title "The Little Prince" 1.4 s; "The End" 0.6 s later. [dunes] DesertEnding.starTiming + [loc]
  - Our version: use our own closing quote paraphrase; the original's dedication line is the author's personal one — use our own or none.
- Then back to the journey map (all 8 planets visited → ending state).

---

## 3. Legacy flow kept in code (OFF)

`homeStory = 0` would run the 2026-10-07/08 flow: the pilot stands up, **leads the prince to the well** (`DesertPilotGuide`: base 1.5 m/s, max 2.4, waits if the prince is > 8 m behind, resumes at 4 m), the well talk together ("next evening"), the pilot coming back. Its line order is in the 22-entry blobs: book lines 36,37,38,39,40,41,59,42,43,44,47,60,48,49,50,51,52,53,55,56,57,58 with pilot-speaks flags F,T,F,T,F,T,F,F,F,T,F,T,F,F,F,F,T,F,F,T,F,F. **Do not implement**; listed so nobody mistakes it for the shipped flow.

## 4. Win / fail

- No fail anywhere. Valley requires Snake → Roses → Fox → Train in order (each later prompt appears only at its stage; INFERRED). Flower and echo are optional. Dunes is linear: wake → (card) → crate talk → well → doorway snake → farewell → end card.

## 5. Test route (`?test=1`)

Valley:
1. Spawn (-271, 8.29, 220). Face the snake (-272.9, 8.59, 220.3); E; advance S1–S9. Assert stage Roses.
2. (Optional) walk to (-272.3, 11.2, 241.5); E flower; L10–L11. Walk to (-272.2, 13.0, 265.5); E "Call out"; L12/L13/L15.
3. Walk via the ridge pass and garden gate to ≈ (-252.0, 7.4, 294.6); E roses; R1–R3 (wait 2.5 s stand). Assert stage Fox.
4. Walk over the bridge to within 2.5 m of the fox at (-227, 8.16, 316.2) (fox sits); E; F1–F12. Assert stage Train.
5. Walk to the station (-209.3, 7.85, 349.4); E "Board the train"; wait for the page/scene change.
Dunes:
6. After step-off at ≈ (-13.2, 7.53, 65.4): walk to (-4.63, 6.27, 51.2); E "Wake him"; L36, old sheep, L73, box, L37, L38, L65.
7. Card; crate talk L39, L71, L74, L75, L49.
8. Walk to the well (51, 6.72, 66.4) via (24.6, 14.8, 79.4) or the shoulder; E "Turn the winch"; L69, memory L79.
9. Walk the sacred way (72.6, 1.24, 60.8) → (93.78, 2.82, 48.43); E snake; L76; wait ≥ 2.7 s.
10. Pilot arrives; E at pilot; L72 → L58; auto walk-out; end card; print `[test:ok earth]`.

## 6. Notes for the rebuild

- Two separate scene states inside one Bevy app (unload valley, load dunes) with a "railway page" overlay as the loading screen.
- The fox and snake need small state machines (above). The pilot needs: Sleep → SitUp → StandUp → Idle; SitOnCrate; Walk (back to the farewell slab).
- Sky: dunes runs an afternoon→dusk ramp driven by the prince's progress along the sacred way, plus a star that fades in.
- Asset keys needed: snake, fox, pilot, plane wreck, crates, well (+bucket/rope), train (loco + 2 cars), rose garden kit, apple tree, signposts, cairns, ruins/columns, sketch pages (old sheep, box), end card art, railway page art (valley, dunes, train).
