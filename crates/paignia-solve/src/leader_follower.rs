//! Exact leader-follower value for one binary inspection family.
//!
//! The leader commits to the probability of the first row. The follower observes
//! that distribution, not the later private draw, and answers with a best
//! response. Optimistic ties maximize the leader's payoff. Pessimistic ties
//! minimize it. A bound that is only a supremum carries no strategy witness.
//!
//! The payoffs are `U_I(p,q) = -c*p - h*(1-p)*q` and
//! `U_E(p,q) = q*(g - (g+f)*p)`, with strictly positive `c`, `h`, `g` and `f`.
//! This is not a general Stackelberg solver, an interval model, or closure of
//! `T18.01`.

use std::cmp::Ordering;

use thiserror::Error;

/// Version of the exact family, tie rule and nonattainment policy.
pub const VERSION: &str = "1.0.0";

/// Reduced rational with a positive denominator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ratio {
    numerator: i64,
    denominator: i64,
}

impl Ratio {
    /// Admit a finite ratio and reduce it. A zero denominator is rejected.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ZeroDenominator`] when `denominator` is zero, and
    /// [`Error::Overflow`] when reduction does not fit in `i64`.
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, Error> {
        if denominator == 0 {
            return Err(Error::ZeroDenominator);
        }
        from_i128(i128::from(numerator), i128::from(denominator))
    }

    #[must_use]
    pub const fn numerator(self) -> i64 {
        self.numerator
    }

    #[must_use]
    pub const fn denominator(self) -> i64 {
        self.denominator
    }

    fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }

    fn one() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }

    fn is_probability(self) -> bool {
        self >= Self::zero() && self <= Self::one()
    }

    /// Add two reduced ratios.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Overflow`] when the sum does not fit in `i64`.
    pub fn checked_add(self, other: Self) -> Result<Self, Error> {
        self.add(other)
    }

    /// Subtract two reduced ratios.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Overflow`] when the reduced difference does not fit in `i64`.
    pub fn checked_sub(self, other: Self) -> Result<Self, Error> {
        self.sub(other)
    }

    /// Multiply two reduced ratios.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Overflow`] when the product does not fit in `i64`.
    pub fn checked_mul(self, other: Self) -> Result<Self, Error> {
        self.mul(other)
    }

    fn neg(self) -> Result<Self, Error> {
        let numerator = self.numerator.checked_neg().ok_or(Error::Overflow)?;
        Ok(Self {
            numerator,
            denominator: self.denominator,
        })
    }

    fn add(self, other: Self) -> Result<Self, Error> {
        from_i128(
            i128::from(self.numerator) * i128::from(other.denominator)
                + i128::from(other.numerator) * i128::from(self.denominator),
            i128::from(self.denominator) * i128::from(other.denominator),
        )
    }

    fn sub(self, other: Self) -> Result<Self, Error> {
        from_i128(
            i128::from(self.numerator) * i128::from(other.denominator)
                - i128::from(other.numerator) * i128::from(self.denominator),
            i128::from(self.denominator) * i128::from(other.denominator),
        )
    }

    fn mul(self, other: Self) -> Result<Self, Error> {
        from_i128(
            i128::from(self.numerator) * i128::from(other.numerator),
            i128::from(self.denominator) * i128::from(other.denominator),
        )
    }

    fn div(self, other: Self) -> Result<Self, Error> {
        if other.numerator == 0 {
            return Err(Error::ZeroDenominator);
        }
        from_i128(
            i128::from(self.numerator) * i128::from(other.denominator),
            i128::from(self.denominator) * i128::from(other.numerator),
        )
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = i128::from(self.numerator) * i128::from(other.denominator);
        let right = i128::from(other.numerator) * i128::from(self.denominator);
        left.cmp(&right)
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Strictly positive parameters of the declared inspection family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Parameters {
    inspection_cost: Ratio,
    undetected_harm: Ratio,
    evasion_gain: Ratio,
    detected_loss: Ratio,
}

impl Parameters {
    /// Admit `c`, `h`, `g` and `f` when each is strictly positive.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NonPositiveParameter`] when any parameter is not
    /// strictly positive.
    pub fn try_new(
        inspection_cost: Ratio,
        undetected_harm: Ratio,
        evasion_gain: Ratio,
        detected_loss: Ratio,
    ) -> Result<Self, Error> {
        for value in [
            inspection_cost,
            undetected_harm,
            evasion_gain,
            detected_loss,
        ] {
            if value <= Ratio::zero() {
                return Err(Error::NonPositiveParameter);
            }
        }
        Ok(Self {
            inspection_cost,
            undetected_harm,
            evasion_gain,
            detected_loss,
        })
    }
}

/// How the follower breaks indifference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FollowerTie {
    /// Choose the follower best response with the greater leader payoff.
    Optimistic,
    /// Choose the follower best response with the lesser leader payoff.
    Pessimistic,
}

/// Expected payoffs of one feasible strategy pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Payoffs {
    /// Leader payoff.
    pub leader: Ratio,
    /// Follower payoff.
    pub follower: Ratio,
}

/// Unilateral pure-deviation gains of a supplied simultaneous strategy pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Gains {
    /// Leader's gain from the better pure row.
    pub leader: Ratio,
    /// Follower's gain from the better pure column.
    pub follower: Ratio,
}

