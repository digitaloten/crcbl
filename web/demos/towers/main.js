// The towers demo's browser shim.
//
// Everything shared with the other demos — boot order, the log drain, the
// canvas, the status bar — is `web/engine/demo.js`. What is here is this
// sample's ten export names and the one line of hint text under its canvas.
//
// **The names are written out literally.** `web/tools/check-exports.mjs` scans
// this shim for `.__crcbl_…` to learn which exports the JS depends on, and then
// checks the built `.wasm` actually has them; a template literal would hide
// every one of them from it.
//
// `hint` leads with the build keys because they are the only thing on this page
// a visitor has to do: the waves arrive on their own after the build phase, and
// a run that is won or lost plays itself again. `N` brings the next wave
// forward rather than being the only way to see one — see
// `apps/towers/src/wave.rs`.
//
// `savedLabel` is "Nothing" and that is literal: the status bar says "Nothing
// saved." when the demo stops, which is the truth about a field that keeps no
// score between visits. Save and resume are milestone 1's slice 5 —
// `docs/plan/sample/07-towers.md`.
import init from './crcbl_towers.js';
import { bootDemo } from '../../engine/demo.js';

bootDemo({
  init,
  hint: 'LEFT/RIGHT pick a build plot · B builds a tower on it · N sends the next wave now · R restarts the run · ESC opens the panel · F3 shows the stats · F11 fullscreen',
  savedLabel: 'Nothing',
  bind: (ex) => ({
    prepare: () => ex.__crcbl_towers_prepare(),
    boot: () => ex.__crcbl_towers_boot(),
    frame: (/** @type {number} */ now) => ex.__crcbl_towers_frame(now),
    status: () => ex.__crcbl_towers_status(),
    shutdown: () => ex.__crcbl_towers_shutdown(),
    logLevel: (/** @type {number} */ level) =>
      ex.__crcbl_towers_log_level(level),
    logTake: ex.__crcbl_towers_log_take,
    logPtr: ex.__crcbl_towers_log_ptr,
    errorPtr: () => ex.__crcbl_towers_error_ptr(),
    errorLen: () => ex.__crcbl_towers_error_len(),
  }),
});
