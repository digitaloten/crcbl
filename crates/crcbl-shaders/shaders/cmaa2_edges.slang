// crcbl-targets: spirv, wgsl, msl, dxil

// `docs/plan/49-antialiasing.md`'s antialiasing ladder, second rung, first of
// the five passes CMAA2 records: the control clear and the edge detect.
//
// # What is transcribed here, and what is this tree's own
//
// The algorithm is Intel's **Conservative Morphological Anti-Aliasing 2**
// (Filip Strugar, 2018). Its four stages are named in
// `docs/plan/49-antialiasing.md` and reproduced across the three `cmaa2_*`
// sources in this directory:
//
//  1. an **edge detect** — a luma delta per pixel boundary against a threshold,
//     with a **local-contrast adaptation** that drops the weaker half of a pair
//     of neighbouring boundaries — which also appends every edge pixel to a
//     **candidate list** (this file);
//  2. a **candidate classification** — the 2×2 `Z` and `U` shapes and the
//     bounded **long-edge search** along the run they belong to — which appends
//     a **blend item** per covered pixel (`cmaa2_shapes.slang`);
//  3. the **deferred colour apply**, which accumulates those items per pixel
//     and resolves them into the target (`cmaa2_shapes.slang`'s
//     `accumulateMain` and `cmaa2_apply.slang`).
//
// **What is not transcribed is the reference's tuned constants.** This engine
// has no network access to the reference source, and a constant recalled from
// memory is a number nothing here could check. So the shape *rules* below are
// analytic — the coverage a straight line reconstructed across a run leaves
// over each of its pixels, which is the morphological family's own derivation
// and is written out at the site — and the two free numbers, `EDGE_THRESHOLD`
// and `LOCAL_CONTRAST_ADAPTATION_FACTOR`, are the ones this tree already
// measured for SMAA on the same fixture. `crates/crcbl/tests/mesh_e2e/cmaa2.rs`
// is what decides whether they hold, and `docs/backlog.md` records that they
// are this transcription's and not the reference's.
//
// # Compute, not a fullscreen triangle
//
// The cost of this tier is meant to be proportional to the frame's *edges*
// rather than to its pixels, which is the whole reason it displaces SMAA — see
// that section. Only the edge detect and the final resolve run per pixel; the
// classification and the accumulate run per candidate and per blend item, so a
// frame with few edges does little work in the two passes between.
//
// # The luma is corrected, because the image reaching here is linear
//
// `tonemap.slang` writes linear values into a target the hardware encodes, so a
// fetch of that target decodes straight back to linear — and an edge threshold
// wants display space, where equal steps look equally different. `luma_of`
// corrects it with one `sqrt`, exactly as `fxaa.slang` and the retired
// `smaa_edges.slang` did; `fxaa.slang`'s header carries the argument at length.
//
// `docs/plan/48-post-processing.md`'s "Pass fusion" would hand this pass the
// luma in the tonemap's alpha channel and save the fetches. That rung has not
// landed — `tonemap.slang` writes an opaque alpha — so this pass computes its
// own, and `docs/backlog.md` carries the fusion.
//
// # No `#include`
//
// `bloom_composite.slang` states the rule: the manifest hashes one source per
// artifact, so an include would be a file whose edits nothing downstream
// notices. The three `cmaa2_*` sources therefore each carry their own copy of
// the parameter block, and the two that share a bind-group layout declare the
// same seven resources in the same order.

/// Everything the CMAA2 passes take from the CPU.
///
/// `crcbl_shaders::cmaa2::Cmaa2Params` is the Rust mirror and
/// `crcbl_render::cmaa2` is what writes the buffer, once per frame. One block
/// for every pass, on the retired SMAA tier's terms: they run at one extent and
/// a second block would be a second thing to keep in step.
struct Cmaa2Params
{
    /// Width of the image being filtered, in texels. Every buffer this tier
    /// owns is indexed `y * viewport_x + x`.
    uint viewport_x;
    /// Its height.
    uint viewport_y;
    /// How many entries the candidate list holds — `crcbl_shaders::cmaa2`'s
    /// `candidate_capacity`. A frame with more edge pixels than this drops the
    /// ones past it, which is a pixel left unfiltered and never a write out of
    /// bounds.
    uint candidate_capacity;
    /// How many blend items the item list holds, on the same terms.
    uint item_capacity;
};

// Declaration order is binding order and it is load-bearing on Metal — see
// `crcbl_shaders::declaration_order`, which lints it over every source. This
// file and `cmaa2_shapes.slang` declare the same seven resources in the same
// order because one bind-group layout serves all four of their compute entry
// points — `crcbl_render::cmaa2` builds exactly one.

