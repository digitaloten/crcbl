# Sample 07 — towers (flagship)

Co-op 3D tower defense, 1–4 players. The flagship sample: every engine pillar in
one shippable game, and the long-lived dogfood project that keeps evolving with
the engine. Genre chosen because command-latency-tolerant gameplay makes
interpolation-only MVP netcode _fully sufficient_ — multiplayer first-class
without needing prediction.

## Proves

- **Everything at once**: the integration test the small samples can't be.
- **Multiplayer as first-class citizen**: same build runs solo (in-memory
  transport) and **LAN co-op** (UDP when it lands at P13, host found by direct
  address or local-network discovery). `PlaceTower`/`UpgradeTower`/`StartWave`
  are commands — server validates, replicates; latency is invisible by genre
  design.
- **Editor as content pipeline**: maps (path splines, build plots, spawn points,
  props) are authored in the stage 8 editor and shipped as `.scn/` scene dirs.
  The editor's real-world usability is measured by building towers maps in it.
- **GPU-driven at gameplay scale**: creep waves (hundreds–thousands),
  projectiles, tower instances — horde's lessons applied in a real game.
- **Full UI surface**: build menu, tower select/upgrade panel, wave timer,
  economy readout, world-space health bars (3D-space UI), minimap via ortho
  second view (the 2D story again, as a feature).
- **System-owned-array ECS as textbook**: `CreepSystem`, `TowerSystem`,
  `ProjectileSystem`, `WaveSystem`, `EconomySystem`, `PathSystem` — the sample's
  code is the ECS documentation.
