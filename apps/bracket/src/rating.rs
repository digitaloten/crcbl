//! Glicko-2 ratings: what a player is worth, how sure of it the system is, and
//! how fast that certainty moves.
//!
//! # The algorithm is Glickman's, transcribed and cited
//!
//! Every formula and every constant below comes from Mark E. Glickman,
//! *Example of the Glicko-2 system*, Boston University, 22 March 2022,
//! published at <http://www.glicko.net/glicko/glicko2.pdf>. `rate` is that
//! paper's steps 2 to 8 in order, and each block names the step it is.
//!
//! **Nothing here is recalled.** A rating system is the shape of code a
//! transcription slip lives through: a scale factor off by a digit, a dropped
//! `g(φ)`, a `+` where the paper has a `−` — each still converges, still orders
//! a ladder plausibly, and nothing else in this crate could tell. What can tell
//! is the paper's own worked example, and the test named
//! `the_paper_s_worked_example` runs it through `rate` and checks all three
//! numbers the paper prints for it.
//!
//! # Why an uncertainty-aware rating, and which half of it does the work
//!
//! Elo moves a rating by a fixed step and has nowhere to put how much it trusts
//! either side of the match. Pairing on a small *observed* rating gap — which
//! is what a matchmaker is for — preferentially picks pairs whose *true* skill
//! gap is larger, because a rating is a noisy estimate of a skill, so the
//! favourite wins more often than the gap predicted and the ladder's scale
//! inflates with every match.
//!
//! Glicko-2 answers that in two places, and **the measurements say it is the
//! second one that matters here**. `g(φ)` flattens the expected score by how
//! unsure the system is of the *opponent* — but once a population has played,
//! every deviation is small and `g` is within a couple of percent of 1, so it
//! corrects almost nothing. What holds the ladder is that the step a rating
//! takes is `φ'²`, the player's own uncertainty, which the system derives from
//! the evidence instead of being told: run at a deviation that settles near 60
//! points, that step lands within a few percent of the K-factor it replaced and
//! the ladder stretches exactly as it did; run at one that settles near 32 and
//! the stretch nearly stops. `START_VOLATILITY` is where that is chosen, and
//! `docs/notes/simulation.md` has all three measurements.
//!
//! # A rating period is one match
//!
//! The paper treats every game inside a *rating period* as simultaneous and
//! updates at the end of it. This sample maps one period to one match, which is
//! the simplest mapping and the one a live matchmaker wants: a result is rated
//! when it is reported rather than at the end of a day. It is not the mapping
//! the paper prefers — it says the system works best at 10 to 15 games per
//! player per period — and the whole cost of that lands on the volatility,
//! which is asked to explain one result instead of a batch of them. That is why
//! `START_VOLATILITY` is the one constant here the paper does not fix.
//!
//! One consequence is worth stating: a player who is not in a match is not in a
//! rating period either, so their deviation does not grow while they sit out.
//! The paper's rule for that case is [`Rating::idled`], and nothing in this
//! sample reaches it, because this population never stops playing.
//!
//! # A rating cannot be built out of range
//!
//! [`Rating`] has no constructor taking arbitrary points outside this module's
//! own tests. One starts at [`Rating::provisional`] and moves only through
//! [`settle`], so a non-finite rating has nowhere to enter from and cannot
//! spread through every later match. That is the contract enforced rather than
//! documented.

use crcbl::core::rand::hash_unit;

/// The ratio between the Glicko scale a rating is *reported* on and the
/// Glicko-2 scale the update runs on.
///
/// The paper's steps 2 and 8, which divide and multiply by exactly this. It is
/// [`SKILL_SCALE`] over `ln 10` to the digits the paper prints, which is what
/// makes a Glicko-2 expected score against a perfectly known opponent the same
/// curve the match stub rolls against — `the_scale_factor_is_elo_s_ten_to_one`
/// pins both halves of that.
const GLICKO2_SCALE: f64 = 173.7178;

/// The *skill* difference at which the stronger player is expected to score ten
/// times as often as the weaker one.
///
/// The match stub's model rather than the rating system's: [`resolve`] weights
/// its roll by it and nothing else reads it. Elo's defining constant, kept
/// because the synthetic population's true skills are stated on that scale.
const SKILL_SCALE: f64 = 400.0;

/// The deviation a player with no record starts at.
///
/// The paper's step 1(a), with the rating it goes with at [`Rating::START`].
const START_DEVIATION: f64 = 350.0;

