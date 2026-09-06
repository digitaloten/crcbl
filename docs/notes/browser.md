# Browser — records

Records kept so they are not re-derived: measurements, investigations, ideas
considered and declined, and lessons. Open work lives in `docs/backlog.md`.

## What the atmosphere shipped without (2026-09-05)

Decision record; the decision is in `docs/backlog.md`.

- **Neither mirror scene is on the browser gate's excuse list, and that is a
  measurement.** `./web/run-render-harness-e2e.sh --expect-fail ssr,ui` was run
  locally on 2026-09-06 — headless Chromium on SwiftShader, which is CI's linux
  leg — and reported `atmosphere_mirror  pass` and `gradient_mirror  pass`, both
  at a max channel delta of 1, in a run where 18 of 20 scenes matched and only
  `ssr` and `ui` were excused. They behave unlike `ssr`, which is excused,
  because every ray in these fixtures _misses_: there is no crossing for two
  rasterisers to land on different taps at, only a smooth environment term. If
  either ever starts failing there, the excuse belongs in
  `.github/workflows/pages.yml`'s `render-harness` matrix beside `ssr` and `ui`,
  not in a wider tolerance.

## sundial's page knobs: what the browser gate presses (2026-09-04)

`/demos/sundial/` is the second page on the site whose controls are HTML rather
than keys — `apps/sundial/src/web.rs` exports one call per knob and
`web/demos/sundial/main.js` binds them. `web/tools/browser-e2e.mjs`'s `sundial`
row presses **every one of them** and reads the effect off the demo's own
heartbeat: the seam button, the seam slider, the filter button, the sun's
stop/start button, the tick slider, the atlas-viewer button and `reset`. It also
holds the tick slider's `max` against the sweep the heartbeat names —
`sun::Sky::row` prints `tick N of SWEEP_TICKS`, so the arc is the engine's own
answer rather than a copy of the constant kept here. Each was watched to fail
with the export behind it made a no-op, and the `reset` sabotage additionally
reddened group D's changed-frame check — which is the evidence that this row is
right to carry no `still`. That is where it differs from alcove's row below,
where four controls are driven by nothing but a person.

## sundial's page: a filter cycle button rather than a select (2026-09-04)

Considered and declined. A `<select>` would need the page to enumerate the set
`crcbl::render::shadow` declares, which means a name-at-index export pair on top
of `__crcbl_sundial_filter` / `_filter_ptr`. The cycle button is alcove's shape
and carries alcove's argument: the set is the engine's, and a page spelling its
members is a copy that goes stale the day a fourth rung lands. Worth revisiting
when the set grows past three, where cycling stops being a reasonable way to
reach a member.

## sundial's sun is adopted a fixed step late (2026-09-04)

Not a bug, and worth knowing before it is reported as one. The filter and the
seam are console cells and move the frame at once, paused or not. The sun's tick
and its run flag live on `crate::app::Sundial`, so `crate::sun`'s `ask_tick`,
`ask_running` and `ask_reset` leave a request that `Clock::advance` adopts on
the next fixed step — and a page whose canvas lost focus is a paused loop that
runs no fixed step. So on a paused page the sun's controls show the request and
the picture catches up when it ticks again; `web/pages/sundial.html` says so.

The channel exists because there is no other route: `crcbl::web::App` keeps the
running `Loop` in a private `Stage` and exposes no accessor for it, so an export
cannot reach the hosted game. The alternative was an engine change
(`App::with_game`, or similar), which was out of this slice's scope. If one
lands for another reason, `crate::sun`'s channel should be deleted in favour of
it.

One consequence nothing clears: a request still outstanding when a run stops is
adopted by the **next** run's first fixed step, because `PAGE` outlives the
loop. For `ask_reset` that is a no-op; for a placed tick it would open the next
run on the sun the page last placed. Neither `crcbl::web_exports!`'s `shutdown`
nor `crate::app` offers a hook to empty it from inside `apps/sundial`.

## Reaching alcove's page knobs costs the pointer (2026-09-04)

**Reaching a control on `/demos/alcove/` at all costs the pointer, and that
surprised us.** The fixture asks for Pointer Lock while it is running,
`web/engine/shell.js` takes it on the first mouse press inside the canvas, and
under a lock every mouse event goes to the canvas — so a click on a page control
never arrives. `Esc` is the way out and the page says so; letting the pointer go
also pauses the fixture, and focus coming back does not resume it. None of that
is a bug — each half is behaviour a check in this tree asserts on purpose — but
together they make a desktop visitor press `Esc` before the knobs answer, where
a finger never does. If the seam ever wants to be reachable while the court is
being flown, the decision to revisit is `Alcove::pointer_mode`, which locks
whenever the run is not paused even though the page opens on the fixed camera.

Nothing else on that page is owed: `web/tools/browser-e2e.mjs`'s `alcove` row
drives every control `web/pages/alcove.html` offers and reads each one off
`Alcove::log_heartbeat`, and `releasePointer` in that file is the release above
made a step of the gate.

### shard runs at 98x in a browser and nothing has profiled it (2026-08-28)

Record; the work this entry still owes is in `docs/backlog.md` under this
heading.

**The harness half of this is shipped and green.** `until()` defaulted to an
unscaled 90-second ceiling while every other budget in
`web/tools/browser-e2e.mjs` was scaled by a measured `slowdown`, and the
measurement itself lived in group E, after the groups that needed it. The
measurement moved to group B, `slowdown`/`budget` moved to module scope, and
`until` now defaults to `pollCeiling()` —
`min(budget(TIMEOUT_MS), POLL_WALL_CAP_MS)`, five minutes. Pages run `4bf0375`
is the confirmation: **all fourteen demo jobs passed and the site deployed**,
where the two runs before it had shard and puppet red on three checks between
them.

