/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Surface-force diagnostics on an immersed cut body (CFD Stage-4
//! `add-slip-boundaries-and-surface-forces`).
//!
//! The hydrodynamic force on an immersed body is `F = ∮_S (−p n + μ(∇u+∇uᵀ)·n) dA`. This module
//! ships the **pressure** term — `F_p = −∮ p n dA`, integrated over the body's `CutFaceFragment`s
//! (each carrying area and outward normal) with a caller-supplied per-cell pressure — and the
//! **viscous (friction)** term `F_μ = ∮ μ(∇u+∇uᵀ)·n dA`, reconstructed from the velocity field at
//! the cut cells, plus the drag/lift coefficient helper. Both are read-only on a field snapshot;
//! neither is on the per-step hot path.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{CutCellRegistry, LatticeCell, LatticeComplex, Manifold};

use crate::solvers::dec::DecNsScalar;
use deep_causality_physics::PhysicsError;

/// The **pressure** surface force on an immersed cut body: `F_p = −∮ p n dA`, summed over every
/// cut cell's fragments with the cell pressure from `cell_pressure` (keyed by the cell's
/// `iter_cells(D)` index — the registry's `CellId`).
///
/// For a closed body the net force is `−p₀ ∮ n dA = 0` under a uniform pressure `p₀` (the
/// fragment normals sum to zero), and `−∇p · V_solid` under a linear pressure field.
pub fn pressure_surface_force<const D: usize, R: DecNsScalar>(
    registry: &CutCellRegistry<D, R>,
    cell_pressure: impl Fn(usize) -> R,
) -> [R; D] {
    let mut force = [R::zero(); D];
    for (&cell_id, cut) in registry.iter() {
        let p = cell_pressure(cell_id);
        for fragment in cut.fragments() {
            let normal = fragment.outward_normal();
            let area = fragment.area();
            for (d, f) in force.iter_mut().enumerate() {
                *f -= p * normal[d] * area;
            }
        }
    }
    force
}

/// The net outward area vector `∮ n dA` of a body's fragments — zero for a closed surface (a
/// consistency check on the fragment normals, independent of any field).
pub fn fragment_area_vector<const D: usize, R: DecNsScalar>(
    registry: &CutCellRegistry<D, R>,
) -> [R; D] {
    let mut area_vector = [R::zero(); D];
    for (_, cut) in registry.iter() {
        for fragment in cut.fragments() {
            let normal = fragment.outward_normal();
            let area = fragment.area();
            for (d, a) in area_vector.iter_mut().enumerate() {
                *a += normal[d] * area;
            }
        }
    }
    area_vector
}

/// The **viscous (friction)** surface force on an immersed cut body:
/// `F_μ = ∮_S μ(∇u + ∇uᵀ)·n dA`, summed over every cut cell's fragments.
///
/// The velocity field is the edge 1-cochain `edge_form` (pass `state.as_one_form()`), `sharp`-
/// reconstructed to vertex vectors. The wall shear is evaluated with a **one-sided wall-normal
/// gradient to the true surface distance `Δh`** (Kirkpatrick et al. 2003): for each fragment the
/// velocity is zero at the wall (no-slip, anchored at the fragment centroid `c`) and sampled by
/// multilinear interpolation at `c + Δh·n` one cell out along the outward normal `n`, so
/// `∂u/∂n ≈ u_sample / Δh`. The Kirkpatrick wall traction `t = μ S·n` with the rank-one wall-normal
/// gradient reduces to `tᵢ = μ (u_sample,i + nᵢ (u_sample·n)) / Δh`, integrated over the fragment
/// area. With `ρ = 1` the dynamic viscosity `mu` equals the kinematic `ν` the solver carries.
///
/// Unlike a central difference straddling the cut (which mixes fluid and solid-side nodes over a
/// full cell), this reads the gradient from the wall to the first fluid sample over the actual
/// perpendicular distance, so it is exact on a no-slip linear shear and far better at a curved wall.
/// It remains a read-only diagnostic, resolution-bound at the body — meaningfully checked against
/// the reference drag, not a fast analytic gate.
///
/// # Errors
/// * `PhysicsError::TopologyError` wrapping a `sharp` failure (wrong edge count or missing metric).
/// * `PhysicsError::TopologyError` when the manifold's geometry is not axis-aligned (per-edge
///   graded geometry has no single per-axis spacing for the wall-normal step).
pub fn viscous_surface_force<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    registry: &CutCellRegistry<D, R>,
    edge_form: &CausalTensor<R>,
    mu: R,
) -> Result<[R; D], PhysicsError> {
    use alloc::format;

    let dx = manifold
        .metric()
        .and_then(|g| g.axis_lengths())
        .ok_or_else(|| {
            PhysicsError::TopologyError(
                "viscous_surface_force requires an axis-aligned geometry (per-axis spacing)".into(),
            )
        })?;

    // Vertex velocity vectors (layout `vertex * D + axis`), keyed by lattice position.
    let vertex_vectors = manifold
        .sharp(edge_form)
        .map_err(|e| PhysicsError::TopologyError(format!("sharp failed: {e}")))?;
    let complex = manifold.complex();
    let velocity: BTreeMap<[usize; D], [R; D]> = complex
        .iter_cells(0)
        .zip(vertex_vectors.as_slice().chunks_exact(D))
        .map(|(vertex, v)| {
            let mut vec = [R::zero(); D];
            vec.copy_from_slice(v);
            (*vertex.position(), vec)
        })
        .collect();

    // The top cells (D-cells) in registry-key order, so a `CellId` resolves to its base position
    // (the start vertex for the multilinear sample's floor search).
    let cells: Vec<LatticeCell<D>> = complex.iter_cells(D).collect();

    let mut force = [R::zero(); D];
    for (&cell_id, cut) in registry.iter() {
        let base = *cells[cell_id].position();
        for fragment in cut.fragments() {
            let area = fragment.area();
            let centroid = fragment.centroid();
            // Unit outward normal (defensively normalised; intersection normals are already unit).
            let raw_n = fragment.outward_normal();
            let mut nn = R::zero();
            for &c in raw_n.iter() {
                nn += c * c;
            }
            if nn <= R::zero() {
                continue;
            }
            let inv = R::one() / nn.sqrt();
            let mut n = [R::zero(); D];
            for (i, ni) in n.iter_mut().enumerate() {
                *ni = raw_n[i] * inv;
            }

            // True wall-normal step Δh ≈ one cell projected on the normal, and the sample point one
            // step into the fluid from the wall anchor (the centroid).
            let mut delta_h = R::zero();
            let mut sample = [R::zero(); D];
            for i in 0..D {
                delta_h += n[i].abs() * dx[i];
            }
            if delta_h <= R::zero() {
                continue;
            }
            for (i, s) in sample.iter_mut().enumerate() {
                *s = centroid[i] + delta_h * n[i];
            }
            let u_sample = sample_velocity(&velocity, &base, &sample, &dx);

            // u·n (the wall-normal component of the sampled velocity).
            let mut u_dot_n = R::zero();
            for i in 0..D {
                u_dot_n += u_sample[i] * n[i];
            }
            // Kirkpatrick rank-one wall traction tᵢ = μ (u_sample,i + nᵢ (u·n)) / Δh.
            for (i, f) in force.iter_mut().enumerate() {
                let traction_i = mu * (u_sample[i] + n[i] * u_dot_n) / delta_h;
                *f += traction_i * area;
            }
        }
    }
    Ok(force)
}

