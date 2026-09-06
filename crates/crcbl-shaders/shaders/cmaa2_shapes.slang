// crcbl-targets: spirv, wgsl, msl, dxil

// `docs/plan/49-antialiasing.md`'s CMAA2 tier, second and third of its five
// passes: the candidate classification with its bounded long-edge search, and
// the fixed-point accumulate that turns the blend items it appends into a
// per-pixel total. `cmaa2_edges.slang`'s header describes the tier, what each
// pass hands the next, and which parts of the reference this transcribes.
//
// # The shapes, and where their coverage comes from
//
// A candidate is a pixel with an edge on its west or north boundary. Runs of
// those boundaries are the aliased staircase, and each run is classified by
// **how the boundary turns at its two ends** — which is the 2×2 pattern the
// reference calls a `Z` or a `U`:
//
// * **`Z`** — the two ends turn opposite ways. The boundary is a straight line
//   crossing the run diagonally, and reconstructing it is exact: at the run's
//   left end it sits half a pixel to one side of the aliased boundary and at
//   the right end half a pixel to the other, so the coverage it leaves over
//   run pixel `i` is the line's mean offset across that pixel.
// * **`U`** — the two ends turn the same way. No straight line has those
//   endpoints, so the reconstruction is two segments meeting in the middle: a
//   half-pixel offset at each end falling to zero between them. That is the
//   morphological family's own answer and it over-blurs a genuine notch, so it
//   is taken at `U_SHAPE_WEIGHT` rather than whole — the "conservative" the
//   tier is named for.
// * **Neither** — an end that does not turn at all, because the run leaves the
//   frame or reaches `MAX_LINE_LENGTH` before it terminates. Nothing is
//   emitted: an unclassified run is a run that keeps its unresolved colour.
//
// The coverage is derived here rather than looked up, which is the structural
// difference from the SMAA tier this replaces — that one carried two committed
// tables and this one carries none.
//
// # Why the accumulate is a pass of its own, and in fixed point
//
// A run's blend items land on pixels other runs also land on, so a pixel's
// total is a sum over items that reach it in whatever order the device
// schedules. **Float addition is not associative**, so a float sum in arrival
// order makes a frame a function of the scheduler rather than of its inputs —
// which is the property `docs/plan/49-antialiasing.md` spends its length
// refusing, and which no golden could hold. Integer addition is associative and
// commutative, so the accumulation is in fixed point at
// `BLEND_FIXED_POINT_SCALE` and the one conversion back to float happens per
// pixel in `cmaa2_apply.slang`, after every item has landed.
//
// That "after" is why it is a separate dispatch: a pixel cannot read its own
// total until every workgroup that might add to it has finished, and the only
// ordering this engine has between workgroups is a pass boundary — the same
// argument `clear_counters.slang` makes about its zero.
//
// # No `#include`
//
// `cmaa2_edges.slang` repeats `bloom_composite.slang`'s rule. The parameter
// block and the seven resource declarations below are that file's, copied,
// because one bind-group layout serves both.

/// Everything the CMAA2 passes take from the CPU — `cmaa2_edges.slang` declares
/// the same block and its header says why there is one.
struct Cmaa2Params
{
    /// Width of the image being filtered, in texels.
    uint viewport_x;
    /// Its height.
    uint viewport_y;
    /// How many entries the candidate list holds.
    uint candidate_capacity;
    /// How many blend items the item list holds.
    uint item_capacity;
};

// Declaration order is binding order and it is load-bearing on Metal — see
// `crcbl_shaders::declaration_order`. These seven are `cmaa2_edges.slang`'s, in
// its order, because one layout serves both files.

[[vk::binding(0, 0)]]
ConstantBuffer<Cmaa2Params> params;

/// The tonemapped frame. Read by `accumulateMain` for the colour a blend item
/// carries a share of; `shapesMain` reads only the edge buffer.
[[vk::binding(1, 0)]]
Texture2D<float4> source;

/// `[0]`: candidates appended. `[1]`: blend items appended.
[[vk::binding(2, 0)]]
RWStructuredBuffer<Atomic<uint> > control;

/// `[pixel]`: the edge bits `cmaa2_edges.slang` wrote.
[[vk::binding(3, 0)]]
RWStructuredBuffer<uint> edges;

/// `[slot]`: the pixel index of each edge pixel.
[[vk::binding(4, 0)]]
RWStructuredBuffer<uint> candidates;

