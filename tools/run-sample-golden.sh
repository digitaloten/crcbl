#!/usr/bin/env bash
# Running a sample's golden suite, for the nine harnesses that were one script.
#
# **Sourced, never run.** It defines one function in the caller's shell, which
# is what each of `apps/*/tests/run-*-golden.sh` did when it carried this
# inline:
#
#   source "${REPO_ROOT}/tools/run-sample-golden.sh"
#   crcbl_sample_golden asteroids "$@"
#   echo "asteroids golden: the field drew, made its claims and matched on $CRCBL_GPU"
#
# It lives at the top level for the reason `tools/nextest-summary.sh` gives:
# the callers span nine app crates and none of them owns the others.
#
# # Why this is a function and not the forty lines it replaces
#
# Because it already was those forty lines, nine times, and they had drifted —
# which is the only way the wrong comment below survived in five of them. The
# nine were byte for byte identical once the sample's name was normalised,
# except for two things:
#
# * The four newest — `alcove`, `lantern`, `shard`, `sundial` — echoed
#   `CRCBL_ADAPTER` on the way past and grepped the suite's log for
#   `device on adapter `. The five older ones echoed nothing and grepped
#   `device on `.
# * The five older ones said, in their own headers, "There is no `CRCBL_ADAPTER`
#   here". **That was false.** A golden run is a `--screenshot` run, and
#   `crates/crcbl/src/screenshot.rs`'s `start_device` is a caller of
#   `crcbl::adapter::pin()` — so an inherited `CRCBL_ADAPTER` steers those runs
#   exactly as it steers the other four, and they neither said so nor printed
#   what they had been handed.
#
# So every run echoes the variable now, and the grep is the one spelling that
# matches both of the log lines the suites emit: five print
# `<sample> golden: device on <adapter>` and four print
# `<sample> golden: device on adapter <id> …`.
#
# # Why the backend must be named
#
# Every backend draws a sample's frame identically by construction — that is
# the point of the seam — so a run that fell back to another backend produces a
# frame that passes and proves nothing about the one that was wanted.
# `crcbl::backend::open` would otherwise answer the question for you, so the
# suites themselves refuse to run without `CRCBL_GPU` and this refuses to start
# without it.
#
# # Pinning a driver and an adapter
#
# `CRCBL_VK_ICD` picks the Vulkan ICD, through the same `crcbl_pin_vk_icd` the
# other harnesses source. `CRCBL_ADAPTER` names a device class — `cpu`,
# `integrated`, `discrete` or `virtual` — and `crcbl::adapter` refuses rather
# than falling back when this machine has none of it. The suite prints the
# adapter it opened and this reads it back: a variable that never reached the
# test process and a pin that was honoured look identical from outside.
#
# # ENVIRONMENT
#
#   CRCBL_GPU       Which backend draws. Required; there is no default.
#   CRCBL_ADAPTER   Which adapter class inside it. Optional.
#   CRCBL_VK_ICD    Which Vulkan driver, when `CRCBL_GPU=vk`.
#   CRCBL_BLESS     Rewrite the references instead of comparing. Never a pass —
#                   `crcbl_golden::Outcome::into_result` reports a blessed run
#                   as a failure, which is what the caller then reports too.
#
# # The zero-tests check is the point
#
# A suite that is both feature-gated and `#[ignore]`d has two ways to run
# nothing. `--no-tests fail` catches an empty selection; parsing nextest's own
# summary catches a filter that matched nothing inside one that was not empty.

CRCBL_SAMPLE_GOLDEN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# shellcheck source=crates/crcbl-vk/tests/vulkan-icd.sh
source "${CRCBL_SAMPLE_GOLDEN_ROOT}/crates/crcbl-vk/tests/vulkan-icd.sh"
# shellcheck source=tools/nextest-summary.sh
source "${CRCBL_SAMPLE_GOLDEN_ROOT}/tools/nextest-summary.sh"
# shellcheck source=tools/vk-validation-log.sh
source "${CRCBL_SAMPLE_GOLDEN_ROOT}/tools/vk-validation-log.sh"

# Remove the run's logs, keeping whatever status the shell was leaving on.
crcbl_sample_golden_cleanup() {
    local status=$?
    rm -f "$CRCBL_SAMPLE_GOLDEN_RAW" "$CRCBL_SAMPLE_GOLDEN_LOG"
    exit "$status"
}

