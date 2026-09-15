/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::ratio::lift_count;
use crate::{LeafOrdinals, SampleSession, UncertainBool, UncertainError};
use deep_causality_rand::RandScalar;

/// Implements the Sequential Probability Ratio Test (SPRT) to evaluate a hypothesis
/// about an uncertain boolean value.
///
/// H0: P(true) <= threshold - epsilon
/// H1: P(true) > threshold + epsilon
///
/// Returns `true` if H1 is accepted, `false` if H0 is accepted.
///
/// # The scalar
///
/// Every quantity below is `R`: the threshold, the confidence, the indifference region, the two
/// hypothesis probabilities, the log-likelihood ratio and the boundaries it is compared against.
/// A probability is dimensionless and the caller states it in their own scalar, so evaluating the
/// test anywhere else would mean narrowing the caller's input and widening the answer back — a
/// round trip that changes the decision at the margins and buys nothing.
///
/// A narrow scalar makes the test coarser rather than wrong: the boundaries are further apart in
/// units of the scalar's resolution, so the decision is reached on the same evidence and reported
/// to the precision the caller asked for.
///
/// # Parameters
/// - `uncertain_bool`: The `UncertainBool<R>` value to test.
/// - `threshold`: The probability threshold to test against (e.g. `0.5` for "more likely than not").
/// - `confidence`: The desired confidence level (e.g. `0.95`).
/// - `epsilon`: The indifference region. Samples within `threshold ± epsilon` are inconclusive.
/// - `max_samples`: The maximum number of samples to draw before making a fallback decision.
/// - `initial_sample_index`: The session index the first draw is taken at.
pub fn evaluate_hypothesis<R: RandScalar>(
    uncertain_bool: &UncertainBool<R>,
    session: &SampleSession,
    threshold: R,
    confidence: R,
    epsilon: R,
    max_samples: usize,
    initial_sample_index: u64,
) -> Result<bool, UncertainError> {
    let one = R::one();
    let zero = R::zero();
    let eps = R::epsilon();

    // Set alpha and beta for the test
    let alpha_error = one - confidence; // Type I error (false positive rate)
    let beta_error = alpha_error; // Type II error (false negative rate), often set equal to alpha

    // Calculate SPRT boundaries. Avoid log(0) by clamping probabilities.
    let a_boundary = (beta_error / (one - alpha_error)).ln();
    let b_boundary = ((one - beta_error) / alpha_error).ln();

    // Set indifference region (p0 and p1)
    let p0 = clamp(threshold - epsilon, zero, one - eps); // Null hypothesis probability
    let p1 = clamp(threshold + epsilon, eps, one); // Alternative hypothesis probability

    // Assigned once: the graph does not change while the test runs, and the loop draws up to
    // `max_samples` times.
    let ordinals = LeafOrdinals::for_bool(uncertain_bool);

    let mut successes = 0usize;
    let mut samples_drawn = 0usize;

    // Batch sampling (as per paper, but simplified for now)
    let batch_size = 10;

    while samples_drawn < max_samples {
        let current_batch_size = (batch_size).min(max_samples - samples_drawn);
        if current_batch_size == 0 {
            break;
        } // Avoid infinite loop if max_samples is reached

        for _ in 0..current_batch_size {
            let sample_result = uncertain_bool.sample_at_with(
                session,
                initial_sample_index + samples_drawn as u64,
                &ordinals,
            )?;
            if sample_result {
                successes += 1;
            }
            samples_drawn += 1;
        }

        // Compute log-likelihood ratio (LLR)
        let n = lift_count::<R>(samples_drawn)?;
        let x = lift_count::<R>(successes)?;

        // Avoid log(0) or log(negative)
        let term1 = if p1 > eps && p0 > eps {
            (p1 / p0).ln()
        } else {
            zero
        };
        let term2 = if (one - p1) > eps && (one - p0) > eps {
            ((one - p1) / (one - p0)).ln()
        } else {
            zero
        };

        let llr = x * term1 + (n - x) * term2;

        if llr <= a_boundary {
            // Accept H0: P(true) <= threshold
            return Ok(false);
        } else if llr >= b_boundary {
            // Accept H1: P(true) > threshold
            return Ok(true);
        }
    }

    // Fallback decision if max_samples reached without clear conclusion
    let final_p = crate::ratio::<R>(successes, samples_drawn)?;
    Ok(final_p > threshold)
}

/// `value` held within `[low, high]`.
///
/// Written out rather than taken from the scalar, because `RandScalar` exposes the order but no
/// `clamp`, and a scalar added tomorrow would have to supply one. Comparison is all this needs.
fn clamp<R: RandScalar>(value: R, low: R, high: R) -> R {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}
