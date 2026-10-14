# Original-design notes — analyst A index

| File | Content |
|---|---|
| `shared-core.md` | Player speeds, input map (incl. clap = C, depart = Q), camera, interactables, story runner (cards, objectives, stall help 40 s / 8 s, dialogue timing), opening template, journey goals, AgentInput / PlaytestRoute harness |
| `b612.md` | Chapter 1: wake-up opening, controls tutorial, rose call (4 m), can → spring → water loop, farewell + glass-cloche ritual, chair + last sunset (sit → 3.5 s hop + 22 s sunset) |
| `king.md` | Chapter 2: palace door, King idle/notice (7 m), audience at stance (0.7, 1.25) in King space, calendar/7:40 beat, ambassador call |
| `vanity.md` | Chapter 3: theatre double door (inward, 105°), clapping with C + hat salutes, two-clap talk, empty-seats look-back |
| `bar.md` | Chapter 4: night tavern, two doors, sit on the guest chair to start the talk, drink / head-down / outburst / 3 s silence |

Each statement is tagged OBSERVED (with the class/method) or INFERRED.
Tools used: `scratchpad/anaA/fd.py` (decode ctor defaults by field offset), `findctor.py`, `ann.py`
(resolve string refs: literal index = d-table index − 32756).
