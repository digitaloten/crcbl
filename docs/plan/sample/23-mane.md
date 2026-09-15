# Sample 23 — mane (S4H, gates P7G)

Hair and fur acceptance test, and the fixture that proves
[58-hair.md](../58-hair.md): fur, card hair and strands on figures that move, in
a wind that gusts, natively and in a browser tab.

**This is the sample that shows hair follows what it is attached to.** Every
figure is driven: it turns, sprints, stops and rides a lift, and the hair's
response to each of those — trailing, overshooting, settling, and not floating
in the lift — is what is measured. Hair that only sways in wind passes nothing
here.

## Proves

- **Every hair rung the engine ships is reachable**: shell fur, wind-mesh cards,
  cards on simulated chains, and guide strands where the device offers them.
- **Parent motion drives the simulation**, and the inertia scale is a live knob
  whose two ends are visible: full inertia trails, zero inertia moves rigidly
  with the head.
- **The solver does not care about the tick rate.** The same motion at two tick
  rates settles to the same shape.
- **Wind is the same field** [56-wind.md](../56-wind.md) serves to grass and
  bodies: a gust from a fan the player places streams the hair and lifts the fur
  in the order the positions predict.
- **Collision holds**: no strand passes through the shoulders or the head.
- **No temporal accumulation**: thin hair stays stable with cutout widening or
  alpha-to-coverage, and every rung is a golden.

## Scope

- **A turntable stage** with three figures on scripted motion — a head with a
  ponytail and fringe on chains, a furred creature on shells, and a head of
  guide strands — each on a key and page buttons.
- **Motion scripts**: turn, sprint and stop, jump, a lift ride, and a teleport
  that must reset rather than whip.
- **Knobs**: inertia scale, stiffness, damping, wind speed and direction, shell
  count, coverage mode.
- **Debug views**: chain particles and constraints, capsules, the parent's
  acceleration vector, the wind sample per segment.
- **Per-pass and per-tick cost** in the debug panel and the headless summary.
- **Pages web demo** at `/demos/mane/`, every knob on the page; strands at the
  count the browser tier is priced for.

## Non-goals (hard cap)

A character creator. Grooming tools. Cloth beyond the tassels a chain carries.
Full strands with a visibility buffer in the browser.

**Exempt from sample rule 11**, on lantern's ground. **Exempt from rules 2 and
10**, on sundial's: no game state, no `World`, no `GameModule` — the hair is
visual and the figures are scripted.

## Status: planned 2026-09-15, nothing built

## Milestones

1. **Fur.** 58's H1 with 57's shells: the creature, the parent spring, the wind
   field's W1, the sample skeleton, the web demo and the CI golden step.
2. **Wind-mesh cards.** H2: the Kajiya-Kay pass and a sway spring.
3. **Chains.** H3: the secondary motion stage, XPBD chains, inertia scale, the
   motion scripts and the tick-invariance check.
4. **Strands.** H4: guide and follow strands in compute, Marschner tables.

## Exit criteria

- Every hair rung the engine ships is reachable, and the sample names any the
  device removed.
- Parent coupling: a measured trail distance under the sprint script that falls
  monotonically as the inertia scale rises, and is zero at zero inertia.
- Tick invariance: the settled shape at two tick rates within the tolerance
  [58-hair.md](../58-hair.md) states.
- A golden per rung, plus one mid-gust.
- Per-pass and per-tick cost in the debug panel and the headless summary.
- Web demo deployed, running on the WebGPU backend.
- Rule 12: selected paths reported, a flag forces a lesser one.
