/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`LadderOutcome`]: the verdict on a refinement ladder, with non-convergence as a first-class
//! result rather than a silent pass.
//!
//! A ladder sweeps one parameter — bond cap, penalization parameter, mask smoothing width — and
//! reports an observable at each rung. The question a gate must answer is whether the sequence is
//! settling. Two failure modes are usually conflated:
//!
//! - the observable *is* settling but has not reached the bound yet, and
//! - the observable is not settling at all, so no bound is meaningful.
//!
//! Collapsing both into a boolean forces a harness to either pass a drifting quantity or fail a
//! quantity that is merely under-refined. A ladder that does not converge is a measurement result
//! in its own right — reporting it is more useful than reporting a pass — so it gets its own
//! variant, and the caller decides the exit policy.
//!
//! Deltas are absolute successive differences; the judgement is made on the finest pair, with the
//! trend across the whole ladder used to explain *why* a sequence failed.

use core::fmt;

/// The minimum number of rungs needed to judge a trend: two differences.
const MIN_RUNGS: usize = 3;

/// The verdict on a refinement ladder.
#[derive(Debug, Clone, PartialEq)]
pub enum LadderOutcome {
    /// The finest successive difference is within tolerance.
    Converged {
        /// The absolute difference between the two finest rungs.
        final_delta: f64,
    },
    /// The finest successive difference exceeds tolerance. The observable may still be drifting,
    /// in which case no bound on its absolute value is meaningful.
    NotConverging {
        /// The absolute difference between the two finest rungs.
        final_delta: f64,
        /// Why the ladder failed: shrinking but not yet inside tolerance, or not shrinking at all.
        detail: String,
    },
    /// Too few rungs to judge a trend, or a non-finite value in the sequence.
    Indeterminate {
        /// What prevented a judgement.
        detail: String,
    },
}

impl LadderOutcome {
    /// Judge a ladder from its rung values, finest last, against `tol` on the finest successive
    /// difference.
    ///
    /// Returns [`Indeterminate`](Self::Indeterminate) for fewer than three rungs or any non-finite
    /// value: a two-rung ladder has one difference and therefore no trend, and a NaN makes every
    /// comparison false, which would otherwise read as convergence.
    pub fn judge(values: &[f64], tol: f64) -> Self {
        if values.len() < MIN_RUNGS {
            return Self::Indeterminate {
                detail: format!(
                    "{} rung(s): need at least {MIN_RUNGS} to judge a trend",
                    values.len()
                ),
            };
        }
        if let Some(bad) = values.iter().position(|v| !v.is_finite()) {
            return Self::Indeterminate {
                detail: format!("non-finite value {} at rung {bad}", values[bad]),
            };
        }
        if !tol.is_finite() || tol <= 0.0 {
            return Self::Indeterminate {
                detail: format!("tolerance {tol} is not a positive finite number"),
            };
        }

        // Signed steps, so a direction reversal is visible; `deltas` is their magnitude.
        let steps: Vec<f64> = values.windows(2).map(|w| w[1] - w[0]).collect();
        // `steps` has at least two entries: `values.len() >= MIN_RUNGS`.
        let final_delta = steps[steps.len() - 1].abs();
        let prev_delta = steps[steps.len() - 2].abs();

        if final_delta <= tol {
            return Self::Converged { final_delta };
        }

        // Outside tolerance. Explain why, because only some of these say the observable has no
        // limit this ladder can reach.
        //
        // A reversal is checked before delta shrinkage: a sequence that rises then falls can have
        // a shrinking final delta while approaching nothing, which is exactly the shape of a
        // penalization ladder whose observable is still being driven by the parameter rather than
        // settling under it. Reporting that as "shrinking, refine further" would understate it.
        let reverses = steps
            .windows(2)
            .any(|w| w[0] > 0.0 && w[1] < 0.0 || w[0] < 0.0 && w[1] > 0.0);

        let detail = if reverses {
            format!(
                "not settling: the sequence reverses direction (non-monotone) with \
                 |Δ| {final_delta:.3e} > {tol:.3e} — no limit is demonstrated, so a bound on the \
                 absolute value is not meaningful"
            )
        } else if final_delta < prev_delta {
            format!(
                "shrinking but outside tolerance: |Δ| {final_delta:.3e} > {tol:.3e} \
                 (previous |Δ| {prev_delta:.3e}) — refine further"
            )
        } else {
            format!(
                "not settling: |Δ| {final_delta:.3e} did not shrink against the previous \
                 |Δ| {prev_delta:.3e} — no limit is demonstrated, so a bound on the absolute \
                 value is not meaningful"
            )
        };
        Self::NotConverging {
            final_delta,
            detail,
        }
    }

    /// Whether the ladder converged. Both [`NotConverging`](Self::NotConverging) and
    /// [`Indeterminate`](Self::Indeterminate) are false — neither establishes a limit.
    pub fn is_converged(&self) -> bool {
        matches!(self, Self::Converged { .. })
    }

    /// Whether the ladder was judged and found not to settle. Distinct from
    /// [`is_converged`](Self::is_converged) being false, which a lack of rungs also produces.
    pub fn is_not_converging(&self) -> bool {
        matches!(self, Self::NotConverging { .. })
    }

    /// The finest successive difference, when one was computed.
    pub fn final_delta(&self) -> Option<f64> {
        match self {
            Self::Converged { final_delta } | Self::NotConverging { final_delta, .. } => {
                Some(*final_delta)
            }
            Self::Indeterminate { .. } => None,
        }
    }
}

impl fmt::Display for LadderOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Converged { final_delta } => {
                write!(f, "converged (|Δ| {final_delta:.3e})")
            }
            Self::NotConverging { detail, .. } => write!(f, "NOT CONVERGING — {detail}"),
            Self::Indeterminate { detail } => write!(f, "indeterminate — {detail}"),
        }
    }
}