/// Unique simultaneous Nash witness for this family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NashWitness {
    /// Probability of the first row.
    pub inspect: Ratio,
    /// Probability of the second column.
    pub evade: Ratio,
    /// Expected payoffs at that witness.
    pub payoffs: Payoffs,
}

/// Leader value under an expected-effort budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Commitment {
    /// Maximum or supremum of the leader's commitment value.
    pub value: Ratio,
    /// `false` when `value` is a supremum with no feasible witness.
    pub attained: bool,
    /// Attaining row probability, absent when `attained` is false.
    pub inspect: Option<Ratio>,
    /// Attaining follower response, absent when `attained` is false.
    pub evade: Option<Ratio>,
}

/// Commitment value while one parameter ranges over a closed interval.
///
/// The value bounds span the pointwise commitment values. On an admitted
/// interval either every game attains its value or every game has an unattained
/// supremum. In the former case each action bound describes one coordinate of
/// the attaining witnesses across the interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitmentProjection {
    /// Lower and upper leader commitment values.
    pub value: RatioBounds,
    /// Whether every game in the admitted interval attains its value.
    pub attained: bool,
    /// Attaining inspection probabilities, absent when `attained` is false.
    pub inspect: Option<RatioBounds>,
    /// Attaining follower responses, absent when `attained` is false.
    pub evade: Option<RatioBounds>,
}

/// Rejection of an inspection-family request.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum Error {
    /// A parameter was zero or negative.
    #[error("inspection parameters must be strictly positive")]
    NonPositiveParameter,
    /// A ratio used as a probability was outside the unit interval.
    #[error("probability {numerator}/{denominator} is outside [0, 1]")]
    ProbabilityOutOfRange {
        /// Numerator of the rejected probability.
        numerator: i64,
        /// Denominator of the rejected probability.
        denominator: i64,
    },
    /// The committed probability exceeds the expected-effort budget.
    #[error("commitment exceeds the expected-effort budget")]
    BudgetExceeded,
    /// A ratio denominator was zero.
    #[error("a ratio denominator must be nonzero")]
    ZeroDenominator,
    /// Inspection cost equals detection-weighted harm, so inspection shares form a Nash segment.
    #[error("inspection cost equals detection-weighted harm, giving multiple Nash witnesses")]
    LeaderIndifferent,
    /// At full inspection, follower indifference admits a segment of evasion shares.
    #[error("follower indifference at full inspection gives multiple Nash witnesses")]
    FollowerIndifferentAtFullInspection,
    /// An intermediate product did not fit in `i64`.
    #[error("exact arithmetic overflowed")]
    Overflow,
    /// A constructed Nash witness had a positive deviation gain.
    #[error("the constructed witness has a profitable deviation")]
    InconsistentWitness,
    /// An interval was ordered from its larger endpoint to its smaller one.
    #[error("the interval lower bound exceeds its upper bound")]
    UnorderedInterval,
    /// Detection probability was outside `(0, 1]`.
    #[error("detection probability {numerator}/{denominator} is outside (0, 1]")]
    DetectionOutOfRange {
        /// Numerator of the rejected probability.
        numerator: i64,
        /// Denominator of the rejected probability.
        denominator: i64,
    },
    /// The detection interval contains more than one Nash regime.
    #[error("detection probability changes the Nash regime inside the interval")]
    DetectionRegimeChange,
    /// A rectangle named the same parameter on both axes.
    #[error("a Nash rectangle must vary two distinct parameters")]
    RepeatedParameter,
    /// The corners of a parameter rectangle are not in one strict Nash regime.
    #[error("parameter rectangle changes the Nash regime")]
    RectangleRegimeChange,
    /// The interval is not contained in one strict commitment piece.
    #[error("the interval is not inside one strict commitment piece")]
    CommitmentRegimeChange,
}

/// Expected payoffs from the four declared cells.
///
/// # Errors
///
/// Returns [`Error::ProbabilityOutOfRange`] when either probability is outside
/// `[0, 1]`, and [`Error::Overflow`] when the exact products do not fit.
pub fn expected_payoffs(game: Parameters, inspect: Ratio, evade: Ratio) -> Result<Payoffs, Error> {
    require_probability(inspect)?;
    require_probability(evade)?;
    let one = Ratio::one();
    let skip = one.sub(inspect)?;
    let comply = one.sub(evade)?;
    let leader = inspect
        .mul(comply)?
        .mul(game.inspection_cost.neg()?)?
        .add(inspect.mul(evade)?.mul(game.inspection_cost.neg()?)?)?
        .add(skip.mul(evade)?.mul(game.undetected_harm.neg()?)?)?;
    let follower = skip
        .mul(evade)?
        .mul(game.evasion_gain)?
        .add(inspect.mul(evade)?.mul(game.detected_loss.neg()?)?)?;
    Ok(Payoffs { leader, follower })
}

/// Pure-deviation gains of a supplied pair. This does not search for equilibria.
///
/// # Errors
///
/// Returns the same errors as [`expected_payoffs`].
pub fn deviation_gains(game: Parameters, inspect: Ratio, evade: Ratio) -> Result<Gains, Error> {
    let current = expected_payoffs(game, inspect, evade)?;
    let leader = expected_payoffs(game, Ratio::zero(), evade)?
        .leader
        .max(expected_payoffs(game, Ratio::one(), evade)?.leader)
        .sub(current.leader)?;
    let follower = expected_payoffs(game, inspect, Ratio::zero())?
        .follower
        .max(expected_payoffs(game, inspect, Ratio::one())?.follower)
        .sub(current.follower)?;
    Ok(Gains { leader, follower })
}