[[vk::binding(0, 0)]]
ConstantBuffer<Cmaa2Params> params;

/// The tonemapped frame, at this tier's own extent.
///
/// `Texture2D<float4>` read by `Load` and with no sampler beside it: every
/// fetch this tier makes is a whole texel read for its own value, which is what
/// a filtered tap would destroy.
[[vk::binding(1, 0)]]
Texture2D<float4> source;

/// `[0]`: how many candidates `edgesMain` appended. `[1]`: how many blend items
/// `cmaa2_shapes.slang` appended.
///
/// `crcbl_shaders::cmaa2::CONTROL_WORDS` long. Both are atomic adds, so both
/// start each frame at zero — `clearMain` is what writes it, a pass of its own
/// on `clear_counters.slang`'s terms exactly: a counter another invocation of
/// the same dispatch may already have added to cannot be zeroed inside it.
[[vk::binding(2, 0)]]
RWStructuredBuffer<Atomic<uint> > control;

/// `[pixel]`: which of a pixel's two owned boundaries carry an edge.
///
/// Bit 0 is `EDGE_LEFT` — the boundary between this pixel and the one to its
/// west — and bit 1 is `EDGE_TOP`, the boundary to its north. A boundary
/// belongs to exactly one of the two pixels either side of it, and taking the
/// west and north sides is the morphological convention throughout.
///
/// Written whole by `edgesMain`, so nothing clears it.
[[vk::binding(3, 0)]]
RWStructuredBuffer<uint> edges;

/// `[slot]`: the pixel index of each edge pixel, appended in whatever order the
/// device schedules.
///
/// The order is not observable: `cmaa2_shapes.slang` derives everything it
/// needs about a candidate from the `edges` buffer, so two runs that appended
/// the same set in different orders classify the same shapes and append the
/// same blend items in a different order — which the fixed-point accumulate is
/// what makes invisible.
[[vk::binding(4, 0)]]
RWStructuredBuffer<uint> candidates;

/// `[pixel * ACCUM_WORDS]`: the fixed-point accumulation, three colour words
/// and a weight word per pixel.
///
/// Zeroed here rather than in `clearMain`: a pixel's four words are written by
/// the one invocation that owns that pixel, so there is no race to separate
/// into its own pass, and folding it in is one fewer full-buffer pass.
[[vk::binding(5, 0)]]
RWStructuredBuffer<Atomic<uint> > accum;

/// `[slot * ITEM_WORDS]`: the blend items, three words each — the pixel being
/// blended, the pixel whose colour it takes a share of, and that share as a
/// fixed-point weight.
///
/// **Last of the seven, and that is not arbitrary.** No entry point of this file
/// touches it, and D3D12 numbers a register per class in declaration order
/// across the whole source — so a resource nothing here uses, declared in the
/// middle, leaves the container with a `u` register nothing accounts for.
/// `crcbl_dx12::dxil`'s `registers_are_dense_from_zero_in_every_committed_container`
/// is what says so. Declared last, it is the register the container simply ends
/// before.
[[vk::binding(6, 0)]]
RWStructuredBuffer<uint> items;

/// Invocations per workgroup, matching `crcbl_shaders::cmaa2::WORKGROUP_SIZE`.
static const uint WORKGROUP_SIZE = 64;

/// Words in `control`, matching `crcbl_shaders::cmaa2::CONTROL_WORDS`.
static const uint CONTROL_WORDS = 2;

/// Words per pixel in `accum`, matching `crcbl_shaders::cmaa2::ACCUM_WORDS`.
static const uint ACCUM_WORDS = 4;

/// Bit 0 of an `edges` word: an edge on the pixel's west boundary.
static const uint EDGE_LEFT = 1;

/// Bit 1: an edge on its north boundary.
static const uint EDGE_TOP = 2;

/// The luma delta across a pixel boundary, in display space, below which there
/// is no edge there — `crcbl_shaders::cmaa2::EDGE_THRESHOLD`.
///
/// **This tree's number rather than the reference's.** It is what
/// `smaa_edges.slang` used on the same fixture before it was retired, and the
/// observer in `crates/crcbl/tests/mesh_e2e/cmaa2.rs` is what holds it.
static const float EDGE_THRESHOLD = 0.1;

