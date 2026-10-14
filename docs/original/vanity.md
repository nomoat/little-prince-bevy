# Chapter 3 — Vanity / the old theatre (original design analysis, analyst A)

Scene `Theater_Main`. Classes (Assembly-CSharp `LittlePrince.Vanity`): `VanityStory`, `VanityPerformer`,
`VanityPerformerLook`, `VanityClap`, `VanityDoor`, `VanityPushPose`, `VanityHUD`, `VanityCameraComfort`,
`VanityAtmosphere`, `VanityPlanarMirror`, `VanityMeadowInstances`, `VanityStaticScenery`.
Text is paraphrased.

## 1. Chapter card and objectives — OBSERVED (`VanityStory` static ctor)
- Chapter 3, title "the vain man's planet", subtitle: a planet nobody ever passes, home of a vain man.
- Narration: a vain man lives here; he hears nothing but praise.
- One event: "go and see the man on the stage".
- Location label on arrival (`VanityHUD.locationName`): "the old theatre".

## 2. World layout — OBSERVED + INFERRED
- Island with a meadow (instanced grass/flowers) and a **tent-like old theatre**. Opening shot (world)
  cam (29, 13, −29) → look (2.6, 4.3, −14.7), FOV 55; push 3 m, settle 4 s.
- World frame: the theatre sits at the origin, **door plane z = 0**, hall interior x ∈ [−7.4, 7.4],
  z ∈ [0.6, 17.6] (`VanityClap.interiorX/Z`).
- **Double door** (`VanityDoor`): leaves swing **inward only** to **105°** in **1.25 s**; doorway 3.2 m × 3.1 m.
  From outside: authored two-hand push ("push the door", approach 0.9 m/s, align 0.25 s); from inside the
  prince steps clear of the swing and **pulls** ("pull the door"); "close the door" when open. The door will not
  swing through the player (`PlayerInLeafSweep`). Creak fades out 0.5 s.
- Inside: rows of **audience chairs** (child-size seat 0.30 m; sit/stand clips), an aisle, a **stage with steps**,
  the vain man on stage (≈ z 14.5), a large **mirror** on the +x side of the stage (planar reflection).
  Warm flickering lamps, threshold blend of interior atmosphere.
- Talk stance (world x, z) **(0, 11.15)**: foot of the stage steps, between the vain man and the seats; the foot of
  the steps is **3.4 m** from him, the end of the aisle **4.5 m**.

## 3. Mechanics — OBSERVED
- **Clap** (`VanityClap`): key **C** (`PlayerInputFrame.clap`). Only inside the hall interior box and when not
  locked. Clip `LP_Clap` 62 frames @30 fps = **2.03 s**; palm contacts at frames 19, 28, 38 (0.63/0.93/1.27 s),
  one random dry clap sound per contact (never the same twice), volume 0.6; blend in 0.2 s, out 0.25 s.
  Works standing or **seated** in an audience chair (`PlaySeated`). Event `Clapped(index)`, `ClapCount`.
- **Vain man answers every clap** with a hat salute (`SaluteOnClap`, triggered on the third palm contact);
  salutes are picked to start from the current idle frame (smooth hand-off). Gestures by name:
  `Performer_TeachClap`, `Performer_Salute`, `Performer_Plead`. Idle loop 360 frames (12 s);
  gesture fade 0.22 s; head look range **15 m**, max yaw 35°, pitch −25/+5, target height 1.1 m.
- HUD hint for clapping shown when `ClapHintPending` (verb "clap" with key C).

## 4. Beats in order — OBSERVED (`VanityStory.Start/Talk/Clap/Salute`, resolved strings)
1. Phase `card` → phase `visit`: star target "the vain man". Prince opens the theatre door, walks down the aisle.
2. Talk prompt "talk": radius **4.6 m** from the vain man (only offered inside the hall, `InHall`).
3. Phase `talk`: prince walked to the stance (0.9 m/s), facing him. Shot `two` (1 s) then `push`
   (slow push-in over **16 s** through lines 1–2 and the two claps).
   - L1 Vain (gesture TeachClap): ah, an admirer is coming!
   - L2 Vain: clap your hands (locked ≥ 2.3 s, max 4 s). The clap hint appears.
   - **Two claps** (loop count 2, `Clap()` routine): each clap is played (wait ≤ 4 s for it), then the vain man
     salutes (wait ≤ 6 s). 0.7 s pause after the second.
   - Shot `vain` (2.2 s). L3 Vain (gesture Salute): does he really admire him very much?
   - L4 Prince: what does "admire" mean?
   - L5 Vain (gesture Plead): to admire means to recognise him as the handsomest, best dressed, richest and
     cleverest man on the planet.
   - Shot `turn` (1.1 s): the prince turns to look back; shot `seats` (1.3 s): over him toward the empty rows.
     **2.4 s** into the look-back (`aloneLineAt`): L6 Prince: but he is the only person on this planet!
   - L7 Vain: please, admire him anyway! After **2.2 s** (`shrugAt`) the prince shrugs.
   - L8 Prince: he admires him — but what difference can that make to him? (1.2 s)
   - Shot `hall` (2.5 s): back on the stage behind the vain man's left shoulder; the prince turns away (+180°),
     the vain man raises his hat to the empty hall.
4. Phase `depart`: chapter complete.

## 5. Camera shots (world) — OBSERVED (`VanityStory` ctor)
| Shot | cam | look | FOV |
|---|---|---|---|
| two | (1.75, 1.6, 8.2) | (0.15, 1.95, 13.0) | 44 |
| twoShotIn (push 16 s) | (1.5, 1.6, 8.7) | (0.1, 2.0, 13.3) | 42 |
| vain | (1.05, 1.95, 10.3) | (0, 2.75, 14.4) | 34 |
| turn | (2.3, 1.8, 11.9) | (0, 1.3, 11.0) | 46 |
| seats | (1.5, 2.55, 13.3) | (0.3, 0.95, 9.4) | 48 |
| hall | (1.4, 2.95, 16.15) | (−0.55, 1.4, 8.0) | 54 |
All on the mirror side (+x). Talk-line timing as shared (0.18 s/char + 1.2, min 2.6).

## 6. Win / fail, hidden goals
- Win: complete the talk → `depart`. No fail state.
- Free claps outside the talk are rewarded by salutes (a toy, not a goal). No hidden goal found (INFERRED).
  A natural optional extension (INFERRED, not in original code): count salutes.

## 7. Test route (INFERRED)
```
wait 8; phase visit
moveto <door outside (0, -1.5)>; interact; wait 3     # push the double door
moveto 0 9                                             # inside, within 4.6 m
interact
phase talk
clap (PressClapNow) when the hint shows, twice; wait 40
phase depart   => [test:ok vanity]
```
