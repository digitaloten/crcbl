//! The sRGB transfer function, in both directions, as a host-side prediction of
//! what a device wrote and a way back from what it wrote.
//!
//! # Why this is in the golden crate
//!
//! A golden suite that compares a *derived* colour with a readback byte has to
//! say what the hardware did on the way out: the engine hands linear colour to
//! an `Rgba8UnormSrgb` attachment and the format encodes it. A suite that
//! *measures* a frame needs the same curve run backwards, because the quantity
//! it is making a claim about — a share of a blend, a ratio of two
//! luminances — is linear and the levels it reads are not. That arithmetic is
//! the one thing every golden suite in the workspace shares, and each carried
//! its own transcription: [`encode`] was `crcbl`'s `render_e2e`,
//! `hal_seam_e2e`, `forward_e2e::depth_probe` and `sprite_e2e` plus
//! `apps/sundial` and `apps/alcove`, and [`decode`] was `render_e2e`,
//! `sprite_e2e`, `apps/alcove`, `apps/viewer`'s `linear_of` and
//! `apps/breakout`'s `to_linear`. A test binary cannot borrow another test
//! binary's helper, so a copy per suite, per direction, is what "keep it local"
//! costs.
//!
//! It lives here rather than in a sample-test crate because it depends on
//! nothing — not on `crcbl`, not on an image — so every one of those callers
//! already has it on the dev-dependency path that gave them
//! [`Image`](crate::Image).
//!
//! `crcbl_render::mip`'s `srgb_to_linear`/`linear_to_srgb` are deliberately not
//! among them: those are shipped engine code, and this is a dev-dependency the
//! engine cannot take.
//!
//! # The curve
//!
//! IEC 61966-2-1's piecewise definition, which is what Vulkan's `*_SRGB`
//! formats, Metal's `*_sRGB` ones and D3D12's `*_SRGB` ones are each defined to
//! apply on a write and undo on a read. The constants are the specification's,
//! and its two arms do not quite meet: they differ by about 3e-8 at the knee,
//! which is a property of the published numbers rather than of this
//! transcription and is what [`tests`](self#tests) allows for.

/// The slope of the straight segment both arms start from.
const LINEAR_SLOPE: f32 = 12.92;

/// The linear value the two arms change over at, which is what [`encode`]
/// compares against.
const KNEE_LINEAR: f32 = 0.003_130_8;

/// The same knee on the encoded side, which is what [`decode`] compares
/// against.
///
/// Derived from [`KNEE_LINEAR`] rather than written out as the specification's
/// rounded 0.04045, so that there is one knee in this module instead of two
/// spellings of it and [`encode`] and [`decode`] change arms at the same signal
/// by construction.
///
/// **The derivation is tidiness, not a fix.** Measured: the two thresholds
/// differ by about 6e-8 of a `[0, 1]` signal, no `byte / 255` lands between
/// them at all, and for the signals that do the two arms answer within 3e-9 of
/// each other in linear light. Nothing in this crate's tests can tell the two
/// apart, and the point of writing it down here is that the next reader does
/// not have to work that out again before deciding it is safe to change.
const KNEE_ENCODED: f32 = LINEAR_SLOPE * KNEE_LINEAR;

