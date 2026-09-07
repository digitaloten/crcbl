// The knob panel under a demo's canvas: the driver every page with one shares.
//
// A fixture page — alcove's occlusion knobs, sundial's shadow knobs — is a row
// of buttons and sliders that write an engine value and then read it straight
// back. What is genuinely the page's is which element ids it has, what each
// control writes, and what `refresh` puts back on the screen. Everything around
// that is the same on every such page and is here:
//
//   * the DOM casts (`el`, `button`, `slider`),
//   * `press`, which wires a button so that pressing it does not pause the demo,
//   * the drive-then-refresh cycle every control goes through,
//   * `open`, which leaves the panel disabled until there is a loop behind it,
//   * `enumName`, the length-then-address read that gets a `&'static str` out of
//     wasm memory.
//
// Nothing here keeps a copy of a knob. Every control is written *and then read
// back* through the sample's own exports, whose answer is what the engine holds
// after the write — so a slider the engine clamped shows where it landed, and a
// value moved by a key, by the pause panel or by a typed console line is picked
// up the next time anything on the page refreshes.
//
// Symbols stay written out literally at the call sites in `web/demos/*/main.js`:
// `web/tools/check-exports.mjs` scans a shim for `ex.__crcbl_…` to learn which
// exports the JS depends on, and a template literal would hide every one of them
// from it. Passing an export to `enumName` as a value is fine — the call site
// still spells it out, so that scan still finds it. **This file must name none
// of them**: it is in the shared half of the shim, which the check reads for
// every demo, so a symbol written here is one every other artifact is required
// to export.

import { STATUS } from './demo.js';
import { readUtf8 } from './wasm.js';

/**
 * The element `id` names.
 *
 * @param {string} id
 * @returns {HTMLElement}
 */
export const el = (id) =>
  /** @type {HTMLElement} */ (document.getElementById(id));

/**
 * …as a button.
 *
 * @param {string} id
 * @returns {HTMLButtonElement}
 */
export const button = (id) => /** @type {HTMLButtonElement} */ (el(id));

/**
 * …as a slider.
 *
 * @param {string} id
 * @returns {HTMLInputElement}
 */
export const slider = (id) => /** @type {HTMLInputElement} */ (el(id));

/**
 * Wires a button so that pressing it does not take the keyboard off the canvas.
 *
 * **A canvas that loses focus is a demo the engine pauses**, and focus coming
 * back does not resume it — so a button that took the focus the way a button
 * normally does would pause the fixture on every press. `preventDefault` on
 * `mousedown` is what stops the focus moving at all; the `click` still fires,
 * because a click is raised on release over the same element whatever the press
 * did.
 *
 * A slider is deliberately *not* wired this way: cancelling its `mousedown`
 * would cancel the drag with it, and the drag is the whole control. Moving one
 * does pause the fixture, and each page's own markup says how to set it ticking
 * again.
 *
 * @param {HTMLButtonElement} control
 * @param {() => void} act
 */
export function press(control, act) {
  control.addEventListener('mousedown', (event) => event.preventDefault());
  control.addEventListener('click', act);
}

/**
 * A `&'static str` the engine holds, out of one of its own variables.
 *
 * The length and the address are one read — `name` answers with the length and
 * moves nothing when it is passed zero — so a page never spells the set of
 * names itself.
 *
 * @param {WebAssembly.Memory} memory the instance's memory
 * @param {(cycle: number) => number} name the export: its length, and non-zero
 *   moves the engine on to the next one
 * @param {() => number} ptr the export that answers with its address
 * @param {number} cycle non-zero moves on before reading
 * @returns {string}
 */
export function enumName(memory, name, ptr, cycle) {
  const len = name(cycle);
  return readUtf8(memory, ptr(), len);
}

/**
 * Wires a page's knob panel and opens it once the demo is actually running.
 *
 * Every control is wired to run its `act` and then `refresh`, so one function
 * puts the whole panel back where the engine is however the value moved. The
 * controls start disabled in the markup and stay that way until there is a loop
 * behind them: a page that let a visitor move a knob while start-up was still
 * polling would write a value `Options::apply` then overwrites on the first
 * frame, which reads as a control that did nothing. The status export is the
 * same one `demo.js` drives its own loop on, and it answers without advancing
 * anything.
 *
 * @param {object} spec
 * @param {() => number} spec.status the sample's status export
 * @param {() => void} spec.refresh puts every control where the engine now is
 * @param {Array<[HTMLButtonElement, () => void]>} spec.buttons a button and
 *   what pressing it writes
 * @param {Array<[HTMLInputElement, (value: number) => void]>} spec.sliders a
 *   slider and what dragging it writes, given the number it now reads
 */
export function installKnobs({ status, refresh, buttons, sliders }) {
  /** @param {() => void} act */
  const drive = (act) => {
    act();
    refresh();
  };

  for (const [control, act] of buttons) press(control, () => drive(act));
  for (const [control, act] of sliders) {
    control.addEventListener('input', () =>
      drive(() => act(Number(control.value)))
    );
  }

  const controls = [
    ...buttons.map(([control]) => control),
    ...sliders.map(([control]) => control),
  ];

  function open() {
    const now = status();
    if (now === STATUS.RUNNING || now === STATUS.PAUSED) {
      refresh();
      for (const control of controls) control.disabled = false;
      return;
    }
    if (now === STATUS.FAILED || now === STATUS.STOPPED) return;
    requestAnimationFrame(open);
  }
  requestAnimationFrame(open);
}
