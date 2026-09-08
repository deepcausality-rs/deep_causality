/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AlfvenSpeed, MagneticPressure};
use crate::{Density, PhysicalField, PhysicsError};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_multivector::MultiVector;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

/// Calculates the characteristic speed of Alfven waves.
/// $$ v_A = \frac{B}{\sqrt{\mu_0 \rho}} $$
///
/// # Arguments
/// *   `b_field` - Magnetic field $B$ (Uses magnitude $|B|$).
/// *   `density` - Plasma density $\rho$.
/// *   `permeability` - Magnetic permeability $\mu_0$.
///
/// # Returns
/// *   `Result<AlfvenSpeed<R>, PhysicsError>` - Alfven speed $v_A$.
pub fn alfven_speed_kernel<R>(
    b_field: &PhysicalField<R>,
    density: &Density<R>,
    permeability: R,
) -> Result<AlfvenSpeed<R>, PhysicsError>
where
    R: RealField,
{
    let b_mag = b_field.inner().squared_magnitude().sqrt();
    let rho = density.value();

    if permeability <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Permeability must be positive".into(),
        ));
    }

    if rho < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Density cannot be negative".into(),
        ));
    }

    if rho == R::zero() {
        return Err(PhysicsError::Singularity(
            "Zero density in Alfven speed".into(),
        ));
    }

    let denom = (permeability * rho).sqrt();
    let va = b_mag / denom;

    AlfvenSpeed::new(va)
}

/// Calculates magnetic pressure.
/// $$ P_B = \frac{B^2}{2\mu_0} $$
///
/// # Arguments
/// *   `b_field` - Magnetic field $B$.
/// *   `permeability` - Magnetic permeability $\mu_0$.
///
/// # Returns
/// *   `Result<MagneticPressure<R>, PhysicsError>` - Magnetic pressure $P_B$.
pub fn magnetic_pressure_kernel<R>(
    b_field: &PhysicalField<R>,
    permeability: R,
) -> Result<MagneticPressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let b_sq = b_field.inner().squared_magnitude();

    if permeability <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Permeability must be positive".into(),
        ));
    }

    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let pb = b_sq / (two * permeability);
    MagneticPressure::new(pb)
}

