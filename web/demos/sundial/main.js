// sundial in the browser.
//
// The boot sequence and the frame loop are `web/engine/demo.js`, shared with
// every other demo. What is left here is the part that genuinely cannot be
// shared: this sample's `__crcbl_sundial_*` symbols, the two strings the status
// bar shows, and the shadow knobs — which are one sample's controls and have no
// business in a file every demo runs.
//
// The knobs reach three different kinds of state and `apps/sundial/src/web.rs`
// is where that table is: the filter, the seam and the two bias counts are
// `r_shadow_*` console cells, the sun is the fixture's own clock, and the atlas
// viewer and the cascade tint are the engine's `r_debug_view` — one cell for
// both of them, so each of those two buttons takes the other's picture down.
//
// The symbols are written out literally rather than built from the sample's
// name. `web/tools/check-exports.mjs` scans the shim for `.__crcbl_…` to learn
// which exports the JS depends on, and fails when one is missing from the
// artifact; a template literal would hide every one of them from it.
//
// `hint` names the page's own knobs first, because they are what this fixture
// is for and what a visitor with no keyboard can reach; the free camera comes
// second, and it says to swap camera first because that is the truth — the page
// opens on the fixed pose the goldens are taken from, and `apps/sundial`
// integrates the free camera whether or not it is the one being drawn from.
// `savedLabel` is "Nothing" and that is literal: the status bar says "Nothing
// saved." when the demo stops, which is the truth about a shadow fixture with
// no score and no save file.

import init from './crcbl_sundial.js';
import { bootDemo } from '../../engine/demo.js';
import {
  button,
  el,
  enumName,
  installKnobs,
  slider,
} from '../../engine/knobs.js';

/**
 * Wires the page's shadow controls to the sample's own exports.
 *
 * **Called from `bind`, which is the one place this page is handed its own wasm
 * instance.** `bootDemo` has no per-demo hook and should not grow one for a
 * single sample — `web/demos/alcove/main.js` and `web/demos/viewer/main.js` say
 * the same thing about their own, for the same reason.
 *
 * The wiring itself — the focus-safe press, the drive-then-refresh cycle, the
 * wait for a running loop before the panel opens — is `web/engine/knobs.js`,
 * shared with the other fixture page that has a panel. What is this sample's is
 * below: its element ids, what each control writes, and `refresh`.
 *
 * **The sun's two controls are the ones that are not instant.** The filter and
 * the seam are console cells and move at once; a tick and a run flag live on the
 * fixture's own clock and are adopted on its next fixed step, so what the sun's
 * exports answer with is the request — which is where the clock is about to be,
 * and what a slider has to show while it is being dragged.
 *
 * @param {Record<string, any>} ex the instance's raw exports
 */