The sweep the fix was sized against, off that run's group B lines — the first
time the site's pace has been measured demo by demo:

| slowdown            | what a poll buys | ceiling  |
| ------------------- | ---------------- | -------- |
| 1.0–1.6x (9 demos)  | 90 s             | 90–146 s |
| 9.0x                | 33.4 s           | capped   |
| 11.9x, 20.6x, 26.6x | 25.3–11.3 s      | capped   |
| 28.4x (puppet)      | 10.6 s           | capped   |
| 37.3x               | 8.1 s            | capped   |
| 98.4x (shard)       | 3.1 s            | capped   |

shard passed on 3.05 simulated seconds, so the cap has margin at today's pace
and no check needs re-denominating in beats yet. A demo slower again would hit
the cap first, and the group B line is what says so.

### The browser gate drives the touch keyboard by arithmetic (2026-08-31)

`CONSOLE_BUTTON_CENTRE`, `KEYBOARD_LETTER_ROWS`, `KEYBOARD_HEIGHT_FRACTION`,
`SPACE_BAR_CENTRE` and `RETURN_KEY_CENTRE` in `web/tools/browser-e2e.mjs` are
copies of constants in `crates/crcbl-ui/src/console/keyboard.rs` and
`crates/crcbl/src/engine/console_button.rs`. The same trade `PAUSE_INSET`
already makes, and it fails loudly rather than quietly when either side moves —
the taps land between keys and the echo never appears — but it is a duplication
and worth knowing about before the layout is changed.

The touch console block also runs on `breakout` only. Every demo's console is
the same engine code, so a second copy would only cost the gate taps; if
breakout's group F block is ever removed, the guard in `web/run-browser-e2e.sh`
moves with it.

### DECIDED — Escape cannot pause a demo that holds the pointer lock (2026-08-26)

Decision record; the decision is in `docs/backlog.md`.

The options, none taken yet:

- **Treat losing the lock as the pause**, which is what browser first-person
  games conventionally do — `pointerlockchange` to unlocked pauses the demo. One
  Escape then reads to the visitor as "pause", and alt-tab pauses too, which is
  the behaviour a player expects anyway. Costs a new edge from the shim into the
  engine.
- **Bind a second pause key in the browser** and say so in the hint. Cheapest,
  but it makes the demo's controls differ per target, which is the divergence
  "the same build runs in both" exists to avoid.
- **Leave it**, and change each demo's hint to say Escape twice. Honest, and
  worse for a visitor.

### A browser that declines `unadjustedMovement` gives adjusted deltas (2026-08-26)

`crcbl-shell`'s web backend sets `ShellCaps::RAW_POINTER_MOTION`, and the thing
behind it is `requestPointerLock({ unadjustedMovement: true })` in `takeLock` in
`web/engine/shell.js`. That option is the OS acceleration bypass the capability
names, and where it is unavailable the shim retries the plain
`requestPointerLock()` and the `movementX`/`movementY` the engine reads are the
**OS-adjusted** ones — the same acceleration curve as the desktop cursor, so aim
speed changes with how fast the hand moves.

**Who is affected is decided by the OS, not by the browser**, which is the half
this entry originally got wrong. Chromium rejects the option with
`NotSupportedError` on **Linux and Android whatever its version**, and grants it
on Windows and macOS — the platform, not the release, is the gate. Measured here
on Chromium 151.0.7922.173 against four configurations, all rejecting: a `data:`
page and an `http://localhost` one (`isSecureContext` true), headless and
headed, with and without `--enable-blink-features=PointerLockOptions`. The
request is made from inside a real `pointerdown`, so transient activation is
satisfied. Safari on iOS and Firefox for Android also lack it.

The consequence worth naming: **every Linux desktop visitor, and CI's own Linux
job, take the fallback**, so `ShellCaps::RAW_POINTER_MOTION`'s "unaccelerated"
half is not honoured on the platform this project is developed on. It is
honoured natively on the same machine — X11 reads XI2's `axisvalues_raw` and
Wayland reads `relative-pointer`'s `dx_unaccel`, both deliberately.

Which path a run took is no longer merely asserted: group `AM` in
`web/tools/probe-e2e.mjs` asserts it per platform — Windows and macOS must be
granted the option, Linux must be refused it and reach the lock through the
fallback. The deltas themselves are still not measured; only the path is.

What it costs: a competitive shooter cannot trust aim on those browsers, which
is one of the reasons `docs/plan/sample/11-breach.md` gives for breach being
native-first. What would close it is nothing on our side — it is a browser
feature — so the honest options are to leave the caveat stated, where it is now
(`ShellCaps::RAW_POINTER_MOTION`'s docs and the `web` backend's module docs), or
to add a _third_ capability bit separating "relative motion" from "unaccelerated
relative motion". The bit was not added: nothing in the engine would branch on
it today, and `ShellCaps::has_mouselook` would still be the check a camera runs.

### `apps/breach`'s practice map is gated through a page navigation, not a second run

`web/tools/browser-e2e.mjs` reaches the practice map by navigating the _same_
browser to `?map=practice` in the middle of group C and then navigating back to
the range, so the groups after it judge the demo they always have. Two full page
boots is about seven seconds of the breach gate's runtime.

The alternative considered and declined: a second gate target, so
`CRCBL_WEB_E2E_DEMO=breach-practice` would be its own CI step. It would need a
second page, a second `web/build.sh` DEMOS row and two more `pages.yml` steps
(`tools/check-browser-gate-demos.sh` enforces both), all to run a second copy of
groups A, B, D, E, F, H and I against the same wasm — which is far more CI
minutes than the two navigations cost. Worth revisiting only if breach grows a
third map.

### `apps/breach`'s browser gate leans on one moving plate for liveness