- **Physics slice it drives** (interleaved build): CCD vs moving targets (TOI
  where both bodies move), kinematic spline-followers in the broadphase, trigger
  volumes (creep-reaches-exit), character controller (dev fly/walk camera on the
  map — the controller's first real terrain).
- **Audio grammar in anger** (topic 13): off-screen creep waves locatable by ear
  (direction + behind-cue), tower fire pans with the camera, occlusion muffles
  lanes behind terrain — the esports-legibility claim demonstrated in a real
  game, native + browser.

## Scope (MVP of the sample)

- 1 map (editor-built), fixed creep path (spline-follow kinematic bodies; no
  dynamic pathfinding/maze-building — that's the RTS trap).
- 3 tower types (single-target, splash, slow) + 1 upgrade tier each. All combat
  through `crcbl-phys`: tower acquisition = sphere overlap (range),
  single-target = swept-projectile CCD vs _moving_ creeps, splash = overlap
  burst at impact point.
- 3 creep types (fast, tanky, swarm), 10 scripted waves, shared team lives +
  shared gold.
- 1–4 players co-op; solo = same game over in-memory transport.
- Win/lose, restart, lobby-lite (join before wave 1; late join post-MVP).
- Save/resume (topic 14): manual + between-wave autosave; solo and
  dedicated-server co-op (world save server-side, clients rejoin into it — save
  = same snapshot machinery as join-in-progress).
- **`.crpix` art throughout the 2D layer** (sample rule 11): tower and creep
  icons, the wave banner, the build menu, the range indicators. As the flagship
  this is also where skinned buttons (`Button::with_skin`, nine-slice) stop
  being a `crcbl-vk` golden and become a real UI — a build menu is the first
  place a button's corners surviving a resize is something a player sees.
- **Debug panel on, with its network module** (sample rule 4). Towers is the
  first sample where that module is not decoration: 1–4 players over a real
  transport is what the netgraph — RTT, jitter, loss, snapshot size, tick-lead —
  was specified for, and this is the sample that finds out whether it reads.

## Non-goals (until engine post-MVP)

Maze-building/dynamic pathing, PvP, campaign/meta-progression, difficulty modes,
cosmetics, matchmaking (direct connect only), ~~audio (engine gap)~~.

**The audio non-goal is withdrawn: there is no engine gap.** `crcbl-audio` ships
— a device seam with a real-time streaming thread natively and an `AudioWorklet`
in the browser, a mixer, a spatial module and a synth — and every 2D sample on
the ladder already emits spatial cues through it. Sample rule 8 therefore
applies to towers with no exemption, and the "audio grammar in anger" bullet
above is a requirement rather than an aspiration.

## Milestones

1. Solo loop on hardcoded map: creeps walk spline, towers shoot, gold, waves
   (buildable from stage 7; genuinely fun checkpoint). **First slice built
   2026-09-07** — see "Where this stands" for what it holds and what it owes.
2. Map from editor: author the real map in stage 8 editor — this milestone _is_
   stage 8 dogfood.
3. Co-op over real transport + browser client (stage 10 exit demo: wasm client
   into native dedicated server).
4. Polish pass: world-space health bars, minimap, game-feel cheap wins.

## Where this stands

**Milestone 1's first slice is built.** `apps/towers` is the solo loop on a
hardcoded map, running natively: creeps walk a path as kinematic bodies, one
tower type acquires and shoots them, a kill pays gold, a scripted table of three
waves runs out, and the run is won at the end of the table or lost at zero lives
and then plays itself again. `PlaceTower` and `StartWave` are commands the
client seals into bytes and the server validates over `InMemoryTransport`, so
solo is already the same game the co-op build will be.

**It is not on the demo site.** There is no `towers` row in `web/build.sh`'s
`DEMOS` array and no directory of its own under `web/demos/`, so rule 7 is
unmet; the library builds for `wasm32` and the browser front end is the next
slice.

| Slice | What it is                                                                                                                                                                                                                   | Status               |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------- |
| 1     | Solo loop on the hardcoded map: the polyline path, one creep archetype, one single-target tower, three scripted waves, shared gold and lives, the three commands over the loopback, a fixed overhead camera, the debug panel | **Built 2026-09-07** |
| 2     | The browser demo: `src/web.rs`, a `towers` directory under `web/demos/`, the `DEMOS` row, the Pages steps, the feature card and the browser gate                                                                             | Next                 |
| 3     | The rest of milestone 1's content: splash and slow towers, one upgrade tier each, the tanky and swarm creeps, all ten waves, `.crpix` art and the build menu it makes possible, spatial audio, world-space health bars       | Owed                 |
| 4     | The dev fly/walk camera — `CharacterController` on this map, which is the controller's first real terrain                                                                                                                    | Owed                 |
| 5     | Save and resume between waves (topic 14)                                                                                                                                                                                     | Owed                 |

**What slice 1 proves is narrower than the "Proves" list above**, and it is
three `crcbl-phys` L0 queries this document named towers as the forcing function
for. Not one line of the engine changed on the sample's behalf.

- **A trigger volume is how a creep reaches the exit.**
  `PhysicsWorld::set_trigger` makes the volume non-solid, so every sweep and
  every ray passes through it and only `overlap_sphere` reports it — which is
  exactly the pair a "did anything get in here?" volume wants. `map::world`
  registers it, `creep::has_reached_the_exit` asks it, and
  `map::tests::the_exit_is_a_volume_a_bolt_flies_through_and_an_overlap_reports`
  holds both halves against the map's own geometry.
- **Acquisition is a sphere overlap, and the filter is the interesting half.**
  The same query hands a tower the ground slab and the exit volume, so
  `tower::acquire` answers with a creep or with nothing — and its test asserts
  the query really does return the other things, without which a build with no
  filter at all would pass.
- **CCD against a target that is itself moving.** A bolt covers more ground in
  one tick than a creep is wide, and the creeps have already walked by the time
  the bolts sweep, so `sweep_sphere` over the segment is the only thing that
  sees the hit.
  `tower::tests::a_bolt_hits_a_creep_that_a_test_at_either_end_of_the_tick_would_miss`
  takes both static readings beside it: an overlap at the start of the tick and
  an overlap at the end, and neither finds the creep the sweep hit.

**The engine gap the slice found is a spline type.** Nothing in `crcbl-phys` or
`crcbl-scene` offers one, so `path` measures straight legs between `map::PATH`'s
waypoints and a creep turns a corner in a single tick. That is sample code over
kinematic bodies, exactly as the bullet below predicted; `docs/backlog.md`
carries it rather than the sample working around it.

**What the slice does not have, stated as gaps rather than as decisions.** Rule
11 is **owed, not exempted**: there is no `.crpix` art anywhere, so the tower
and creep icons, the wave banner and the build menu are untextured rectangles
and the built-in font. Rule 8 is **owed, not exempted**: the sample ships
silent, and the "audio grammar in anger" bullet above has nothing behind it yet.
Rule 12 is half met — the three selectors are on the debug panel, the `[HUD]`
line and the summary, but there is no flag to hold a path below what the device
offers. There is no pointer or touch input, which the browser slice needs. And
the debug panel's network module has nothing to report on, which is milestone
3's problem rather than this slice's.

**What it is waiting on, and it is not one thing.**

- **Milestone 1 is under way rather than waiting.** Slice 1 is built; the table
  above is what the rest of it costs. The four ingredients this section used to
  list as present are present and three of them are now exercised by code: the
  per-collider trigger flag, `sweep_sphere` and `overlap_sphere` are what the
  sample runs on, and `CharacterController` is the one still untouched here — it
  is slice 4's, and `apps/puppet`, `apps/breach` and `apps/shard` drive it from
  three different cameras in the meantime.
- **Milestone 2 waits on the editor, which does not exist. The scene directory
  no longer holds it up.** `crcbl_scene::scn`
  ([06-assets-scenes.md](../06-assets-scenes.md)'s task 4) landed 2026-09-07 and
  `apps/breakout` reads its board out of a `.scn/` directory, so a wave that
  saves and reloads has a format to save into. What is missing is `apps/editor`:
  the workspace `Cargo.toml` records its absence as deliberate — it is a later
  phase — and `docs/plan/08-editor.md` is its design. Every "editor-built" and
  "authored in the editor" line in this doc still inherits P12, and slice 1's
  map is a table in `apps/towers/src/map.rs` for exactly that reason.
- **Milestone 3 waits on a wire.** `crcbl-net` ships `InMemoryTransport` and
  nothing else: no UDP transport, no LAN host discovery, no lobby browser. So
  "co-op over real transport" and the 4-player LAN exit criterion have no
  implementation to sit on, and the netgraph's network module has no connection
  to report on in this sample any more than it does in breakout's. The commands
  are already shaped for it, which is the one thing slice 1 could do about it.

## Exit criteria

- 4-player **LAN** co-op session completes 10 waves on a dedicated headless
  server found through the lobby browser — the engine's marquee demo, recorded.
  All clients native: a browser cannot host, cannot discover LAN hosts, and
  cannot reach a LAN server from an HTTPS page (topic 23's LAN correction).
- The **web build ships and is single player**, like every other sample's — same
  game over `InMemoryTransport`, so the wasm target cannot rot.
- Map authored 100% in the editor, zero hand-edited scene text.
- New tower type addable in one sitting by one dev following the sample's own
  docs — extensibility proof.
- It's actually fun for a session with friends. Flagship carries the bar the
  benchmarks don't.