/// Unique Nash witness when inspection is strictly dominated or strictly mixed.
///
/// # Errors
///
/// Returns [`Error::LeaderIndifferent`] when cost equals harm, because every
/// inspection probability up to the follower's threshold is then a best response.
/// Also returns [`Error::InconsistentWitness`] if the closed form has a positive gain.
pub fn simultaneous_nash(game: Parameters) -> Result<NashWitness, Error> {
    let (inspect, evade) = match game.inspection_cost.cmp(&game.undetected_harm) {
        Ordering::Less => (
            game.evasion_gain
                .div(game.evasion_gain.add(game.detected_loss)?)?,
            game.inspection_cost.div(game.undetected_harm)?,
        ),
        Ordering::Greater => (Ratio::zero(), Ratio::one()),
        Ordering::Equal => return Err(Error::LeaderIndifferent),
    };
    let payoffs = expected_payoffs(game, inspect, evade)?;
    let gains = deviation_gains(game, inspect, evade)?;
    if gains.leader != Ratio::zero() || gains.follower != Ratio::zero() {
        return Err(Error::InconsistentWitness);
    }
    Ok(NashWitness {
        inspect,
        evade,
        payoffs,
    })
}

/// Unique Nash witness when an inspection detects evasion with probability `detection`.
///
/// Perfect detection is `1` and matches [`simultaneous_nash`]. If inspection cost
/// exceeds detection-weighted harm (`c > d h`), skipping is strictly dominant
/// even when the follower would be indifferent at full inspection.
///
/// # Errors
///
/// Returns [`Error::LeaderIndifferent`] when `c = d h`, or
/// [`Error::FollowerIndifferentAtFullInspection`] when `c < d h` and
/// `d(g+f) = g`. Also returns [`Error::DetectionOutOfRange`],
/// [`Error::Overflow`], or [`Error::InconsistentWitness`].
pub fn simultaneous_nash_with_detection(
    game: Parameters,
    detection: Ratio,
) -> Result<NashWitness, Error> {
    require_detection(detection)?;
    if detection == Ratio::one() {
        return simultaneous_nash(game);
    }
    let (inspect, evade) = detection_actions(game, detection)?;
    let payoffs = expected_payoffs_with_detection(game, detection, inspect, evade)?;
    let gains = detection_gains(game, detection, inspect, evade)?;
    if gains.leader != Ratio::zero() || gains.follower != Ratio::zero() {
        return Err(Error::InconsistentWitness);
    }
    Ok(NashWitness {
        inspect,
        evade,
        payoffs,
    })
}

/// Project the unique Nash witness while detection stays inside one strict regime.
///
/// # Errors
///
/// Returns [`Error::DetectionRegimeChange`] when the endpoints are not both
/// interior, both pure-skip, or both pure-inspect. Endpoint errors from
/// [`simultaneous_nash_with_detection`] propagate.
pub fn detection_nash_projection(
    game: Parameters,
    lower: Ratio,
    upper: Ratio,
) -> Result<NashProjection, Error> {
    require_detection(lower)?;
    require_detection(upper)?;
    if lower > upper {
        return Err(Error::UnorderedInterval);
    }
    let low = simultaneous_nash_with_detection(game, lower)?;
    let high = simultaneous_nash_with_detection(game, upper)?;
    let skip_changes = (low.inspect == Ratio::zero()) != (high.inspect == Ratio::zero());
    let full_inspection_changes = (low.inspect == Ratio::one()) != (high.inspect == Ratio::one());
    if skip_changes || full_inspection_changes {
        return Err(Error::DetectionRegimeChange);
    }
    Ok(NashProjection {
        inspect: span_bounds(low.inspect, high.inspect),
        evade: span_bounds(low.evade, high.evade),
        leader: span_bounds(low.payoffs.leader, high.payoffs.leader),
        follower: span_bounds(low.payoffs.follower, high.payoffs.follower),
        endpoint_gap: Ratio::zero(),
    })
}

fn span_bounds(left: Ratio, right: Ratio) -> RatioBounds {
    if left <= right {
        RatioBounds {
            lower: left,
            upper: right,
        }
    } else {
        RatioBounds {
            lower: right,
            upper: left,
        }
    }
}

fn require_detection(detection: Ratio) -> Result<(), Error> {
    if detection > Ratio::zero() && detection <= Ratio::one() {
        Ok(())
    } else {
        Err(Error::DetectionOutOfRange {
            numerator: detection.numerator(),
            denominator: detection.denominator(),
        })
    }
}

fn detection_actions(game: Parameters, detection: Ratio) -> Result<(Ratio, Ratio), Error> {
    let scaled_harm = detection.mul(game.undetected_harm)?;
    match game.inspection_cost.cmp(&scaled_harm) {
        Ordering::Greater => return Ok((Ratio::zero(), Ratio::one())),
        Ordering::Equal => return Err(Error::LeaderIndifferent),
        Ordering::Less => {}
    }
    let follow = detection.mul(game.evasion_gain.add(game.detected_loss)?)?;
    match follow.cmp(&game.evasion_gain) {
        Ordering::Greater => Ok((
            game.evasion_gain.div(follow)?,
            game.inspection_cost.div(scaled_harm)?,
        )),
        Ordering::Less => Ok((Ratio::one(), Ratio::one())),
        Ordering::Equal => Err(Error::FollowerIndifferentAtFullInspection),
    }
}