/// Multilinear interpolation of the vertex velocity field at the physical point `p` (lattice
/// coordinates `position · dx`). The floor of each grid coordinate is found by a short bounded
/// search seeded at the cut cell's `base` vertex (the sample sits within ~one cell of it), so no
/// real-to-integer cast is needed. Corners outside the domain contribute the no-slip zero.
fn sample_velocity<const D: usize, R: DecNsScalar>(
    velocity: &BTreeMap<[usize; D], [R; D]>,
    base: &[usize; D],
    p: &[R; D],
    dx: &[R; D],
) -> [R; D] {
    let mut lo = [0usize; D];
    let mut frac = [R::zero(); D];
    for j in 0..D {
        let g = p[j] / dx[j];
        // Floor of `g`, found by stepping from `base[j]` (clamped at 0).
        let mut k = base[j];
        while R::from_usize(k + 1).unwrap_or_else(R::one) <= g {
            k += 1;
        }
        while k > 0 && R::from_usize(k).unwrap_or_else(R::zero) > g {
            k -= 1;
        }
        lo[j] = k;
        frac[j] = g - R::from_usize(k).unwrap_or_else(R::zero);
    }

    let mut out = [R::zero(); D];
    for corner in 0..(1usize << D) {
        let mut pos = [0usize; D];
        let mut weight = R::one();
        for j in 0..D {
            let bit = (corner >> j) & 1;
            pos[j] = lo[j] + bit;
            weight *= if bit == 1 {
                frac[j]
            } else {
                R::one() - frac[j]
            };
        }
        if let Some(v) = velocity.get(&pos) {
            for (o, &vi) in out.iter_mut().zip(v.iter()) {
                *o += weight * vi;
            }
        }
    }
    out
}