function shadowKnobs(ex) {
  const memory = /** @type {WebAssembly.Memory} */ (ex.memory);

  const filter = button('knob-filter');
  const seam = button('knob-seam');
  const atlas = button('knob-atlas');
  const cascades = button('knob-cascades');
  const sun = button('knob-sun');
  const reset = button('knob-reset');
  const seamAt = slider('knob-seam-at');
  const bias = slider('knob-bias');
  const offset = slider('knob-normal-offset');
  const sunTick = slider('knob-sun-tick');
  const seamValue = el('knob-seam-value');
  const biasValue = el('knob-bias-value');
  const offsetValue = el('knob-normal-offset-value');
  const sunValue = el('knob-sun-value');

  // The arc the tick slider spans, off the engine's own `sun::SWEEP_TICKS`
  // rather than a number written into the markup — `web/pages/sundial.html`
  // gives the slider no `max` at all, and this is what sets it. A sweep read
  // once: it is a constant of the fixture, not a knob.
  const sweep = ex.__crcbl_sundial_sun_sweep();
  sunTick.max = String(sweep - 1);

  // The same argument for the two bias counts, and here it is one track per
  // count: `r_shadow_bias` and `r_shadow_normal_offset` are declared with
  // ceilings of their own, so one number read for both would give one of the two
  // sliders a track the engine never reaches the end of. The floor is `min="0"`
  // in the markup and stays there — a drag below what the engine accepts is
  // clamped by the call itself and `refresh` puts the thumb where it landed.
  bias.max = String(ex.__crcbl_sundial_bias_ceiling());
  offset.max = String(ex.__crcbl_sundial_normal_offset_ceiling());

  /**
   * The filter's own name, out of the engine's variable, so this page never
   * spells the set of filters itself.
   *
   * @param {number} cycle non-zero moves the shadow on to the next filter
   * @returns {string}
   */
  function filterName(cycle) {
    return enumName(
      memory,
      ex.__crcbl_sundial_filter,
      ex.__crcbl_sundial_filter_ptr,
      cycle
    );
  }

  /** Puts every control where the engine now is. */
  function refresh() {
    filter.textContent = `${filterName(0)} →`;

    const at = ex.__crcbl_sundial_seam_at(-1);
    seam.textContent = at > 0 ? 'lower it' : 'raise it';
    seamAt.value = String(at);
    seamValue.textContent =
      at > 0 ? `${at.toFixed(2)} of the width` : 'no seam';

    // The engine holds one debug view, so each of these is "is *this* the
    // picture" rather than a flag of its own control — a view some other route
    // put up leaves both readings off, and putting one of the two up takes the
    // other down. That is the truth about the frame rather than about the page.
    atlas.textContent = ex.__crcbl_sundial_atlas_view(0)
      ? 'hide it'
      : 'show it';
    cascades.textContent = ex.__crcbl_sundial_cascades(0)
      ? 'hide it'
      : 'show it';

    // Both counts are in texels of whichever cascade the fragment landed in,
    // and the unit is printed for `Knobs::bias_row`'s reason: a bare `1.50`
    // beside a seam printed as a fraction of the width is two numbers a reader
    // has no reason to read differently.
    const biasNow = ex.__crcbl_sundial_bias(-1);
    bias.value = String(biasNow);
    biasValue.textContent = `${biasNow.toFixed(2)} texels`;
    const offsetNow = ex.__crcbl_sundial_normal_offset(-1);
    offset.value = String(offsetNow);
    offsetValue.textContent = `${offsetNow.toFixed(2)} texels`;

    const running = ex.__crcbl_sundial_sun_running(-1);
    sun.textContent = running ? 'stop it' : 'start it';
    const tick = ex.__crcbl_sundial_sun_tick(-1);
    // The slider spans one sweep and the clock counts past it — `Sky::at` takes
    // the remainder, so this is the same pose the frame is drawn at rather than
    // a second opinion about where the sun is.
    sunTick.value = String(tick % sweep);
    sunValue.textContent = `tick ${tick} of ${sweep}`;
  }

  // A slider here costs something the court in alcove did not: a drag pauses
  // the fixture, a paused loop runs no fixed step, and a sun request is adopted
  // on the next one. `web/pages/sundial.html` says how to set it ticking again,
  // and the control's own label is the request either way.
  installKnobs({
    status: () => ex.__crcbl_sundial_status(),
    refresh,
    buttons: [
      [filter, () => ex.__crcbl_sundial_filter(1)],
      [seam, () => ex.__crcbl_sundial_seam(1)],
      [atlas, () => ex.__crcbl_sundial_atlas_view(1)],
      [cascades, () => ex.__crcbl_sundial_cascades(1)],
      [
        sun,
        () =>
          ex.__crcbl_sundial_sun_running(
            ex.__crcbl_sundial_sun_running(-1) ? 0 : 1
          ),
      ],
      [reset, () => ex.__crcbl_sundial_reset()],
    ],
    sliders: [
      [seamAt, (at) => ex.__crcbl_sundial_seam_at(at)],
      [bias, (texels) => ex.__crcbl_sundial_bias(texels)],
      [offset, (texels) => ex.__crcbl_sundial_normal_offset(texels)],
      [sunTick, (tick) => ex.__crcbl_sundial_sun_tick(tick)],
    ],
  });
}

bootDemo({
  init,
  hint: 'the knobs under the canvas drive the filter, the seam, the two bias counts, the shadow atlas, the cascade tint and the sun · ESC opens the panel — CAMERA swaps to the free one · then WASD, Space/Shift and the arrows fly it · F3 shows the panel · F11 fullscreen',
  savedLabel: 'Nothing',
  bind: (ex) => {
    shadowKnobs(ex);
    return {
      prepare: () => ex.__crcbl_sundial_prepare(),
      boot: () => ex.__crcbl_sundial_boot(),
      frame: (/** @type {number} */ now) => ex.__crcbl_sundial_frame(now),
      status: () => ex.__crcbl_sundial_status(),
      shutdown: () => ex.__crcbl_sundial_shutdown(),
      logLevel: (/** @type {number} */ level) =>
        ex.__crcbl_sundial_log_level(level),
      logTake: ex.__crcbl_sundial_log_take,
      logPtr: ex.__crcbl_sundial_log_ptr,
      errorPtr: () => ex.__crcbl_sundial_error_ptr(),
      errorLen: () => ex.__crcbl_sundial_error_len(),
    };
  },
});
