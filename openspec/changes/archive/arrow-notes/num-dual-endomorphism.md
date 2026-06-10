# Note: `Dual<T>` and `Endomorphism` — two CT-adjacent primitives

Status: **parked / exploratory**. Not committed to any change. Captured from a
discussion comparing DeepCausality's category-theory foundation against the
"Category Theory for Tiny ML in Rust" book (Ghalebi & Jafarranmani, working
draft). Neither item is needed for current BRCD/SURD work; both are doors, not
refactors.

Two unrelated primitives surfaced. They are the **discrete** and **continuous**
halves of "how a system changes":

- `Endomorphism<T>` — discrete dynamics (iterate / fixpoint).
- `Dual<T>` — continuous dynamics (differentiate / tangent).

---

## 1. `Endomorphism<T>` — iteration & fixpoint primitive

An endomorphism is a morphism from a type back to itself: `T -> T`. As a marker
it is trivial (`Endomorphism<T>: Morphism<T, T>` with a blanket impl). The value
is structural, not the trait itself:

1. **Type-enforced iterability.** `T -> T` is the only shape that can legally be
   iterated (`f(f(f(x)))`). The marker makes "safe to loop" a compile-time fact
   and gives a home for reusable combinators instead of ad-hoc `loop { … }`:
   - `iterate_n(f, x, StepCount)` — apply exactly n times
   - `iterate_to_fixpoint(f, x, eq)` — apply until `f(x) == x`
   - `iterate_until(f, x, predicate)` — apply until converged/stable
2. **Algebra: `End(T)` is a monoid** under composition (identity = unit,
   composition associative). Formal license to fold a list of state-transitions
   into one and regroup/batch freely without changing the result. The
   composition monoid on transformations — complements the value monoids already
   in `num` (`AddMonoid`/`MulMonoid`).
3. **Pairs with the existing Comonad.** Iterated endomorphism = discrete
   dynamical system *in time*; comonadic `extend` over a context = evolution *in
   space*. `haft` already has `CoMonad`; this completes the temporal/spatial duo.

### Where it lands in causality (concrete, present-day)
- **BRCD Meek/orientation rules** are literally `MixedGraph -> MixedGraph`
  applied until no edge changes = `iterate_to_fixpoint` of a composed
  endomorphism. Currently hand-rolled; this names and consolidates it.
- **Context / effect propagation** in the hypergraph until stable.
- **Time-stepping** the causal monad's `State`.

### Honest scope
Thin abstraction. Win = consolidation (one tested fixpoint combinator vs. N
ad-hoc loops) + compiler refusing to iterate a non-square transform + lawful
folding. Lands exactly on BRCD code being written now. Don't expect more.

### Placement
`haft` (sits next to `Morphism`/`CoMonad`)g. Keep it type-preserving: **no `CtResult`
fallibility** — that was incidental to the book's ML domain and is strictly
weaker than the `Effect5`/`CausalityError` channel `haft` already has.

---

## 2. `Dual<T>` — dual numbers / type-based automatic differentiation

A dual number extends a value with an infinitesimal ε where ε² = 0: `a + b·ε`.
Evaluate any function on `a + 1·ε` and you get `f(a) + f'(a)·ε` — the ε-channel
carries the **exact** derivative (machine precision, not finite-difference, not
symbolic). Implement `Dual<T>`, overload `Add`/`Mul`/…, and differentiation falls
out of the trait impls. This is **forward-mode automatic differentiation** — the
canonical *type-based* AD. 

Terminology (for searching): **dual numbers / forward-mode AD / differentiable
programming**; categorical framing is Conal Elliott, *"The Simple Essence of
Automatic Differentiation"* ("backprop as a functor"); deep theory is
*differential / tangent categories*. ("differential types" is not the standard
name.)

### Algebraic home: `num`
`Dual<T>` over a `Field` is the **ring of dual numbers** — a well-defined object
that slots into the existing `num` algebra tower (`Field`, `Ring`, `Module`,
`Algebra`). Because Elliott's framing makes differentiation a **functor**, once
`num` has `Dual<T>` and `haft` has `Functor`/`Profunctor`, gradients compose
through the same functorial machinery as everything else — AD as another arrow
in the uniform-math story.

### Primary consumer: differential geometry in `deep_causality_topology` ← key insight
`Dual<T>` over a `Field` **is** the algebraic model of a **tangent vector**:
`a + 1·ε` = a point plus a tangent direction, and `f(a + ε) = f(a) + df(a)·ε` is
exactly the pushforward / directional derivative. So the natural first consumer
is the **differential geometry** already in the topology crate:
- tangent spaces / tangent vectors,
- directional derivatives, pushforwards (df),
- metric/curvature computations that currently need analytic or numeric
  derivatives → exact instead.

This reframes the whole idea: `Dual<T>` is foundational **for differential
geometry**, not a discovery feature. That is its cleanest justification.

### Other potential consumers (parametric / continuous side only)
- **`deep_causality_uncertain`**: delta-method uncertainty propagation needs the
  Jacobian; dual numbers give it exactly.
- **`deep_causality_physics`**: forces are gradients of potentials (F = −∇U),
  Euler–Lagrange, geodesics → exact via AD.
- **Effect sensitivity**: ∂(outcome)/∂(parent) of a structural equation is the
  local quantitative causal effect (Pearl). AD gives it alongside evaluation.
- **Gradient-based / continuous structure discovery** (NOTEARS family): AD is the
  enabling substrate — but a *different* methodology from current SURD/BRCD.

### Boundary — what AD does NOT help
AD is for the **continuous/parametric** side only. It is **irrelevant** to the
discrete/combinatorial core: BRCD Meek orientation, MEC enumeration, PC-style
CI-test discovery, SURD's information-theoretic decomposition. Do not expect
`Dual<T>` to touch the BRCD machinery.

### Honest caveats
- Forward-mode (dual numbers) is efficient for **few-inputs/many-outputs**
  (Jacobian-vector, sensitivities, tangent vectors). Scalar-loss-over-many-
  parameters learning wants **reverse-mode** (a tape/graph/macro) — a second,
  larger build. `Dual<T>` alone gives exact sensitivities and tangent-space ops;
  it does **not** by itself give NOTEARS-scale gradient learning.
- This is a strategic direction (differentiable modeling), not a free win.

---

## Suggested first proof point
`Dual<T>` in `num` (over the existing `Field`/`Ring` traits) → consumed by
**differential geometry in `deep_causality_topology`** (tangent vector /
directional derivative). Cleanest, most self-justifying entry; avoids over-
committing to a causal-discovery direction.

`Endomorphism` is independent and lighter — could land first as the
`iterate_to_fixpoint` home for the in-flight BRCD Meek-rule loop.
