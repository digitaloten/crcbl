// alcove in the browser.
//
// The boot sequence and the frame loop are `web/engine/demo.js`, shared with
// every other demo. What is left here is the part that genuinely cannot be
// shared: this sample's `__crcbl_alcove_*` symbols, the two strings the status
// bar shows, and the occlusion knobs — which are one sample's controls and have
// no business in a file every demo runs.
//
// The knobs reach two different kinds of state: every `r_ssao_*` control is a
// console cell this sample owns, and the two view buttons name a
// `crcbl::render::DebugView`, which is the engine's one cell — shared with every
// other sample and with the `debug_view` console command.
// `apps/alcove/src/web.rs` is where that table is.
//
// The symbols are written out literally rather than built from the sample's
// name. `web/tools/check-exports.mjs` scans the shim for `.__crcbl_…` to learn
// which exports the JS depends on, and fails when one is missing from the
// artifact; a template literal would hide every one of them from it.
//
// `hint` names the page's own knobs first, because they are what this fixture
// is for and what a visitor with no keyboard can reach; the free camera comes
// second, and it says to swap camera first because that is the truth — the page
// opens on the fixed pose the goldens are taken from, and `apps/alcove`
// integrates the free camera whether or not it is the one being drawn from.
// `savedLabel` is "Nothing" and that is literal: the status bar says "Nothing
// saved." when the demo stops, which is the truth about an occlusion fixture
// with no score and no save file.

import init from './crcbl_alcove.js';
import { bootDemo } from '../../engine/demo.js';
import {
  button,
  el,
  enumName,
  installKnobs,
  slider,
} from '../../engine/knobs.js';

/**
 * Wires the page's occlusion controls to the sample's own exports.
 *
 * **Called from `bind`, which is the one place this page is handed its own wasm
 * instance.** `bootDemo` has no per-demo hook and should not grow one for a
 * single sample — `web/demos/viewer/main.js` says the same thing about its drop
 * target, for the same reason.
 *
 * The wiring itself — the focus-safe press, the drive-then-refresh cycle, the
 * wait for a running loop before the panel opens — is `web/engine/knobs.js`,
 * shared with the other fixture page that has a panel. What is this sample's is
 * below: its element ids, what each control writes, and `refresh`.
 *
 * @param {Record<string, any>} ex the instance's raw exports
 */
function occlusionKnobs(ex) {
  const memory = /** @type {WebAssembly.Memory} */ (ex.memory);

  const view = button('knob-view');
  const bentView = button('knob-bent-view');
  const technique = button('knob-technique');
  const bent = button('knob-bent');
  const seam = button('knob-seam');
  const reset = button('knob-reset');
  const seamAt = slider('knob-seam-at');
  const radius = slider('knob-radius');
  const intensity = slider('knob-intensity');
  const seamValue = el('knob-seam-value');
  const radiusValue = el('knob-radius-value');
  const intensityValue = el('knob-intensity-value');

  /** Where the seam stands when the page raises it: the middle of the frame. */
  const SEAM_CENTRE = 0.5;

  /**
   * The technique's own name, out of the engine's variable, so this page never
   * spells the set of techniques itself.
   *
   * @param {number} cycle non-zero moves the gather on to the next one
   * @returns {string}
   */
  function techniqueName(cycle) {
    return enumName(
      memory,
      ex.__crcbl_alcove_technique,
      ex.__crcbl_alcove_technique_ptr,
      cycle
    );
  }

  /** Puts every control where the console now is. */
  function refresh() {
    view.textContent = ex.__crcbl_alcove_view(-1) ? 'AO only' : 'shaded';
    // The engine holds one debug view, so this is "is the bent direction the
    // picture" rather than a flag of this control's own — a view some other
    // route put up leaves this reading off, which is the truth about the frame.
    bentView.textContent = ex.__crcbl_alcove_bent_view(0)
      ? 'hide it'
      : 'show it';
    technique.textContent = `${techniqueName(0)} →`;
    bent.textContent = ex.__crcbl_alcove_bent_normals(-1) ? 'on' : 'off';

    const at = ex.__crcbl_alcove_seam(-1);
    seam.textContent = at > 0 ? 'lower it' : 'raise it';
    seamAt.value = String(at);
    seamValue.textContent =
      at > 0 ? `${at.toFixed(2)} of the width` : 'no seam';

    radius.value = String(ex.__crcbl_alcove_radius_dial());
    radiusValue.textContent = `${ex.__crcbl_alcove_radius(-1).toFixed(2)} m`;
    intensity.value = String(ex.__crcbl_alcove_intensity_dial());
    intensityValue.textContent = ex.__crcbl_alcove_intensity(-1).toFixed(2);
  }

  // Dragging a slider pauses the fixture, and here that costs nothing to look
  // at: a court with nothing in it that moves draws the same picture paused,
  // and the knobs go on changing that picture. `web/pages/alcove.html` says
  // how to set it ticking again.
  installKnobs({
    status: () => ex.__crcbl_alcove_status(),
    refresh,
    buttons: [
      [view, () => ex.__crcbl_alcove_view(ex.__crcbl_alcove_view(-1) ? 0 : 1)],
      [bentView, () => ex.__crcbl_alcove_bent_view(1)],
      [technique, () => ex.__crcbl_alcove_technique(1)],
      [
        bent,
        () =>
          ex.__crcbl_alcove_bent_normals(
            ex.__crcbl_alcove_bent_normals(-1) ? 0 : 1
          ),
      ],
      [
        seam,
        () =>
          ex.__crcbl_alcove_seam(
            ex.__crcbl_alcove_seam(-1) > 0 ? 0 : SEAM_CENTRE
          ),
      ],
      [reset, () => ex.__crcbl_alcove_reset()],
    ],
    sliders: [
      [seamAt, (at) => ex.__crcbl_alcove_seam(at)],
      [radius, (at) => ex.__crcbl_alcove_radius(at)],
      [intensity, (at) => ex.__crcbl_alcove_intensity(at)],
    ],
  });
}

bootDemo({
  init,
  hint: 'the knobs under the canvas drive the occlusion · ESC opens the panel — CAMERA swaps to the free one · then WASD, Space/Shift and the arrows fly it · F3 shows the panel · F11 fullscreen',
  savedLabel: 'Nothing',
  bind: (ex) => {
    occlusionKnobs(ex);
    return {
      prepare: () => ex.__crcbl_alcove_prepare(),
      boot: () => ex.__crcbl_alcove_boot(),
      frame: (/** @type {number} */ now) => ex.__crcbl_alcove_frame(now),
      status: () => ex.__crcbl_alcove_status(),
      shutdown: () => ex.__crcbl_alcove_shutdown(),
      logLevel: (/** @type {number} */ level) =>
        ex.__crcbl_alcove_log_level(level),
      logTake: ex.__crcbl_alcove_log_take,
      logPtr: ex.__crcbl_alcove_log_ptr,
      errorPtr: () => ex.__crcbl_alcove_error_ptr(),
      errorLen: () => ex.__crcbl_alcove_error_len(),
    };
  },
});
