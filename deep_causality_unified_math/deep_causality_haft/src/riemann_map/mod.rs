/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */



/// The `RiemannMap` trait models high-arity geometric interactions, specifically the
/// Riemann Curvature Tensor and Scattering Matrices.
///
/// # Category Theory
/// This is a **typed interface** (a signature) for rank-4 interactions shaped like a
/// multilinear map $R: V \otimes V \otimes V \to V$. The trait itself carries no equational
/// theory: multilinearity (additivity/homogeneity per argument) and the curvature symmetries
/// (antisymmetry $R(u,v)w = -R(v,u)w$, first Bianchi identity — do Carmo, *Riemannian
/// Geometry*, Ch. 4) are properties of concrete implementations whose types carry algebra
/// (`deep_causality_topology` / `deep_causality_physics`), and are to be stated and tested
/// there.
///
/// # Mathematical Definition
/// The Riemann Curvature Tensor $R$ is defined in terms of the covariant derivative $\nabla$:
/// $$ R(u, v)w = \nabla_u \nabla_v w - \nabla_v \nabla_u w - \nabla_{[u, v]} w $$
/// It measures the non-commutativity of parallel transport around a loop defined by $u$ and $v$.
///
/// # Use Cases
/// *   **General Relativity**: Calculating gravity as spacetime curvature.
/// *   **Particle Physics**: Scattering matrices (S-Matrix) taking 2 inputs and producing 2 outputs.
/// *   **Differential Geometry**: Measuring the holonomy of a connection.
/// # Why this is not an arity-4 higher-kinded trait
///
/// A rank-4 multilinear map $R: V \otimes V \otimes V \to V$ has **one** domain. Its three inputs
/// and its output are elements of the same vector space, which is what makes $R(u,v)w$ meaningful
/// and what makes the antisymmetry $R(u,v)w = -R(v,u)w$ statable at all.
///
/// An earlier version made `curvature` generic in four independent type parameters bounded only by
/// `Satisfies<P::Constraint>`. Under `NoConstraint` that admits every type, so the one real
/// implementation had to reinterpret its arguments through raw pointers to recover the concrete
/// vector type. That made a safe function undefined behaviour for inputs its own signature
/// accepted. Naming the space as an associated type removes the possibility: the implementation
/// receives the type it needs, and a caller passing anything else is a compile error.
pub trait RiemannMap {
    /// The rank-4 tensor this witness reads.
    type Tensor;

    /// The vector space the map acts on.
    type Vector;

    /// The Curvature Operator: $R(u, v)w$.
    /// Consumes two directions ($u, v$) and a vector ($w$) to measure curvature.
    fn curvature(
        tensor: &Self::Tensor,
        u: &Self::Vector,
        v: &Self::Vector,
        w: &Self::Vector,
    ) -> Self::Vector;

    /// The Scattering Matrix: two in-states produce two out-states in the same space.
    fn scatter(
        interaction: &Self::Tensor,
        in_1: &Self::Vector,
        in_2: &Self::Vector,
    ) -> (Self::Vector, Self::Vector);
}
