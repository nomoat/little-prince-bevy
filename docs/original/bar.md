# Chapter 4 — Bar / the drinker (original design analysis, analyst A)

Scene `Bar_Main`. Assembly `Bar.Runtime`: `BarStory`, `BarTableScene`, `DrinkerActor`, `DrinkerAutoPlay`,
`BarSeat` (a `ChairSeat`), `BarSeatAssist`, `BarApproach`, `BarDoor`, `BarDoorWedge`, `BarPushPose`,
`BarPrinceLook`, `BottleLiquid`, `BarZones`, `BarCameraSafety`, `BarAudio`, `BarLightFlicker`, `BarLightLod`,
`BarChimneySmoke`, `DrinkerSounds`, `BarFootsteps`. Text is paraphrased.

## 1. Chapter card and objectives — OBSERVED (`BarStory` static ctor)
- Chapter 4, title "the drinker's planet", subtitle: a man who drinks to forget everything.
- Narration: a drinker lives here; he drinks to forget what shames him — drinking.
- One event: "go and sit in the tavern".
- Verbs used: push the door, sit down, stand up, depart.

## 2. World layout — OBSERVED (fields/tooltips) + INFERRED
- **Night** island with a lit **tavern**, moon top right, chimney smoke; opening shot (world)
  cam (−10, 4.5, −26) → look (−6.53, 3.98, −6.31), FOV 52; push 3 m, settle 4 s.
- Tavern (`BarZones`): a low **hall** (beams; camera kept under the beam underside indoors), a **stairwell** and an
  **upstairs** floor, a **yard** with a wall outside. Outdoor night air is muffled indoors (low-pass, dB drop,
  cross-fade) (`BarAudio`).
- Two **doors** (`BarDoor`): the **front door** (two-palm push clip, push ≈ 2.2 s, opens 100°, leaf 1.3 m,
  lights inside brighten with the opening angle, full at 75°) and a **back door** (quick push, its swing compressed
  into `coastSeconds`). Approach 0.9 m/s, align 0.25 s. A wedge holds a door open (`BarDoorWedge`).
- Inside: the drinker at a **table** covered with bottles (full and empty), a **guest chair** opposite him
  (high adult chair: seat ~0.46 m, feet off the floor, clip set "High": SitHighDown/SitHighIdle/SitHighUp,
  stand-up clip 3.3 s), an open window to the night.
- `BarSeat.requireReach`: the chair is only offered from where a probe capsule can actually walk to its standing
  spot (grid cell 0.1 m, step 0.05 m, ≤ 80 steps).

## 3. NPC behaviour — OBSERVED
- `DrinkerActor` animator triggers: **Drink**, **HeadDown**, **Outburst**, **LookWindow**.
- `DrinkerAutoPlay` (ambient): a sip every **9–16 s** (random), occasional look to the window; paused during the
  story. Story pauses: before first sip 1.0 s, after sip 2.5 s, head-down hold 2.5 s, after outburst 3.0 s.
- `BottleLiquid`: bottle fill 0.8 drops at 0.025/s while the mouth is under the surface, never below 0.35;
  liquid sways (2.2 Hz, damping 0.35).
- Prince while seated (`BarPrinceLook`): looks at the drinker (weight 0.7, head 0.75, body 0.08, ease 0.6 s);
  during the silence plays a sad seated pose (`SitHighSadIn`).

## 4. Beats in order — OBSERVED (`BarStory.Start/Talk`, `BarTableScene.Scene`, resolved strings)
1. Phase `card` → `visit`: star target "the drinker" (on the guest chair). The prince opens the tavern door.
2. **Sitting on the guest chair starts the scene** (`B612GameEvents.SitStarted` for that seat; there is no "talk"
   prompt). `BarSeatAssist`: incoming speed is dropped if it cannot brake before the standing spot (margin 0.35 m).
3. Phase `talk`: wait until the drinker is idle (≤ 12 s). Shot `two`, then `push` (slow push-in over **22 s**).
   **0.8 s** after he is seated the drinker takes his first sip.
   - L1 Prince: what are you doing here? — L2 Drinker: I am drinking.
   - L3 Prince: why are you drinking? — L4 Drinker: to forget.
   - L5 Prince: to forget what? → drinker **drinks** → shot `drinker` (1.8 s, over the bottles).
   - L6 Drinker: to forget that he is ashamed. — L7 Prince: ashamed of what?
   - Drinker **lowers his head**, then the **outburst**: L8 Drinker: ashamed of drinking! (locked **1.0 s**, no
     click, until he has shouted it).
   - Shot `silence` (2.2 s): side-on, table and empty bottles between them, window behind. **3 s** of silence,
     both heads down; then his last sip.
   - Phase `done`; camera moves back to the seated view (**2.5 s**).
   (BarTableScene fallback pacing: question 2.4 s, answer 2.1 s, before shame 1.2 s, silence 3 s.)
4. Phase `depart`: he can stand up ("stand up"; view turns back behind him 0.3 s into the 3.3 s stand-up, over
   2.2 s, if it is > 90° off) and leave; departure UI.

## 5. Camera shots — OBSERVED (`BarStory` ctor; INFERRED to be in the guest-chair frame)
| Shot | cam | look | FOV |
|---|---|---|---|
| two | (−1.24, 1.4, 0.75) | (−1.97, 0.92, 2.55) | 50 |
| twoShotIn (push 22 s) | (−1.37, 1.32, 1.07) | (−1.97, 0.92, 2.55) | 48 |
| drinker | (−1.35, 1.3, 2.4) | (−2.03, 1.04, 3.38) | 42 |
| silence | (0.4, 1.25, 2.7) | (−1.2, 0.99, 2.73) | 48 |
All from the room side (+x). Bar camera safety: interior distance/pitch clamp, lens under the beams, side views
along walls instead of top-down lifts outdoors.

## 6. Win / fail, hidden goals
- Win: sit at the table and let the conversation play → `depart`. If he stands up mid-scene the talk is
  aborted (`Abort()`, OBSERVED method) and replays on the next sit (INFERRED from `_played/_talkRoutine`).
- No fail state, no hidden goal found (INFERRED).

## 7. Test route (INFERRED)
```
wait 8; phase visit
moveto <front door stance>; interact; wait 4    # push the front door
moveto <guest chair stand spot>; interact       # sit down
phase talk
wait 45
phase depart   => [test:ok bar]
interact                                        # stand up (optional)
```