/// Four-cell expectation when detection is imperfect.
///
/// # Errors
///
/// Returns [`Error::DetectionOutOfRange`], [`Error::ProbabilityOutOfRange`], or
/// [`Error::Overflow`].
pub fn expected_payoffs_with_detection(
    game: Parameters,
    detection: Ratio,
    inspect: Ratio,
    evade: Ratio,
) -> Result<Payoffs, Error> {
    require_detection(detection)?;
    require_probability(inspect)?;
    require_probability(evade)?;
    let one = Ratio::one();
    let miss = one.sub(detection)?;
    let skip = one.sub(inspect)?;
    let comply = one.sub(evade)?;
    let inspect_evade_leader = game
        .inspection_cost
        .neg()?
        .sub(miss.mul(game.undetected_harm)?)?;
    let inspect_evade_follower = miss
        .mul(game.evasion_gain)?
        .sub(detection.mul(game.detected_loss)?)?;
    let leader = inspect
        .mul(comply)?
        .mul(game.inspection_cost.neg()?)?
        .add(inspect.mul(evade)?.mul(inspect_evade_leader)?)?
        .add(skip.mul(evade)?.mul(game.undetected_harm.neg()?)?)?;
    let follower = inspect
        .mul(evade)?
        .mul(inspect_evade_follower)?
        .add(skip.mul(evade)?.mul(game.evasion_gain)?)?;
    Ok(Payoffs { leader, follower })
}

fn detection_gains(
    game: Parameters,
    detection: Ratio,
    inspect: Ratio,
    evade: Ratio,
) -> Result<Gains, Error> {
    let current = expected_payoffs_with_detection(game, detection, inspect, evade)?;
    let leader = expected_payoffs_with_detection(game, detection, Ratio::zero(), evade)?
        .leader
        .max(expected_payoffs_with_detection(game, detection, Ratio::one(), evade)?.leader)
        .sub(current.leader)?;
    let follower = expected_payoffs_with_detection(game, detection, inspect, Ratio::zero())?
        .follower
        .max(expected_payoffs_with_detection(game, detection, inspect, Ratio::one())?.follower)
        .sub(current.follower)?;
    Ok(Gains { leader, follower })
}

/// Follower best response and both payoffs at one feasible commitment.
///
/// # Errors
///
/// Returns [`Error::ProbabilityOutOfRange`] for a probability outside `[0, 1]`
/// and [`Error::BudgetExceeded`] when the commitment exceeds the budget.
pub fn commitment_at(
    game: Parameters,
    inspect: Ratio,
    budget: Ratio,
    tie: FollowerTie,
) -> Result<(Ratio, Payoffs), Error> {
    commitment_at_with_detection(game, Ratio::one(), inspect, budget, tie)
}

/// Commitment response when detection may be imperfect.
///
/// # Errors
///
/// Returns [`Error::DetectionOutOfRange`], [`Error::ProbabilityOutOfRange`], or
/// [`Error::BudgetExceeded`].
pub fn commitment_at_with_detection(
    game: Parameters,
    detection: Ratio,
    inspect: Ratio,
    budget: Ratio,
    tie: FollowerTie,
) -> Result<(Ratio, Payoffs), Error> {
    require_probability(budget)?;
    require_probability(inspect)?;
    if inspect > budget {
        return Err(Error::BudgetExceeded);
    }
    let comply = (
        Ratio::zero(),
        expected_payoffs_with_detection(game, detection, inspect, Ratio::zero())?,
    );
    let evade = (
        Ratio::one(),
        expected_payoffs_with_detection(game, detection, inspect, Ratio::one())?,
    );
    let mut options = Vec::with_capacity(2);
    match comply.1.follower.cmp(&evade.1.follower) {
        Ordering::Greater => options.push(comply),
        Ordering::Less => options.push(evade),
        Ordering::Equal => {
            options.push(comply);
            options.push(evade);
        }
    }
    options
        .first()
        .copied()
        .map(|_| select_tie(&options, tie))
        .ok_or(Error::InconsistentWitness)
}

/// Exact commitment value on `[0, budget]`, including a one-sided supremum.
///
/// Leader payoff is affine on each side of `g / (g + f)`. The value is the
/// best feasible endpoint, threshold, or one-sided threshold limit. It is
/// attained only when a feasible commitment reaches it.
///
/// # Errors
///
/// Returns [`Error::ProbabilityOutOfRange`] when `budget` is outside `[0, 1]`.
pub fn commitment_bound(
    game: Parameters,
    budget: Ratio,
    tie: FollowerTie,
) -> Result<Commitment, Error> {
    commitment_bound_with_detection(game, Ratio::one(), budget, tie)
}

