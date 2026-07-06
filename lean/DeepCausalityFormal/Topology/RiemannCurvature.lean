/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Topology — Riemann curvature laws (the opener of the Topology formalization layer).

Rust source: `deep_causality_topology/src/types/curvature_tensor/mod.rs` (`CurvatureTensor`,
operations `contract`, `check_bianchi_identity`; symmetry taxonomy `CurvatureSymmetry`).
The haft-level `RiemannMap` trait is a bare signature (deviation D10 of the formalization
audit); the laws live HERE, where the carrier types have algebra — resolution of proposal P-3.

Accepted theory (do Carmo, *Riemannian Geometry*, Ch. 4; Kobayashi–Nomizu Vol. 1, Ch. III):
the Riemann curvature operator satisfies
  1. Antisymmetry:            R(u,v)w = −R(v,u)w
  2. First Bianchi identity:  R(u,v)w + R(v,w)u + R(w,u)v = 0
  3. Multilinearity in each slot.

Model: vectors are integer-valued functions over an abstract index type (pointwise algebra;
no finite sums needed), and the curvature operator is the canonical **constant-curvature
(maximally symmetric) form** built from a symmetric bilinear form `g`:
    R(u,v)w := g(v,w)·u − g(u,w)·v
(do Carmo Ch. 4, Lemma 3.4 — the curvature of a space of constant sectional curvature, here
with K = 1 absorbed). Antisymmetry holds unconditionally; the Bianchi identity needs exactly
the symmetry of `g`; linearity in `w` needs exactly bilinearity of `g` — each hypothesis is
stated, so the theorems exhibit which property of the metric carries which law. The Rust
witness instantiates the same form on the concrete `CurvatureTensor` via `from_generator`
(components `R^d_abc = K(δ^d_a g_bc − δ^d_b g_ac)`) and checks the same three laws through
`contract` / `check_bianchi_identity`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_topology/tests/types/curvature_tensor/curvature_tensor_law_tests.rs`.
-/

namespace DeepCausalityFormal.Topology.RiemannCurvature

/-- Vectors: integer-valued functions over an abstract index type `I`. -/
def V (I : Type) : Type := I → Int

/-- Pointwise vector addition. -/
def vadd {I : Type} (x y : V I) : V I := fun i => x i + y i

/-- Scalar multiple. -/
def vsmul {I : Type} (k : Int) (x : V I) : V I := fun i => k * x i

/-- The canonical constant-curvature Riemann operator built from a bilinear form `g`:
    `R(u,v)w = g(v,w)·u − g(u,w)·v` (do Carmo Ch. 4, Lemma 3.4). -/
def R {I : Type} (g : V I → V I → Int) (u v w : V I) : V I :=
  fun i => g v w * u i - g u w * v i

variable {I : Type}

/-- Antisymmetry: `R(u,v)w = −R(v,u)w` — swapping the loop directions negates the holonomy
    (do Carmo Ch. 4, Prop. 2.5(a)). Holds pointwise for every bilinear form, no hypotheses.

    THEOREM_MAP: `topology.curvature.antisymmetry` -/
theorem antisymmetry (g : V I → V I → Int) (u v w : V I) (i : I) :
    R g u v w i = -(R g v u w i) := by
  show g v w * u i - g u w * v i = -(g u w * v i - g v w * u i)
  rw [Int.neg_sub]

/-- First Bianchi identity: the cyclic sum `R(u,v)w + R(v,w)u + R(w,u)v = 0`
    (do Carmo Ch. 4, Prop. 2.5(b)). Needs exactly the SYMMETRY of `g`.

    THEOREM_MAP: `topology.curvature.bianchi_first` -/
theorem bianchi_first (g : V I → V I → Int) (gsymm : ∀ x y, g x y = g y x)
    (u v w : V I) (i : I) :
    R g u v w i + R g v w u i + R g w u v i = 0 := by
  show (g v w * u i - g u w * v i) + (g w u * v i - g v u * w i)
      + (g u v * w i - g w v * u i) = 0
  rw [gsymm w u, gsymm v u, gsymm w v]
  generalize g v w * u i = a
  generalize g u w * v i = b
  generalize g u v * w i = c
  omega

/-- Additivity in the transported vector: `R(u,v)(w₁ + w₂) = R(u,v)w₁ + R(u,v)w₂`.
    Needs exactly the additivity of `g` in its second slot (the u/v slots are symmetric
    in the argument).

    THEOREM_MAP: `topology.curvature.linearity` -/
theorem linear_w_add (g : V I → V I → Int)
    (gadd : ∀ x y z, g x (vadd y z) = g x y + g x z) (u v w₁ w₂ : V I) (i : I) :
    R g u v (vadd w₁ w₂) i = R g u v w₁ i + R g u v w₂ i := by
  show g v (vadd w₁ w₂) * u i - g u (vadd w₁ w₂) * v i
      = (g v w₁ * u i - g u w₁ * v i) + (g v w₂ * u i - g u w₂ * v i)
  rw [gadd, gadd, Int.add_mul, Int.add_mul]
  generalize g v w₁ * u i = a
  generalize g v w₂ * u i = a'
  generalize g u w₁ * v i = b
  generalize g u w₂ * v i = b'
  omega

/-- Homogeneity in the transported vector: `R(u,v)(k·w) = k·R(u,v)w`. Needs exactly the
    homogeneity of `g` in its second slot.

    THEOREM_MAP: `topology.curvature.linearity` -/
theorem linear_w_smul (g : V I → V I → Int)
    (gsmul : ∀ (k : Int) (x y : V I), g x (vsmul k y) = k * g x y)
    (k : Int) (u v w : V I) (i : I) :
    R g u v (vsmul k w) i = k * R g u v w i := by
  show g v (vsmul k w) * u i - g u (vsmul k w) * v i
      = k * (g v w * u i - g u w * v i)
  rw [gsmul, gsmul, Int.mul_sub, Int.mul_assoc, Int.mul_assoc]

end DeepCausalityFormal.Topology.RiemannCurvature
