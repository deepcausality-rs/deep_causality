# CATE via the Causal Monad

Computes the Conditional Average Treatment Effect on `PropagatingProcess<FloatType, (), PatientContext>` (`FloatType` is a local alias for `f64`; `PatientContext` is a `Context` of `Data<FloatType>` nodes with no space or time slot) with the [`Alternatable`](../../../../deep_causality_core/src/traits/alternatable/mod.rs) family.

## How to run

```bash
cargo run -p classical_causality_examples --example cate_via_monad
```

## The estimand

```
CATE(S) = E[ Y(do(T=1)) - Y(do(T=0)) | X in S ]
```

For each patient in the subgroup `S` (age > 65, the `AGE_THRESHOLD` constant that defines the estimand), the example computes the individual treatment effect by running the same chain twice (factually under treatment, then via `alternate_context(control)`) and takes the mean.

## The mechanism

Each patient is a `PatientContext` built by `patient_world(age, initial_bp, dose, drug_administered)`, holding four Datoid contextoids: `AGE`, `INITIAL_BP`, `DOSE` (the BP change the drug produces when administered, `-10.0` for every patient) and `ASSIGNMENT` (`1.0` treated, `0.0` control). The chain is a two-stage `bind`:

| Stage | Input | Output | Reads |
|---|---|---|---|
| `stage_drug_effect` | seed value (initial BP, ignored) | drug-effect delta | `ASSIGNMENT` and `DOSE` from Context |
| `stage_final_bp` | drug-effect delta | final BP | `INITIAL_BP` from Context |

Per patient, the operator:

```rust
let treatment = patient_world(age, initial_bp, dose, true)?;
let control = patient_world(age, initial_bp, dose, false)?;

let y1 = run_binds(start(treatment.clone()));
let y0 = run_binds(start(treatment).alternate_context(control));
let ite = value_of(&y1)? - value_of(&y0)?;
```

The CATE is the mean of `ite` across the subgroup. `alternate_context` must sit *between* `start` and the binds so both stages read the alternated context, the same shape as the RCM example.

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/cate` | `classical_via_causal_monad/cate` |
|---|---|---|
| Patient data lives in | One `PatientContext` per patient, three Datoids (age, initial BP, dose) built by `create_patient_population` | One `PatientContext` per patient, four Datoids built by `patient_world` |
| Subgroup filter | Read every patient's `AGE` with `?`, then `.filter(\|&(_, age)\| age > 65.0)` | Read every patient's `AGE` with `?`, then `.filter(\|&(_, age)\| age > AGE_THRESHOLD)` |
| Counterfactual mechanism | `model::arm` clones the patient's Context and adds a `DRUG_ADMINISTERED` Datoid; a new contextual Causaloid per arm | `start(treatment).alternate_context(control)` |
| Lines of code | ~240 across 2 files | ~255 in a single file |
| Audit trail | None by default | `!!ContextAlternation!!` entries per patient |

Both versions compute the same CATE for the same population. In the monad version, the iteration over patients is `.iter().filter().map()`, the counterfactual is one method call, and the type signature carries the patient context generically.

## Reference

For background, see the [Counterfactuals concept page](https://docs.deepcausality.com/concepts/counterfactuals/) and the RCM example, whose single-patient pattern this one stratifies.