/// `[pixel * ACCUM_WORDS]`: the fixed-point accumulation, three colour words
/// and a weight word per pixel.
[[vk::binding(5, 0)]]
RWStructuredBuffer<Atomic<uint> > accum;

/// `[slot * ITEM_WORDS]`: the blend items — the pixel being blended, the pixel
/// whose colour it takes a share of, and that share as a fixed-point weight.
[[vk::binding(6, 0)]]
RWStructuredBuffer<uint> items;

/// Words per pixel in `accum`, matching `crcbl_shaders::cmaa2::ACCUM_WORDS`.
static const uint ACCUM_WORDS = 4;

/// Words per blend item, matching `crcbl_shaders::cmaa2::ITEM_WORDS`.
static const uint ITEM_WORDS = 3;

/// Bit 0 of an `edges` word: an edge on the pixel's west boundary.
static const uint EDGE_LEFT = 1;

/// Bit 1: an edge on its north boundary.
static const uint EDGE_TOP = 2;

/// The longest run this pass will classify, in pixels — matching
/// `crcbl_shaders::cmaa2::MAX_LINE_LENGTH`.
///
/// A bound rather than a quality knob: the search is a loop and an unbounded
/// one over a frame-sized buffer is a shader a driver may time out on. A run
/// longer than this is left unclassified rather than truncated and guessed at,
/// because a truncated run's ends are not its ends and the coverage derived
/// from them would be a shape the frame does not contain.
static const uint MAX_LINE_LENGTH = 64;

/// What a fixed-point weight of one is — matching
/// `crcbl_shaders::cmaa2::BLEND_FIXED_POINT_SCALE`.
///
/// Two to the twentieth. The precision argument is in two halves and both
/// matter. **Below**: one part in 1048576 is three orders of magnitude finer
/// than the `1/255` step of an eight-bit target, so the quantisation cannot
/// move a written texel. **Above**: a `uint` holds 4096 of these, and a pixel
/// takes a handful of blend items, so the accumulation has no path to a wrap —
/// the weights it sums are each at most `MAX_BLEND_WEIGHT` and the colours they
/// scale are saturated into `[0, 1]`.
static const float BLEND_FIXED_POINT_SCALE = 1048576.0;

/// How much of the `U` reconstruction is applied — matching
/// `crcbl_shaders::cmaa2::U_SHAPE_WEIGHT`.
///
/// A `U` is the one shape whose reconstruction is a guess: two segments meeting
/// in the middle is only one of the surfaces that could have produced those
/// ends, and taking it whole softens a notch that may be geometry rather than
/// aliasing. **This transcription's number, not the reference's** — see
/// `cmaa2_edges.slang`'s header and `docs/backlog.md`.
static const float U_SHAPE_WEIGHT = 0.5;

/// The smallest share worth an item, as a fraction.
///
/// Half the `1/255` step of an eight-bit target: a contribution below it cannot
/// move the pixel it lands on, and every one of them is a list slot, an atomic
/// add and a dispatched invocation.
static const float MIN_BLEND_WEIGHT = 0.001953125;

/// The largest share one item may carry.
///
/// Half: a reconstructed line is at most half a pixel off the aliased boundary
/// it replaces, so a pixel is at most half its neighbour's.
static const float MAX_BLEND_WEIGHT = 0.5;

/// The edge bits of `(x, y)`, or none where that is outside the frame.
///
/// `x` and `y` are unsigned, so a step off the west or north side wraps to a
/// very large number and is caught by the same comparison as a step off the
/// east or south one.
uint edge_at(uint x, uint y)
{
    if (x >= params.viewport_x || y >= params.viewport_y)
    {
        return 0;
    }
    return edges[y * params.viewport_x + x];
}

/// Which way a horizontal run turns at the boundary column `column` of row
/// `row`: `1` where the vertical segment continuing it lies in the row above,
/// `-1` where it lies in this row, and `0` where there is neither or both.
///
/// A run of north boundaries along row `row` separates the pixels of row
/// `row - 1` from those of row `row`. Where it ends, the boundary has to keep
/// going vertically, and it can only do that as a west boundary in one of those
/// two rows.
int horizontal_turn(uint column, uint row)
{
    bool below = (edge_at(column, row) & EDGE_LEFT) != 0;
    bool above = row > 0 && (edge_at(column, row - 1) & EDGE_LEFT) != 0;
    if (above && !below)
    {
        return 1;
    }
    if (below && !above)
    {
        return -1;
    }
    return 0;
}