/// Commitment bound for a detection probability in `(0, 1]`.
///
/// The follower's indifference point is `g / (detection * (g + f))`. When that
/// point lies above 1, evasion is the unique response on the whole budget and
/// the bound is an endpoint. Perfect detection matches [`commitment_bound`].
///
/// # Errors
///
/// Returns [`Error::DetectionOutOfRange`] or [`Error::ProbabilityOutOfRange`].
pub fn commitment_bound_with_detection(
    game: Parameters,
    detection: Ratio,
    budget: Ratio,
    tie: FollowerTie,
) -> Result<Commitment, Error> {
    require_probability(budget)?;
    let threshold = detection_threshold(game, detection)?;
    let mut points = vec![Ratio::zero(), budget];
    if threshold <= budget {
        points.push(threshold);
    }
    points.sort_unstable();
    points.dedup();
    let attained = points
        .into_iter()
        .map(|inspect| {
            commitment_at_with_detection(game, detection, inspect, budget, tie)
                .map(|(evade, payoffs)| (inspect, evade, payoffs))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut values: Vec<Ratio> = attained
        .iter()
        .map(|(_, _, payoffs)| payoffs.leader)
        .collect();
    if threshold <= budget {
        values.push(
            expected_payoffs_with_detection(game, detection, threshold, Ratio::one())?.leader,
        );
    }
    if threshold < budget {
        values.push(
            expected_payoffs_with_detection(game, detection, threshold, Ratio::zero())?.leader,
        );
    }
    let value = values.into_iter().max().ok_or(Error::InconsistentWitness)?;
    let witness = attained
        .into_iter()
        .filter(|(_, _, payoffs)| payoffs.leader == value)
        .min_by_key(|(inspect, _, _)| *inspect);
    Ok(match witness {
        Some((inspect, evade, _)) => Commitment {
            value,
            attained: true,
            inspect: Some(inspect),
            evade: Some(evade),
        },
        None => Commitment {
            value,
            attained: false,
            inspect: None,
            evade: None,
        },
    })
}

/// Commitment value while one parameter ranges over a closed positive interval.
///
/// When the follower's threshold `θ` lies below the budget, the competing
/// leader values are `-h` for skipping and `-cθ` for deterrence. When `θ`
/// exceeds the budget, the follower evades throughout the feasible interval
/// and the sign of `h-c` selects its optimal endpoint. Each comparison is
/// monotone in any one parameter, so matching strict choices give sharp
/// coordinate bounds.
///
/// # Errors
///
/// Returns [`Error::CommitmentRegimeChange`] if the interval includes `θ = B`,
/// `cθ = h` with `θ < B`, or `h = c` with `θ > B`, where `B` is the budget.
/// Interval, budget and exact arithmetic errors propagate.
pub fn commitment_projection(
    baseline: Parameters,
    interval: ParameterInterval,
    budget: Ratio,
    tie: FollowerTie,
) -> Result<CommitmentProjection, Error> {
    admit_interval(interval)?;
    require_probability(budget)?;
    let lower_game = with_varied(baseline, interval.parameter, interval.lower)?;
    let upper_game = with_varied(baseline, interval.parameter, interval.upper)?;
    project_commitment(
        lower_game,
        upper_game,
        Ratio::one(),
        Ratio::one(),
        budget,
        tie,
    )
}

/// Commitment value while detection ranges over a closed subinterval of `(0, 1]`.
///
/// Uses the same strict comparisons as [`commitment_projection`], with threshold
/// `θ = g/(d(g+f))` and evasion-branch slope `d h - c`. Both the comparison
/// of `cθ` with `h` and the slope are monotone in detection `d`.
///
/// # Errors
///
/// Returns [`Error::CommitmentRegimeChange`], [`Error::DetectionOutOfRange`],
/// [`Error::UnorderedInterval`], [`Error::ProbabilityOutOfRange`], or [`Error::Overflow`].
pub fn detection_commitment_projection(
    baseline: Parameters,
    detection_lower: Ratio,
    detection_upper: Ratio,
    budget: Ratio,
    tie: FollowerTie,
) -> Result<CommitmentProjection, Error> {
    require_detection(detection_lower)?;
    require_detection(detection_upper)?;
    if detection_lower > detection_upper {
        return Err(Error::UnorderedInterval);
    }
    require_probability(budget)?;
    project_commitment(
        baseline,
        baseline,
        detection_lower,
        detection_upper,
        budget,
        tie,
    )
}

fn project_commitment(
    lower_game: Parameters,
    upper_game: Parameters,
    lower_detection: Ratio,
    upper_detection: Ratio,
    budget: Ratio,
    tie: FollowerTie,
) -> Result<CommitmentProjection, Error> {
    let lower_piece = commitment_piece(lower_game, lower_detection, budget)?;
    let upper_piece = commitment_piece(upper_game, upper_detection, budget)?;
    if lower_piece != upper_piece {
        return Err(Error::CommitmentRegimeChange);
    }
    let lower = commitment_bound_with_detection(lower_game, lower_detection, budget, tie)?;
    let upper = commitment_bound_with_detection(upper_game, upper_detection, budget, tie)?;
    if lower.attained != upper.attained {
        return Err(Error::CommitmentRegimeChange);
    }
    let value = Span::cover(lower.value, upper.value).bounds();
    if !lower.attained {
        return Ok(CommitmentProjection {
            value,
            attained: false,
            inspect: None,
            evade: None,
        });
    }
    Ok(CommitmentProjection {
        value,
        attained: true,
        inspect: Some(
            Span::cover(
                lower.inspect.ok_or(Error::InconsistentWitness)?,
                upper.inspect.ok_or(Error::InconsistentWitness)?,
            )
            .bounds(),
        ),
        evade: Some(
            Span::cover(
                lower.evade.ok_or(Error::InconsistentWitness)?,
                upper.evade.ok_or(Error::InconsistentWitness)?,
            )
            .bounds(),
        ),
    })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CommitmentPiece {
    DeterrencePreferred,
    SkippingPreferred,
    EvadeIncreasing,
    EvadeDecreasing,
}

fn commitment_piece(
    game: Parameters,
    detection: Ratio,
    budget: Ratio,
) -> Result<CommitmentPiece, Error> {
    let theta = detection_threshold(game, detection)?;
    if theta < budget {
        let deterrence_value = game.inspection_cost.neg()?.mul(theta)?;
        let skip_value = game.undetected_harm.neg()?;
        return match deterrence_value.cmp(&skip_value) {
            Ordering::Greater => Ok(CommitmentPiece::DeterrencePreferred),
            Ordering::Less => Ok(CommitmentPiece::SkippingPreferred),
            Ordering::Equal => Err(Error::CommitmentRegimeChange),
        };
    }
    if theta == budget {
        return Err(Error::CommitmentRegimeChange);
    }
    let prevented_harm = detection.mul(game.undetected_harm)?;
    match prevented_harm.cmp(&game.inspection_cost) {
        Ordering::Greater => Ok(CommitmentPiece::EvadeIncreasing),
        Ordering::Less => Ok(CommitmentPiece::EvadeDecreasing),
        Ordering::Equal => Err(Error::CommitmentRegimeChange),
    }
}

fn threshold(game: Parameters) -> Result<Ratio, Error> {
    detection_threshold(game, Ratio::one())
}

fn detection_threshold(game: Parameters, detection: Ratio) -> Result<Ratio, Error> {
    require_detection(detection)?;
    game.evasion_gain
        .div(detection.mul(game.evasion_gain.add(game.detected_loss)?)?)
}

fn require_probability(value: Ratio) -> Result<(), Error> {
    if value.is_probability() {
        Ok(())
    } else {
        Err(Error::ProbabilityOutOfRange {
            numerator: value.numerator,
            denominator: value.denominator,
        })
    }
}

fn select_tie(options: &[(Ratio, Payoffs)], tie: FollowerTie) -> (Ratio, Payoffs) {
    let mut best = options[0];
    for candidate in options.iter().skip(1) {
        let order = candidate.1.leader.cmp(&best.1.leader);
        let replace = match tie {
            FollowerTie::Optimistic => {
                order == Ordering::Greater || (order == Ordering::Equal && candidate.0 < best.0)
            }
            FollowerTie::Pessimistic => {
                order == Ordering::Less || (order == Ordering::Equal && candidate.0 > best.0)
            }
        };
        if replace {
            best = *candidate;
        }
    }
    best
}

fn from_i128(mut numerator: i128, mut denominator: i128) -> Result<Ratio, Error> {
    if denominator == 0 {
        return Err(Error::ZeroDenominator);
    }
    if denominator < 0 {
        numerator = numerator.checked_neg().ok_or(Error::Overflow)?;
        denominator = denominator.checked_neg().ok_or(Error::Overflow)?;
    }
    let factor = gcd(numerator.checked_abs().ok_or(Error::Overflow)?, denominator);
    let numerator = i64::try_from(numerator / factor).map_err(|_| Error::Overflow)?;
    let denominator = i64::try_from(denominator / factor).map_err(|_| Error::Overflow)?;
    Ok(Ratio {
        numerator,
        denominator,
    })
}

fn gcd(mut left: i128, mut right: i128) -> i128 {
    while right != 0 {
        let next = left % right;
        left = right;
        right = next;
    }
    left
}

/// One strictly positive parameter of the inspection family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariedParameter {
    /// Inspection cost `c`.
    InspectionCost,
    /// Undetected harm `h`.
    UndetectedHarm,
    /// Undetected evasion gain `g`.
    EvasionGain,
    /// Detected loss `f`.
    DetectedLoss,
}

/// Closed interval for the parameter named by [`VariedParameter`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParameterInterval {
    /// Parameter that moves. The other three stay at the baseline.
    pub parameter: VariedParameter,
    /// Lower endpoint, strictly positive.
    pub lower: Ratio,
    /// Upper endpoint, strictly positive and at least `lower`.
    pub upper: Ratio,
}

/// Closed projection of one scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RatioBounds {
    /// Lower endpoint of this scalar projection.
    pub lower: Ratio,
    /// Upper endpoint of this scalar projection.
    pub upper: Ratio,
}