/// The volatility a player with no record starts at.
///
/// **The one constant here that is this application's rather than the
/// paper's**, and the paper says so: step 1(a) sets it to 0.06 and adds that
/// the value depends on the particular application. 0.06 goes with the period
/// length the paper recommends, 10 to 15 games; a period here is one game, and
/// the volatility is a per-period budget, so the same budget spread over one
/// game instead of twelve is `0.06 / sqrt(12)` —
/// `the_start_volatility_is_the_paper_s_over_a_period_of_one` is that
/// arithmetic and `the_deviation_settles_where_the_volatility_puts_it` is the
/// consequence it was chosen for.
///
/// It matters because the deviation settles where the volatility's growth
/// balances a period's worth of information, and the *step* a settled rating
/// takes is that deviation squared. At the paper's 0.06 over one-game periods
/// the deviation settles near 60 points and the step lands within a few percent
/// of the Elo K-factor this replaced — which is measurable: the ladder stretched
/// exactly as it did before. See `docs/notes/simulation.md`.
const START_VOLATILITY: f64 = 0.017_320_508;

/// The system constant τ, which constrains how fast a volatility may move.
///
/// The paper's step 1 asks for a value between 0.3 and 1.2 and says the system
/// should be tested to find the one with the best predictive accuracy. This is
/// the value the paper's own example is worked at, in the middle of that range,
/// and taking it is what makes the example usable: at any other τ the paper
/// prints no numbers to check against, and the one test here that can catch a
/// transcription slip would have nothing to assert.
const TAU: f64 = 0.5;

/// How narrow the bracket around `ln(σ'²)` must be before the volatility
/// iteration stops.
///
/// The paper's step 5.1, which names this value as a sufficiently small choice.
const CONVERGENCE_TOLERANCE: f64 = 0.000_001;

/// The most passes either loop in [`volatility`] may take.
///
/// Neither can reach it. The bracket search walks left in multiples of τ until
/// `f` turns positive, and the paper reports `k` is almost always 1; the
/// Illinois iteration's maximum over ten thousand of the paper's simulations
/// was 19. The cap is here because both are `while` loops over floating point,
/// so a non-finite input would spin one of them forever — and this sample runs
/// in a browser tab.
const MAX_ITERATIONS: u32 = 100;

/// The deviation at or below which a rating stops being called provisional.
///
/// Not the paper's — it names no such threshold — but built on the one summary
/// of RD the paper does give: a player's strength is 95% likely to lie inside
/// their rating plus or minus twice their deviation. At this value that
/// interval reaches a fifth of the way across the 1000-point skill range these
/// populations span, so a settled rating places a player somewhere in the
/// ladder rather than merely in the population. It reads a rating and changes
/// nothing about the update.
const PROVISIONAL_DEVIATION: f64 = 110.0;

/// What one player scored in one match.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The first player won.
    Win,
    /// Neither did.
    Draw,
    /// The second player won.
    Loss,
}

impl Outcome {
    /// The score the *first* player takes from this outcome.
    ///
    /// The paper's `s`: 0 for a loss, 0.5 for a draw, 1 for a win.
    #[must_use]
    pub const fn score(self) -> f64 {
        match self {
            Self::Win => 1.0,
            Self::Draw => 0.5,
            Self::Loss => 0.0,
        }
    }

    /// The same outcome told from the second player's side.
    #[must_use]
    pub const fn mirrored(self) -> Self {
        match self {
            Self::Win => Self::Loss,
            Self::Draw => Self::Draw,
            Self::Loss => Self::Win,
        }
    }
}

/// A player's rating, how uncertain it is, and how erratic their results have
/// been.
///
/// The three quantities the paper carries from one rating period to the next,
/// held on the Glicko scale it reports on rather than the Glicko-2 scale it
/// updates on. The conversion between them is step 2 going in and step 8 coming
/// out, and both live inside `rate`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rating {
    points: f64,
    deviation: f64,
    volatility: f64,
}

impl Rating {
    /// Where a player with no record starts.
    ///
    /// The paper's step 1(a), and also the middle of the range this sample's
    /// synthetic populations are drawn from, so a new player is equally wrong
    /// about a strong opponent and a weak one.
    pub const START: f64 = 1500.0;

    /// A rating with no games behind it.
    #[must_use]
    pub const fn provisional() -> Self {
        Self {
            points: Self::START,
            deviation: START_DEVIATION,
            volatility: START_VOLATILITY,
        }
    }

    /// The rating itself, on the scale it is reported on.
    #[must_use]
    pub const fn points(self) -> f64 {
        self.points
    }

    /// How unsure of that rating the system is, in the same points.
    ///
    /// The paper's RD: the player's strength is 95% likely to lie inside
    /// `points` plus or minus twice this.
    #[must_use]
    pub const fn deviation(self) -> f64 {
        self.deviation
    }

    /// How erratic this player's results have been.
    ///
    /// The paper's σ, and the quantity Glicko-2 has that Glicko does not: it
    /// rises when a run of results is hard to explain from the rating, which
    /// lets the rating move faster without the deviation having to widen.
    #[must_use]
    pub const fn volatility(self) -> f64 {
        self.volatility
    }