/// The same for a vertical run, at the boundary row `row` of column `column`:
/// `1` where the horizontal segment continuing it lies in the column to the
/// west, `-1` where it lies in this column.
int vertical_turn(uint column, uint row)
{
    bool right = (edge_at(column, row) & EDGE_TOP) != 0;
    bool left = column > 0 && (edge_at(column - 1, row) & EDGE_TOP) != 0;
    if (left && !right)
    {
        return 1;
    }
    if (right && !left)
    {
        return -1;
    }
    return 0;
}

/// Where the reconstructed boundary sits, relative to the aliased one, a
/// fraction `t` of the way along a run.
///
/// In pixels and **signed**: negative is toward the pixel on the `-offset` side
/// of the boundary, positive toward the one on the other side. A half-pixel
/// offset at an end is the most a morphological reconstruction can claim, which
/// is where the halves come from.
///
/// A `Z` is a straight line between the two ends. A `U` cannot be one — no
/// straight line has both ends on the same side — so it is two segments meeting
/// at zero in the middle, taken at [`U_SHAPE_WEIGHT`].
float boundary_height(float t, int start, int end, bool u_shape)
{
    if (u_shape)
    {
        return -0.5 * float(start) * (2.0 * abs(t - 0.5)) * U_SHAPE_WEIGHT;
    }
    return -0.5 * float(start) * (1.0 - t) - 0.5 * float(end) * t;
}

/// Appends one blend item, dropping it where the list is already full.
///
/// **A slot past the capacity is dropped**, and that is the whole of what an
/// overflowing frame costs: the counter has moved, so `accumulateMain` clamps
/// what it reads back to the same capacity, and the pixel the dropped item was
/// for keeps its unresolved colour. There is no path from an overflow to a
/// write outside this buffer.
///
/// A share below [`MIN_BLEND_WEIGHT`] is not appended at all: it cannot move
/// the pixel it lands on, and every item is a slot, an atomic add and a
/// dispatched invocation.
void append_item(uint target, uint from, float share)
{
    if (share < MIN_BLEND_WEIGHT)
    {
        return;
    }
    uint slot = control[1].add(1);
    if (slot < params.item_capacity)
    {
        items[slot * ITEM_WORDS + 0] = target;
        items[slot * ITEM_WORDS + 1] = from;
        items[slot * ITEM_WORDS + 2] = uint(min(share, MAX_BLEND_WEIGHT) * BLEND_FIXED_POINT_SCALE);
    }
}

/// Appends the blend items one classified run leaves.
///
/// `first` is the pixel index of the run's first element, `stride` the index
/// step between consecutive elements — one along a row, a row's width down a
/// column — and `offset` the index distance to the pixel on the *other* side of
/// the run's boundary, which is a row's width for a north boundary and one for
/// a west boundary.
///
/// `start` and `end` are [`horizontal_turn`]'s or [`vertical_turn`]'s answers at
/// the two ends. A run with a zero at either end is not a shape and leaves
/// nothing.
///
/// # The coverage is an integral over the pixel, not the height at its middle
///
/// Both sides of the boundary can be wrong inside **one** pixel, and taking the
/// mean signed height would cancel them. The clearest case is the one this tier
/// exists for: a 45° staircase, where every run is a single pixel long and the
/// reconstructed line runs corner to corner across it. Its mean height is
/// exactly zero — and an eighth of the pixel above the line belongs below it and
/// an eighth below belongs above, which is a mix a filter must make and a mean
/// says is unnecessary. So the two areas are integrated separately, and a pixel
/// the line crosses appends **two** items.
void emit_line(uint first, uint stride, uint len, uint offset, int start, int end)
{
    if (start == 0 || end == 0)
    {
        return;
    }
    float inv_len = 1.0 / float(len);
    bool u_shape = start == end;
    for (uint i = 0; i < len; ++i)
    {
        // The reconstructed boundary at this element's two edges. It is linear
        // between them — a `U`'s only bend is at the middle of the run, which
        // falls on an element edge whenever the run has an even length and is
        // inside one element when it does not, where the error is a fraction of
        // one pixel's coverage.
        float near = boundary_height(float(i) * inv_len, start, end, u_shape);
        float far = boundary_height(float(i + 1) * inv_len, start, end, u_shape);

        // The area of this element on each side of the reconstruction: a
        // trapezoid where the two heights share a sign, and two triangles where
        // the line crosses inside the element.
        float below;
        float above;
        if (near <= 0.0 && far <= 0.0)
        {
            below = -0.5 * (near + far);
            above = 0.0;
        }
        else if (near >= 0.0 && far >= 0.0)
        {
            below = 0.0;
            above = 0.5 * (near + far);
        }
        else
        {
            float crossing = near / (near - far);
            below = 0.5 * abs(min(near, far)) * (near < 0.0 ? crossing : 1.0 - crossing);
            above = 0.5 * max(near, far) * (near > 0.0 ? crossing : 1.0 - crossing);
        }

        uint pixel = first + i * stride;
        // `below` is area this pixel should have had from the other side of the
        // boundary, and `above` the same for the pixel across it.
        append_item(pixel, pixel - offset, below);
        append_item(pixel - offset, pixel, above);
    }
}

