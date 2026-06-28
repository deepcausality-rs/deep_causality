## 1. Operator additive algebra

- [x] 1.1 Add public `add(&self, &Self) -> Result<Self>` to `CausalTensorTrainOperator` — dimension-guarded, delegating to `as_combined_train().add(...)` + `from_combined_train`.
- [x] 1.2 Add public `scale(&self, T) -> Self` (rank-preserving), delegating to `as_combined_train().scale(...)`.
- [x] 1.3 Add public `neg(&self) -> Self` (`scale(0 − 1)`) and `sub(&self, &Self) -> Result<Self>` (`add(other.neg())`).
- [x] 1.4 Tests (`check_operator_algebra`, f64 + Float106): `add`/`sub` densify to the elementwise sum/difference; `scale`/`neg` densify and preserve bond; `apply` linearity `(a+b)·x = a·x + b·x`; mismatched dimensions return `ShapeMismatch`.
- [x] 1.5 `cargo fmt`; clippy `--all-targets` clean; tensor suite green (581 tests). No `unsafe`, no `dyn`, no lib-code macros, no concrete float literals in lib code.
