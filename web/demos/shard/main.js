// shard in the browser.
//
// The boot sequence and the frame loop are `web/engine/demo.js`, shared with
// every other demo. What is left here is the part that genuinely cannot be
// shared: this sample's `__crcbl_shard_*` symbols, the two strings the status
// bar shows, and the one reading `docs/plan/sample/15-shard.md` asks of this
// sample and of no other — the peak wasm heap, argued for inside `bind`.
//
// The symbols are written out literally rather than built from the sample's
// name. `web/tools/check-exports.mjs` scans the shim for `.__crcbl_…` to learn
// which exports the JS depends on, and fails when one is missing from the
// artifact; a template literal would hide every one of them from it.
//
// There is no extra symbol here beyond the ten every demo has. breach has an
// eleventh because it ships two maps for `?map=` to choose between; shard's
// milestone 1 is one zone, so a query string has nothing to pick and the shim
// has nothing to validate.
//
// `hint` leads with the walk keys because the page opens standing in the zone
// with the torches already lit, so the first useful thing is to move. The blow
// comes next, because the zone has three things in it that will not wait
// forever. L is listed after them and named for what it does rather than for the
// key, because dousing the torches is the one control that changes the *picture*
// rather than the position — it is what the browser gate presses to prove the
// lighting is being computed rather than declared. `savedLabel` is "Character":
// the status bar says "Character saved." when the demo stops, and what it names
// is `apps/shard/src/save.rs`'s payload — where the character was standing, what
// they had left, how many times they were put down, and which foes are felled —
// written into the Origin Private File System through the shared shim's OPFS
// backend.

import init from './crcbl_shard.js';
import { bootDemo, STATUS } from '../../engine/demo.js';

/** Bytes in a mebibyte, the unit an address-space budget is stated in. */
const MIB = 1024 * 1024;

bootDemo({
  init,
  hint: 'W/A/S/D walk, relative to where the camera is looking · Q/E swing it a quarter turn · SPACE strikes everything in reach · L douses the torches and lights them again · ESC opens the panel · F3 shows the stats · F11 fullscreen',
  savedLabel: 'Character',
  bind: (ex) => {
    // **THE PEAK WASM HEAP, PRINTED BY THE PAGE ITSELF**, because
    // `docs/plan/sample/15-shard.md`'s milestone 1 asks how close a browser
    // build of real 3D content comes to the wasm32 address-space ceiling, and
    // this is the sample it asks it of. A `WebAssembly.Memory` only ever grows,
    // so `memory.buffer.byteLength` read at any moment *is* the high-water mark
    // to that moment and the last line a run prints is the run's peak — no
    // sampling cadence to get wrong, and nothing to accumulate.
    //
    // Printed on growth rather than on every frame for the same reason: a
    // figure that cannot fall says nothing new when it has not moved, and a
    // line a frame would bury the handful that matter under thousands. The one
    // unconditional reading is the last frame's, so a log always states the
    // figure at a known point instead of leaving a reader to trust that the
    // final growth line was really the last one.
    //
    // **Here rather than in `web/engine/demo.js`** because the criterion is
    // this sample's: shard is the only demo on the site whose content could
    // plausibly approach the ceiling, and a reading every demo prints is a
    // reading nobody asked for on every other page. `web/tools/browser-e2e.mjs`
    // reads these lines back — its `wasmHeap` row is what opts a demo in — and
    // CI's `web-e2e-shard` artifact is the page log, which is how the figure
    // survives a run nobody watched.
    const memory = /** @type {WebAssembly.Memory} */ (ex.memory);
    let peak = 0;
    let ended = false;
    /** @param {string} at what prompted the reading */
    const reportHeap = (at) => {
      peak = memory.buffer.byteLength;
      const mib = (peak / MIB).toFixed(1);
      console.log(`[MEM] wasm heap: ${peak} bytes (${mib} MiB) at ${at}`);
    };
    return {
      prepare: () => ex.__crcbl_shard_prepare(),
      boot: () => ex.__crcbl_shard_boot(),
      frame: (/** @type {number} */ now) => {
        const status = ex.__crcbl_shard_frame(now);
        if (memory.buffer.byteLength > peak) reportHeap('a frame');
        if (!ended && (status === STATUS.STOPPED || status === STATUS.FAILED)) {
          ended = true;
          reportHeap('the last frame');
        }
        return status;
      },
      status: () => ex.__crcbl_shard_status(),
      shutdown: () => ex.__crcbl_shard_shutdown(),
      logLevel: (/** @type {number} */ level) =>
        ex.__crcbl_shard_log_level(level),
      logTake: ex.__crcbl_shard_log_take,
      logPtr: ex.__crcbl_shard_log_ptr,
      errorPtr: () => ex.__crcbl_shard_error_ptr(),
      errorLen: () => ex.__crcbl_shard_error_len(),
    };
  },
});
