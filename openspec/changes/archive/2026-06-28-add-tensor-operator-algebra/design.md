## Context

`CausalTensorTrainOperator` (MPO) is a rank-4-core chain. It already realizes the multiplicative side of
an operator algebra (`compose` = product, `identity` = one) but exposes no additive operations, while
the state type `CausalTensorTrain` is already a full `AddGroup` / `Module` / `Ring`. The missing additive
operators are the foundational primitive for assembling differential MPOs in the CFD ↔ QTT bridge.

## Goals / Non-Goals

Goals: complete the additive operator algebra (`add` / `sub` / `neg` / `scale`) with minimal, reused
code and full tests, at the right abstraction level (the tensor crate, not the CFD crate).

Non-Goals: implementing the `AddGroup` / `Module` / `Ring` *trait* impls for the operator (inherent
methods suffice and mirror `CausalTensorTrain::scale`, which is inherent to avoid `Module::scale`
collision); a closed-form Laplacian MPO (a CFD-bridge concern, kept as a later optimization); any change
to `compose` / `apply` / `round` / defaults.

## Decisions

- **Delegate to the combined-index view.** The internal `as_combined_train()` / `from_combined_train()`
  already bridge an MPO to a `CausalTensorTrain` over the merged `(out, in)` index, where `add` (trait)
  and `scale` (inherent) exist. `add`/`scale` wrap that bridge; `neg = scale(0 − 1)`;
  `sub = add(other.neg())`. ~30 lines total, no new algorithms.
- **Inherent methods, not trait.** Mirrors the existing `scale` precedent and avoids any `Module` method
  collision.
- **Bond growth is explicit.** `add`/`sub` concatenate bonds (`r ← rₐ + r_b`); the caller rounds to
  recompress — consistent with how `compose` already inflates and `round` restores. `scale`/`neg` are
  rank-preserving.
- **Dimension guard.** `add`/`sub` return `CausalTensorError::ShapeMismatch` on mismatched in/out dims.

## Risks / Trade-offs

Minimal. The only behavioural note is the additive bond growth, which is expected and documented; the
result is exact (a round is the caller's choice, not forced).
