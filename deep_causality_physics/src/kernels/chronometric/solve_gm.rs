/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::fmt::Debug;
use deep_causality_num::RealField;

use crate::{CentralBody, PhysicsError, SPEED_OF_LIGHT, SpaceTimeCoordinate};

/// Recovers GM within the weak-field 1PN limit from two space-time coordinates.
///
///
/// Inverts the relativistic clock equation
///
/// $\dot\tau = 1 + \Phi(r,\theta)/c^2 - v^2/(2c^2)$
///
/// using the J2-corrected geopotential
///
/// $\Phi(r,\theta) = -\dfrac{GM}{r}\left[1 - J_2 \left(\dfrac{R_{eq}}{r}\right)^2 \dfrac{3\cos^2\theta - 1}{2}\right]$
///
/// The colatitude cosine is taken as `position.z / r` from the input coordinate,
/// where `z` is assumed to lie along the central body's rotation axis (the same
/// axis $J_2$ is referenced to).
///
/// # Assumptions
///
/// This kernel is a *first post-Newtonian* (1PN) inversion. It is physically
/// valid only when **all** of the following hold:
///
/// 1. **Weak field**: $|\Phi|/c^2 \ll 1$. At Earth's surface this ratio is
///    ~$7\times 10^{-10}$; at GNSS altitude ~$1.7\times 10^{-10}$. The
///    expansion breaks down only near compact objects (neutron stars,
///    black holes), where $|\Phi|/c^2$ approaches $O(1)$.
///
/// 2. **Slow motion**: $v^2/c^2 \ll 1$. Satisfied trivially for any
///    sub-relativistic motion (~$10^{-10}$ for GNSS satellites). Fails only
///    for relativistic probes by design.
///
/// 3. **Axially symmetric, static source**: the geopotential is modeled as
///    monopole + $J_2$ quadrupole. Higher zonal harmonics ($J_3$, $J_4$, …),
///    sectoral / tesseral terms, solid-Earth and ocean tides, atmospheric
///    loading, and time-varying mass redistributions are **not** accounted
///    for. At GNSS scale, $J_3$/$J_4$ contribute below ~$10^{-6}$ relative;
///    tides ~$10^{-9}$. Cosmology (FLRW) and strong-lensing regimes fall
///    outside this assumption entirely.
///
/// 4. **Inertial-frame velocity**: `coord.v_ms` *must* be measured against
///    an inertial (non-rotating) frame, e.g. ECI for Earth. Passing a
///    body-fixed/rotating-frame velocity (e.g. ECEF) silently corrupts the
///    result via the omitted Sagnac term $-2(\vec r \cdot \vec v)/c^2$ — at
///    Earth's equator this can bias recovered $GM$ by up to ~1%, far worse
///    than any of the truncated GR terms. The caller is responsible for the
///    frame transform.
///
/// 5. **Body-fixed `position` orientation**: `position[2]` ($z$) is assumed
///    to align with the body's rotation axis (the $J_2$ symmetry axis). For
///    Earth this corresponds to ECEF/ITRF — geographic pole along $+z$. An
///    arbitrary inertial frame does *not* satisfy this; rotate first if
///    needed.
///
/// 6. **Truncation order (1PN)**: drops $O(\Phi^2/c^4)$ (2PN Schwarzschild
///    correction), cross terms $O(\Phi v^2/c^4)$, frame-dragging
///    (gravitomagnetic / Lense–Thirring), and gravitational-wave back-reaction.
///    At Earth scale all of these are $\lesssim 10^{-19}$, irrelevant for any
///    current clock metrology, but begin to matter near the Sun's limb (Shapiro
///    delay) and dominate near compact objects.
///
/// 7. **Two-point differential measurement**: a single coordinate cannot
///    isolate $GM$ — the gravitational and SR kinematic contributions to
///    clock drift are degenerate at one point. Both inputs must be valid,
///    simultaneous(-ish) samples of the *same* body's gravity field with
///    sufficient radial separation $|1/r_{\text{eff},a} - 1/r_{\text{eff},b}|$.
///
/// 8. **Clock-drift sign convention**: `clock_drift_rate` is interpreted as
///    $\dot\tau \equiv d\tau/dt - 1$ — the fractional frequency offset
///    relative to coordinate time, *negative* deep in the gravitational well.
///    A clock at higher altitude has a less-negative (or more-positive)
///    drift than one deeper in the well.
///
/// 9. **Static `CentralBody` parameters**: `body.j2` and `body.equatorial_radius_m`
///    must be a self-consistent pair from a single gravity model (e.g. JGM-3,
///    EGM2008). Mixing $J_2$ from one model with $R_{eq}$ from another
///    introduces a model-inconsistency bias of order $J_2 \cdot \Delta R_{eq}^2$.
///
/// # Coverage
///
/// The kernel is altitude-agnostic for Earth orbit. The math is identical
/// across regimes — only the signal magnitude and which corrections dominate
/// change:
///
/// | Regime    | Altitude       | Examples                          | Notes                                                                 |
/// |-----------|----------------|-----------------------------------|-----------------------------------------------------------------------|
/// | LEO       | 200–2000 km    | ISS/ACES, GRACE-FO, LEO-PNT       | $J_2$ contribution ~10× stronger than at GNSS; drag perturbs orbit   |
/// | MEO       | 20–35k km      | Galileo, GPS, GLONASS, BeiDou     | Design-target regime                                                  |
/// | GEO       | 35,786 km      | Telecom satellites with onboard clocks | $\Phi/c^2 \sim 10^{-11}$, smaller signal but cleaner $J_2$        |
/// | HEO       | varies         | Molniya, TESS-like elliptical orbits | One orbit naturally probes a wide $r$ range — favorable for GM recovery |
/// | Lab/tower | 1 mm – 1 km    | Pound–Rebka, optical clock height comparisons | $J_2$ irrelevant at this scale; needs $\sim 10^{-15}$ or better clock stability |
///
/// **Non-Earth bodies**: the kernel itself is body-agnostic — swapping
/// [`CentralBody`] to Mars, Moon, Jupiter, etc. will produce a valid
/// inversion *in principle*. In practice, callers must verify that the
/// chosen $J_2$ / $R_{eq}$ values come from a self-consistent gravity model
/// for that body (assumption 9), and that the body's higher-order harmonics
/// or non-axial mass distribution (e.g. lunar mascons, asteroid irregularity)
/// do not dominate the residual at the precision they care about. Validate
/// against a forward-modeled drift before trusting the recovered GM.
///
/// # Where it stops working
///
/// - **N-body / Lagrange-point regimes** — fails because the kernel assumes
///   a single dominant central body. Near L1/L2 or in three-body resonances,
///   the potential is not $-GM/r$ even to leading order. *Watch out for*:
///   spacecraft in halo or Lissajous orbits, lunar transfer trajectories,
///   anything where a second body's gravity is comparable.
///
/// - **Irregular / non-axisymmetric bodies** (asteroids, comets, small moons)
///   — fails because the geopotential expansion past $J_2$ is dominated by
///   sectoral and tesseral terms or non-continuous mass distribution.
///   *Watch out for*: any small Solar System body where shape and density
///   vary strongly with longitude.
///
/// - **Strong-field regimes** (neutron stars, black hole vicinity) — fails
///   because $|\Phi|/c^2$ is no longer $\ll 1$, so 1PN truncation breaks.
///   *Watch out for*: orbits with periapsis inside ~$10^3$ Schwarzschild radii;
///   need full Schwarzschild or Kerr geodesics instead.
///
/// - **Cosmological scales** (Hubble flow, FLRW) — fails because there is no
///   asymptotically-flat coordinate time to define $\dot\tau$ against.
///   *Watch out for*: anyone confusing cosmological redshift for gravitational
///   redshift; this kernel is not the right tool.
///
/// - **Frame-dragging / gravitomagnetic measurements** (Gravity Probe B,
///   LARES) — fails because the kernel has no body-spin sector.
///   *Watch out for*: experiments deliberately targeting Lense–Thirring;
///   they need the full 1PN gravitomagnetic potential, not just the static
///   Newtonian + $J_2$ piece.
///
/// - **Single-clock GM recovery** — fails because gravitational and SR
///   kinematic contributions are degenerate at one point. *Watch out for*:
///   any caller pattern that tries to invoke this with effectively one
///   measurement (e.g. two coords at the same $r$ and $v$); the $\epsilon$
///   guard catches the worst case but near-degenerate inputs amplify noise.
///
/// - **Non-gravitational accelerations** (radiation pressure, atmospheric
///   drag, thrust) — kernel cannot distinguish them from gravity. *Watch
///   out for*: LEO satellites with significant drag, deep-space probes with
///   solar radiation pressure, anything maneuvering during the measurement
///   window. The orbit-determination upstream must remove these before the
///   inputs reach this kernel.
///
/// # Errors
///
/// Returns [`PhysicsError`] if either input has a non-positive radius, or if
/// the effective radial separation is below numerical precision.
pub fn solve_gm_analytical_kernel<R>(
    coord_a: &SpaceTimeCoordinate<R>,
    coord_b: &SpaceTimeCoordinate<R>,
    body: &CentralBody<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + From<f64> + Debug,
{
    let c: R = R::from(SPEED_OF_LIGHT);
    let c_sq = c * c;

    // Term 1: clock-drift difference (gravitational redshift contribution)
    let term_time = c_sq * (coord_b.clock_drift_rate - coord_a.clock_drift_rate);

    // Term 2: SR kinetic-energy difference per unit mass
    let half: R = R::from(0.5);
    let term_kinetic = half * (coord_b.v_ms * coord_b.v_ms - coord_a.v_ms * coord_a.v_ms);

    // Term 3: J2-corrected potential geometry
    let inv_r_eff_a = inv_r_effective(coord_a, body)?;
    let inv_r_eff_b = inv_r_effective(coord_b, body)?;
    let term_potential = inv_r_eff_a - inv_r_eff_b;

    let epsilon: R = R::from(1e-20);
    if term_potential.abs() < epsilon {
        return Err(PhysicsError::TopologyError(
            "Insufficient effective radial separation for GM derivation".to_string(),
        ));
    }

    Ok((term_time + term_kinetic) / term_potential)
}

/// Computes 1/r_eff = 1/r − J2 · R_eq² · P₂(cos θ) / r³
/// where P₂(cos θ) = (3cos²θ − 1)/2 is the second Legendre polynomial.
#[inline]
fn inv_r_effective<R>(
    coord: &SpaceTimeCoordinate<R>,
    body: &CentralBody<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + From<f64> + Debug,
{
    let r = coord.r_m;
    if r <= R::zero() {
        return Err(PhysicsError::TopologyError(
            "Non-positive radial distance".to_string(),
        ));
    }

    let cos_theta = coord.position[2] / r;
    let three: R = R::from(3.0);
    let half: R = R::from(0.5);
    let legendre_p2 = half * (three * cos_theta * cos_theta - R::one());

    let r_cubed = r * r * r;
    let req_sq = body.equatorial_radius_m * body.equatorial_radius_m;

    Ok(R::one() / r - body.j2 * req_sq * legendre_p2 / r_cubed)
}
