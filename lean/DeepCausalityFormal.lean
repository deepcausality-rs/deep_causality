/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Root module of the DeepCausality formalization (Lean 4 + Mathlib).

Layered to mirror the Rust crate tiers:
  * `Num`      — foundational algebraic laws (monoid/group/ring/field), mirroring `deep_causality_num`.
  * `Haft`     — HKT / functor / monad / arrow laws, mirroring `deep_causality_haft`.
  * `Core`     — the Causal Monad `pure`/`bind` laws, mirroring `deep_causality_core`.
  * `Topology` — curvature-tensor laws at the concrete carriers, mirroring `deep_causality_topology`.

Each theorem is bound to a Rust witness via `lean/THEOREM_MAP.md`. See `lean/README.md`.

Scope (what is proved end-to-end here, each bound to a Rust witness):
  * `Num`      — the add-monoid laws (associativity, identity).
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

import DeepCausalityFormal.Num.Monoid
import DeepCausalityFormal.Core.CausalMonad
import DeepCausalityFormal.Core.CausalArrow
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
import DeepCausalityFormal.Haft.Morphism
import DeepCausalityFormal.Haft.Endomorphism
import DeepCausalityFormal.Haft.Adjunction
import DeepCausalityFormal.Haft.Foldable
import DeepCausalityFormal.Haft.Traversable
import DeepCausalityFormal.Haft.NaturalIso
import DeepCausalityFormal.Haft.Either
import DeepCausalityFormal.Haft.EffectSystem
import DeepCausalityFormal.Haft.Io
import DeepCausalityFormal.Haft.Signatures
import DeepCausalityFormal.Haft.FreeMonad
import DeepCausalityFormal.Topology.RiemannCurvature
