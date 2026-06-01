## Context

The numerical stack already runs on `RealField` (`deep_causality_num`), a real-number trait sitting atop the crate's algebra tower (magma → … → field → real). `RealField` supplies `sqrt`, `abs`, `exp`, `ln`, `log(base)`, `powf`, the full trig set, `pi`/`e`/`epsilon`, `clamp`, rounding, and the `Field` arithmetic with `PartialOrd`, `Neg`, and `Copy`. That surface covers SURD's information-theoretic logarithms and MRMR's correlation arithmetic, so one bound, `T: RealField`, suffices for both crates.

`CausalTensor<T>` is already generic with no struct-level bound. The two laggard crates are `deep_causality_algorithms` (SURD, MRMR) and `deep_causality_discovery` (the CDL pipeline), which pin `f64` throughout: result types, the typestate states (`CausalTensor<Option<f64>>`, `SurdResult<f64>`), every stage trait, the configs, and the CSV/Parquet loaders.

Constraints carried from the repo: no external numeric crates, `unsafe_code = "forbid"`, static dispatch (no `dyn`), one-type-one-module, full test coverage of changed code.

## Goals / Non-Goals

**Goals:**
- SURD and MRMR generic over `T: RealField`, results at `T = f64` identical to today.
- The CDL pipeline generic over `T: RealField`, precision chosen at the call site by type alias (the math-stack paradigm).
- Existing `f64` call sites keep compiling with minimal change.

**Non-Goals:**
- New algorithms or any behavior change at `f64`.
- The BRCD preparatory foundations (numeric primitives, causal-graph layer, two-dataset carriage, result enum). Those stay in `brcd-prep-foundations`, sequenced after this change.
- Recovering precision beyond the source data: widening a CSV-sourced `f64` to `Float106` does not invent new significant digits (see risks).

## Decisions

**D1. One bound: `T: RealField`.**
The trait surface already covers every operation SURD and MRMR use. Where a routine needs integer-to-real conversion (counts, sample sizes), add the existing `FromPrimitive` bound alongside `RealField`, matching the pattern in the conjugate-gradient solver. *Alternative considered:* a narrower custom bound per algorithm. Rejected: it fragments the abstraction the rest of the stack already shares.

**D2. Default the precision type parameter to `f64` on types, require it explicitly on free functions.**
Give generic structs and the typestate a `T = f64` default (`CDL<S, T = f64>`, `SurdResult<T = f64>`) so existing callers compile unchanged, while new callers (BRCD) pick `Float106` or `f32` by alias. Free functions take `T` by inference or turbofish. *Alternative considered:* no default, forcing every call site to name `T`. Rejected: needless churn for SURD users and the examples, with no benefit. The default keeps the migration mechanical and the diff reviewable.

**D3. Replace `f64` literals with `RealField` constructors.**
Build constants through the tower (`T::zero()`, `T::one()`) and through `T::from(f64)` / `FromPrimitive` for other literals, exactly as the reference example does (`FloatType::from(3.0)`, `FloatType::pi()`). Centralize the few recurring constants per module to keep the conversion auditable. *Alternative considered:* a local `two()`/`half()` helper trait. Rejected unless a literal recurs enough to justify it; prefer the existing constructors.

**D4. Loaders parse to the source numeric, then convert to `T`.**
CSV and Parquet carry `f64`-class numbers; parse them as the format dictates and convert into `T` via `From<f64>`/`FromPrimitive`, producing `CausalTensor<Option<T>>`. *Alternative considered:* a generic `FromStr`-based parse directly into `T`. Rejected: it complicates the loaders for no gain, since the source precision bounds the input regardless.

**D5. Preserve `f64` results by construction.**
The generic code instantiated at `T = f64` must lower to the same operations as today. Guard with golden tests that compare the generic path at `f64` against recorded outputs for SURD and MRMR, end to end through the CDL pipeline.

## Risks / Trade-offs

- **Large, mostly mechanical diff (~300 `f64` sites).** → The `T = f64` default (D2) keeps callers compiling; the change is reviewable crate by crate; the full test suite plus golden tests (D5) guard correctness.
- **`f64`-literal to `T`-constant conversion can silently change a constant.** → Centralize constant construction per module and assert `f64`-equality against recorded outputs.
- **Source-precision ceiling.** Parsing a `f64`-precision CSV and storing it as `Float106` does not recover digits the file never had. → Document that `T` governs downstream compute precision, not input fidelity; input precision is bounded by the source format.
- **Ordering against `brcd-prep-foundations`.** Both touch the discovery trait and the `WithCausalResults` state. → Land this change first (precision generification), then `brcd-prep-foundations` layers the algorithm-specific result enum and two-dataset carriage as `DiscoveryOutcome<T>`. Update that change to declare the dependency.
- **`from(f64)` availability across all target types.** → Confirm `RealField`/`Field` provides the integer/float conversion path on every concrete type (`f32`, `f64`, `Float106`, and the upcoming `f16`/`f128`); add the `FromPrimitive` bound where a routine needs it.

## Open Questions

- Whether to default `T = f64` only on public types or also on the stage traits (trait default type parameters interact with object safety and inference); settle per trait during implementation.
- Exact home and shape of any shared "real constant" helpers if D3 reveals a recurring literal across modules.
- Whether MRMR's F-statistic path needs `FromPrimitive` beyond `RealField` (degrees-of-freedom counts); confirm at implementation.
