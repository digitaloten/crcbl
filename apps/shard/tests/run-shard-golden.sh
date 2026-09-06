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
# # Why the backend must be named
#
# Every backend draws this zone identically by construction — that is the point
# of the seam — so a run that fell back to another backend produces a frame that
# passes and proves nothing about the one that was wanted. `crcbl::backend::open`
# would otherwise answer the question for you.
#
# # Pinning a driver and an adapter
#
# `CRCBL_VK_ICD` picks the Vulkan ICD, through the same `crcbl_pin_vk_icd` the
# other harnesses source. `CRCBL_ADAPTER` names a device class — `cpu`,
# `integrated`, `discrete` or `virtual` — and `crcbl::adapter` refuses rather
# than falling back when this machine has none of it. The suite prints the
# adapter it opened, and this script reads it back: a variable that never
# reached the test process and a pin that was honoured look identical from
# outside.
#
# # ENVIRONMENT
#
#   CRCBL_GPU       Which backend draws. Required; there is no default.
#   CRCBL_ADAPTER   Which adapter class inside it. Optional.
#   CRCBL_VK_ICD    Which Vulkan driver, when `CRCBL_GPU=vk`.
#   CRCBL_BLESS     Rewrite the references instead of comparing. Never a pass —
#                   `crcbl_golden::Outcome::into_result` reports a blessed run
#                   as a failure, which is what this script then reports too.
#
# # Two checks after the run, not one
#
# A suite that is both feature-gated and `#[ignore]`d has two ways to run
# nothing, and this one has a third. `--no-tests fail` catches an empty
# selection; parsing nextest's own summary catches a filter that matched
# nothing inside one that was not empty; and the check on the "matched" lines
# catches the case peculiar to this suite — an adapter that cannot be held to a
# geometry path says so and returns without comparing, which is honest and is
# also indistinguishable from a pass in the summary above. A run in which
# *every* frame took that path would otherwise be green while nothing was
# compared at all.

set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${APP_DIR}/../.." && pwd)"

# shellcheck source=crates/crcbl-vk/tests/vulkan-icd.sh
source "${REPO_ROOT}/crates/crcbl-vk/tests/vulkan-icd.sh"
crcbl_pin_vk_icd "shard golden"

# shellcheck source=tools/nextest-summary.sh
source "${REPO_ROOT}/tools/nextest-summary.sh"
# shellcheck source=tools/vk-validation-log.sh
source "${REPO_ROOT}/tools/vk-validation-log.sh"

if [ -z "${CRCBL_GPU:-}" ]; then
    cat >&2 <<'NOBACKEND'
shard golden: CRCBL_GPU is not set, so nothing would pin the backend and a
  fallback would pass. Name one:

    CRCBL_GPU=vk   $0     # Vulkan
    CRCBL_GPU=mtl  $0     # Metal, macOS
    CRCBL_GPU=dx12 $0     # Direct3D 12, Windows
NOBACKEND
    exit 1
fi

# A vk run validates whatever the shell says: the check after the run reads
# what the layer said, and a `CRCBL_VK_VALIDATION=0` left over from profiling
# would hand it a log with no messenger in it — which it rejects, for the
# wrong reason.
if [ "$CRCBL_GPU" = vk ]; then
    export CRCBL_VK_VALIDATION=1
fi

cd "$REPO_ROOT"

# Echoed rather than defaulted: unset means "whatever this machine enumerated
# first", which is a legitimate thing to run deliberately.
echo "shard golden: CRCBL_ADAPTER=${CRCBL_ADAPTER:-<unset>}"

LOG="$(mktemp -t crcbl-shard-golden.XXXXXX.log)"
cleanup() {
    local status=$?
    rm -f "$LOG" "${LOG}.plain"
    exit "$status"
}
trap cleanup EXIT INT TERM

# `--success-output immediate` because the lines this suite exists to record —
# the adapter, the path each frame drew on, the colour count and the two block
# readings, and the golden's own numbers — are printed by a passing test, which
# is exactly the run nextest captures them on.
set +e
cargo nextest run --locked -p shard --features golden-e2e --test golden \
    --run-ignored all --no-tests fail --success-output immediate "$@" 2>&1 | tee "$LOG"
STATUS=${PIPESTATUS[0]}
set -e

if [ "$STATUS" -ne 0 ]; then
    echo "shard golden: the suite failed on $CRCBL_GPU" >&2
    exit "$STATUS"
fi

# CI sets `CARGO_TERM_COLOR: always`, so nextest wraps its counts in escapes and
# a match on digits next to "tests run" sees none — a check that then fires on a
# suite which ran everything and passed.
crcbl_nextest_plain "$LOG" "${LOG}.plain"

# What the validation layer said, which nothing here read until a forward_e2e
# run went green on radv with an `ERROR … vk validation:` line in its log: a
# violation reaches `crcbl_core::log::error!` and the test still passes, because
# the fixture's `finish` checks only the seam's out-of-band channel.
# `tools/vk-validation-log.sh` asks both halves — the layer announced itself,
# and then said nothing — and `CRCBL_VK_VALIDATION_SELF_TEST=1` is how to watch
# this go red.
if [ "$CRCBL_GPU" = vk ] \
    && ! crcbl_validation_saw_nothing "${LOG}.plain" "the shard golden suite"; then
    exit 1
fi

if ! crcbl_nextest_summary "${LOG}.plain" "shard golden" \
    "The golden-e2e feature or the ignore attribute stopped matching the tests."; then
    exit 1
fi

# Which adapter the frames were drawn on, from the suite rather than from the
# variable this script exported.
ADAPTER="$(grep -F 'shard golden: device on adapter ' "${LOG}.plain" | head -1 || true)"
if [ -z "$ADAPTER" ]; then
    echo "shard golden: the suite never named the adapter it drew on." >&2
    echo "  The test must print it and this script must be able to find it, or a" >&2
    echo "  green run claims evidence about a device nobody wrote down." >&2
    exit 1
fi
echo "shard golden: ${ADAPTER#*shard golden: }"

# **Which geometry paths this adapter could actually be held to**, named on the
# way past rather than left in the log: a path that cannot be forced here is a
# gap in what this run is evidence for, and a reader of the CI output should not
# have to go looking for it.
if grep -F 'cannot be drawn on ' "${LOG}.plain" >&2; then
    echo "shard golden: the bearings above were compared by whichever test asked" >&2
    echo "  for the path the device did open on — see the lines for the reason." >&2
fi

# **A frame was actually compared**, rather than every test reporting that its
# path could not be forced. That report is honest and it passes the summary
# above while nothing was held to a golden at all.
if ! grep -qF ' and matched — ' "${LOG}.plain"; then
    echo "shard golden: no frame was compared against a golden on $CRCBL_GPU." >&2
    echo "  Either no path could be forced on this adapter, or every test took" >&2
    echo "  its skip path — both of which pass the summary above while proving" >&2
    echo "  nothing about this backend." >&2
    exit 1
fi

echo "shard golden: the zone drew from every bearing and matched on $CRCBL_GPU"
