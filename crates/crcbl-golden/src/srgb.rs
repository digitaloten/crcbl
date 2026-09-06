//! The sRGB transfer function, as a host-side prediction of what a device
//! wrote.
//!
//! # Why this is in the golden crate
//!
//! A golden suite that compares a *derived* colour with a readback byte has to
//! say what the hardware did on the way out: the engine hands linear colour to
//! an `Rgba8UnormSrgb` attachment and the format encodes it. That arithmetic is
//! the one thing every golden suite in the workspace shares, and until now each
//! carried its own transcription — `crcbl`'s `render_e2e`, `hal_seam_e2e`,
//! `forward_e2e::depth_probe` and `sprite_e2e`, plus `apps/sundial` and
//! `apps/alcove`. A test binary cannot borrow another test binary's helper, so
//! six copies of one curve is what "keep it local" costs.
//!
//! It lives here rather than in a sample-test crate because it depends on
//! nothing — not on `crcbl`, not on an image — so every one of those callers
//! already has it on the dev-dependency path that gave them
//! [`Image`](crate::Image).
//!
//! # The curve
//!
//! IEC 61966-2-1's piecewise definition, which is what Vulkan's `*_SRGB`
//! formats, Metal's `*_sRGB` ones and D3D12's `*_SRGB` ones are each defined to
//! apply on a write. The constants are the specification's, and they do not
//! quite meet: the two arms differ by about 3e-8 at the knee, which is a
//! property of the published numbers rather than of this transcription and is
//! what [`tests`](self#tests) allows for.

/// Encodes linear light into sRGB's `[0, 1]` signal.
///
/// The inverse of the decode a sampler applies on the way in. Values outside
/// `[0, 1]` are carried through the same two arms rather than clamped — a
/// caller predicting a byte clamps when it quantises, and a caller checking an
/// intermediate wants to see what it actually got.
#[must_use]
pub fn encode(linear: f32) -> f32 {
    if linear <= 0.003_130_8 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
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
