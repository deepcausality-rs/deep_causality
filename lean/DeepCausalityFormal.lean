/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Root module of the DeepCausality formalization (Lean 4 + Mathlib).

Layered to mirror the Rust crate tiers:
  * `Num`      — numeric-core laws (identity, integer ring, cast round-trips, and the
                 real-field model of the `Float106` double-double), mirroring `deep_causality_num`.
  * `Algebra`  — the algebra trait tower (monoid/group/ring/field/module/algebra, division algebra,
                 conjugation, norm), mirroring `deep_causality_algebra`.
  * `Complex`  — `Complex` (field, conjugation, norm) and `Quaternion` (division ring, norm,
                 non-commutativity), mirroring `deep_causality_num_complex`.
  * `Dual`     — the dual number `R[ε]` (commutative ring, ε² = 0, real projection, product rule),
                 mirroring `deep_causality_num_dual`.
  * `Rational` — the field of fractions and its canonical form, mirroring
                 `deep_causality_num_rational`.
  * `Haft`     — HKT / functor / monad / arrow laws, mirroring `deep_causality_haft`.
  * `Core`     — the Causal Monad `pure`/`bind` laws, mirroring `deep_causality_core`.
  * `Linear`   — rank–nullity over 𝔽₂ and the Betti-number identity read off it, mirroring
                 `deep_causality_linear`.
  * `Homology` — the chain condition `∂ₖ ∘ ∂ₖ₊₁ = 0` and the Betti identity standing on it rather
                 than assuming it, mirroring `deep_causality_homology`.
  * `Topology` — curvature-tensor laws at the concrete carriers, mirroring `deep_causality_topology`.
  * `Quantum`  — linearity of the Choi action and of the partial trace, and where trace preservation
                 fails, mirroring `deep_causality_quantum`.

Each theorem is bound to a Rust witness via `lean/THEOREM_MAP.md`. See `lean/README.md`.

Scope (what is proved end-to-end here, each bound to a Rust witness):
  * `Num`      — identity (`Zero`/`One`), integer ring laws (commutativity, distributivity,
                 Euclidean division), cast round-trips, and the `Float106` real-field model.
                 The bit-exact double-double error bounds remain [open] (out of L1 scope).
  * `Algebra`  — the trait-tower laws over Mathlib carriers: monoid/commutative-monoid/semilattice,
                 group/abelian-group, ring/commutative-ring, field/real-field, module/algebra,
                 division algebra, conjugation (star), and norm multiplicativity.
  * `Complex`  — `ℂ` is a field with involutive conjugation and multiplicative norm; `ℍ` is a
                 division ring with multiplicative norm and a non-commutativity witness. (Octonions
                 are out of L1 scope — not in Mathlib — and remain covered by the Rust tests.)
  * `Dual`     — `R[ε]` is a commutative ring, `ε² = 0`, the real projection is a ring map, and the
                 tangent part satisfies the Leibniz product rule (forward-mode AD).
  * `Core`     — the causal-monad laws over the single-channel carrier: bind left identity,
                 bind right identity (unconditional, including errored carriers), associativity,
                 and the error left-zero.
  * `Rational` — ℚ is a field (inverse, commutativity, associativity, distributivity) and an abelian
                 group under addition; the canonical form has a positive denominator coprime with
                 the numerator, so equality is structural; and the order is dense.
  * `Haft`     — the algebraic-layer laws (functor, pure, applicative, monad, comonad, bifunctor,
                 profunctor, monoidal-merge, parametric-monad, arrow, morphism, endomorphism,
                 adjunction, foldable, traversable, natural-iso, either, effect-system, io,
                 signatures) — ~60 theorems.
  * `Linear`   — rank–nullity over 𝔽₂, the nullity-as-count-minus-rank substitution that
                 `betti_number_over` performs without materialising a kernel, and that what it
                 computes is the dimension of mod-2 homology.
  * `Homology` — that `∂ₖ ⬝ ∂ₖ₊₁ = 0` implies `im ∂ₖ₊₁ ⊆ ker ∂ₖ`, which discharges the hypothesis
                 `Linear`'s Betti identity takes and never supplies, and that identity restated over
                 the matrix condition. The subspace inclusion cannot be tested; the matrix product
                 can, and the conformance harness checks it at every grade of every complex.
  * `Topology` — the curvature laws at the concrete carrier (antisymmetry, first Bianchi identity,
                 linearity).
  * `Quantum`  — the Choi action and the partial trace are additive and homogeneous, the partial
                 trace is a bimodule map on both sides and acts as expected on a Kronecker product,
                 and trace preservation holds only at a stated boundary — with an explicit
                 counterexample and its value where it does not.