/// Coordinate-wise bounds on Nash action shares and payoffs.
///
/// Each endpoint is attained by some witness. A point inside the four intervals
/// at once is not claimed to be a witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NashProjection {
    /// Projection of the inspection probability.
    pub inspect: RatioBounds,
    /// Projection of the evasion probability.
    pub evade: RatioBounds,
    /// Projection of the leader payoff.
    pub leader: RatioBounds,
    /// Projection of the follower payoff.
    pub follower: RatioBounds,
    /// Gap between each reported endpoint and a witness that attains it.
    /// The closed form makes this zero. It is not a floating-point tolerance.
    pub endpoint_gap: Ratio,
}

/// Exact Nash projections while one parameter ranges over a closed interval.
///
/// Within each strict regime the witness is monotone, so the image is the span
/// of the regime endpoints. Where cost equals harm, the whole indifference
/// segment is included. The result is a projection, not a joint equilibrium set.
///
/// # Errors
///
/// Returns [`Error::NonPositiveParameter`], [`Error::UnorderedInterval`], or
/// [`Error::Overflow`].
pub fn nash_projection(
    baseline: Parameters,
    interval: ParameterInterval,
) -> Result<NashProjection, Error> {
    if interval.lower <= Ratio::zero() || interval.upper <= Ratio::zero() {
        return Err(Error::NonPositiveParameter);
    }
    if interval.lower > interval.upper {
        return Err(Error::UnorderedInterval);
    }
    let image = match interval.parameter {
        VariedParameter::InspectionCost => {
            project_threshold(baseline, interval, baseline.undetected_harm, true)?
        }
        VariedParameter::UndetectedHarm => {
            project_threshold(baseline, interval, baseline.inspection_cost, false)?
        }
        VariedParameter::EvasionGain | VariedParameter::DetectedLoss => {
            project_without_threshold(baseline, interval)?
        }
    };
    Ok(NashProjection {
        inspect: image.inspect.bounds(),
        evade: image.evade.bounds(),
        leader: image.leader.bounds(),
        follower: image.follower.bounds(),
        endpoint_gap: Ratio::zero(),
    })
}

