# Predict / Correct / Verify Inside the Causal Monad

## Introduction

This example runs the skeleton of a Kalman filter inside the causal monad. A Kalman filter estimates a hidden state: a model predicts how the state evolves, and noisy sensor readings correct the prediction. GPS receivers, inertial navigation, robot localization, autopilots, VR head-tracking, missile guidance, sensor fusion in self-driving cars, and trading-signal smoothing all run on this loop.

`predict` is the model: where the state went since the last tick. `correct` is the measurement update: it nudges the prediction toward what the sensor saw. `verify` is the sanity gate: did anything blow up. A full Kalman filter also carries a covariance matrix alongside the mean, but the pipeline is the same: predict, correct, check, repeat. `bind` does the bookkeeping: it threads the state through the steps and stops at the first failure.

A 2-vector state passes through three monadic steps: a tensor matrix-multiply (predict), a Clifford rotor (correct), and a NaN check (verify). Each step is a `bind` on `CausalEffectPropagationProcess`.

## How to Run

```bash
cargo run -p mathematics_examples --example effect_kalman_predict_correct_examples
```

## What It Demonstrates

Tensor and Clifford algebra share one monadic spine. The intermediate state has type `Process<CausalTensor<FloatType>>`; the closures swap between tensor-style and multivector-style computation, and `bind` carries the value across each swap.

The monad handles the error channel. If `predict` returned `fail(...)`, the rest of the chain would short-circuit and `result.error()` would carry the cause. The step log accumulates across all `bind` calls and produces the trace.

## Mathematical Content

- Predict: `x' = F x` where `F` is a 2D rotation matrix.
- Correct: apply an additional rotor `R x R~` in `Cl(2,0)`.
- Verify: assert finiteness of the resulting components.

## What This Example Skips

A production filter also tracks a covariance alongside the mean state; those pieces go inside the existing `bind` chain without changing its structure. The example omits:

- **Covariance matrix `P`.** A Kalman filter carries an `n x n` symmetric matrix alongside the state, expressing its confidence in each component and the correlations between them. Promote the carried value from `CausalTensor<FloatType>` to a `(mean, covariance)` pair.
- **Process noise `Q`.** Every predict step inflates the covariance to model the uncertainty added by the dynamics. Update rule: `P_pred = F P F^T + Q`.
- **Measurement noise `R`.** Every correct step uses it to weigh the new measurement against the prior.
- **Innovation and Kalman gain.** The correct step is `K = P_pred H^T (H P_pred H^T + R)^-1`, then `x_new = x_pred + K (z - H x_pred)`, then `P_new = (I - K H) P_pred`. In this example, the rotor stands in for the gain application.
- **Joseph form for the covariance update.** `P_new = (I - K H) P_pred (I - K H)^T + K R K^T` is numerically more stable than the simple form above and prevents `P` from losing positive-definiteness under finite-precision arithmetic.
- **Outlier rejection.** A chi-squared gate on the innovation (`(z - H x_pred)^T S^-1 (z - H x_pred) < threshold`) discards measurements that disagree too strongly with the prediction. Adding it is a fourth `bind` between correct and verify.
- **Square-root or UD factored form.** For long runs or ill-conditioned dynamics, `P` is stored as `S` such that `S S^T = P` to keep half the digit loss.
- **Multivariate state.** Real applications carry position, velocity, orientation, bias states, often 9 to 30 dimensions. The matrix algebra grows; the pipeline shape does not.

Adding any of the above is a local edit to `predict`, `correct` or `verify` plus a richer carried-value type; the monadic chain stays the same.

## Key APIs

- `CausalEffectPropagationProcessWitness::pure` and `bind`
- `EinSumOp::mat_mul` for the linear predict step
- `CausalMultiVector::geometric_product` for the algebraic correct step
- `effect_helpers::Process`, `ProcessWitness`, `ok`, `fail`

## Adaptation

- Inject a deliberate NaN in `predict` to observe the short-circuit.
- Add a fourth step that runs a Kalman gain update.
- Replace `Process<CausalTensor<FloatType>>` with a richer state type carrying the covariance.