# `crcbl_sample_golden <package> [extra nextest args…]`
#
# Runs `<package>`'s `golden` suite and makes every check the nine harnesses
# made: the backend was named, the validation layer was loaded and said
# nothing, nextest ran a complete suite of at least one test, and the suite
# printed which adapter it drew on.
#
# The label every message carries is `<package> golden`, which is what all nine
# already used and what their goldens' own `eprintln!`s are prefixed with.
#
# **The success line stays with the caller.** It is the one sentence in these
# scripts that is genuinely the sample's — "the field drew", "the court drew and
# made its occlusion claims" — and `apps/shard` has two further checks of its
# own to make between the last shared one and that line.
#
# It exits rather than returning on a failure: the caller has nothing to print
# on the way out, which is where this parts company with
# `crcbl_nextest_summary`. What it leaves behind for a caller that does have
# more to ask is `CRCBL_SAMPLE_GOLDEN_LOG`, the colour-stripped copy of the
# run's output, valid until the trap fires on the way out.
crcbl_sample_golden() {
    local package="$1"
    shift
    local label="${package} golden"

    crcbl_pin_vk_icd "$label"

    if [ -z "${CRCBL_GPU:-}" ]; then
        cat >&2 <<NOBACKEND
${label}: CRCBL_GPU is not set, so nothing would pin the backend and a
  fallback would pass. Name one:

    CRCBL_GPU=vk   \$0     # Vulkan
    CRCBL_GPU=mtl  \$0     # Metal, macOS
    CRCBL_GPU=dx12 \$0     # Direct3D 12, Windows
NOBACKEND
        exit 1
    fi

    # A vk run validates whatever the shell says: the check after the run reads
    # what the layer said, and a `CRCBL_VK_VALIDATION=0` left over from
    # profiling would hand it a log with no messenger in it — which it rejects,
    # for the wrong reason.
    if [ "$CRCBL_GPU" = vk ]; then
        export CRCBL_VK_VALIDATION=1
    fi

    cd "$CRCBL_SAMPLE_GOLDEN_ROOT"

    # Echoed rather than defaulted: unset means "whatever this machine
    # enumerated first", which is a legitimate thing to run deliberately.
    echo "${label}: CRCBL_ADAPTER=${CRCBL_ADAPTER:-<unset>}"

    # Globals rather than locals, because the trap below runs in the shell that
    # fires it — long after this function has returned and its locals are gone.
    CRCBL_SAMPLE_GOLDEN_RAW="$(mktemp -t "crcbl-${package}-golden.XXXXXX.log")"
    CRCBL_SAMPLE_GOLDEN_LOG="${CRCBL_SAMPLE_GOLDEN_RAW}.plain"
    trap crcbl_sample_golden_cleanup EXIT INT TERM
    local log="$CRCBL_SAMPLE_GOLDEN_RAW"

    # `--success-output immediate` because the lines these suites exist to
    # record — the adapter, the ratios they measured and the goldens' own
    # numbers — are printed by a passing test, which is exactly the run nextest
    # captures them on.
    local status
    set +e
    cargo nextest run --locked -p "$package" --features golden-e2e --test golden \
        --run-ignored all --no-tests fail --success-output immediate "$@" 2>&1 | tee "$log"
    status=${PIPESTATUS[0]}
    set -e

    if [ "$status" -ne 0 ]; then
        echo "${label}: the suite failed on $CRCBL_GPU" >&2
        exit "$status"
    fi

    # CI sets `CARGO_TERM_COLOR: always`, so nextest wraps its counts in escapes
    # and a match on digits next to "tests run" sees none — a check that then
    # fires on a suite which ran everything and passed.
    crcbl_nextest_plain "$log" "$CRCBL_SAMPLE_GOLDEN_LOG"

    # What the validation layer said, which nothing here read until a
    # forward_e2e run went green on radv with an `ERROR … vk validation:` line
    # in its log: a violation reaches `crcbl_core::log::error!` and the test
    # still passes, because the fixture's `finish` checks only the seam's
    # out-of-band channel. `tools/vk-validation-log.sh` asks both halves — the
    # layer announced itself, and then said nothing — and
    # `CRCBL_VK_VALIDATION_SELF_TEST=1` is how to watch this go red.
    if [ "$CRCBL_GPU" = vk ] \
        && ! crcbl_validation_saw_nothing "$CRCBL_SAMPLE_GOLDEN_LOG" "the ${label} suite"; then
        exit 1
    fi

    if ! crcbl_nextest_summary "$CRCBL_SAMPLE_GOLDEN_LOG" "$label" \
        "The golden-e2e feature or the ignore attribute stopped matching the tests."; then
        exit 1
    fi

    # Which adapter the frames were drawn on, from the suite's own log rather
    # than from the variable this exported. `device on ` and not
    # `device on adapter `, because both spellings are in the tree and either is
    # the answer to the question.
    local adapter
    adapter="$(grep -F "${label}: device on " "$CRCBL_SAMPLE_GOLDEN_LOG" | head -1 || true)"
    if [ -z "$adapter" ]; then
        echo "${label}: the suite never named the adapter it drew on." >&2
        echo "  The test must print it and this script must be able to find it, or a" >&2
        echo "  green run claims evidence about a device nobody wrote down." >&2
        exit 1
    fi
    echo "${label}: ${adapter#*"${label}": }"
}