/// A nondimensional force coefficient `C = F / (½ ρ U² A)` at `ρ = 1` (drag with the streamwise
/// component, lift with the transverse, given the reference speed `u_ref` and frontal area/length
/// `reference_area`).
pub fn force_coefficient<R: DecNsScalar>(force_component: R, u_ref: R, reference_area: R) -> R {
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    force_component / (half * u_ref * u_ref * reference_area)
}

/// The **Fourier-law wall heat flux** on an immersed cut body:
/// `q = −k ∮_S ∇T·n dA`, summed over every cut cell's fragments.
///
/// This is Fourier's law as an actual surface integral — a gradient, a conductivity and a wall
/// normal, integrated over the wetted area. It is the thermal counterpart of
/// [`viscous_surface_force`] and shares its geometry and its wall-normal reconstruction, so the two
/// diagnostics agree about where the wall is and how far the first fluid sample sits from it; a
/// discrepancy between friction and heat flux is then physics rather than two different wall models.
///
/// `scalar` is the temperature 0-cochain (one value per vertex) that
/// [`DecScalarRate`](crate::solvers::dec::DecScalarRate) marches, `t_wall` the temperature the body
/// is held at, and `k` the thermal conductivity. If the scalar is marched with diffusivity `κ`, the
/// physically consistent conductivity is `k = ρ·c_p·κ`; the two are separate inputs because the
/// crate carries no `c_p`, and a caller supplying both consistently gets the physical answer.
///
/// **Sign convention:** with `n` the body's **outward** normal, a positive `q` is heat flowing from
/// the wall into the fluid. A body hotter than the surrounding fluid therefore reports `q > 0`.
///
/// The wall-normal derivative is reconstructed **one-sided to the true surface distance**
/// (Kirkpatrick et al. 2003), exactly as the viscous force reconstructs the shear: the wall value
/// `t_wall` is anchored at the fragment centroid `c`, the field is sampled by multilinear
/// interpolation at `c + Δh·n` one cell out along the outward normal, and
/// `∂T/∂n ≈ (T_sample − t_wall)/Δh`. A central difference straddling the cut is not used — it mixes
/// fluid and solid-side nodes over a full cell, and the solid side is at the wall value by
/// construction, so it would systematically halve the gradient.
///
/// This is **not** the QTT path's
/// [`penalization_heat_integral`](crate::solvers::qtt::penalization_heat_integral), which is a
/// volumetric penalization rate `(1/η)∫χ(T_w−T)dV` with no gradient, conductivity or normal. That
/// quantity is not a flux and cannot be scaled into one; volume penalization has no wall surface,
/// only a mask smoothed over a numerical width. The two are not interchangeable.
///
/// Like the viscous force this is a read-only diagnostic, resolution-bound at the body.
///
/// # Errors
/// * [`PhysicsError::DimensionMismatch`] when `scalar` is not one value per vertex.
/// * [`PhysicsError::NumericalInstability`] when `k` or `t_wall` is not finite.
/// * [`PhysicsError::TopologyError`] when the manifold's geometry is not axis-aligned (per-edge
///   graded geometry has no single per-axis spacing for the wall-normal step).
pub fn wall_heat_flux<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    registry: &CutCellRegistry<D, R>,
    scalar: &CausalTensor<R>,
    t_wall: R,
    k: R,
) -> Result<R, PhysicsError> {
    use alloc::format;
    use deep_causality_topology::ChainComplex;

    if !k.is_finite() || !t_wall.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "wall_heat_flux: conductivity and wall temperature must be finite".into(),
        ));
    }
    let complex = manifold.complex();
    let n0 = complex.num_cells(0);
    if scalar.len() != n0 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "wall_heat_flux: expected {} scalar values (one per vertex), got {}",
            n0,
            scalar.len()
        )));
    }

    let dx = manifold
        .metric()
        .and_then(|g| g.axis_lengths())
        .ok_or_else(|| {
            PhysicsError::TopologyError(
                "wall_heat_flux requires an axis-aligned geometry (per-axis spacing)".into(),
            )
        })?;

    // Vertex temperatures keyed by lattice position, for the multilinear sample.
    let temperature: BTreeMap<[usize; D], R> = complex
        .iter_cells(0)
        .zip(scalar.as_slice().iter())
        .map(|(vertex, &t)| (*vertex.position(), t))
        .collect();

    let cells: Vec<LatticeCell<D>> = complex.iter_cells(D).collect();

    let mut flux = R::zero();
    for (&cell_id, cut) in registry.iter() {
        let Some(cell) = cells.get(cell_id) else {
            continue;
        };
        let base = *cell.position();
        for fragment in cut.fragments() {
            let area = fragment.area();
            let centroid = fragment.centroid();
            let raw_n = fragment.outward_normal();
            let mut nn = R::zero();
            for &c in raw_n.iter() {
                nn += c * c;
            }
            if nn <= R::zero() {
                continue;
            }
            let inv = R::one() / nn.sqrt();
            let mut n = [R::zero(); D];
            for (i, ni) in n.iter_mut().enumerate() {
                *ni = raw_n[i] * inv;
            }

            // True wall-normal step Δh ≈ one cell projected on the normal.
            let mut delta_h = R::zero();
            for i in 0..D {
                delta_h += n[i].abs() * dx[i];
            }
            if delta_h <= R::zero() {
                continue;
            }
            let mut sample = [R::zero(); D];
            for (i, s) in sample.iter_mut().enumerate() {
                *s = centroid[i] + delta_h * n[i];
            }
            let t_sample = sample_scalar(&temperature, &base, &sample, &dx, t_wall);

            // ∂T/∂n from the wall to the first fluid sample, then Fourier's law. With n outward,
            // a fluid colder than the wall gives ∂T/∂n < 0 and hence q > 0 — heat leaving the wall.
            let dt_dn = (t_sample - t_wall) / delta_h;
            flux += (R::zero() - k) * dt_dn * area;
        }
    }
    Ok(flux)
}

