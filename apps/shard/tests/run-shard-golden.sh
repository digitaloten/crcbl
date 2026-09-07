#!/usr/bin/env bash
# Draw shard's zone on a named backend, from the fixed camera set, on every
# geometry path this adapter can be held down to, and compare each frame
# against its checked-in golden.
#
#   CRCBL_GPU=vk apps/shard/tests/run-shard-golden.sh [extra nextest args…]
#
# # What this is for
#
# `docs/plan/sample/15-shard.md`'s milestone 1 exit criterion "golden frames per
# `GeometryPath` from a fixed camera set". The suite is
# `apps/shard/tests/golden.rs`, and this sample is the one whose whole subject
# is the *fallback* paths carrying real content — a browser's frame goes
# through `IndirectPerBatch`, `ArrayPages` and `LightingPath::Rasterised` by
# construction — so a frame only ever checked on the tail a desktop adapter
# selects for itself is a frame checked on the path nobody ships to.
#
# # Two checks after the shared ones
#
# A suite that is both feature-gated and `#[ignore]`d has two ways to run
# nothing, which `tools/run-sample-golden.sh` catches, and this one has a third:
# an adapter that cannot be held to a geometry path says so and returns without
# comparing, which is honest and is also indistinguishable from a pass in
# nextest's summary. A run in which *every* frame took that path would otherwise
# be green while nothing was compared at all.
#
# # What every sample's golden harness shares
#
# `tools/run-sample-golden.sh`: why the backend must be named, how a driver and
# an adapter are pinned, the `CRCBL_GPU` / `CRCBL_ADAPTER` / `CRCBL_VK_ICD` /
# `CRCBL_BLESS` those are read from, and the three checks made after the run.

set -euo pipefail

# shellcheck source=tools/run-sample-golden.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)/tools/run-sample-golden.sh"

crcbl_sample_golden shard "$@"

# **Which geometry paths this adapter could actually be held to**, named on the
# way past rather than left in the log: a path that cannot be forced here is a
# gap in what this run is evidence for, and a reader of the CI output should not
# have to go looking for it.
if grep -F 'cannot be drawn on ' "$CRCBL_SAMPLE_GOLDEN_LOG" >&2; then
    echo "shard golden: the bearings above were compared by whichever test asked" >&2
    echo "  for the path the device did open on — see the lines for the reason." >&2
fi

# **A frame was actually compared**, rather than every test reporting that its
# path could not be forced. That report is honest and it passes the summary in
# the shared script while nothing was held to a golden at all.
if ! grep -qF ' and matched — ' "$CRCBL_SAMPLE_GOLDEN_LOG"; then
    echo "shard golden: no frame was compared against a golden on $CRCBL_GPU." >&2
    echo "  Either no path could be forced on this adapter, or every test took" >&2
    echo "  its skip path — both of which pass the summary above while proving" >&2
    echo "  nothing about this backend." >&2
    exit 1
fi

echo "shard golden: the zone drew from every bearing and matched on $CRCBL_GPU"