/// How much stronger a neighbouring boundary has to be before this one is
/// dropped — `crcbl_shaders::cmaa2::LOCAL_CONTRAST_ADAPTATION_FACTOR`.
///
/// The local-contrast adaptation CMAA2 and SMAA both carry, in the form this
/// tree measured: an edge whose neighbour has more than this much more contrast
/// is the quiet half of a pair and is not marked, so a step gets one line of
/// edge pixels rather than two. The factor is SMAA's own; the reference's is
/// not something this transcription can check.
static const float LOCAL_CONTRAST_ADAPTATION_FACTOR = 2.0;

/// Perceptual luma of a linear colour.
///
/// Rec. 709 weights, then one `sqrt` standing in for the sRGB encode. The
/// header says why the correction is here and what it approximates.
float luma_of(float3 color)
{
    return sqrt(dot(color, float3(0.2126, 0.7152, 0.0722)));
}

/// Luma of `texel`, clamped to the image.
///
/// Clamping rather than a border colour: a boundary at the frame's own edge has
/// no outside to step to, and repeating the edge texel makes that delta zero,
/// which is the answer that marks no edge there.
float luma_at(int2 texel)
{
    int2 inside = clamp(texel, int2(0, 0), int2(int(params.viewport_x) - 1, int(params.viewport_y) - 1));
    return luma_of(source.Load(int3(inside, 0)).rgb);
}

/// Zeroes the two counters `edgesMain` and `cmaa2_shapes.slang` add to.
///
/// One workgroup, dispatched over `CONTROL_WORDS` rather than over the frame,
/// so the guard below is the only thing between a rounded-up group count and a
/// write past the end.
[shader("compute")]
[numthreads(64, 1, 1)]
void clearMain(uint3 thread: SV_DispatchThreadID)
{
    if (thread.x >= CONTROL_WORDS)
    {
        return;
    }
    control[thread.x].store(0);
}

/// One invocation per pixel: mark this pixel's two boundaries, clear its
/// accumulation, and append it to the candidate list if either boundary carries
/// an edge.
///
/// **A linear dispatch over the frame rather than a two-dimensional one**, on
/// `exposure.slang`'s terms: the workgroup is the one constant every compute
/// pass here declares and the texel is recovered with a divide and a modulo.
///
/// The two boundaries a pixel owns are its west and north ones, so the first
/// column can carry no `EDGE_LEFT` and the first row no `EDGE_TOP` — there is
/// no pixel on the other side for the boundary to separate it from.
[shader("compute")]
[numthreads(64, 1, 1)]
void edgesMain(uint3 thread: SV_DispatchThreadID)
{
    uint index = thread.x;
    uint pixels = params.viewport_x * params.viewport_y;
    if (index >= pixels)
    {
        return;
    }
    uint x = index % params.viewport_x;
    uint y = index / params.viewport_x;
    int2 texel = int2(int(x), int(y));

    // The accumulation this pixel owns, zeroed before any blend item can reach
    // it — every pass that adds to it runs after this one.
    for (uint word = 0; word < ACCUM_WORDS; ++word)
    {
        accum[index * ACCUM_WORDS + word].store(0);
    }

    float c = luma_at(texel);
    float w = luma_at(texel + int2(-1, 0));
    float n = luma_at(texel + int2(0, -1));
    float e = luma_at(texel + int2(1, 0));
    float s = luma_at(texel + int2(0, 1));

    // The west and north deltas, which are the two boundaries this pixel owns.
    float2 delta = abs(c - float2(w, n));
    float2 marked = step(float2(EDGE_THRESHOLD, EDGE_THRESHOLD), delta);

    // Local contrast adaptation: the strongest delta in the neighbourhood
    // against this pixel's own two.
    float2 other = abs(c - float2(e, s));
    float strongest = max(max(delta.x, delta.y), max(other.x, other.y));
    marked *= step(float2(strongest, strongest), LOCAL_CONTRAST_ADAPTATION_FACTOR * delta);

    uint bits = 0;
    if (x > 0 && marked.x > 0.0)
    {
        bits |= EDGE_LEFT;
    }
    if (y > 0 && marked.y > 0.0)
    {
        bits |= EDGE_TOP;
    }
    edges[index] = bits;

    if (bits != 0)
    {
        // The append. A slot past the capacity is **dropped**: the counter has
        // already moved, so `cmaa2_shapes.slang` clamps what it reads back to
        // the capacity and this list is never written outside it. What that
        // costs is a pixel nobody classifies, which is a pixel that keeps its
        // unresolved colour — the degradation this tier's caps are chosen for.
        uint slot = control[0].add(1);
        if (slot < params.candidate_capacity)
        {
            candidates[slot] = index;
        }
    }
}