Deviations from accepted category theory are recorded in
`openspec/notes/causal-algebra/haft-formalization-deviations.md`; the full program and its
remaining layers are described in `openspec/notes/causal-algebra/Formalization.md`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

/- `Num` -/
import DeepCausalityFormal.Num.Identity
import DeepCausalityFormal.Num.Integer
import DeepCausalityFormal.Num.Cast
import DeepCausalityFormal.Num.Float106
/- `Algebra` -/
import DeepCausalityFormal.Algebra.Monoid
import DeepCausalityFormal.Algebra.MonoidGeneric
import DeepCausalityFormal.Algebra.CommutativeMonoid
import DeepCausalityFormal.Algebra.Verdict
import DeepCausalityFormal.Algebra.Group
import DeepCausalityFormal.Algebra.Ring
import DeepCausalityFormal.Algebra.Field
import DeepCausalityFormal.Algebra.EuclideanDomain
import DeepCausalityFormal.Algebra.Module
import DeepCausalityFormal.Algebra.DivisionAlgebra
import DeepCausalityFormal.Algebra.Scalar
/- `Complex` -/
import DeepCausalityFormal.Complex.Complex
import DeepCausalityFormal.Complex.Quaternion
/- `Dual` -/
import DeepCausalityFormal.Dual.Dual
/- `Rational` -/
import DeepCausalityFormal.Rational.Rational
/- `Core` -/
import DeepCausalityFormal.Core.EffectLog
import DeepCausalityFormal.Core.CausalEffect
import DeepCausalityFormal.Core.CausalCommand
import DeepCausalityFormal.Core.CausalMonad
import DeepCausalityFormal.Core.CausalArrow
import DeepCausalityFormal.Core.Alternatable
import DeepCausalityFormal.Core.CausalFlow
import DeepCausalityFormal.Core.Csv
import DeepCausalityFormal.Core.Consistency
import DeepCausalityFormal.Core.Causaloid
import DeepCausalityFormal.Core.VerdictClosure
import DeepCausalityFormal.Core.GraphAlgebra
import DeepCausalityFormal.Core.Catamorphism
import DeepCausalityFormal.Core.CommandInput
import DeepCausalityFormal.Core.ContextGraph
/- `Haft` -/
import DeepCausalityFormal.Haft.Hkt
import DeepCausalityFormal.Haft.Functor
import DeepCausalityFormal.Haft.Pure
import DeepCausalityFormal.Haft.Applicative
import DeepCausalityFormal.Haft.Monad
import DeepCausalityFormal.Haft.Comonad
import DeepCausalityFormal.Haft.Bifunctor
import DeepCausalityFormal.Haft.Profunctor
import DeepCausalityFormal.Haft.MonoidalMerge
import DeepCausalityFormal.Haft.ParametricMonad
import DeepCausalityFormal.Haft.Arrow
import DeepCausalityFormal.Haft.ArrowChoice
import DeepCausalityFormal.Haft.ArrowTerm
import DeepCausalityFormal.Haft.Category
import DeepCausalityFormal.Haft.Kleisli
import DeepCausalityFormal.Haft.Morphism
import DeepCausalityFormal.Haft.Endomorphism
import DeepCausalityFormal.Haft.Adjunction
import DeepCausalityFormal.Haft.Foldable
import DeepCausalityFormal.Haft.Interpreter
import DeepCausalityFormal.Haft.Traversable
import DeepCausalityFormal.Haft.NaturalIso
import DeepCausalityFormal.Haft.Either
import DeepCausalityFormal.Haft.EffectSystem
import DeepCausalityFormal.Haft.Io
import DeepCausalityFormal.Haft.Signatures
import DeepCausalityFormal.Haft.SymmetricMonoidal
import DeepCausalityFormal.Haft.FreeMonad
import DeepCausalityFormal.Haft.Cofree
/- `Homology` -/
import DeepCausalityFormal.Homology.ChainCondition
/- `Linear` -/
import DeepCausalityFormal.Linear.RankNullity
/- `Topology` -/
import DeepCausalityFormal.Topology.RiemannCurvature
/- `Quantum` -/
import DeepCausalityFormal.Quantum.PartialTrace
import DeepCausalityFormal.Quantum.PartialTraceCounterexample
import DeepCausalityFormal.Quantum.Choi
