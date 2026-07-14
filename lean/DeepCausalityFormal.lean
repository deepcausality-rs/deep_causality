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
  * `Haft`     — HKT / functor / monad / arrow laws, mirroring `deep_causality_haft`.
  * `Core`     — the Causal Monad `pure`/`bind` laws, mirroring `deep_causality_core`.
  * `Topology` — curvature-tensor laws at the concrete carriers, mirroring `deep_causality_topology`.

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
  * `Haft`     — the algebraic-layer laws (functor, pure, applicative, monad, comonad, bifunctor,
                 profunctor, monoidal-merge, parametric-monad, arrow, morphism, endomorphism,
                 adjunction, foldable, traversable, natural-iso, either, effect-system, io,
                 signatures) — ~60 theorems.
  * `Topology` — the curvature laws at the concrete carrier (antisymmetry, first Bianchi identity,
                 linearity).
Deviations from accepted category theory are recorded in
`openspec/notes/causal-algebra/haft-formalization-deviations.md`; the full program and its
remaining layers are described in `openspec/notes/causal-algebra/Formalization.md`.
-/

import DeepCausalityFormal.Num.Identity
import DeepCausalityFormal.Num.Integer
import DeepCausalityFormal.Num.Cast
import DeepCausalityFormal.Num.Float106
import DeepCausalityFormal.Algebra.Monoid
import DeepCausalityFormal.Algebra.MonoidGeneric
import DeepCausalityFormal.Algebra.CommutativeMonoid
import DeepCausalityFormal.Algebra.Verdict
import DeepCausalityFormal.Algebra.Group
import DeepCausalityFormal.Algebra.Ring
import DeepCausalityFormal.Algebra.Field
import DeepCausalityFormal.Algebra.Module
import DeepCausalityFormal.Algebra.DivisionAlgebra
import DeepCausalityFormal.Algebra.Scalar
import DeepCausalityFormal.Complex.Complex
import DeepCausalityFormal.Complex.Quaternion
import DeepCausalityFormal.Dual.Dual
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
import DeepCausalityFormal.Topology.RiemannCurvature
import DeepCausalityFormal.Quantum.PartialTrace
import DeepCausalityFormal.Quantum.PartialTraceCounterexample
import DeepCausalityFormal.Quantum.Choi