/// Multilinear interpolation of the vertex temperature field at the physical point `p`, by the same
/// bounded floor search [`sample_velocity`] uses. Corners outside the domain contribute the wall
/// value rather than zero — for a scalar, zero is a temperature, not a natural "absent" state, and
/// using it would fabricate a gradient at a domain edge.
fn sample_scalar<const D: usize, R: DecNsScalar>(
    temperature: &BTreeMap<[usize; D], R>,
    base: &[usize; D],
    p: &[R; D],
    dx: &[R; D],
    fallback: R,
) -> R {
    let mut lo = [0usize; D];
    let mut frac = [R::zero(); D];
    for j in 0..D {
        let g = p[j] / dx[j];
        let mut k = base[j];
        while R::from_usize(k + 1).unwrap_or_else(R::one) <= g {
            k += 1;
        }
        while k > 0 && R::from_usize(k).unwrap_or_else(R::zero) > g {
            k -= 1;
        }
        lo[j] = k;
        frac[j] = g - R::from_usize(k).unwrap_or_else(R::zero);
    }

    let mut out = R::zero();
    for corner in 0..(1usize << D) {
        let mut pos = [0usize; D];
        let mut weight = R::one();
        for j in 0..D {
            let bit = (corner >> j) & 1;
            pos[j] = lo[j] + bit;
            weight *= if bit == 1 {
                frac[j]
            } else {
                R::one() - frac[j]
            };
        }
        out += weight * temperature.get(&pos).copied().unwrap_or(fallback);
    }
    out
}
