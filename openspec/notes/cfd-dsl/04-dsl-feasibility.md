[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DSL Feasibility Study: Compile Spike Results

STATUS: complete, PASS. Gates the `rework-cfd-flow-dsl` specification per
[03-dsl-acceptance.md §1.1](03-dsl-acceptance.md). Ran before any OpenSpec delta was
written. Companion to `02-dsl-study-io.md` (rev 4.1).

## What was tested

The acceptance note named three genuinely novel, load-bearing type constructs and
required a compile spike proving they compose against the real generic shapes before
the spec freezes. The spike (`scratchpad/dsl_spike.rs`, `rustc --edition 2024`) mirrors
the real crate arities exactly:

- `MarchConfig<'c, const D, R, Z, C>` → `MarchPipeline<'c, D, R, Z, C>` (geometry stage)
- `DuctConfig<R>` → `DuctMarchRun<'a, R>` (no geometry, different arity)
- `Coupled<C, S>` → coupled run (the stage `S` supplied at run time)
- `CarrierPause<'c, R, S, M, D>` with the two-lifetime `CarrierFork<'p, 'c, …>`
- the real `scoped_map` bound (`F: Fn(&T) -> U + Send + Sync`) and the `Scalar: Send + Sync`
  model of `CfdScalar: … + MaybeParallel`

## Results

| Construct | Verdict | Evidence |
|---|---|---|
| A. `Marchable` GAT unifying config families of different arity | PASS | `CfdFlow::march(&duct)` and `CfdFlow::march(&march_config)` both dispatch; `type Pipeline<'c> where Self: 'c` hides the family pipeline type |
| A′. `Coupled<C, S>` is itself `Marchable` | PASS | `.couple(stack).march()` composes onto the one trait; the coupled path needs no second march verb |
| B. `GateSeq` higher-ranked over a borrowed view | PASS | `for<'a> fn(&StudyView<'a, Row>) -> (bool, String)` stores and calls |
| C. Pause borrow through `scoped_map` over cases | PASS | compiles; and the shipped `CarrierPause::continue_branches` already borrows `&self` across `scoped_map` in-repo |
| D. `StudyEffect` phase chain (CDL pattern) | PASS | fluent double-impl transitions `Cases → Swept → Judged`; already proven by `deep_causality_discovery` |

Forbidden programs are rejected by the type system, confirmed by three negative probes:

- `verdict()` before `gates()` → `no method named verdict found for StudyEffect<Swept<MapRow>>`
- `gates()` before `sweep()` → `no method named gates found for StudyEffect<Cases<…>>`
- `GateSeq<OtherRow>` into a `Swept<MapRow>` study → `mismatched types`

## Three refinements the spike forced into the spec

These are corrections to the design sketch that the compile spike surfaced. The spec
must carry them; the sketch's placeholder forms do not compile.

### F1. `GateSeq<Row>`, not `GateSeq<StudyView<'static, Row>>`

The rev 4.1 sketch typed the sequence over `StudyView<'static, Row>`. A `'static` view
cannot borrow the run's rows. The compiling form parameterizes the sequence by the
**row** and makes each gate check higher-ranked in the view lifetime:

```rust
type GateFn<Row> = for<'a> fn(&StudyView<'a, Row>) -> (bool, String);
pub struct GateSeq<Row> { title: String, gates: Vec<(&'static str, GateFn<Row>)> }
```

`Swept<Row>::gates` and `Judged<Row>::gates` take `GateSeq<Row>`. This is exactly the
"higher-ranked fn signatures rather than the `'static` placeholder" §1.1 anticipated.

### F2. `Marchable` spans uncoupled families and the `Coupled` wrapper; the coupling stage is never in the config

The five march families split by where the report comes from: uncoupled (`DuctConfig`,
`MarchConfig`, `QttMarchConfig`) report from `run()`/`run_owned()`, while coupled
(`CompressibleMarchConfig`, `UncertainMarchConfig`) report from `run_until(coupling, …)`
where the stage `S` is a **run-time** argument, absent from the config type. Therefore:

- `Marchable` is implemented directly by the three uncoupled config types.
- The campaign's `.couple(stack)` produces a `Coupled<C, S>` wrapper, and
  `Coupled<C, S>: Marchable` runs the `run_until` path to a fixed horizon and reduces to
  a report. `.couple(stack).march()` composes onto the same trait; no second march verb.

The design's two-level split already reflected this (`.case(cfg).march()` uncoupled vs
`.case(cfg).couple(stack).march()` coupled); the spike confirms one trait covers both.

### F3. `ForkStudy` threads the pause's full parameterization

The real pause is `CarrierPause<'c, R, S, M, D>` and the fork carries **two** lifetimes
(`CarrierFork<'p, 'c, R, S, M, D>`) plus the coupling stage `S`. The campaign's
`ForkStudy` holding `&'p pause` must thread `'c, R, S, M` (or hide them behind a
`Forkable` trait bound). The borrow itself is not at risk: `continue_branches` ships the
identical `&self`-through-`scoped_map` pattern with the bounds
`Self: MaybeParallel, M::Config: MaybeParallel, Report<R>: MaybeParallel`. The spec's
`fork`/`branch`/`continue_for` lowers directly onto `fork()` + `alternate_context` +
`continue_march`, which already exist.

## Consequence for the spec

No construct fails; the grammar does not degrade back into closures. The spec proceeds
with F1–F3 folded in. The compile-fail probes become the `compile_fail` doctests
required by §1.3.