/// Exact Nash projection on a rectangle that stays inside one strict regime.
///
/// The four corners are solved. Inside the interior regime and inside the
/// pure-skip regime, each coordinate is monotone, so the corner span is the
/// image. A rectangle that reaches equal cost and harm, or that mixes the two
/// regimes, is rejected. The result remains a projection.
///
/// # Errors
///
/// Returns [`Error::RepeatedParameter`], [`Error::RectangleRegimeChange`],
/// [`Error::LeaderIndifferent`], or the interval admission errors.
pub fn nash_rectangle(
    baseline: Parameters,
    first: ParameterInterval,
    second: ParameterInterval,
) -> Result<NashProjection, Error> {
    admit_interval(first)?;
    admit_interval(second)?;
    if first.parameter == second.parameter {
        return Err(Error::RepeatedParameter);
    }
    let mut image: Option<Image> = None;
    let mut pure_skip: Option<bool> = None;
    for (left, right) in [
        (first.lower, second.lower),
        (first.lower, second.upper),
        (first.upper, second.lower),
        (first.upper, second.upper),
    ] {
        let game = with_varied(
            with_varied(baseline, first.parameter, left)?,
            second.parameter,
            right,
        )?;
        let witness = simultaneous_nash(game)?;
        let corner_is_pure = witness.inspect == Ratio::zero();
        match pure_skip {
            None => pure_skip = Some(corner_is_pure),
            Some(previous) if previous != corner_is_pure => {
                return Err(Error::RectangleRegimeChange);
            }
            Some(_) => {}
        }
        push_image(&mut image, witness_image(witness));
    }
    let image = image.ok_or(Error::InconsistentWitness)?;
    Ok(NashProjection {
        inspect: image.inspect.bounds(),
        evade: image.evade.bounds(),
        leader: image.leader.bounds(),
        follower: image.follower.bounds(),
        endpoint_gap: Ratio::zero(),
    })
}

