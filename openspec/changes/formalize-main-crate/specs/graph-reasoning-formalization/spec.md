<!--
Synced 2026-07-10. The original two requirements are OBSOLETE as written: (1) "engine as a
Free::fold catamorphism" is DELIVERED by equivalent — `core.causal_effect.fold_universal` (fold is
the unique handler), `core.causal_effect.relay_termination` (fuel-bounded total relay handler),
and `core.causaloid.graph_fold_order_invariant` (the engine as a schedule-invariant dataflow
fold), now in the main specs via `core-formalization` and `causaloid-graph-algebra`;
(2) "Reconvergence merge (∇) is out of scope" is FALSE since Stage 4 landed the defined merge
`∇ ∘ (Λ₁ ⊗ Λ₂)` with `∇ = Verdict::join` (corpus-gated). What REMAINS in this change is the one
residual theorem below.
-->

## ADDED Requirements

### Requirement: Relay-round composition is sequential Kleisli composition

The formalization SHALL state and prove the cross-round equation of adaptive reasoning: a
multi-round evaluation — where a round ends in `RelayTo(target, sub)` and the next round starts at
`target` seeded with `sub` — is the **sequential (Kleisli) composition of its rounds**, so the
per-round correctness theorems (`core.causaloid.graph_fold_order_invariant` within a round;
`core.causal_effect.relay_termination` for the fuel bound) compose to a statement about the whole
adaptive run: one composite arrow, total under the fuel bound, with state/context/log threading
across the round boundary matching the engine. The theorem SHALL carry a `THEOREM_MAP.md` row and
a Rust witness exercising a two-round relay on the real engine.

#### Scenario: Two rounds compose as one arrow

- **WHEN** a graph evaluation relays once (round 1 ends in `RelayTo`, round 2 runs from the target
  with the sub-program as seed)
- **THEN** the model proves the two-round result equals the Kleisli composite of the two
  single-round evaluations, and the Rust witness confirms it (value, logs concatenated across the
  boundary, fuel decremented once)

#### Scenario: The fuel bound composes

- **WHEN** the per-round fuel theorem is composed across rounds
- **THEN** the whole adaptive run is total with at most `MAX_RELAY_ROUNDS` rounds, inheriting
  `core.causal_effect.relay_termination` — no new termination argument is introduced