    /// Whether the system is still too unsure of this rating to call it
    /// settled.
    #[must_use]
    pub fn is_provisional(self) -> bool {
        self.deviation > PROVISIONAL_DEVIATION
    }

    /// This rating after a period the player sat out.
    ///
    /// The paper's note under step 8: rating and volatility are unchanged and
    /// only step 6 applies, so the deviation grows by the volatility — the
    /// system forgets. Nothing in this sample reaches it through a queue,
    /// because a period here *is* a match; `rate` falls back to it for a
    /// period with no games in it, which is what makes that function total.
    #[must_use]
    pub fn idled(self) -> Self {
        let phi = self.deviation / GLICKO2_SCALE;
        Self {
            deviation: GLICKO2_SCALE * (phi * phi + self.volatility * self.volatility).sqrt(),
            ..self
        }
    }
}

impl Default for Rating {
    fn default() -> Self {
        Self::provisional()
    }
}

/// The paper's `g(φ)`: how much an opponent's uncertainty flattens the score
/// they are expected to concede.
///
/// Step 3. It is 1 for an opponent whose rating is known exactly and falls
/// towards 0 as theirs becomes a guess, which is the term Elo has no room for.
fn attenuation(phi: f64) -> f64 {
    let pi_squared = std::f64::consts::PI * std::f64::consts::PI;
    1.0 / (1.0 + 3.0 * phi * phi / pi_squared).sqrt()
}

/// The paper's `E(µ, µj, φj)`: the score a player is expected to take off one
/// opponent.
///
/// Step 3.
fn expectation(mu: f64, opponent_mu: f64, opponent_phi: f64) -> f64 {
    1.0 / (1.0 + (-attenuation(opponent_phi) * (mu - opponent_mu)).exp())
}

/// The paper's step 5: the new volatility, found by its Illinois iteration.
///
/// Every line is the paper's, including the halving of `fA` that makes this the
/// Illinois variant of regula falsi rather than plain false position. The paper
/// replaced a Newton-Raphson procedure with this one in 2012 precisely because
/// the old one sometimes did not converge, so the variant is the point rather
/// than a detail.
fn volatility(delta: f64, phi: f64, v: f64, sigma: f64) -> f64 {
    let a = (sigma * sigma).ln();
    let f = |x: f64| {
        let e = x.exp();
        let denominator = phi * phi + v + e;
        e * (delta * delta - phi * phi - v - e) / (2.0 * denominator * denominator)
            - (x - a) / (TAU * TAU)
    };

    // Step 5.2. A and B bracket `ln(σ'²)`. They are not in numeric order, and
    // B is the end that moves first.
    let mut bound_a = a;
    let mut bound_b = if delta * delta > phi * phi + v {
        (delta * delta - phi * phi - v).ln()
    } else {
        // Walk left in multiples of τ until `f` turns positive. This terminates
        // for every finite input: as `x` falls, the first term of `f` vanishes
        // and `−(x − a)/τ²` grows without bound.
        let mut k = 1.0;
        while f(a - k * TAU) < 0.0 && k < f64::from(MAX_ITERATIONS) {
            k += 1.0;
        }
        a - k * TAU
    };

    // Steps 5.3 and 5.4.
    let mut f_a = f(bound_a);
    let mut f_b = f(bound_b);
    let mut passes = 0;
    while (bound_b - bound_a).abs() > CONVERGENCE_TOLERANCE && passes < MAX_ITERATIONS {
        let c = bound_a + (bound_a - bound_b) * f_a / (f_b - f_a);
        let f_c = f(c);
        if f_c * f_b <= 0.0 {
            bound_a = bound_b;
            f_a = f_b;
        } else {
            f_a /= 2.0;
        }
        bound_b = c;
        f_b = f_c;
        passes += 1;
    }

    // Step 5.5.
    (bound_a / 2.0).exp()
}

