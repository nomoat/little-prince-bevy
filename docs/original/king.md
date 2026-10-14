# Chapter 2 — King (original design analysis, analyst A)

Scene `RoyalCourt_Walkthrough`. Assembly `King.Runtime`: `KingStory`, `KingIdleDriver`, `KingHeadLook`,
`CourtDoor`, `CourtPushPose`, `CourtArea`, `CourtArrivalHint`, `CourtCameraSafety`, `CourtCandleFlicker`,
`CourtPlanarReflection`, `CourtAmbience`, `CourtFootsteps`, `CourtInteriorGrade`.
Text is paraphrased.

## 1. Chapter card and objectives — OBSERVED (`KingStory` static ctor)
- Chapter 2, title "the King's planet", subtitle: an empty palace where a king lives.
- Narration (2 lines): a king lives here; to him everyone is a subject, even the stars obey him.
- One event: "go and see the King" (no further steps observed).
- Journey goal: Story (completes when the audience ends).

## 2. World layout — OBSERVED (tooltips/fields) + INFERRED
- Small island planet with a **palace**; the camera opening shows the whole planet from above-front-right
  (`openingShot` world cam (26, 12, −30) → look (−1.83, 4.31, −10.22), FOV 50; push 3 m during the card,
  settle 4 s).
- **Palace door** (`CourtDoor`, an `Interactable`): prompt "push the palace door"; one leaf, opens to **100°**;
  the prince walks to the stance at **0.9 m/s**, aligns 0.25 s, plays a two-handed push clip (`PushDoor`,
  clip blend in 0.15 / out 0.3); creak/latch/settle sounds; states Closed → Approaching → Reaching →
  Pushing → Releasing → Open. Only usable from outside (`Outside(p)`).
- Short **arrival hint** text on entering (`CourtArrivalHint`): visible (alpha 0.85) for 12 s, fades over 3 s.
- **Throne hall** (`CourtArea` box): long hall along +Z (hall shot looks from z ≈ 12 toward the throne at
  z ≈ 15.5), red carpet on the centre line, high windows, **polished floor with planar reflection**, candelabra
  with flickering candles, a **dais with steps** and the throne. Camera keeps 2 m from the doorway in passages
  (`CourtCameraSafety.passageDistance`), lens radius 0.16 m, wall margin 0.035 m.
- King space (origin at the King, +x his right = scepter side, +z in front of him):
  - dais sides at |x| > **2.3 m** are too high to step on; the foot of the steps is **2.6 m** in front;
  - candelabra stand within |x| < **3.25 m** and up to z = **2.05 m**;
  - audience stance **(0.7, 0, 1.25)**: off the carpet line to his right so his calendar (on his left thigh)
    stays visible; leave point **(1.6, 0, 2.9)**.
- The King: seated, scepter in right hand, **a calendar/book on his left thigh** (used in the sunset beat).

## 3. NPC behaviour — OBSERVED (`KingIdleDriver`, `KingHeadLook` ctors)
- Idle loop 450 frames (15 s at 30 fps) with **random fidgets every 2–5 s** (cross-fade 0.3 s, action cut 0.45 s,
  exact-frame wait ≤ 0.25 s). Named gestures: `King_Notice`, `King_Talk_A`, `King_Talk_B`, `King_Consult`.
- **Notices the player** (auto `King_Notice`) when the prince comes within **7 m**; re-arms after the prince is
  farther than **10 m** for **3 s**. Head-look weight fades 0.8 s around clips.
- Head look: target height 1.0 m, range 9 m, max yaw 30°, pitch down 18° / up 5°, turn speed 2.5, neck share 0.35.

## 4. Beats in order — OBSERVED (`KingStory.Start`, `Audience` coroutine, resolved strings)
1. Phase `card`: card 3 s over the opening shot; camera descends to the play view.
2. Phase `visit`: star target on the King ("King" label). The prince opens the palace door himself and walks in.
3. Talk prompt "talk" (`BuildTalk`): `Interactable` at the KingMarker, **radius 3.1 m**, must face it within
   **70°**, and stand on its front side (half angle **70°**), priority 1.
4. Phase `audience` (after E):
   - Path: `AudienceRoute(from)` — from beside the dais he first walks straight out to the foot of the steps,
     from behind a candelabrum he first steps out past it, then up to the stance (walk 0.9 m/s), turning to face
     the King. Shot `two` (move 1 s), then `push` = slow push-in over **14 s** (`greetingPush`) during lines 1–3.
   - L1 King (gesture `King_Talk_A`): delighted — here comes a subject!
   - L2 Prince: how can he know I am a subject when he has never seen me?
   - L3 King: everyone who comes before him is his subject; everything here obeys him, even the stars.
   - Shot `king` (2 s move). L4 Prince: then could he make the sun set right now? He wants to see a sunset.
   - The question stays **2.2 s** (`consultAskAt`), then shot `calendar` (1.2 s): the King runs his finger down
     the calendar (`King_Consult`); at **5.2 s** into that clip (finger finds the date, he nods) shot `king`
     again and L5 King: of course, but an order must be given at the right time — tonight at **7:40** he will
     order it to set.
   - Shot `prince` (2 s). L6 Prince (aside): at 7:40 it would set anyway. He shrugs (one-shot, ~1.4 s).
   - L7 Prince: at home he only had to move his chair a few steps to see a sunset.
   - Shot `hall` (world, 2 s): from the hall, he walks toward us with the King on the throne behind him.
   - L8 Prince: he must go. He walks away toward the leave point (0.9 m/s).
   - After he has walked **0.35 m** (`callAfter`), the King leans out after him (gesture) and calls
     L9: wait — he appoints him his **ambassador**! (King lines start 0.35 s into the gesture, wait ≤ 1 s for it.)
     The prince does not answer; he keeps walking.
5. Phase `depart`: chapter complete; departure UI.

Line timing: auto `0.18 s/char + 1.2 s`, min 2.6 s; click after 0.5 s.

## 5. Camera shots (King space unless noted) — OBSERVED (`KingStory` ctor)
| Shot | cam | look | FOV |
|---|---|---|---|
| two | (−1.75, 1.2, 2.75) | (0.25, 0.82, 0.7) | 40 |
| twoShotIn (push, 14 s) | (−1.5, 1.2, 2.5) | (0.2, 0.9, 0.6) | 39 |
| king | (−0.95, 1.2, 2.05) | (0, 1.15, 0.15) | 36 |
| calendar | (−0.62, 1.38, 1.05) | (−0.15, 0.94, 0.3) | 32 |
| prince | (−0.95, 1.1, 0.8) | (0.7, 0.95, 1.35) | 42 |
| hall (world) | (−1.35, 1.15, 12.2) | (−0.5, 1.05, 15.5) | 40 |
All on the King's left (the calendar side); the camera moves only when someone acts.

## 6. Win / fail, hidden goals
- Win: finish the audience → `depart`. No fail state. No hidden goal found (INFERRED).
- The sunset joke is a story beat only: no time/sunset mechanic in this chapter (INFERRED: no day-cycle
  calls in `King.Runtime`).

## 7. Test route (INFERRED)
```
wait 8; phase visit
moveto <outside door stance>; interact; wait 4     # push the palace door
moveto <king space (0.7, 3.5)>                      # inside talk radius 3.1, facing him
interact
phase audience
wait 75                                             # or click through lines
phase depart   => [test:ok king]
```
