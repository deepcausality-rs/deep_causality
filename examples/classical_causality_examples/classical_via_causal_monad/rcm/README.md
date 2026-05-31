# RCM via the Causal Monad

This example mirrors the [`classical_via_causaloid/rcm`](../../classical_via_causaloid/rcm) example but is implemented directly on the carrier `PropagatingProcess<f64, (), TreatmentContext>` using the [`Alternatable`](https://docs.rs/deep_causality_core/latest/deep_causality_core/trait.Alternatable.html) family. Same estimand, same numbers, different mechanism.

## How to run

```bash
cargo run -p classical_causality_examples --example rcm_via_monad
```

## The estimand

Rubin's potential-outcomes definition of a causal effect:

```
ITE = Y(do(T = 1)) - Y(do(T = 0))
```

For a single patient with baseline BP, what is the BP change caused by administering the drug versus not administering it?

## The mechanism

1. **Treatment assignment lives in the Context.** `TreatmentContext { drug_administered, drug_effect_if_administered }` is the only thing that distinguishes the two worlds. Both runs use the same bind chain and the same baseline value.
2. **Build the seed carrier** with `start(treatment_ctx)` — a `PropagatingProcess::pure(PATIENT_INITIAL_BP)` with the treatment context attached.
3. **Factual run:** `start(treatment).bind(apply_drug_effect).bind(compute_final_bp)` → `Y(1)`.
4. **Counterfactual run:** `start(treatment).alternate_context(control).bind(apply_drug_effect).bind(compute_final_bp)` → `Y(0)`. The `alternate_context` call rewrites the carrier's Context channel *before* either bind runs, so both stages read the control assignment.
5. **ITE = Y(1) - Y(0).**

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/rcm` | `classical_via_causal_monad/rcm` |
|---|---|---|
| Causal logic lives in | `Causaloid` + `CausaloidGraph` | `bind` closures on `PropagatingProcess` |
| Treatment assignment carried in | `RcmState` (the value) | `TreatmentContext` (the Context channel) |
| Counterfactual mechanism | Construct two `RcmState` values, run graph twice | Build one seed, swap Context via `alternate_context`, run binds |
| Audit-log artefact | None by default; user must instrument | `!!ContextAlternation!!` entry appended automatically |
| Number of "world" representations | Two state values | One chain definition, one factual seed, one alternated context |

Both implementations produce the same ITE for the same patient. Neither is more correct; they are two faces of the same Causaloid/Context separation.

## Reference

For the conceptual background, see the [Counterfactuals concept page](https://docs.deepcausality.com/concepts/counterfactuals/) and the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf).