/// Calculates the time evolution of the magnetic field (Frozen-in flux).
/// $$ \frac{\partial \mathbf{B}}{\partial t} = \nabla \times (\mathbf{v} \times \mathbf{B}) $$
///
/// **Discrete exterior calculus implementation**:
/// $$ \partial_t B = -d(i_v B) $$
/// where $B$ is a 2-form (flux through faces), $v$ is a vector field carried as a 1-form
/// (circulation along edges), $i_v$ is the interior product (contraction), and $d$ is the exterior
/// derivative — here the complex's own coboundary on 1-forms.
///
/// # Arguments
/// *   `v_manifold` - Manifold carrying the velocity field $v$ as a 1-form.
/// *   `b_manifold` - Manifold carrying the magnetic flux 2-form $B$. **Its complex must equal
///     `v_manifold`'s**; a call whose two manifolds disagree is refused, because the whole
///     computation is performed on `v_manifold`'s complex and a 2-form measured on other geometry
///     would be silently reinterpreted on it.
///
/// # Returns
/// *   `Result<CausalTensor<R>, PhysicsError>` — $\partial_t B$, a 2-form.
///
/// # The contraction is the crate's, not this kernel's
///
/// The interior product is [`deep_causality_topology::SimplicialManifold::interior_product`]
/// (`unified-math-next` task 6.7u). This kernel previously open-coded the star–wedge identity
/// `i_v B = ⋆(v ∧ ⋆B)` against `hodge_star_operators()[2]`, and that chain could not be made to
/// work: it needs a **degree-changing** star `Λ² → Λ¹`, and what the crate vends is the square
/// diagonal one. The shape check refused every complex built from real geometry, which was correct
/// and left the kernel unusable in its own domain.
///
/// The replacement does not repair that chain, it takes the other standard route. The crate's
/// simplicial contraction interpolates both cochains with **Whitney forms** on each tetrahedron and
/// contracts the reconstructed vector fields, which needs no Hodge star at all — and is exact for
/// constant fields, which is the property its tests pin.
///
/// That matters here beyond convenience, because the star this kernel used to reach for is wrong at
/// intermediate grades: `build_lumped_mass_hodge_star` returns the dual/primal volume ratio only at
/// `k = 0` and `k = n`, and the primal volume `|σ|` in between. On a regular tetrahedron scaled by
/// `h`, `⋆₂` measures as `O(h²)` where a Hodge star on 2-forms in three dimensions must scale as
/// `O(h⁻¹)`. Any formulation composing it would have returned a plausible number that is wrong by a
/// factor of `h³`. That defect is recorded separately; this kernel no longer depends on it.
///
/// # Sign convention
///
/// `i_v B = (B × V)♭`, read off the components of the contraction, so `−d(i_v B)` is `∇×(v × B)` —
/// the induction equation as written above. The previous implementation returned `+d(i_v B)` from a
/// chain whose wedge order was also reversed, and no test could tell, because no in-domain input
/// ever reached the arithmetic.
///
/// # Domain
///
/// **Three dimensions only.** A 2-form on a 3-complex contracts to a 1-form; the same identity in
/// two dimensions is a different operator on different skeletons.
pub fn ideal_induction_kernel<R>(
    v_manifold: &SimplicialManifold<R, R>,
    b_manifold: &SimplicialManifold<R, R>,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + FromPrimitive + Default + PartialEq + Debug,
{
    let complex = v_manifold.complex();
    let skeletons = complex.skeletons();

    if skeletons.len() < 4 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "ideal induction needs a 3D complex: a 2-form contracts to a 1-form only there. This \
             complex has {} skeletons, so it is {}-dimensional",
            skeletons.len(),
            skeletons.len().saturating_sub(1)
        )));
    }

    // The two cochains must live on the *same* complex. The signature takes two independent
    // manifolds, and every offset below — the slice bounds, the Whitney interpolation, the
    // coboundary — is read off `v_manifold`'s complex alone, so a `b_manifold` carrying a
    // different complex has its 2-form reinterpreted on geometry it was never measured against.
    // Simplex counts do not catch that: two meshes with the same connectivity and different vertex
    // positions agree on every count, and the mixed call then returns a plausible number for
    // physics that is not being computed. Measured on two tetrahedron pairs differing only in one
    // vertex: flux freezing demands `∂ₜB = 0` exactly for a uniform field in a uniform flow, and
    // the mixed call returned `1.34`.
    //
    // The comparison is structural and therefore linear in the size of the mesh — skeletons,
    // both operator sets, and the coordinates. That is the same order as the contraction below,
    // so it is a constant-factor cost rather than a new one; a timestep loop that calls this per
    // step with one manifold can hoist the check if it ever shows up in a profile. It reads
    // `SimplicialComplex`'s own `PartialEq`, which deliberately excludes the lazily-populated
    // Hodge ⋆ cache, so two equal complexes never differ by cache state alone.
    if v_manifold.complex() != b_manifold.complex() {
        return Err(PhysicsError::DimensionMismatch(
            "ideal induction reads the velocity 1-form and the magnetic 2-form on one complex; \
             the two manifolds carry different complexes"
                .into(),
        ));
    }

    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();

    // Both manifolds are checked, and against the same complex: the signature takes two
    // independent manifolds, so a `b_manifold` built over a smaller complex is a legal call and
    // must be refused rather than indexed past.
    if v_manifold.data().len() < n0 + n1 + n2 {
        return Err(PhysicsError::DimensionMismatch(
            "v_manifold data too small".into(),
        ));
    }
    if b_manifold.data().len() < n0 + n1 + n2 {
        return Err(PhysicsError::DimensionMismatch(
            "b_manifold data too small".into(),
        ));
    }

    // v is the 1-form at offset n0; B is the 2-form at offset n0 + n1.
    let v_slice = &v_manifold.data().as_slice()[n0..n0 + n1];
    let b_slice = &b_manifold.data().as_slice()[n0 + n1..n0 + n1 + n2];

    let v_form = CausalTensor::new(v_slice.to_vec(), vec![n1])?;
    let b_form = CausalTensor::new(b_slice.to_vec(), vec![n2])?;

    // i_v B, a 1-form.
    let i_v_b = v_manifold.interior_product(&v_form, &b_form, 2)?;

    // d(i_v B), a 2-form, through the complex's own coboundary on 1-forms.
    if complex.coboundary_operators().len() <= 1 {
        return Err(PhysicsError::CalculationError(
            "Coboundary operator for 1-forms not available".into(),
        ));
    }
    let d_1 = &complex.coboundary_operators()[1];
    if d_1.shape() != (n2, n1) {
        return Err(PhysicsError::DimensionMismatch(format!(
            "ideal induction needs the coboundary on 1-forms with shape ({n2}, {n1}); \
             coboundary_operators()[1] has shape {:?}",
            d_1.shape()
        )));
    }
    let d_i_v_b = d_1.vec_mult_real(i_v_b.as_slice())?;

    // ∂ₜB = −d(i_v B).
    let dt_b: Vec<R> = d_i_v_b.into_iter().map(|x| R::zero() - x).collect();
    let len = dt_b.len();
    CausalTensor::new(dt_b, vec![len]).map_err(PhysicsError::from)
}