/// One player's side of one rating period.
///
/// The paper's steps 2 to 8, over however many opponents the period held. Both
/// [`settle`] and the worked-example test go through here, so the arithmetic
/// the sample runs on is the arithmetic the paper's numbers check.
fn rate(player: Rating, games: &[(Rating, f64)]) -> Rating {
    // The paper's note under step 8, and the reason this function is total:
    // a period with no games in it moves nothing but the deviation.
    if games.is_empty() {
        return player.idled();
    }

    // Step 2.
    let mu = (player.points - Rating::START) / GLICKO2_SCALE;
    let phi = player.deviation / GLICKO2_SCALE;

    // The sums of steps 3 and 4 share their per-opponent terms, so they are
    // accumulated together. `performance` is the sum step 7 wants; ∆ is that
    // sum times `v`, and only step 5 reads it.
    let mut variance_sum = 0.0;
    let mut performance = 0.0;
    for (opponent, score) in games {
        let opponent_mu = (opponent.points - Rating::START) / GLICKO2_SCALE;
        let opponent_phi = opponent.deviation / GLICKO2_SCALE;
        let weight = attenuation(opponent_phi);
        let expected = expectation(mu, opponent_mu, opponent_phi);
        variance_sum += weight * weight * expected * (1.0 - expected);
        performance += weight * (score - expected);
    }

    // Steps 3 and 4.
    let v = 1.0 / variance_sum;
    let delta = v * performance;

    // Step 5, then step 6's new pre-period deviation.
    let sigma = volatility(delta, phi, v, player.volatility);
    let phi_star = (phi * phi + sigma * sigma).sqrt();

    // Step 7.
    let phi_next = 1.0 / (1.0 / (phi_star * phi_star) + 1.0 / v).sqrt();
    let mu_next = mu + phi_next * phi_next * performance;

    // Step 8.
    Rating {
        points: GLICKO2_SCALE * mu_next + Rating::START,
        deviation: GLICKO2_SCALE * phi_next,
        volatility: sigma,
    }
}

/// What `rating` is expected to score against `opponent`, in `0.0..=1.0`.
///
/// The paper's `E` over two ratings on the scale they are reported on. It is
/// **not** symmetric, because it is attenuated by how uncertain the
/// *opponent's* rating is: two players whose deviations differ do not expect a
/// whole point between them, and which side of one the pair falls depends on
/// whether the surer of the two is the one ahead.
#[must_use]
pub fn expected_score(rating: Rating, opponent: Rating) -> f64 {
    expectation(
        (rating.points - Rating::START) / GLICKO2_SCALE,
        (opponent.points - Rating::START) / GLICKO2_SCALE,
        opponent.deviation / GLICKO2_SCALE,
    )
}

/// The chance the first of two *skills* beats the second.
///
/// The match stub's model rather than a rating: a true skill has no deviation
/// to be uncertain by, which is why this takes bare points. It is the same
/// logistic [`expected_score`] runs on — `SKILL_SCALE` over `ln 10` is
/// `GLICKO2_SCALE` — so the curve the outcomes come from and the curve the
/// ratings chase are one curve.
#[must_use]
pub fn expected_from_points(points: f64, opponent: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((opponent - points) / SKILL_SCALE))
}

/// Apply one result, returning both players' new ratings.
///
/// `outcome` is told from `a`'s side. Each side is a rating period of one match
/// against the other's *pre-match* rating, so the two updates are independent
/// and neither sees the other's. That is why this is not zero-sum: a player the
/// system is unsure of moves further than the settled opponent who beat them,
/// which is the correction a fixed step cannot make.
#[must_use]
pub fn settle(a: Rating, b: Rating, outcome: Outcome) -> (Rating, Rating) {
    (
        rate(a, &[(b, outcome.score())]),
        rate(b, &[(a, outcome.mirrored().score())]),
    )
}

/// Resolve one match between players of known true skill.
///
/// The outcome is a seeded roll weighted by [`expected_from_points`], so a
/// population's ratings converge on its true skills if and only if the
/// arithmetic in this module is right. It is deliberately *not* decided by
/// whoever is stronger: an outcome that always favoured the higher skill would
/// make convergence trivial and prove nothing.
#[must_use]
pub fn resolve(skill_a: f64, skill_b: f64, seed: u64, index: u64) -> Outcome {
    if hash_unit(seed, index) < expected_from_points(skill_a, skill_b) {
        Outcome::Win
    } else {
        Outcome::Loss
    }
}

