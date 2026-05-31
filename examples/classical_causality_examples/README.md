# Classical Causality Examples

Five classical causal-inference methods (CATE, DBN, Granger, RCM, SCM), each implemented **twice** in the DeepCausality framework so the two implementation styles can be read side by side.

```bash
cargo run -p classical_causality_examples --example <example_name>
```

## The two columns

| Folder | Mechanism | Where the alternation lives |
|---|---|---|
| [`classical_via_causaloid/`](classical_via_causaloid/) | `Causaloid` + `CausaloidGraph` + manual Context construction | Contextoid clones outside the carrier |
| [`classical_via_causal_monad/`](classical_via_causal_monad/) | `PropagatingProcess` + `bind` chain + the [`Alternatable`](https://docs.rs/deep_causality_core/latest/deep_causality_core/trait.Alternatable.html) family | `alternate_value` / `alternate_context` / `alternate_state` on the carrier itself |

Both columns avoid Pearl's abduction step. They differ only in *where* the alternation happens: in the Context outside the carrier (causaloid column), or in the carrier's channels (monad column). Same estimand, same numbers, two faces of the same Causaloid/Context separation.

## Example matrix

| Method | `via_causaloid` | `via_monad` |
|---|---|---|
| **CATE** (heterogeneity) | [cate](classical_via_causaloid/cate/README.md) | _pending_ |
| **DBN** (Umbrella World) | [dbn](classical_via_causaloid/dbn/README.md) | _pending_ |
| **Granger** (time-series predictive causality) | [granger](classical_via_causaloid/granger/README.md) | _pending_ |
| **RCM** (Rubin potential outcomes) | [rcm](classical_via_causaloid/rcm/README.md) | [rcm](classical_via_causal_monad/rcm/README.md) |
| **SCM** (Pearl's Ladder) | [scm](classical_via_causaloid/scm/README.md) | [scm](classical_via_causal_monad/scm/README.md) |

## Run commands

### Causaloid column

| Example | Command |
|---|---|
| CATE | `cargo run -p classical_causality_examples --example cate_via_causaloid` |
| DBN | `cargo run -p classical_causality_examples --example dbn_via_causaloid` |
| Granger | `cargo run -p classical_causality_examples --example granger_via_causaloid` |
| RCM | `cargo run -p classical_causality_examples --example rcm_via_causaloid` |
| SCM | `cargo run -p classical_causality_examples --example scm_via_causaloid` |

### Causal-Monad column

| Example | Command |
|---|---|
| RCM | `cargo run -p classical_causality_examples --example rcm_via_monad` |
| SCM | `cargo run -p classical_causality_examples --example scm_via_monad` |

(Remaining monadic ports — CATE, DBN, Granger — are staged for upcoming rounds.)
