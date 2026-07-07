# SCM via the Causal Monad

Pearl's Ladder of Causation on the smoking-tar-cancer chain, implemented directly on `PropagatingProcess<f64, (), SmokingContext>` using the [`Alternatable`](../../../../deep_causality_core/src/traits/alternatable/mod.rs) family. Each rung uses one operator from the family.

## How to run

```bash
cargo run -p classical_causality_examples --example scm_via_monad
```

## The three rungs

| Rung | Operator | Question | Mechanism |
|---|---|---|---|
| 1. Association | none | "If I see high nicotine, what about cancer?" | Plain `bind` chain: `start(world).bind(stage_has_tar).bind(stage_cancer_risk)`. |
| 2. Intervention | `alternate_value` | "If I *do* `Tar := 0.0`, what about cancer?" | Mid-chain value substitution: `start(world).bind(stage_has_tar).alternate_value(0.0).bind(stage_cancer_risk)`. Severs the Smoking -> Tar link. |
| 3. Counterfactual | `alternate_context` | "Given a smoker with high tar, what if they had not smoked?" | Same chain run twice, second time with `.alternate_context(counterfactual_world)` between `start` and the binds. |

## Why one file, not three

The Causaloid version ([`classical_via_causaloid/scm`](../../classical_via_causaloid/scm)) splits Pearl's three rungs across `rung1_association.rs`, `rung2_intervention.rs`, `rung3_counterfactual.rs` because each rung carries non-trivial `CausaloidGraph` and `Contextoid` construction. The monad version does not need that scaffolding: a two-stage bind chain plus one operator per rung fits in a single file under 150 lines. The rungs are functions, not modules.

## How this differs from the Causaloid version

| Concern | `classical_via_causaloid/scm` | `classical_via_causal_monad/scm` |
|---|---|---|
| Causal logic lives in | `Causaloid` + `CausaloidGraph` (one graph for rungs 1-2, one contextual Causaloid for rung 3) | Two `bind` closures (`stage_has_tar`, `stage_cancer_risk`) |
| Rung 2 mechanism | CSM-style "decide to act on observed risk" | Pearl's `do(...)` via `.alternate_value(...)` mid-chain |
| Rung 3 mechanism | Contextoid clone + modify + re-evaluate against a separate `BaseContext` | `.alternate_context(other_world)` at the seed |
| Audit log | None by default; user must instrument | `!!ValueAlternation!!` / `!!ContextAlternation!!` entries appended automatically (visible in stdout) |
| Lines of code | ~250 across 5 files | ~150 in a single file |

Both implementations reach the same conclusions for the same worlds. Neither is more correct; they are two faces of the same Causaloid/Context separation.

## Reference

For the conceptual background, see the [Counterfactuals concept page](https://docs.deepcausality.com/concepts/counterfactuals/) and the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf).