/// Nash projection while detection and one other parameter vary together.
///
/// Every corner must stay in one strict regime: interior, pure skip, or pure
/// inspect. Those regimes are monotone in the two coordinates, so the corner
/// span is the image. A corner with no unique witness is returned as that
/// error instead of being clipped.
///
/// # Errors
///
/// Returns [`Error::RectangleRegimeChange`], [`Error::LeaderIndifferent`],
/// [`Error::DetectionOutOfRange`], or the interval admission errors.
pub fn detection_nash_rectangle(
    baseline: Parameters,
    parameter: ParameterInterval,
    detection_lower: Ratio,
    detection_upper: Ratio,
) -> Result<NashProjection, Error> {
    admit_interval(parameter)?;
    require_detection(detection_lower)?;
    require_detection(detection_upper)?;
    if detection_lower > detection_upper {
        return Err(Error::UnorderedInterval);
    }
    let mut image: Option<Image> = None;
    let mut regime: Option<DetectionRegime> = None;
    for (parameter_value, detection) in [
        (parameter.lower, detection_lower),
        (parameter.lower, detection_upper),
        (parameter.upper, detection_lower),
        (parameter.upper, detection_upper),
    ] {
        let game = with_varied(baseline, parameter.parameter, parameter_value)?;
        let witness = simultaneous_nash_with_detection(game, detection)?;
        let corner = detection_regime(witness);
        match regime {
            None => regime = Some(corner),
            Some(previous) if previous != corner => return Err(Error::RectangleRegimeChange),
            Some(_) => {}
        }
        push_image(&mut image, witness_image(witness));
    }
    let image = image.ok_or(Error::InconsistentWitness)?;
    Ok(NashProjection {
        inspect: image.inspect.bounds(),
        evade: image.evade.bounds(),
        leader: image.leader.bounds(),
        follower: image.follower.bounds(),
        endpoint_gap: Ratio::zero(),
    })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DetectionRegime {
    Interior,
    PureSkip,
    PureInspect,
}

fn detection_regime(witness: NashWitness) -> DetectionRegime {
    if witness.inspect == Ratio::zero() {
        DetectionRegime::PureSkip
    } else if witness.inspect == Ratio::one() {
        DetectionRegime::PureInspect
    } else {
        DetectionRegime::Interior
    }
}

fn admit_interval(interval: ParameterInterval) -> Result<(), Error> {
    if interval.lower <= Ratio::zero() || interval.upper <= Ratio::zero() {
        return Err(Error::NonPositiveParameter);
    }
    if interval.lower > interval.upper {
        return Err(Error::UnorderedInterval);
    }
    Ok(())
}

fn witness_image(witness: NashWitness) -> Image {
    Image {
        inspect: Span::point(witness.inspect),
        evade: Span::point(witness.evade),
        leader: Span::point(witness.payoffs.leader),
        follower: Span::point(witness.payoffs.follower),
    }
}

#[derive(Clone, Copy)]
struct Span {
    lower: Ratio,
    upper: Ratio,
}

impl Span {
    fn point(value: Ratio) -> Self {
        Self {
            lower: value,
            upper: value,
        }
    }

    fn cover(left: Ratio, right: Ratio) -> Self {
        if left <= right {
            Self {
                lower: left,
                upper: right,
            }
        } else {
            Self {
                lower: right,
                upper: left,
            }
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            lower: if self.lower <= other.lower {
                self.lower
            } else {
                other.lower
            },
            upper: if self.upper >= other.upper {
                self.upper
            } else {
                other.upper
            },
        }
    }

    fn bounds(self) -> RatioBounds {
        RatioBounds {
            lower: self.lower,
            upper: self.upper,
        }
    }
}

#[derive(Clone, Copy)]
struct Image {
    inspect: Span,
    evade: Span,
    leader: Span,
    follower: Span,
}

impl Image {
    fn union(self, other: Self) -> Self {
        Self {
            inspect: self.inspect.union(other.inspect),
            evade: self.evade.union(other.evade),
            leader: self.leader.union(other.leader),
            follower: self.follower.union(other.follower),
        }
    }
}

fn with_varied(
    baseline: Parameters,
    parameter: VariedParameter,
    value: Ratio,
) -> Result<Parameters, Error> {
    let (mut cost, mut harm, mut gain, mut loss) = (
        baseline.inspection_cost,
        baseline.undetected_harm,
        baseline.evasion_gain,
        baseline.detected_loss,
    );
    match parameter {
        VariedParameter::InspectionCost => cost = value,
        VariedParameter::UndetectedHarm => harm = value,
        VariedParameter::EvasionGain => gain = value,
        VariedParameter::DetectedLoss => loss = value,
    }
    Parameters::try_new(cost, harm, gain, loss)
}

fn unique_image(game: Parameters) -> Result<Image, Error> {
    let witness = simultaneous_nash(game)?;
    Ok(Image {
        inspect: Span::point(witness.inspect),
        evade: Span::point(witness.evade),
        leader: Span::point(witness.payoffs.leader),
        follower: Span::point(witness.payoffs.follower),
    })
}

fn segment_image(game: Parameters) -> Result<Image, Error> {
    let theta = threshold(game)?;
    Ok(Image {
        inspect: Span::cover(Ratio::zero(), theta),
        evade: Span::point(Ratio::one()),
        leader: Span::point(game.inspection_cost.neg()?),
        follower: Span::cover(Ratio::zero(), game.evasion_gain),
    })
}

fn unique_at(
    baseline: Parameters,
    parameter: VariedParameter,
    value: Ratio,
) -> Result<Image, Error> {
    unique_image(with_varied(baseline, parameter, value)?)
}

fn project_without_threshold(
    baseline: Parameters,
    interval: ParameterInterval,
) -> Result<Image, Error> {
    if baseline.inspection_cost == baseline.undetected_harm {
        let lower = segment_image(with_varied(baseline, interval.parameter, interval.lower)?)?;
        let upper = segment_image(with_varied(baseline, interval.parameter, interval.upper)?)?;
        return Ok(lower.union(upper));
    }
    Ok(
        unique_at(baseline, interval.parameter, interval.lower)?.union(unique_at(
            baseline,
            interval.parameter,
            interval.upper,
        )?),
    )
}

fn project_threshold(
    baseline: Parameters,
    interval: ParameterInterval,
    boundary: Ratio,
    interior_below: bool,
) -> Result<Image, Error> {
    let interior = |value: Ratio| {
        if interior_below {
            value < boundary
        } else {
            value > boundary
        }
    };
    let dominated = |value: Ratio| {
        if interior_below {
            value > boundary
        } else {
            value < boundary
        }
    };
    let mut image: Option<Image> = None;
    if interior(interval.lower) {
        push_image(
            &mut image,
            unique_at(baseline, interval.parameter, interval.lower)?,
        );
    }
    if interior(interval.upper) {
        push_image(
            &mut image,
            unique_at(baseline, interval.parameter, interval.upper)?,
        );
    }
    if interval.lower <= boundary && boundary <= interval.upper {
        push_image(
            &mut image,
            segment_image(with_varied(baseline, interval.parameter, boundary)?)?,
        );
    }
    if dominated(interval.lower) {
        push_image(
            &mut image,
            unique_at(baseline, interval.parameter, interval.lower)?,
        );
    }
    if dominated(interval.upper) {
        push_image(
            &mut image,
            unique_at(baseline, interval.parameter, interval.upper)?,
        );
    }
    image.ok_or(Error::InconsistentWitness)
}

fn push_image(image: &mut Option<Image>, next: Image) {
    *image = Some(match image.take() {
        Some(current) => current.union(next),
        None => next,
    });
}