/// A rating at chosen values, for tests that need a spread.
///
/// Deliberately behind `cfg(test)`: outside a test a rating is only reachable
/// through [`Rating::provisional`] and [`settle`], which is what keeps a
/// non-finite one out. `queue`'s tests use it too, which is why it is not
/// inside the test module below.
#[cfg(test)]
pub(crate) const fn at(points: f64, deviation: f64, volatility: f64) -> Rating {
    Rating {
        points,
        deviation,
        volatility,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crcbl::core::rand::salt;

    /// The volatility the paper's worked example is stated at.
    ///
    /// Step 1(a)'s default, and an *input* to that example rather than a
    /// system constant — which is why this sample's own
    /// [`START_VOLATILITY`] is free to be a different number and the example
    /// still checks the arithmetic.
    const PAPER_VOLATILITY: f64 = 0.06;

    #[test]
    fn the_start_volatility_is_the_paper_s_over_a_period_of_one() {
        // The arithmetic `START_VOLATILITY`'s doc comment claims: the paper's
        // default spread from the middle of the 10-to-15-game period it goes
        // with down to the one-game period this sample rates on.
        let games_per_period: f64 = 12.0;
        let derived = PAPER_VOLATILITY / games_per_period.sqrt();
        assert!(
            (START_VOLATILITY - derived).abs() < 1.0e-9,
            "{START_VOLATILITY} is not {PAPER_VOLATILITY}/sqrt({games_per_period}) = {derived}"
        );
    }

    #[test]
    fn the_deviation_settles_where_the_volatility_puts_it() {
        // The mechanism the choice above was made for. A settled rating steps
        // by its deviation squared, and the deviation settles where the
        // volatility's growth balances a period's worth of information — so
        // the volatility, and nothing else, is what sets the step a converged
        // ladder takes. Two players drawing forever is the cleanest way to
        // watch it happen.
        fn settled(volatility: f64) -> f64 {
            let mut a = at(Rating::START, START_DEVIATION, volatility);
            let mut b = a;
            for _ in 0..500 {
                (a, b) = settle(a, b, Outcome::Draw);
            }
            a.deviation()
        }

        let ours = settled(START_VOLATILITY);
        let paper = settled(PAPER_VOLATILITY);
        assert!(
            paper > ours * 1.5,
            "the paper's default settles at {paper} and this sample's at {ours}, \
             which is not the difference the constant was chosen for"
        );
        assert!(
            ours < PROVISIONAL_DEVIATION,
            "a rating that never leaves provisional ({ours}) would make \
             `is_provisional` a constant"
        );
    }

    /// **The paper's own worked example, and the only test here that could
    /// catch a transcription slip.**
    ///
    /// A player rated 1500 whose deviation is 200 and whose volatility is 0.06
    /// meets players rated 1400, 1550 and 1700, whose deviations are 30, 100
    /// and 300; they win the first and lose the other two, and τ is 0.5. The
    /// paper prints the answer to two decimal places for the rating and the
    /// deviation and to five for the volatility, and those three numbers are
    /// what this asserts.
    ///
    /// Every constant in this module is loaded by it — the scale factor twice
    /// over, `g`, `E`, the variance, ∆, the whole volatility iteration and both
    /// halves of step 7 — so a wrong digit anywhere moves one of the three.
    ///
    /// **The tolerances are the paper's own rounding, not slack.** It carries
    /// four figures from step to step and rounds µ' to four decimals before the
    /// final multiply by the scale factor, so its printed rating is that
    /// arithmetic's answer rather than a full-precision one: this module lands
    /// at 1464.0507 against the paper's 1464.06, which is that last rounding
    /// and no more. `the_worked_example_s_intermediate_quantities` is where
    /// each step is checked against the figure the paper prints for it, and it
    /// reproduces the paper's rounded arithmetic to say so.
    #[test]
    fn the_paper_s_worked_example() {
        let player = at(1500.0, 200.0, PAPER_VOLATILITY);
        let updated = rate(
            player,
            &[
                (at(1400.0, 30.0, PAPER_VOLATILITY), 1.0),
                (at(1550.0, 100.0, PAPER_VOLATILITY), 0.0),
                (at(1700.0, 300.0, PAPER_VOLATILITY), 0.0),
            ],
        );

        assert!(
            (updated.points() - 1464.06).abs() < 0.02,
            "rating {} is not the paper's 1464.06",
            updated.points()
        );
        assert!(
            (updated.deviation() - 151.52).abs() < 0.01,
            "deviation {} is not the paper's 151.52",
            updated.deviation()
        );
        assert!(
            (updated.volatility() - 0.05999).abs() < 0.000_01,
            "volatility {} is not the paper's 0.05999",
            updated.volatility()
        );
    }

    /// The intermediate quantities the paper prints on the way to that answer.
    ///
    /// The three final numbers can only say that something is wrong; these say
    /// which step it is in.
    ///
    /// Where a quantity is computed from the values the player and opponents
    /// were *given* — µj, φj, `g`, `E` — it is asserted to the precision the
    /// paper prints. Where the paper computes one from figures it has already
    /// rounded — `v` and ∆ come from the four-decimal `g` and `E` in its own
    /// table, φ* from its four-decimal φ — both halves are checked: the paper's
    /// arithmetic is reproduced from the paper's own figures and has to land on
    /// the number it prints, and this module's full-precision value has to sit
    /// within the rounding that separates them.
    #[test]
    fn the_worked_example_s_intermediate_quantities() {
        let mu = 0.0;
        // A row per opponent: their rating and deviation, the score against
        // them, then the paper's own µj, φj, g(φj) and E for that row.
        let opponents = [
            (1400.0, 30.0, 1.0, -0.5756, 0.1727, 0.9955, 0.639),
            (1550.0, 100.0, 0.0, 0.2878, 0.5756, 0.9531, 0.432),
            (1700.0, 300.0, 0.0, 1.1513, 1.7269, 0.7242, 0.303),
        ];

        let mut variance_sum = 0.0;
        let mut performance = 0.0;
        let mut paper_variance_sum = 0.0;
        let mut paper_performance = 0.0;
        for (points, deviation, score, paper_mu, paper_phi, paper_g, paper_e) in opponents {
            let opponent_mu = (points - Rating::START) / GLICKO2_SCALE;
            let opponent_phi = deviation / GLICKO2_SCALE;
            let weight = attenuation(opponent_phi);
            let expected = expectation(mu, opponent_mu, opponent_phi);
            assert!(
                (opponent_mu - paper_mu).abs() < 0.000_05,
                "mu {opponent_mu}"
            );
            assert!(
                (opponent_phi - paper_phi).abs() < 0.000_05,
                "phi {opponent_phi}"
            );
            assert!((weight - paper_g).abs() < 0.000_05, "g {weight}");
            assert!((expected - paper_e).abs() < 0.000_5, "E {expected}");
            variance_sum += weight * weight * expected * (1.0 - expected);
            performance += weight * (score - expected);
            paper_variance_sum += paper_g * paper_g * paper_e * (1.0 - paper_e);
            paper_performance += paper_g * (score - paper_e);
        }

        // Step 3, and step 4's sum before and after the multiply by `v`.
        let v = 1.0 / variance_sum;
        let paper_v = 1.0 / paper_variance_sum;
        assert!(
            (paper_v - 1.7785).abs() < 0.000_05,
            "the paper's v is {paper_v}"
        );
        assert!(
            (v - paper_v).abs() < 0.001,
            "v {v} against the paper's {paper_v}"
        );
        let delta = v * performance;
        let paper_delta = paper_v * paper_performance;
        assert!(
            (paper_delta + 0.4834).abs() < 0.000_05,
            "the paper's delta is {paper_delta}"
        );
        assert!(
            (delta - paper_delta).abs() < 0.001,
            "delta {delta} against the paper's {paper_delta}"
        );
        assert!(
            (performance + 0.272).abs() < 0.000_5,
            "the sum step 7 wants is {performance}, not the paper's -0.272"
        );

        // Step 5, which the paper reaches on its second Illinois pass.
        let phi = 200.0 / GLICKO2_SCALE;
        assert!((phi - 1.1513).abs() < 0.000_05, "phi {phi}");
        let sigma = volatility(delta, phi, v, PAPER_VOLATILITY);
        assert!((sigma - 0.05999).abs() < 0.000_01, "sigma {sigma}");

        // Step 6, printed to six decimals and computed from the four-decimal
        // phi above it.
        let phi_star = (phi * phi + sigma * sigma).sqrt();
        let paper_phi_star = (1.1513f64 * 1.1513 + 0.05999 * 0.05999).sqrt();
        assert!(
            (paper_phi_star - 1.152_862).abs() < 0.000_000_5,
            "the paper's phi star is {paper_phi_star}"
        );
        assert!(
            (phi_star - paper_phi_star).abs() < 0.000_05,
            "phi star {phi_star} against the paper's {paper_phi_star}"
        );

        // Step 7, both halves.
        let phi_next = 1.0 / (1.0 / (phi_star * phi_star) + 1.0 / v).sqrt();
        assert!((phi_next - 0.8722).abs() < 0.000_05, "phi next {phi_next}");
        let mu_next = mu + phi_next * phi_next * performance;
        assert!((mu_next + 0.2069).abs() < 0.000_05, "mu next {mu_next}");
    }

    #[test]
    fn the_scale_factor_is_elo_s_ten_to_one() {
        // The Glicko-2 scale factor is the skill scale over `ln 10`, and the
        // paper prints it rounded to four decimals. Written as the relation
        // rather than as the constant, so it fails on a mistyped digit instead
        // of proving the module agrees with itself.
        let exact = SKILL_SCALE / std::f64::consts::LN_10;
        assert!(
            (GLICKO2_SCALE - exact).abs() < 0.000_05,
            "{GLICKO2_SCALE} is not {SKILL_SCALE}/ln 10 = {exact}"
        );

        // And the consequence: against an opponent whose rating is known
        // exactly, a player a skill scale ahead expects ten of every eleven
        // points from the rating system *and* from the match stub.
        let expected = expected_score(
            at(Rating::START + SKILL_SCALE, 0.0, START_VOLATILITY),
            at(Rating::START, 0.0, START_VOLATILITY),
        );
        let rolled = expected_from_points(Rating::START + SKILL_SCALE, Rating::START);
        assert!(
            (expected - 10.0 / 11.0).abs() < 1.0e-7 && (rolled - 10.0 / 11.0).abs() < 1.0e-12,
            "the rating expects {expected} and the stub rolls {rolled}"
        );
    }

    #[test]
    fn equal_ratings_expect_half_a_point() {
        let player = Rating::provisional();
        assert_eq!(expected_score(player, player), 0.5);
    }

    #[test]
    fn an_uncertain_opponent_flattens_the_expected_score() {
        // `g(φ)` is the term Elo has no room for: the further an opponent's
        // rating is from being known, the closer to even the match is treated
        // as, whichever way the points point.
        let player = at(Rating::START + 300.0, 50.0, START_VOLATILITY);
        let known = at(Rating::START, 30.0, START_VOLATILITY);
        let guessed = at(Rating::START, START_DEVIATION, START_VOLATILITY);
        let against_known = expected_score(player, known);
        let against_guessed = expected_score(player, guessed);
        assert!(
            against_known > against_guessed && against_guessed > 0.5,
            "known {against_known}, guessed {against_guessed}"
        );
        assert_eq!(
            attenuation(0.0),
            1.0,
            "an opponent who is known exactly was attenuated"
        );
        assert!(
            attenuation(START_DEVIATION / GLICKO2_SCALE) < 1.0,
            "an opponent nothing is known about was not"
        );
    }

    #[test]
    fn two_equally_known_players_expect_a_whole_point_between_them() {
        for delta in [0.0, 1.0, 37.5, 400.0, 1200.0, -250.0] {
            let a = at(Rating::START, 80.0, START_VOLATILITY);
            let b = at(Rating::START + delta, 80.0, START_VOLATILITY);
            let total = expected_score(a, b) + expected_score(b, a);
            assert!(
                (total - 1.0).abs() < 1.0e-12,
                "delta {delta}: total {total}"
            );
        }

        // And a mismatched pair deliberately does not, because each side's
        // expectation is attenuated by the *other's* deviation — which way it
        // misses depends on whether the surer player is the one ahead.
        let sure = at(Rating::START + 200.0, 30.0, START_VOLATILITY);
        let unsure = at(Rating::START, START_DEVIATION, START_VOLATILITY);
        let ahead = expected_score(sure, unsure) + expected_score(unsure, sure);
        assert!(
            ahead < 0.99,
            "the surer player ahead summed to {ahead}, so `g` is doing nothing"
        );

        let sure = at(Rating::START, 30.0, START_VOLATILITY);
        let unsure = at(Rating::START + 200.0, START_DEVIATION, START_VOLATILITY);
        let behind = expected_score(sure, unsure) + expected_score(unsure, sure);
        assert!(
            behind > 1.01,
            "the surer player behind summed to {behind}, not past a whole point"
        );
    }

    #[test]
    fn a_win_raises_and_a_loss_lowers() {
        let start = Rating::provisional();
        let (winner, loser) = settle(start, start, Outcome::Win);
        assert!(winner.points() > start.points(), "{winner:?}");
        assert!(loser.points() < start.points(), "{loser:?}");
    }

    #[test]
    fn a_draw_between_equals_moves_nothing_but_the_certainty() {
        let start = Rating::provisional();
        let (a, b) = settle(start, start, Outcome::Draw);
        assert_eq!(a.points(), start.points());
        assert_eq!(b.points(), start.points());
        // The result still said something: the system is surer of both of them
        // than it was, which is what a rating with no games behind it has to
        // earn.
        assert!(a.deviation() < start.deviation(), "{a:?}");
        assert!(b.deviation() < start.deviation(), "{b:?}");
    }

    #[test]
    fn an_upset_moves_more_than_the_expected_result() {
        let favourite = at(Rating::START + SKILL_SCALE, 60.0, START_VOLATILITY);
        let underdog = at(Rating::START, 60.0, START_VOLATILITY);

        let (_, upset_winner) = settle(favourite, underdog, Outcome::Loss);
        let (expected_winner, _) = settle(favourite, underdog, Outcome::Win);

        let upset_gain = upset_winner.points() - underdog.points();
        let expected_gain = expected_winner.points() - favourite.points();
        assert!(
            upset_gain > expected_gain,
            "upset gained {upset_gain}, the expected result gained {expected_gain}"
        );
    }

    #[test]
    fn a_rating_the_system_is_unsure_of_moves_faster() {
        // What Glicko-2 has instead of a K-factor schedule, and it is not a
        // schedule: the step is `φ'²`, so it falls out of how uncertain the
        // rating already was rather than out of a game count.
        let new = Rating::provisional();
        assert!(new.is_provisional());
        let settled = at(Rating::START, 50.0, START_VOLATILITY);
        assert!(!settled.is_provisional());

        let (new_after, _) = settle(new, new, Outcome::Win);
        let (settled_after, _) = settle(settled, settled, Outcome::Win);
        assert!(
            new_after.points() - new.points() > settled_after.points() - settled.points(),
            "provisional {new_after:?} did not outrun settled {settled_after:?}"
        );
    }

    #[test]
    fn playing_settles_a_rating_and_sitting_out_unsettles_it() {
        let mut player = Rating::provisional();
        let mut filler = Rating::provisional();
        assert!(player.is_provisional());

        let mut games = 0;
        while player.is_provisional() {
            (player, filler) = settle(player, filler, Outcome::Draw);
            games += 1;
            assert!(
                games < 1_000,
                "still provisional at a deviation of {}",
                player.deviation()
            );
        }

        // And the paper's idle rule pushes it back the other way, which is the
        // half of step 6 no queue in this sample reaches.
        let idle = player.idled();
        assert!(
            idle.deviation() > player.deviation(),
            "sitting out narrowed the deviation: {idle:?}"
        );
        assert_eq!(idle.points(), player.points(), "sitting out moved a rating");
        assert_eq!(
            rate(player, &[]),
            idle,
            "an empty period is not an idle one"
        );
    }

    #[test]
    fn the_volatility_solver_takes_both_of_the_paper_s_branches() {
        // `∆² > φ² + v` skips the walk left and takes B from a logarithm. A
        // large ∆ for a player the system is sure of is what reaches it, and
        // the answer has to stay a plausible volatility rather than a number
        // the iteration wandered off to.
        let phi = 30.0 / GLICKO2_SCALE;
        let v = 4.0;
        let delta = 6.0;
        assert!(
            delta * delta > phi * phi + v,
            "this no longer reaches the branch it is about"
        );
        let surprised = volatility(delta, phi, v, START_VOLATILITY);
        assert!(
            surprised > START_VOLATILITY && surprised < 1.0,
            "a result nothing explains left the volatility at {surprised}"
        );

        // The other branch, which is the one the worked example takes: a result
        // in line with the rating leaves the volatility where it was.
        let quiet = volatility(0.0, phi, v, START_VOLATILITY);
        assert!(
            (quiet - START_VOLATILITY).abs() < 0.001,
            "an unremarkable result moved the volatility to {quiet}"
        );
    }

    /// The population the convergence test runs, and the error it ends with.
    ///
    /// Returns the mean and worst absolute distance between a player's rating
    /// and their true skill, in points.
    fn converge(players: usize, matches: u64, seed: u64) -> (f64, f64) {
        // True skills spread evenly across the range a rating starts in the
        // middle of, so nobody's starting rating is already right.
        let skill: Vec<f64> = (0..players)
            .map(|index| {
                let across = index as f64 / (players - 1) as f64;
                Rating::START - 500.0 + across * 1000.0
            })
            .collect();
        let mut ratings = vec![Rating::provisional(); players];

        for round in 0..matches {
            // Pair at random rather than by rating: this test is about the
            // arithmetic converging, and skill-based pairing would let a good
            // matchmaker hide a bad update rule.
            let a = (hash_unit(seed, round * 2) * players as f64) as usize % players;
            let b = (hash_unit(seed, round * 2 + 1) * players as f64) as usize % players;
            if a == b {
                continue;
            }
            let outcome = resolve(skill[a], skill[b], salt(seed, round), round);
            let (next_a, next_b) = settle(ratings[a], ratings[b], outcome);
            ratings[a] = next_a;
            ratings[b] = next_b;
        }

        let errors: Vec<f64> = ratings
            .iter()
            .zip(&skill)
            .map(|(rating, truth)| (rating.points() - truth).abs())
            .collect();
        let mean = errors.iter().sum::<f64>() / errors.len() as f64;
        let worst = errors.iter().copied().fold(0.0, f64::max);
        (mean, worst)
    }

    /// The mean absolute error a population has before a single match is
    /// played, which is what convergence has to beat.
    fn starting_error(players: usize) -> f64 {
        let total: f64 = (0..players)
            .map(|index| {
                let across = index as f64 / (players - 1) as f64;
                (Rating::START - (Rating::START - 500.0 + across * 1000.0)).abs()
            })
            .sum();
        total / players as f64
    }

    #[test]
    fn ratings_converge_on_true_skill() {
        // Every player starts at `Rating::START` while true skills are spread
        // evenly 500 either side of it, so the error to be closed is a known
        // quantity — and asserting it first is what stops the bounds below
        // passing on a run that never moved a rating at all.
        let start_error = starting_error(64);
        assert!(
            start_error > 200.0,
            "the population barely spreads at all ({start_error:.1}), so closing \
             on it would prove nothing"
        );

        const MEAN_ERROR: f64 = 60.0;
        const WORST_ERROR: f64 = 200.0;
        for seed in [0xB2ACu64, 1, 7, 99, 123_456] {
            let (mean, worst) = converge(64, 40_000, seed);
            assert!(
                mean < MEAN_ERROR,
                "seed {seed}: mean error {mean:.1} did not close on the true skills"
            );
            assert!(worst < WORST_ERROR, "seed {seed}: worst error {worst:.1}");
            assert!(
                mean < start_error / 3.0,
                "seed {seed}: mean error {mean:.1} barely improved on {start_error:.1}"
            );
        }
    }
}