An indoor range with a ceiling has no sun, no sky and nothing else that moves,
so the only thing that changes on a breach canvas with nobody touching it is the
far lane's travelling plate — `map::MOVER_LANE`, driven by `map::plate_x` off
the simulated clock. Two of the browser gate's generic claims rest on it: the
`moving` probe in group C reads the `mover:` field off the `[HUD]` line, and
group D's "the canvas changes between frames" needs the plate to be **in shot**.

Two consequences worth knowing before touching either:

- Freeze or remove the travelling plate and two checks go red, one of them in a
  group that has nothing to do with breach. Verified by sabotage: making
  `plate_x` ignore its `seconds` argument fails
  `the travelling target keeps crossing its lane under its own steam` _and_
  `the canvas changes between frames while the simulation runs`.
- The `range` block in `web/tools/browser-e2e.mjs` therefore puts the view back
  down the range after its own checks, measuring the turn rate off the look
  check rather than carrying a copy of it. Before that existed the block left
  the camera pitched at the ceiling and group D failed on a demo that was
  running perfectly well.

A second moving fixture — a swinging lamp, a fan — would take the weight off one
plate. Not built: it is scenery for a map that has none yet.

The **practice map does not have this problem**: three bots walk their patrols
whatever the player does, and the gate's block for that map reads a bot's own
feet rather than a plate. But the range is what group D judges, because that is
the map the page opens on and the map the block navigates back to.

### The sRGB gate was reading whichever menu the run ended on

**A red Pages run on a docs-only commit, 2026-08-20**, and the interesting part
is how nearly it was misread. Group G reported
`expected rgb(107,173,229) … the dominant colour is rgb(63,105,141) at 32.5%`
under the name "the clear reaches the canvas sRGB-encoded" — the message for the
bug that shipped to users once already.

**The numbers said it was not that.** The observed colour is a **uniform 0.61
multiply** of the expected one (ratios 0.589, 0.607, 0.616); a transfer-function
error is a power curve, and `rgb(107,173,229)` sRGB-decoded is
`rgb(37,107,200)`, nothing like what arrived. A uniform multiply is an overlay.
Confirmed by printing the demo's own HUD line beside each sample: six
consecutive samples read `[HUD] Dead score: 0` — flappy's death screen dims the
whole sky, and by group G the bird had died.

**Why it passed here every time and failed there.** Nothing about the runner:
the state at group G depends on how group F's taps went, so it is a race, and
this desktop happened to land on the winning side five runs out of five. That is
what a race looks like from the machine that never loses it.

**Fixed by establishing the state, not by moving or widening anything.** Group G
now presses the demo's own start key until its own `started` line appears — the
same state group C establishes — and checks that it got there, so the sample is
never taken in an unknown state. It cannot hide a broken encode: a live frame
with no encode shows the linear colour, which is what the row's `unencoded` is
compared against. Red-checked by making flappy's `started` predicate match
nothing: the new check fails with "the demo never reported its started state in
3 presses", and the sRGB check fails _beside_ it, so the misdiagnosis cannot
repeat.

**Two things tried first and rejected, recorded so they are not retried.**
Rebooting the page before sampling is worse — `crcbl.status()` reaches RUNNING
before the first frame is presented, so flappy sampled an all-black canvas, and
breakout's clear is only uncovered once its start menu has been dismissed.
Moving the group before E and F is not enough either: the bird is already dead
by then, which is how the `Dead` line was found.

**One thing worth keeping from the detour:** the eight backdrop samples were
taken back to back, spanning a few milliseconds, so they were eight looks at one
frame rather than eight frames. They are spaced now, and the spacing is scaled
by the measured slowdown like every other budget.

### The one device-loss ordering that does not hold

`gpu-replay.js` now watches `GPUDevice.lost` and files the loss once, first,
with its reason, and both comments in `#requestReadback` and `#loseDevice` point
here for the case it cannot make clean.

**The ordering that does hold.** The specification's "lose the device" resolves
`lost` **before** completing the steps waiting on a loss, so a map rejection
caused by a genuine loss arrives after `#loseDevice` has already filed the
readback with the loss text, and the rejection handler leaves it alone. That is
the path a real device failure takes.

**The one that does not.** `GPUDevice.destroy()` does not follow that route: it
cancels an outstanding map through the **buffer**, and Chromium was watched
rejecting one with "Buffer was unmapped before mapping was resolved" a whole
task _ahead_ of `lost`. `#loseDevice` re-files such an entry so the readback
ends up carrying the loss either way — but the rejection was pushed to the
**error queue** when it landed, and a queued error cannot be taken back. So on
that single path a reader sees the browser's sentence before the loss.