/// One invocation per candidate slot: classify the runs that **start** at this
/// candidate and append their blend items.
///
/// Only a run's first pixel does the work, so a run of length `n` is walked
/// once rather than `n` times — which is what keeps this pass proportional to
/// the frame's edges. A pixel may start a horizontal run and a vertical one,
/// and both are processed.
[shader("compute")]
[numthreads(64, 1, 1)]
void shapesMain(uint3 thread: SV_DispatchThreadID)
{
    // **Clamped to the capacity**, not to the counter: an overflowing frame
    // left the counter above the list it could write, and a dispatch that
    // trusted the counter would read slots nothing filled.
    uint count = min(control[0].load(), params.candidate_capacity);
    uint slot = thread.x;
    if (slot >= count)
    {
        return;
    }

    uint index = candidates[slot];
    uint width = params.viewport_x;
    uint x = index % width;
    uint y = index / width;
    uint own = edges[index];

    // A horizontal run starts here when this pixel carries a north boundary and
    // the pixel to its west does not.
    if ((own & EDGE_TOP) != 0 && (edge_at(x - 1, y) & EDGE_TOP) == 0)
    {
        uint len = 1;
        while (len <= MAX_LINE_LENGTH && (edge_at(x + len, y) & EDGE_TOP) != 0)
        {
            len += 1;
        }
        if (len <= MAX_LINE_LENGTH)
        {
            emit_line(index, 1, len, width, horizontal_turn(x, y), horizontal_turn(x + len, y));
        }
    }

    // A vertical run starts here when this pixel carries a west boundary and
    // the pixel to its north does not.
    if ((own & EDGE_LEFT) != 0 && (edge_at(x, y - 1) & EDGE_LEFT) == 0)
    {
        uint len = 1;
        while (len <= MAX_LINE_LENGTH && (edge_at(x, y + len) & EDGE_LEFT) != 0)
        {
            len += 1;
        }
        if (len <= MAX_LINE_LENGTH)
        {
            emit_line(index, width, len, 1, vertical_turn(x, y), vertical_turn(x, y + len));
        }
    }
}

/// One invocation per blend item: add its share of one pixel's colour to
/// another pixel's fixed-point total.
///
/// **Order-independent by construction.** Every invocation here does four
/// integer atomic adds and reads nothing back, so the total a pixel ends with
/// is the same whatever order the device runs them in — which is what makes a
/// frame this tier resolved a function of its inputs.
[shader("compute")]
[numthreads(64, 1, 1)]
void accumulateMain(uint3 thread: SV_DispatchThreadID)
{
    uint count = min(control[1].load(), params.item_capacity);
    uint slot = thread.x;
    if (slot >= count)
    {
        return;
    }

    uint target = items[slot * ITEM_WORDS + 0];
    uint from = items[slot * ITEM_WORDS + 1];
    uint weight = items[slot * ITEM_WORDS + 2];
    uint pixels = params.viewport_x * params.viewport_y;
    if (target >= pixels || from >= pixels)
    {
        return;
    }

    uint width = params.viewport_x;
    int2 texel = int2(int(from % width), int(from / width));
    // Saturated, so the products below cannot exceed the weight and the
    // accumulation's headroom argument holds for any frame — including one an
    // earlier pass left above one.
    float3 color = saturate(source.Load(int3(texel, 0)).rgb);
    float share = float(weight);

    accum[target * ACCUM_WORDS + 0].add(uint(color.r * share));
    accum[target * ACCUM_WORDS + 1].add(uint(color.g * share));
    accum[target * ACCUM_WORDS + 2].add(uint(color.b * share));
    accum[target * ACCUM_WORDS + 3].add(weight);
}
