# Chaining

`bind` threads a value through dependent steps. Each step may fail, and a failure short-circuits
the rest, so the error path is part of the chain itself. The carried type is free to change at
every step, which is what lets a chain leave one crate and arrive in another.

Every monad comes with a Kleisli category, so these chains compose as morphisms — see
`1_foundation/haft/category.rs`.

| Example | What it shows | Command |
|---|---|---|
| [einstein_field_causal_tensor](einstein_field_causal_tensor/) | `G_uv + Λ g_uv = κ T_uv` solved for `T_uv`, with `pure`, `fmap`, `apply`, `extend` and a shape-changing `bind` | `cargo run -p mathematics_examples --example einstein_field_causal_tensor_examples` |
| [effect_tensor_algebra_roundtrip](effect_tensor_algebra_roundtrip/) | The carried type changes at every step, tensor → multivector → tensor → scalar, at `Float106` | `cargo run -p mathematics_examples --example effect_tensor_algebra_roundtrip_examples` |
| [effect_kalman_predict_correct](effect_kalman_predict_correct/) | Predict / correct / verify in one chain: a tensor matmul and a Clifford rotor side by side | `cargo run -p mathematics_examples --example effect_kalman_predict_correct_examples` |
| [effect_system_causal_tensor](effect_system_causal_tensor/) | The typed effect system: `bind` aggregates the value, the error and the warnings together | `cargo run -p mathematics_examples --example effect_system_causal_tensor_examples` |