/// Encodes linear light into sRGB's `[0, 1]` signal.
///
/// The inverse of the [`decode`] a sampler applies on the way in. Values
/// outside `[0, 1]` are carried through the same two arms rather than clamped —
/// a caller predicting a byte clamps when it quantises, and a caller checking an
/// intermediate wants to see what it actually got.
#[must_use]
pub fn encode(linear: f32) -> f32 {
    if linear <= KNEE_LINEAR {
        LINEAR_SLOPE * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// Decodes sRGB's `[0, 1]` signal back into linear light.
///
/// [`encode`]'s inverse, and the electro-optical transfer function a sampler
/// applies to an `*_SRGB` format on the way in. It is what a suite reading a
/// frame needs before it can state a ratio: the encode turns an even ramp into
/// a curve three times steeper at one end than the other, so a share of a blend
/// or a ratio of two luminances measured in levels is measuring the transfer
/// function as much as the picture.
///
/// Values outside `[0, 1]` are carried through the same two arms rather than
/// clamped, for the reason [`encode`] gives.
#[must_use]
pub fn decode(encoded: f32) -> f32 {
    if encoded <= KNEE_ENCODED {
        encoded / LINEAR_SLOPE
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

/// [`encode`] scaled to a channel's 255 levels, without rounding.
///
/// What a suite comparing against a readback byte wants: the level the format
/// would have written, still fractional so the comparison can carry a tolerance
/// rather than argue about which side of a half the two landed on.
#[must_use]
pub fn encode_level(linear: f32) -> f32 {
    encode(linear) * 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The curve is the specification's, checked against its own anchors.**
    ///
    /// Every value here comes from IEC 61966-2-1 rather than from a run of this
    /// code: the two ends, the linear segment's slope, the knee both arms are
    /// defined to meet at, and two rows of the 8-bit table every image editor
    /// ships. A transcription slip — a coefficient, an exponent, a comparison
    /// the wrong way round — moves at least one of them.
    #[test]
    fn the_transfer_function_matches_the_specifications_own_values() {
        // The ends. `encode(1.0)` is `1.055 - 0.055` and is exact by
        // construction, which is why the specification picked those two.
        assert!(
            encode(0.0).abs() < 1e-7,
            "encode(0.0) is {}, and the curve starts at zero",
            encode(0.0)
        );
        assert!(
            (encode(1.0) - 1.0).abs() < 1e-6,
            "encode(1.0) is {}, and 1.055 - 0.055 is one",
            encode(1.0)
        );

        // The linear segment is a straight 12.92, so a value well under the
        // knee is a slope check with no exponent in it at all.
        assert!(
            (encode(0.001) - 0.012_92).abs() < 1e-6,
            "encode(0.001) is {}, so the linear segment's slope is not 12.92",
            encode(0.001)
        );

        // The knee at 0.0031308, where the two arms are defined to meet. They
        // meet to about 3e-8 with the published constants; a swapped comparison
        // or a mistyped 12.92 opens that to a whole level.
        let knee = 0.003_130_8_f32;
        let power_arm = 1.055 * knee.powf(1.0 / 2.4) - 0.055;
        assert!(
            (encode(knee) - power_arm).abs() < 1e-6,
            "the arms disagree at the knee: {} against {power_arm}",
            encode(knee)
        );

        // Two rows of the 8-bit table. Level 128 sits at linear 0.215_860_5,
        // and linear 0.5 — middle grey — is the 187.5 that rounds to 188.
        assert!(
            (encode_level(0.215_860_5) - 128.0).abs() < 0.02,
            "linear 0.215_860_5 is level 128, and this puts it at {}",
            encode_level(0.215_860_5)
        );
        assert!(
            (encode_level(0.5) - 187.516).abs() < 0.02,
            "linear 0.5 is level 187.516, and this puts it at {}",
            encode_level(0.5)
        );
    }

    /// **The inverse is the specification's too, checked against the same
    /// anchors from the other side.**
    ///
    /// Written against the published values rather than against [`encode`], so
    /// that this and the round trip below are two claims and not one: a pair of
    /// mutually inverse mistakes round-trips perfectly and lands every readback
    /// measurement in the wrong place.
    #[test]
    fn the_inverse_matches_the_specifications_own_values() {
        // The ends, which the specification fixes at exactly these.
        assert!(
            decode(0.0).abs() < 1e-7,
            "decode(0.0) is {}, and the curve starts at zero",
            decode(0.0)
        );
        assert!(
            (decode(1.0) - 1.0).abs() < 1e-6,
            "decode(1.0) is {}, and (1 + 0.055) / 1.055 is one",
            decode(1.0)
        );

        // The straight segment, with no exponent in it: a signal of 0.01292 is
        // 12.92 times a linear 0.001.
        assert!(
            (decode(0.012_92) - 0.001).abs() < 1e-6,
            "decode(0.012_92) is {}, so the straight segment's slope is not 12.92",
            decode(0.012_92)
        );

        // The knee, from the encoded side. The arms meet to about 3e-9 here,
        // which is the published constants rather than this transcription; a
        // swapped comparison or a mistyped 12.92 opens that to a whole level.
        let power_arm = ((KNEE_ENCODED + 0.055) / 1.055).powf(2.4);
        assert!(
            (decode(KNEE_ENCODED) - power_arm).abs() < 1e-6,
            "the arms disagree at the knee: {} against {power_arm}",
            decode(KNEE_ENCODED)
        );

        // The same two rows of the 8-bit table `encode` is pinned against,
        // read the other way: level 128 is linear 0.215_860_5, and level
        // 187.516 is middle grey.
        assert!(
            (decode(128.0 / 255.0) - 0.215_860_5).abs() < 1e-5,
            "level 128 is linear 0.215_860_5, and this puts it at {}",
            decode(128.0 / 255.0)
        );
        assert!(
            (decode(187.516 / 255.0) - 0.5).abs() < 1e-5,
            "level 187.516 is middle grey, and this puts it at {}",
            decode(187.516 / 255.0)
        );
    }

    /// **The two are inverses to the precision an `f32` has**, over the whole
    /// range and across the knee.
    ///
    /// The anchors above pin each arm's shape; this is what pins the *pair* —
    /// that the two really are inverses and not merely two plausible curves.
    /// It does **not** pin where they change arms: the band the two candidate
    /// thresholds disagree over is far too narrow for any tolerance here to
    /// see, which is measured on [`KNEE_ENCODED`].
    ///
    /// The bound is absolute rather than relative because the quantity is a
    /// `[0, 1]` signal. It is a few times the worst error measured over this
    /// sweep, which is a handful of the `f32` levels available at the top of
    /// the range, so a platform whose `powf` rounds a little differently is not
    /// a red run: `powf` is a libm call and libm is not bit-portable.
    #[test]
    fn the_pair_round_trips_to_f32_precision_across_the_whole_range() {
        /// The largest round-trip error the sweep may show.
        const TOLERANCE: f32 = 1e-6;

        let mut worst = (0.0f32, 0.0f32);
        for step in 0u16..=1000 {
            let linear = f32::from(step) / 1000.0;
            let error = (decode(encode(linear)) - linear).abs();
            if error > worst.1 {
                worst = (linear, error);
            }
        }
        assert!(
            worst.1 < TOLERANCE,
            "decode(encode({})) is off by {}, which is more than {TOLERANCE}",
            worst.0,
            worst.1
        );

        // The knee itself is not on that grid, and it is the step the two
        // constants have to agree about. Either side of it too, so a threshold
        // that moved is caught rather than straddled.
        for linear in [KNEE_LINEAR, KNEE_LINEAR * 0.999, KNEE_LINEAR * 1.001] {
            let error = (decode(encode(linear)) - linear).abs();
            assert!(
                error < TOLERANCE,
                "decode(encode({linear})) is off by {error} at the knee"
            );
        }
    }

    /// **It is monotonic across the knee**, which the anchors alone do not say.
    ///
    /// A pair of arms that each pass their own anchor can still step backwards
    /// where they join, and a predicted byte drawn from the wrong side of that
    /// step is off by more than any tolerance here allows.
    #[test]
    fn the_encode_rises_everywhere_including_across_the_knee() {
        let mut previous = f32::NEG_INFINITY;
        for step in 0u16..=1000 {
            let linear = f32::from(step) / 1000.0;
            let encoded = encode(linear);
            assert!(
                encoded > previous,
                "encode({linear}) is {encoded}, which does not rise above {previous}"
            );
            previous = encoded;
        }
    }
}