Closing it means either not filing a map rejection until a turn has passed, so a
loss can still claim it — which delays every honest rejection to tidy one — or
making the error queue support retraction, which is a wire-format change to
`Reply::DeviceErrors` for a cosmetic ordering. Neither is obviously worth it,
and the entry that matters (the readback's own failure reason) is already
correct in both orderings. Recorded so the next reader does not mistake the
ordering for an oversight.

### What the three browser gates still keep to themselves

The launch-and-poll loop, the CDP client, `openPage`, `evaluate`, `until`, the
browser registry and the exit hooks are one copy each in
`web/tools/browser-launch.mjs`. What is left is deliberate and worth not
"fixing":

- **`fail` stays per gate.** The prefix differs and so does the meaning of the
  exit code — `render-harness-e2e.mjs` documents 0/1/2 as a contract in its
  usage header. A `makeFail(prefix)` factory would be a helper whose whole body
  is its parameter. The part that _was_ shared knowledge, "kill the browsers
  before you go", is now the exit hook rather than something each copy must
  remember.
- **`render-harness-e2e.mjs`'s own poll stays.** It must hard-fail on its
  deadline rather than answer `null`, must let a throwing `evaluate` through as
  its exit 2, and polls at 250 ms against `until`'s 16. Merging it needs a flag
  per difference, which is the shape that argues against merging.
- **`check` and `group` are still two copies**, shared between `browser-e2e.mjs`
  and `probe-e2e.mjs`. `check` is byte-identical; `group` differs only in the
  gate name it prints. Not moved because they are the checks-and-verdict layer
  rather than the browser layer, and `render-harness-e2e.mjs` has no counterpart
  — so a third caller does not exist and may never.

**One thing found by doing it, and it was a real defect rather than a tidiness
question:** `render-harness-e2e.mjs` leaked its whole browser process tree on
every error it diagnosed. It stopped the browser from a `finally` in `main`, but
`fail` calls `process.exit`, which does not unwind one, and it had no signal
handlers where the other two did. Interrupted mid-run it left **12 chromium
processes and a profile directory**; it leaves none now. Worth keeping because
it is the argument for the sharing: the registry and the hooks live with the
`launch` that registers every browser, so a fourth gate cannot be written
without them.

### DECIDED — the browser has no WebGL2 fallback, and the deletion closed the alternative

Un-linking `crcbl-wgpu` from the wasm dropped a capability: **a browser without
WebGPU has no fallback at all.** The WebGL2 path came from `wgpu`, which stopped
being linkable there and was deleted outright on 2026-08-21, so on such a
browser the engine has no backend to open rather than a slower one.

**Detect-and-message shipped**, which was the recommendation. `demo.js`'s `main`
answers a missing `navigator.gpu` with "This browser has no WebGPU" and the
browsers that have it, and — the case that actually happens — answers a
`requestAdapter()` that resolves to `null` with a sentence aimed at a person,
before the megabytes of wasm load. A blank canvas is now an explanation.

**The fallback itself was the open half, and the deletion settled it by
foreclosure.** Both options are kept so neither is re-argued:

- **Accept it.** WebGPU shipped in Chrome, Edge and Firefox; Safari has it
  from 26. The floor rises on its own, and the engine now refuses gracefully.
  This is what happened.
- **A second artifact.** Build a wgpu/WebGL2 wasm alongside and pick at load
  time — the fallback and the small default both, at the cost of two builds, two
  toolchains (that build needs `wasm-bindgen` back) and a loader that chooses.
  **This one is gone**: with `crcbl-wgpu` deleted there is no WebGL2 path left
  to build, so choosing it now means reviving a deleted backend rather than
  re-enabling a flag.

**Revisit only if someone reports a browser that needs it.** Nothing in the
samples requires WebGL2 and no telemetry says anyone is on such a browser — and
reopening it is now a revival rather than a flag, which is the thing to know
before promising anyone a fallback.

### The debug markers have no browser probe, and probably never can

`begin_debug_label`, `end_debug_label` and `insert_debug_marker` are wired and
covered by the node replayer's stubs, but **nothing drives them against a real
browser**, and this is not a gap waiting on effort. WebGPU gives a page no way
to observe them: the three calls return nothing, change no resource, and are
readable only inside a native capture tool attached to the browser process. A
probe group would therefore encode them and then assert the frame still
submitted — a check that passes identically whether they were replayed or
dropped.

**The reasoning now lives in the code**, in a block comment above
`Replayer#debugScope` in `web/engine/gpu-replay.js`, which is where a reader
asking "why is there no group for these?" arrives. It also names what _is_
checked without a browser: `web/tools/gpu-replay.mjs` replays all three against
a stub device whose encoder, render pass and compute pass objects each record
their own debug calls, so a push landing on the wrong object — the unbalanced
group that costs a real `finish()` — fails there by name. Delete that comment,
and this entry, if a WebGPU extension ever reports recorded markers back.

The `dispatch_indirect` half of this entry **shipped** as probe group AG, which
reads the three workgroup counts back per axis rather than asserting a dispatch
happened.

### The Windows probe gate has no adapter; macOS is proven

Record; the two things still open are in docs/backlog.md under the same heading.
**macOS works, first run, exactly as predicted.** `probe-macos` ran headless on
`macos-15` against a real Apple adapter — Chrome 150.0.7871.187, adapter mode
`hardware` resolved automatically on darwin, **57/57 over groups `G…AA`**. So
headless Chrome really does close the WebGPU canvas readback gap on macOS, and
that is the first proof of `crcbl-webgpu` on Metal-backed Dawn. Take
`continue-on-error` off, having served three green runs.

**Windows has no WebGPU adapter in the default mode.** The job found Chrome
151.0.7922.109 exactly where the registry said, resolved `hardware` on win32,
and died at `requestAdapter() returned no adapter — no GPU to drive`. So a
GPU-less `windows-latest` exposes nothing through D3D, and that mode gates
nothing. The runner reported it correctly — "the driver reported no checks — the
gate is not gating" — rather than passing on zero.

**The remaining route is SwiftShader, now being measured.** It is the one this
file warned against, and the warning is still true: it moves Dawn to SwiftShader
while Chromium's shared-image device stays on D3D11, so a canvas handed between
them reads back as uninitialised memory. But that is a _canvas_ fault, and most
of the probe is not a canvas — G through W and AA drive the command stream
against textures the replayer owns. **Expected: real seam coverage with X, Y and
Z failing.** If that is what comes back, the honest end state is a Windows job
that runs the groups Windows can serve and says which, not one that pretends to
run them all.

`pages.yml` runs the seam probe on `macos-15` (headless, against real Metal) and
`windows-latest` (headless, SwiftShader, with four groups expected to fail).

**Both jobs now gate.** macOS came off `continue-on-error` after three green
runs at 57/57. Windows came off once it was taught its four expected failures,
and passes at 53/57 with them excused — and a listed group that _passes_ fails
the run as stale, so the list cannot rot into a blanket suppression.

**What the first runs answer**, and the line to read for each:

- **macOS.** `browser: /Applications/Google Chrome.app/…` — absent means the
  image assumption is wrong rather than the gate. Then
  `probe e2e: adapter mode "hardware" (auto on darwin)`, then the `groups` line.
  `G…AA` and 57/57 means headless macOS really does close the canvas readback
  gap. A missing `X Y Z` means it does not, and macOS needs a headed session
  too.
- **Windows.** The open question is whether a GPU-less runner exposes any WebGPU
  adapter at all. No group letters means it does not; `X Y Z AA` missing while
  everything else passes means headed did not close the readback gap either, and
  Windows cannot host this gate.

**~~Coverage gap in what landed~~ — both halves are spent, 2026-08-22.** This
said `render-harness-e2e.mjs`'s _launch_ path had never been executed and that
it was "wired into no workflow at all". Neither holds: `pages.yml` runs
`./web/run-render-harness-e2e.sh` and two later steps consume its readbacks, and
the launch path has since been run here repeatedly — which is how the browser
leak it was hiding got found.

### The stream decoder caps a SPIR-V module at 65 536 words

`crcbl-webgpu`'s decoder bounds `ShaderModuleDesc::spirv` by `MAX_ELEMENT_COUNT`
(`1 << 16` words = 256 KiB), the same cap every counted list on this stream
uses. A real SPIR-V module can be larger — a big compute shader clears 256 KiB
easily — so pointing the decoder at one would refuse it as `InvalidLength`.

**It cannot bite the WebGPU path**, and that is why it was left. A browser
consumes only `wgsl`; on a browser build the engine hands `create_shader_module`
a descriptor whose `spirv` is empty, so the field the cap guards is never
populated on the only path this decoder runs. The ceiling is reachable solely by
aiming the Rust decoder at a stream carrying real SPIR-V, which is a test or a
tool, never production.

If a future use does stream SPIR-V through this decoder, the fix is a per-field
cap sized to a shader rather than the shared element count — the two limits
answer different questions and `MAX_ELEMENT_COUNT` was sized for the second.

### Anisotropy: the limit says one, the replayer passes more through

**DECIDED 2026-09-06 —** confirmed as it stands: the limit is reported as 1 and
an ask above it is passed through to `createSampler` for the device to clamp.
Precedent: wgpu's WebGPU backend does the same, because WebGPU exposes no query
for the maximum a device supports, so a reported 1 means "no ceiling this
backend can guarantee" rather than "more than one is refused". Nothing owed.
`halLimitsFor` reports `max_sampler_anisotropy: 1` and withholds
`Features::SAMPLER_ANISOTROPY`, while `webgpuMaxAnisotropyFor` passes an ask
above 1 straight to `createSampler` and lets the device clamp. Both halves are
argued where they are written and neither is a bug, but together they mean a
caller who respects the reported limit never exercises the pass-through, and one
who ignores it gets whatever the device does.

The alternative is refusing everything above 1, which would make the seam's
anisotropic filtering permanently unreachable on WebGPU. That is why it was not
done, and it remains **a decision worth confirming** rather than one that sits
implicit. WebGPU has no query for the maximum a device supports, which is why
the reported limit is 1: it is "no ceiling this backend can guarantee", not
"more than one is refused".

**Corrected 2026-08-24:** this entry said the two halves live "in two files that
do not reference each other". They are in one file, `web/engine/gpu-replay.js`,
and `webgpuMaxAnisotropyFor` already linked to `halLimitsFor`; the reference was
one-way, and `halLimitsFor` now links back. So what is left here is only the
decision, not the implicitness.

## The browser leak check watches occupancy, not just totals (2026-08-24)

Record; the decision it still needs is in docs/backlog.md under the same
heading. Check I in `web/tools/browser-e2e.mjs` ("a steady-state frame gives
back everything it takes") took one `liveObjects()` sample, waited 60 frames,
took another, and failed on any kind that had risen. It failed twice on CI and
neither was a leak: lantern `readbacks 2 -> 5` (commit `fa7fc11`) and quarry
`readbacks 1 -> 2` (commit `68e278c`), neither reproducible on this machine.

**`readbacks` is not a monotone tally — it is the occupancy of a fixed-size
ring.** `CullStatsRing` (`crates/crcbl-render/src/cull_stats.rs`) holds
`FRAMES_IN_FLIGHT + 1` = 3 slots per `ForwardRenderer`, and `take_slot` matches
only `Recording` or `Idle`, never `Polling` — so a slot with a live readback is
never reused and the count is hard-bounded at three per renderer. quarry has one
renderer (ceiling 3, observed 2); lantern has two, `renderer` and `monitor` in
`apps/lantern/src/gpu.rs` (ceiling 6, observed 5). Both failures were strictly
inside the bound. Occupancy rises with the readback round trip measured _in
frames_, which is exactly what a loaded CI runner stretches.

The check now asks whether a kind climbs across **every** one of four windows
rather than one: a ring saturates and stops, a leak does not. Verified both ways
— the acquire-path leak it was written for (a fresh image and view per frame,
never retired) still fails it, at `imageViews 520 -> 580 -> 641 -> 701`.

**Per-kind ceilings were the alternative and were declined**: they would put the
engine's ring depths in a JS test file, to be silently wrong the day one
changed. If a future leak turns out to be slow enough to saturate within three
windows, the answer is more windows, not a table of engine constants.

**It happened a third time on 2026-09-02, and this one survived the three-window
hardening.** `render lantern in a real browser` failed the Pages run for
`d0bc715` with `readbacks 2 -> 4 -> 5 -> 6`. That is the same false positive
again, and the arithmetic says so rather than the resemblance:
`FRAMES_IN_FLIGHT` is 2 and `CullStatsRing::new` takes `frames_in_flight + 1`
slots, so three per renderer; lantern's `gpu.rs` holds two, `renderer` and
`monitor`; the ceiling is six, and the run stopped **at exactly six**. The
increments decelerate — `+2`, `+1`, `+1` — which is a ring filling, and an
unbounded leak does not stop at the bound.

So a loaded runner can now stretch the round trip far enough that occupancy
takes three whole windows to saturate, and three windows was no longer enough to
see it stop. The cheap remedy this entry already named was taken — a fourth
window, so the flat step after saturation is inside the sample — and the comment
above the constants labels it a stopgap.

**Not caused by the autoexec that landed in the same commit**, though the timing
invites the reading: an autoexec run on a page with no `autoexec.cfg` reads the
resident map once and allocates nothing, and `readbacks` counts `CullStatsRing`
slots, which nothing on that path touches. The evidence is weaker than it looks
in one respect worth stating — the three Pages runs between this commit and the
last green lantern were **cancelled by my own pushes**, so there are no
observations in between and "it first failed here" is not evidence it started
here.

## What the deleted WebGPU plan left behind (2026-08-22)

Record; what the deletion left open is in docs/backlog.md under the same
heading.

- **The ordering that makes `crcbl-webgpu`'s `suboptimal: false` true is guarded
  nowhere.** Settled 2026-08-22: the hard-coded `false` is correct, and correct
  by construction of the platform rather than by anything this engine does.
  WebGPU's _Canvas Context sizing_ algorithm re-derives a canvas context's
  texture descriptor from `canvas.width`/`canvas.height` whenever either is set,
  and `configure()`'s own note says its early validation "remains valid until
  the next `configure()` call, **except** for validation of the `size`, which
  changes when the canvas is resized". A canvas context is not a swapchain that
  can outgrow a size, so the API has no out-of-date signal for a backend to
  translate. `crcbl-webgpu` now says this at the literal, and two tests hold the
  two links it can reach: `the_shim_reports_the_canvas_size_it_just_wrote` pins
  `web/engine/shell.js` to sizing the backing store and then reporting those
  same locals, from exactly one place;
  `the_acquired_extent_is_the_one_last_configured` drives the device and proves
  a reconfigure moves what a later acquire reports.

  **All three links are held now.** The third is `Loop::frame_body`'s
  pump-resize-frame ordering in `crcbl`, guarded by
  `a_resize_reaches_the_gpu_before_the_frame_that_follows_it`, which asserts on
  the extent the GPU **held when `frame` ran** rather than the one it ended on —
  both orderings leave the same extent behind, so an end-state assertion would
  have passed either way. Measured when it was written: moving the resize below
  the frame fails that test and **nothing else in `crcbl`'s suite**, which is
  what it was written for.

## `/favicon.ico` is still a 404, deliberately

`web/favicon.svg` is declared by the layout, which is what stops the browsers in
the requirements list (Chrome/Edge 113+, Safari 18+, Firefox) asking for
`/favicon.ico` at all — verified: `curl` against the live site returned 404 for
that path before the change, and the built pages now carry
`<link rel="icon" href="/favicon.svg">`. A browser that ignores the declaration
still gets a 404 and no icon.

Not fixed because an `.ico` is a binary blob and this repo bakes its art from
committed text. `web/build.sh` has no image toolchain and adding one for a 16×16
icon is a worse trade than the miss. `web/tools/browser-e2e.mjs` still filters
`favicon.ico` out of its 404 assertion for the same reason.

## Should the click that refocuses a canvas reach the game at all?

**DECIDED 2026-09-06 —** keep delivering the click. Activation blocking is
declined: browsers and every web game deliver the click that focuses the page,
and for a paused game the current behaviour is the friendlier one — the player
clicked on `RESUME` and got a resume. Nothing owed; what holds the line is
described below. **Behaviour that surprised us, deliberately left alone.** A
canvas has no title bar, so `web/engine/shell.js` gives it the keyboard from its
own `pointerdown` handler — which makes the click that "clicks back into the
window" also a press at a real position inside the game. With the pause menu on
screen and `RESUME` under the cursor, clicking back in resumes. That is each
half behaving correctly and the combination being surprising; it is what put the
browser gate at 23/25 for a slice, because section E clicked the canvas's
_centre_ to restore focus and the menu is centred there.

The alternative is click-to-focus **activation blocking**: the first press after
a focus gain restores focus and is swallowed rather than delivered, which is
what several desktop toolkits do. Not done, and not obviously right — swallowing
a click is its own surprise, and for a paused game the current behaviour is
arguably the friendlier one (the player clicked on `RESUME`; they got a resume).
It needs a decision rather than a patch, and it would have to be decided for
native and web together, since `Loop` cannot tell the two apart.

What holds the line meanwhile:
`a_focusing_click_off_every_button_leaves_the_game_paused` in all four games'
`app.rs` asserts the corner is over no button and the centre is over `RESUME`,
so a menu that grew until it reached the corner fails a fast Rust test rather
than the slow browser one. Four copies of it, plus horde's
`a_focusing_click_off_every_button_leaves_the_title_screen_up`, because the menu
geometry is per-sample even though `FOCUS_CLICK_INSET` — 8 pixels, in
`web/tools/browser-e2e.mjs` — is not. The loop around them is
`crcbl::engine::Loop` now, so this is one of the few things still written out
per sample, and it is per sample for a reason rather than by omission.

## `apps/hud` milestone 1: what was deliberately left out

Record; what the sample still owes is in docs/backlog.md under the same heading.

- **Settled: Chrome 151 broke the browser gate, and it was a device mismatch,
  not a readback quirk.** This entry predicted the failure and it arrived
  exactly as described — GitHub's runner moved from Chrome 150.0.7871.128 to
  151.0.7922.108 between `4eb0d65` (Pages green) and `77fa401` (Pages red),
  group A went red for all five demos at once, and the deploy was **skipped**
  rather than failed, which is the shape that hides a broken publish.

  The cause: a WebGPU canvas is handed between two devices — Dawn renders into
  it and Chromium's compositor reads it back for `toDataURL` — and those must be
  the same Vulkan implementation. `--use-webgpu-adapter=swiftshader` moves
  **only Dawn**; the shared-image device stayed on whatever the machine had, and
  on 151 the hand-off fails. The snapshot was therefore **uninitialised memory
  rather than black** — decoding the raw PNG outside the browser gave 2427
  distinct colours, almost all at alpha 0, which is why it decoded as
  `rgb(0,0,0)`. Chromium said so in its own stderr:
  `ReadPixels: Source shared image is not accessible` and
  `CopyTextureForBrowser from [Invalid Texture]`.

  The fix is `--enable-features=Vulkan --use-vulkan=swiftshader` in
  `browserFlags`, pointing the shared-image device at SwiftShader too. Neither
  flag works alone, and `--use-angle=swiftshader` does **not** substitute — it
  is specifically Chromium's shared-image Vulkan device that has to match
  Dawn's. Nothing about how pixels are read changed: `toDataURL` was never the
  problem, and the control and every render check still read through the same
  path.

  **It was never confined to the control.** With group A bypassed, the real
  breakout demo failed identically at its own canvas size with 16 device errors.
  The control was faithfully representing group D, which is its whole purpose.

  **No browser pin.** The gate passes on 151, and the control is what turned a
  silent regression into a loud one — a pin would have hidden this rather than
  fixed it.

- **An unexplained workaround, found and deliberately not used.** Creating a 2D
  canvas in the page _before_ the WebGPU context and reading it back with
  `toDataURL` also makes the SwiftShader readback work, with the old flags.
  Priming it after the fact does not work, the mechanism is unexplained, and it
  would have to be injected into every demo page. Recorded only in case the flag
  fix stops working.

- **Xvfb + `--hardware` reads transparent black on this machine** and loses the
  WebGPU device mid-run, on Chromium 151 with an RX 7900 XTX under RADV. It is
  harmless because `auto` falls through to SwiftShader — which is also what CI
  does, since the runner has no GPU at all — but a developer who passes
  `--hardware` under Xvfb gets a confusing failure. Not investigated.

## Mobile input: what it decided, and the bugs it found

Record; what is left is in docs/backlog.md under the same heading.

### Decisions taken

- **`Binding::PointerPosition { axis }` feeding an `Axis1`**, normalised to the
  surface at −1…+1 with +X right and +Y up. Not an `Axis2`, which would put a
  _place_ in the same value shape as `Binding::Wasd`'s _direction_ — handed
  `(0.5, 0.0)` a consumer cannot tell "half way right" from "moving right at
  half speed". The pixel→normalised step happens once in the engine loop; the
  surface→world step stays in the game, because the play field is not the
  canvas.
- **An absolute binding replaces the relative ones within one `Axis1`** rather
  than summing: a place plus a rate is neither.
- **The pointer wins on the tick it moves; the keyboard owns every other tick.**
  A resting mouse is not a command, so arrow keys still work on a desktop with a
  cursor over the field, and a lifted finger has not asked for anything. That is
  what `Axis1Action::pointer_moved` exists for — the edge an absolute source has
  and a relative one does not.
- **A pointer that leaves keeps its last position.** A leave carries no
  coordinate so nothing is fed to the map at all, which is why the paddle stays
  put instead of walking to the middle on every tap — a touch pointer is
  destroyed on `pointerup`, so a lift is a leave.
- **Breakout's launch is bound to the pointer too.** A lost life returns to
  `WaitingForLaunch` with no menu on screen, so without it a phone could move
  the paddle and never serve again.
- **The viewport meta is unchanged and zoom is not suppressed.** `layout.html`
  is shared with every prose page, iOS Safari has ignored `user-scalable=no`
  since iOS 10, and `touch-action: none` on the canvas already kills double-tap
  zoom — which is the actual complaint. Suppressing it would be an accessibility
  regression that does nothing on the platform it targets.

### Three touch bugs the survey did not predict

All were real, all are fixed, and all three are invisible to a mouse:

- **`pointercancel` was unhandled.** The OS taking over a gesture leaves the
  button down forever, and a held button raises no _edge_ — so the tap silently
  stops working.
- **Non-primary contacts were forwarded** into a seam with no contact ids, so a
  second finger read as the first one teleporting.
- **A tap that opened and closed inside one pump dropped its release** — which
  on a phone is every tap. The first tap worked and the second did nothing.
  Found by writing the test first and watching it fail.

## The browser gate's staleness guard did not cover Rust (fixed 2026-08-20)

Kept because the _shape_ recurs, not because the fix is pending.
`web/run-browser-e2e.sh` warns when `target/site` is older than its sources, and
that warning is the only thing standing between a reused site and a green run
about code that is not under test — the block says so in capitals.

It scanned `web/engine` and `web/tools` for `.js`/`.mjs` and nothing else. Its
comment excused the gap with "the wasm has `build.sh`'s own staleness handling",
which is true of `build.sh` and irrelevant in the branch where the warning
lives: that branch is precisely the one `build.sh` does not run in, so no
`cargo` invocation ever compares a `.rs` against the artifact.

**It cost a false red-check the day it was found.** A deliberately frozen camera
was re-run through the gate without `--build`; the gate reported
`the dolly keeps running down the face under its own steam — it took 2 values`
and passed 34/34, against a wasm built before the sabotage. With `--build` the
same tree failed 32/34. The guard printed nothing either way.

The find now also walks `apps` and `crates` for `.rs`, `.slang` and
`Cargo.toml`. **The general lesson:** a staleness guard has to cover every input
to the artifact, and an exemption reasoned from another code path is how one
ends up covering the half nobody edits.

## The browser gate's budgets are measured, not fixed (2026-08-20)

Record; the unscaled `TAP_INTERVAL_MS` and the unverified scaled second wait are
in `docs/backlog.md` under this heading.

**Both 3D demos now render in a real browser on CI**, which closes the decision
this entry used to be. What is kept is how it was closed, because the shape
recurs and the reasoning is not recoverable from the diff.

**The decision was never about the demos.** The old blocker — `crcbl-render`'s
draw-argument pass binding fourteen storage buffers against SwiftShader's
ceiling of ten — went when that pass was packed down to eight. What was left was
"nobody has run either on a GitHub runner", and turning the step on twice bought
two precise defects rather than an opinion:

1. **32 of 34.** Both failures were group E's HUD heartbeat,
   `0 HUD line(s) in 4000 ms`. `TICK_WINDOW_MS` was a constant chosen on this
   desktop; the runner advances quarry's simulated second every 27 seconds.
   Worse, "a paused demo runs no ticks at all" **passed for free** on that run —
   no heartbeat could appear in any state — which is the exact failure mode the
   group's first check exists to catch, and it caught it.
2. **33 of 34.** Group E green after the window was derived from the measured
   beat; `PADDLE_SETTLE_MS`'s flat 1500 ms failed instead, one budget along.

**So the fix was not a bigger number, it was a measured one.** The heartbeat's
nominal value is one simulated second, so `slowdown = max(1, beat / 1000)` is
how far behind real time a machine is running the demo, and every budget in the
harness is scaled by it. Clamped at 1, so nothing is ever shorter than the
constant already gave. The control prints the factor.

**Measured on the runner, 2026-08-20**, which is what the whole exercise was
for:

| demo      | beat     | factor | result | step  |
| --------- | -------- | ------ | ------ | ----- |
| breakout  | 993 ms   | 1.0x   | 47/47  | —     |
| flappy    | 1034 ms  | 1.0x   | 43/43  | —     |
| asteroids | 992 ms   | 1.0x   | 38/38  | —     |
| horde     | 1023 ms  | 1.0x   | 47/47  | —     |
| hud       | 1003 ms  | 1.0x   | 37/37  | —     |
| quarry    | 26243 ms | 26.2x  | 37/37  | 5m54s |
| lantern   | 19339 ms | 19.3x  | 37/37  | 5m23s |

The five 2D demos sit at exactly nominal and are untouched by the scaling, which
is the property that makes it safe; only the two heavy frames are behind, and by
the factor their own heartbeat reports.

**Both stale prose claims are fixed**: `web/pages/index.html`'s excusing clause
and `web/pages/lantern.html`'s "This one wants a real GPU" note, which said the
canvas stays black on a software adapter and had been false since the pass was
packed.

## The browser entry point is shared; what the move left behind (2026-08-15)

Record; `crates/crcbl/src/web.rs`'s size is in `docs/backlog.md` under this
heading.

S1B finding 2 is closed: `crcbl::web_exports!` writes the ten
`#[unsafe(no_mangle)]` symbols and the page state, and every sample's `web.rs`
invokes it. It was landed as a move, and these are the things it deliberately
did not fix.

### `asset_source` has no caller in the four samples that define it

The four samples that define it — asteroids, breakout, flappy, horde — export
`pub fn asset_source() -> Option<Rc<FetchSource>>` and none of the four calls
it. `opfs_store` is genuinely used (`crate::best` in three of them,
`crate::high_score` in breakout).

**It is no longer the speculative half, though: `apps/viewer` has a caller.**
`apps/viewer/src/shelf.rs` resolves every browser shelf key through it, which
makes the viewer the pattern any of the four would copy rather than an argument
for deleting the accessor.

Left alone because the task was a move and deleting it is a public-API change to
four sample crates in the same commit as the migration. It is **not** a wasm
export — it has no `#[unsafe(no_mangle)]`, so removing it cannot change what the
shim resolves. Deleting it is a two-line-per-sample change whenever someone
wants it gone.

### The unit test cannot observe `prepare`'s log line

`web::tests::the_generated_exports_drive_the_page` invokes the macro over the
`FakePending` fixture and drives nine of the ten symbols. It **cannot** assert
that `prepare` logs, because `log::set_logger` is process-global and
`args::tests::the_front_end_returns_the_contract_exit_codes` calls
`crcbl::core::log::init_logging` in the same test binary — whichever runs first
wins, and the assertion passed alone and failed in the suite. It was observed
failing both ways before being rewritten to push onto `LOG` directly.

What covers the line instead is `web/tools/smoke.mjs`, which `web/build.sh` runs
per demo against the real artifact and which asserts "the log queue delivers the
line prepare wrote". The exact rendered text was also read out of all five
browser-gate page logs: `<name>: prepared; assets from assets/`, unchanged from
the literal each sample used to carry, because the macro reaches it through
`HostedGame::NAME`.

`boot` is the tenth symbol and is not driven by the unit test at all: it opens a
`Web` shell, which exists only on `wasm32`. The browser gate is its only cover.

### The macro is reachable by two paths

`#[macro_export]` puts it at `crcbl::web_exports!`, and a
`#[doc(inline)] pub use` in the module makes `crcbl::web::web_exports!` work
too. The samples all call it as `crcbl::web_exports!`. Nothing enforces that; a
future sample writing the longer path is not wrong, just inconsistent.
