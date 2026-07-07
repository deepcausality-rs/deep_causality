# Predict / Correct / Verify Inside the Causal Monad

## Introduction

This is the shape of a Kalman filter, stripped to its skeleton. A Kalman filter is the standard tool whenever you have a system whose state you cannot measure directly, but you have a model that predicts how it should evolve plus noisy sensor readings that correct the prediction. GPS receivers, inertial navigation, robot localization, autopilots, head-tracking in VR, missile guidance, sensor fusion in self-driving cars, and trading-signal smoothing in quant finance all run on this loop.

The example keeps the two steps recognizable. `predict` is the model: where did the state go since the last tick. `correct` is the measurement update: how do we nudge the prediction toward what the sensor saw. `verify` is the sanity gate: did anything blow up. Real Kalman filters have richer state (a covariance matrix in addition to the mean), but the pipeline structure is the same: predict, correct, check, repeat. The point of this example is to show that the bookkeeping of "thread state through these steps and stop if something fails" is handled by `bind` rather than by the engineer.

A 2-vector state passes through three monadic steps: a tensor matrix-multiply (predict), a Clifford rotor (correct), and a NaN check (verify). Each step is a `bind` on `CausalEffectPropagationProcess`.

## How to Run

```bash
cargo run -p mathematics_examples --example effect_kalman_predict_correct_examples
```

## What It Demonstrates

Tensor and Clifford algebra share a single monadic spine. The intermediate state has type `Process<CausalTensor<f64>>`; the closures swap between tensor-style and multivector-style computation without unwrapping anything by hand.

The error channel is handled by the monad. If `predict` returned `fail(...)`, the rest of the chain would short-circuit and `after_verify.error()` would carry the cause. The step log accumulates across all `bind` calls; that is how the trace is produced.

## Mathematical Content

- Predict: `x' = F x` where `F` is a 2D rotation matrix.
- Correct: apply an additional rotor `R x R~` in `Cl(2,0)`.
- Verify: assert finiteness of the resulting components.

## What This Example Skips

This example shows the predict / correct / verify skeleton, not a full Kalman filter. A production filter tracks more than the mean state. A skilled engineer can add the missing pieces directly inside the existing `bind` chain; the structure does not have to change.

Specifically, the example omits:

- **Covariance matrix `P`.** A Kalman filter carries an `n x n` symmetric matrix alongside the state. It expresses how confident the filter is in each component and the correlations between them. Promote the carried value from `CausalTensor<FloatType>` to a `(mean, covariance)` pair.
- **Process noise `Q`.** Every predict step inflates the covariance to model the uncertainty added by the dynamics. Update rule: `P_pred = F P F^T + Q`.
- **Measurement noise `R`.** Every correct step uses this to weight the trust given to the new measurement against the trust in the prior.
- **Innovation and Kalman gain.** The correct step is `K = P_pred H^T (H P_pred H^T + R)^-1`, then `x_new = x_pred + K (z - H x_pred)`, then `P_new = (I - K H) P_pred`. In this example, the rotor stands in for the gain application but does not compute a gain.
- **Joseph form for the covariance update.** `P_new = (I - K H) P_pred (I - K H)^T + K R K^T` is numerically more stable than the simple form above and prevents `P` from losing positive-definiteness under finite-precision arithmetic.
- **Outlier rejection.** A chi-squared gate on the innovation (`(z - H x_pred)^T S^-1 (z - H x_pred) < threshold`) discards measurements that disagree too strongly with the prediction. Adding it is a fourth `bind` between correct and verify.
- **Square-root or UD factored form.** For long runs or ill-conditioned dynamics, `P` is stored as `S` such that `S S^T = P` to keep half the digit loss.
- **Multivariate state.** Real applications carry position, velocity, orientation, bias states, often 9 to 30 dimensions. The matrix algebra grows; the pipeline shape does not.

Adding any of the above is a local edit to one of `predict`, `correct`, or `verify` plus a richer carried-value type. The monadic chain itself stays the same. That is the property the example was built to expose.

## Key APIs

- `CausalEffectPropagationProcessWitness::pure` and `bind`
- `EinSumOp::mat_mul` for the linear predict step
- `CausalMultiVector::geometric_product` for the algebraic correct step
- `effect_helpers::Process`, `ProcessWitness`, `ok`, `fail`

## Adaptation

- Inject a deliberate NaN in `predict` to observe the short-circuit.
- Add a fourth step that runs a Kalman gain update.
- Replace `Process<CausalTensor<f64>>` with a richer state type carrying the covariance.
