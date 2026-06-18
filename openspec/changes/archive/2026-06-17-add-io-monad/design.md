# Design — `Io`: a full IO abstraction (monad + effect)

## Context

IO is the workspace's first genuinely side-effecting effect. This document records the design
decisions, the one architectural fork that was resolved, and how IO relates to the existing effect
system.

## Relationship to the existing effect system

`deep_causality_haft`'s effect system (`Effect3/4/5`, `MonadEffect*`) and the causal monad
(`CausalEffectPropagationProcess`, itself an `Effect5`-shaped carrier of value + state + context +
error + log) track effects **statically and purely**: each "effect" is a *fixed type parameter*
threading **data** through a computation. Nothing there performs a real-world side effect.

IO cannot be modeled that way. A filesystem read/write is a **real** effect whose *execution* must be
deferred to the program edge. Therefore IO is **orthogonal** to `Effect3/4/5` — it neither extends nor
replaces them. It is a value-level monad that **bridges into** the pure effect system at two seams:

- **Error seam:** an IO failure becomes a `CausalityError` (`CausalityErrorEnum::IoError`), so it
  short-circuits a `CausalFlow` exactly like any other error.
- **Audit seam:** running an IO action inside a `CausalFlow` appends an `EffectLog` entry, recording
  the effect in the same audit trail as a Pearl `do(...)` intervention.

Summary: *the effect system describes effects as data; the IO monad executes effects at the edge; they
meet at `CausalityError` + `EffectLog`.*

## Decision: defunctionalized, `dyn`-free encoding (Encoding A)

### The fork

A lazy, **mono-parametric** `Io<A>` — the shape the witness `Monad` trait requires (`F::Type<A>` has
exactly one hole) — cannot store data-dependent continuations without `Box<dyn FnOnce>`. This is
structural: a free-monad/AST encoding still needs boxed `A -> Io<B>` continuations. Two consequences:

1. The witness `Monad`/`Functor` traits cannot host a lazy IO bind regardless of encoding: their
   method signatures (`Func: FnMut(A) -> F::Type<B>`, no `'static`) cannot express the `'static`
   capture a deferred thunk requires. This is the **same** structural reason `PropagatingProcessWitness`
   and `CausalEffectPropagationProcessWitness` deliberately do not implement value-only `Monad` (see
   the `NOTE` blocks in their `hkt.rs`).
2. So the realistic choice gives up the generic witness `Monad` impl either way, and differs only in
   whether it uses `dyn`.

The workspace forbids `dyn` (test-enforced; zero `dyn` in haft/core `src`). The codebase already
contains the precedent for "an abstraction that cannot be witnessed without `dyn`": the **`Arrow`
algebra** — a value-level algebra of concrete combinator structs (`Lift`, `Compose`, `First`…), chosen
precisely because composing closures yields unnameable types and `Box<dyn Fn>` is forbidden.

### Resolution

**IO belongs with `Arrow`, not with the witnessed containers.** An `Io<A>` is a nullary Kleisli arrow
`() ⇝ A` over the `Result`/causal monad — the same Kleisli category `CausalArrow` inhabits. The IO
monad is realized as an `IoAction` trait whose `map`/`and_then` return new concrete combinator structs:
total composition, monomorphized, zero-cost, **no `dyn`**.

It does **not** implement the witness `Monad` trait, and that is consistent with the codebase: haft has
two idioms — the witness-pattern hierarchy for pure containers, and the value-level combinator algebra
(`Arrow`) for the rest. IO joins the latter.

### Why generic over `E`

The abstract layer lives in `deep_causality_haft`, which must stay dependency-free and `no_std`-safe.
`IoAction` carries `type Error`, so haft names no concrete error. `deep_causality_core` specializes the
file actions to `Error = CausalityError`, mirroring how `PropagatingEffectWitness<E, L>` fixes its
effect parameters at the core layer.

## Laziness, the run seam, and `CausalFlow`

`CausalFlow` is **eager** — each `bind` executes immediately. IO is **lazy** — nothing runs until
`run`. The bridge reconciles these: the IO action is **composed lazily** and the flow verb is the
single controlled `run` point, folding the `Result` into the flow and appending the audit entry.

### Two directions, two value semantics

A read and a write are not symmetric, and one method cannot honestly be both. The earlier
`commit_io`/`from_io` pair conflated them (and would have replaced a write's flow value with the
action's `()` output, collapsing `CausalFlow<V>` to `CausalFlow<()>` — wrong). They are split by the
direction's effect on the carried value:

- **Read = constructor, value-producing.** The read result *is* the value, so a read begins a flow.
- **Write = step, value-preserving.** A write runs for its effect and the carried value flows on
  unchanged; the `()` output is discarded and an `EffectLog` entry is appended. A write is the IO
  cousin of `guard` (inspect-and-pass-through), not of `try_step` (transform).

### Naming convention

Name the **intent** (data in/out of the world via a *place*), not the *mechanism* (`io`), and let the
preposition take a file path. The `IoAction` argument already carries read/write + format, so the flow
verbs are format-qualified path verbs rather than re-stating "io":

- **`CausalFlow::read_text_from(path)` / `read_csv_from(path)`** — read constructors (value in).
- **`flow.write_text_to(path, |v| contents(v))` / `write_csv_to(path, header, |v| rows(v))`** —
  value-preserving write steps (effect out, value through).

These are thin wrappers over a generic, format-agnostic bridge kept for the power case where an
`IoAction` is composed first and only then entered:

- **`CausalFlow::source(io)`** — start a flow from any composed `IoAction` (its `Output` is the value).
- **`flow.commit(|v| io)`** — run a value-preserving `IoAction<Output = ()>` step.

Rejected alternatives for the verbs: `from_io`/`commit_io` (mechanism-named, and `commit_io` had the
value-collapsing bug above); bare `read_from`/`write_to` taking an `IoAction` (doubles the verb —
"write to a write_csv" — and the preposition would not name a place).

## Bit-identical migration

`write_csv` takes **pre-rendered row strings** (`Vec<Vec<String>>`) and emits `header.join(",")` then
each `row.join(",")`, `'\n'`-terminated. The exact bytes are caller-determined, so a migrated example
reproduces its prior `println!`/`writeln!` output byte-for-byte (verified against a captured baseline).

## Alternatives considered

- **Encoding B (boxed thunk `Io<A,E> = Box<dyn FnOnce()->Result<A,E>>` + `IoWitness` + an `IoMonad`
  trait mirroring `CausalMonad`).** Literally witness-nameable, but introduces `dyn` against a
  test-enforced ban, and *still* cannot implement the generic witness `Monad`. Rejected.
- **Free monad / instruction-set AST.** Still requires boxed result-dependent continuations → `dyn`.
  Rejected.
- **Placing the abstraction in `core` (the old, rolled-back proposal).** Couples the generic IO monad
  to `CausalityError` and keeps it out of the functional foundation where Functor/Monad/Arrow live.
  Rejected; the abstraction goes in haft, the specialization in core.

## Risks

- `IoAction` chains build nested generic types; deep ad-hoc chains can produce long type names. Mitigated
  by keeping the action set small and providing named helpers (`write_csv`, `write_series_csv`).
- No type erasure means heterogeneous IO actions cannot be collected into a single `Vec` without a
  future, separately-documented escape hatch. Out of scope; not needed by the CFD use case.
