# CATE via the Causal Monad

Conditional Average Treatment Effect on `PropagatingProcess<f64, (), PatientContext>` using the [`Alternatable`](https://docs.rs/deep_causality_core/latest/deep_causality_core/trait.AlternatableContext.html) family.

## How to run

```bash
cargo run -p classical_causality_examples --example cate_via_monad
```

## The estimand

```
CATE(S) = E[ Y(do(T=1)) - Y(do(T=0)) | X in S ]
```

For each patient in the subgroup `S` (age > 65), compute the individual treatment effect by running the same chain twice — factually under treatment, then via `alternate_context(control)` — and take the mean.

## The mechanism

The patient's full context (age, baseline BP, treatment assignment) lives in `PatientContext`. The chain is a two-stage `bind`:

| Stage | Input | Output | Reads |
|---|---|---|---|
| `stage_drug_effect` | seed value (placeholder) | drug-effect delta | `drug_administered` from Context |
| `stage_final_bp` | drug-effect delta | final BP | `initial_bp` from Context |

Per patient, the operator:

```rust
let y1 = run_binds(start(treatment_ctx.clone()));
let y0 = run_binds(start(treatment_ctx).alternate_context(control_ctx));
let ite = y1 - y0;
```

The CATE is the mean of `ite` across the subgroup. `alternate_context` must land *between* `start` and the binds so both stages read the alternated context — the same shape the RCM port establishes.

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/cate` | `classical_via_causal_monad/cate` |
|---|---|---|
| Patient data lives in | `BaseContext` with manual Datoid additions per patient | `PatientContext` struct |
| Subgroup filter | Iterate Context nodes looking for `AGE_ID` Datoid | Plain `.filter(|p| p.age > AGE_THRESHOLD)` on a `Vec` |
| Counterfactual mechanism | Clone Context, push a `DRUG_ADMINISTERED_ID` Datoid, build a new contextual Causaloid per arm | `start(treatment).alternate_context(control)` |
| Lines of code | ~210 across 2 files | ~155 in a single file |
| Audit trail | None by default | `!!ContextAlternation!!` entries per patient |

Both versions compute the same CATE for the same population. The monad version reads as straight Rust — the iteration over patients is `.iter().filter().map()`, the counterfactual is one method call, and the type signature carries the patient context generically.

## Reference

For the conceptual background, see the [Counterfactuals concept page](https://docs.deepcausality.com/concepts/counterfactuals/) and the RCM example which establishes the single-patient pattern this one stratifies.
