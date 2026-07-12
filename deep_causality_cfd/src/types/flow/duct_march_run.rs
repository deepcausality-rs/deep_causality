/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The quasi-one-dimensional duct driver behind
//! [`CfdFlow::march`](crate::CfdFlow::march) (design D4).
//!
//! **Solver-composition choice, recorded per D4:** the crate's 1-D
//! compressible Euler solver ([`CompressibleEuler1d`](crate::CompressibleEuler1d))
//! is a QTT marcher on a *periodic* `2^L` grid with no boundary-condition or
//! source-term seam and a fixed-time `run`, so it cannot host the duct's
//! inlet/outlet characteristics or the quasi-1D area source. The less-code
//! honest alternative is this driver: a small dense first-order finite-volume
//! march (Rusanov / local Lax–Friedrichs flux, the same flux family the QTT
//! solver uses) private to the flow module, reusing the solver module's
//! [`ideal_gas_pressure`](crate::ideal_gas_pressure) primitive. No new public
//! solver type; the analytic gates in the mirrored tests judge it against the
//! area–Mach and normal-shock closed forms.
//!
//! **Scheme.** The conservative quasi-1D system
//! `∂(uA)/∂t + ∂(f(u)A)/∂x = (0, p·dA/dx, 0)` is discretized with cell areas
//! `A_i` at centers and face areas `A_{i±1/2}`, a Rusanov face flux, and the
//! pressure source `p_i·(A_{i+1/2} − A_{i−1/2})/Δx` — exactly balancing the
//! flux difference of a uniform state, so a quiescent duct stays quiescent.
//! The march runs in stagnation-scaled variables (`p̂ = p/p₀`, `T̂ = T/T₀`,
//! `ρ̂ = ρ/ρ₀`, `û = u/√(p₀/ρ₀)`), which removes the gas constant from the
//! problem; every reported quantity is either dimensionless (Mach, thrust
//! coefficient) or rescaled back (pressure).
//!
//! **Boundary conditions**, first-order on purpose: the inlet ghost holds the
//! stagnation state — velocity extrapolated from the first cell (clamped to
//! `[0, u*]`, the sonic speed on the inlet isentrope) and the static state
//! recovered from `(p₀, T₀)` along the isentrope; the outlet ghost fixes the
//! static back pressure while the exit is subsonic and extrapolates
//! everything once it is supersonic.

use crate::CfdScalar;
use crate::solvers::ideal_gas_pressure;
use crate::types::flow::Report;
use crate::types::flow_config::DuctConfig;
use deep_causality_physics::PhysicsError;

/// One primitive state `(ρ̂, û, p̂)` in stagnation-scaled variables.
#[derive(Clone, Copy)]
struct Prim<R> {
    rho: R,
    u: R,
    p: R,
}

/// The runnable duct march composed by
/// [`CfdFlow::march`](crate::CfdFlow::march). Borrows the owned
/// [`DuctConfig`]; [`run`](Self::run) returns the owned
/// [`Report`](crate::Report) and the borrow never escapes it.
pub struct DuctMarchRun<'a, R: CfdScalar> {
    config: &'a DuctConfig<R>,
}

impl<'a, R: CfdScalar> DuctMarchRun<'a, R> {
    pub(crate) fn new(config: &'a DuctConfig<R>) -> Self {
        Self { config }
    }

    /// March the duct to a quasi-steady state and report.
    ///
    /// The report carries the series `"x"` (cell centers), `"mach_profile"`,
    /// and `"pressure_profile"` (static pressure, in the unit of the config's
    /// `p0`), plus the one-element scalars `"thrust_coefficient"`
    /// (`C_f = (ṁ·u_e + (p_e − p_b)·A_e) / (p₀·A*)`, with `A*` the profile's
    /// minimum area and the back pressure as ambient) and — only when the
    /// exit is subsonic while the throat ran supersonic (the shocked regime)
    /// — `"shock_position"`, the steepest-pressure-gradient location
    /// downstream of the throat. A shock-free run **omits** the
    /// `"shock_position"` series rather than reporting a sentinel value.
    ///
    /// # Errors
    /// [`PhysicsError::CalculationError`] naming the final residual, the
    /// tolerance, and the step budget when the stop condition expires before
    /// the residual settles; [`PhysicsError::PhysicalInvariantBroken`] if the
    /// state loses positivity mid-march; numeric-conversion failures.
    pub fn run(&self) -> Result<Report<R>, PhysicsError> {
        let cfg = self.config;
        let n = cfg.cells;
        let gamma = cfg.gamma;
        let one = R::one();
        let lift = |v: f64, what: &str| -> Result<R, PhysicsError> {
            R::from_f64(v).ok_or_else(|| {
                PhysicsError::NumericalInstability(format!("R::from_f64({what}) failed"))
            })
        };
        let half = lift(0.5, "0.5")?;
        let two = lift(2.0, "2.0")?;
        // The explicit-march CFL number; 0.5 keeps the first-order Rusanov
        // update comfortably inside its stability bound.
        let cfl = half;

        // --- Grid: cell centers and face positions over [x_start, x_end]. ---
        let x_start = cfg.profile.x_start();
        let x_end = cfg.profile.x_end();
        let n_r = R::from_usize(n)
            .ok_or_else(|| PhysicsError::NumericalInstability("usize lift failed".into()))?;
        let dx = (x_end - x_start) / n_r;
        let mut x_centers = Vec::with_capacity(n);
        let mut a_centers = Vec::with_capacity(n);
        let mut a_faces = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let i_r = R::from_usize(i)
                .ok_or_else(|| PhysicsError::NumericalInstability("usize lift failed".into()))?;
            a_faces.push(cfg.profile.area_at(x_start + i_r * dx));
            if i < n {
                let xc = x_start + (i_r + half) * dx;
                x_centers.push(xc);
                a_centers.push(cfg.profile.area_at(xc));
            }
        }

        // --- Initial state: the inlet-to-back-pressure isentrope at rest. ---
        // p̂ falls linearly from 1 to p_b/p₀; T̂ and ρ̂ follow the isentrope
        // (T̂ = p̂^((γ−1)/γ), ρ̂ = p̂/T̂); û starts at zero and the pressure
        // gradient starts the flow.
        let pb = cfg.back_pressure / cfg.p0;
        let g_exp = (gamma - one) / gamma;
        let mut prim: Vec<Prim<R>> = Vec::with_capacity(n);
        for (i, _) in x_centers.iter().enumerate() {
            let i_r = R::from_usize(i)
                .ok_or_else(|| PhysicsError::NumericalInstability("usize lift failed".into()))?;
            let frac = (i_r + half) / n_r;
            let p = one - (one - pb) * frac;
            let t = p.powf(g_exp);
            prim.push(Prim {
                rho: p / t,
                u: R::zero(),
                p,
            });
        }
        let mut cons = conserved(&prim, gamma, half);

        // Sonic speed on the inlet isentrope, the inlet-ghost velocity cap:
        // T̂* = 2/(γ+1), û* = √(γ·T̂*).
        let u_star = (gamma * two / (gamma + one)).sqrt();

        // --- Quasi-steady march under the stop condition. ---
        let mut residual = R::one();
        let mut settled = false;
        let mut steps = 0usize;
        while steps < cfg.max_steps {
            steps += 1;
            let (ghost_in, ghost_out) = self.ghosts(&prim, gamma, half, u_star, pb);

            // Global wave speed for the CFL step; face-local speeds set the
            // Rusanov dissipation.
            let mut s_global = R::zero();
            for w in &prim {
                let c = (gamma * w.p / w.rho).sqrt();
                let s = w.u.abs() + c;
                if s > s_global {
                    s_global = s;
                }
            }
            if !(s_global > R::zero() && s_global.is_finite()) {
                return Err(PhysicsError::NumericalInstability(
                    "duct march: non-physical wave speed".into(),
                ));
            }
            let dt = cfl * dx / s_global;

            // Face fluxes (Rusanov), including the two boundary faces.
            let mut flux = Vec::with_capacity(n + 1);
            for i in 0..=n {
                let left = if i == 0 { ghost_in } else { prim[i - 1] };
                let right = if i == n { ghost_out } else { prim[i] };
                flux.push(rusanov_flux(left, right, a_faces[i], gamma, half));
            }

            // Conservative update `u_i ← u_i − Δt/(Δx·A_i)·(F_{i+1/2} − F_{i−1/2})
            // + Δt·s_i/A_i` with the pressure area-source
            // `s_i = p_i·(A_{i+1/2} − A_{i−1/2})/Δx`; for a uniform state the
            // source cancels the flux difference exactly (well-balanced).
            let mut new_cons = Vec::with_capacity(n);
            let mut max_delta = [R::zero(); 3];
            let mut max_scale = [R::zero(); 3];
            for i in 0..n {
                let coeff = dt / (dx * a_centers[i]);
                let src = prim[i].p * (a_faces[i + 1] - a_faces[i]) / dx;
                let z = cons[i];
                let znew = [
                    z[0] - coeff * (flux[i + 1][0] - flux[i][0]),
                    z[1] - coeff * (flux[i + 1][1] - flux[i][1]) + dt * src / a_centers[i],
                    z[2] - coeff * (flux[i + 1][2] - flux[i][2]),
                ];
                for k in 0..3 {
                    let d = (znew[k] - z[k]).abs();
                    if d > max_delta[k] {
                        max_delta[k] = d;
                    }
                    let s = z[k].abs();
                    if s > max_scale[k] {
                        max_scale[k] = s;
                    }
                }
                new_cons.push(znew);
            }
            cons = new_cons;
            prim = primitives(&cons, gamma)?;

            // The residual: per-component L∞ change scaled by the component's
            // grid-wide magnitude — the "maximum relative change per step".
            residual = R::zero();
            for k in 0..3 {
                if max_scale[k] > R::zero() {
                    let r = max_delta[k] / max_scale[k];
                    if r > residual {
                        residual = r;
                    }
                }
            }
            if residual < cfg.residual_tol {
                settled = true;
                break;
            }
        }
        if !settled {
            return Err(PhysicsError::CalculationError(format!(
                "duct march did not reach quasi-steady state: residual {residual} is still \
                 above the tolerance {tol} after the full step budget of {budget} steps",
                tol = cfg.residual_tol,
                budget = cfg.max_steps,
            )));
        }

        self.report(&prim, &x_centers, &a_centers, dx)
    }

    /// The inlet and outlet ghost states of the current field.
    fn ghosts(&self, prim: &[Prim<R>], gamma: R, half: R, u_star: R, pb: R) -> (Prim<R>, Prim<R>) {
        let one = R::one();
        // Inlet: extrapolate velocity, clamp to [0, u*], recover the static
        // state from the stagnation pair along the isentrope.
        let mut u_in = prim[0].u;
        if u_in < R::zero() {
            u_in = R::zero();
        }
        if u_in > u_star {
            u_in = u_star;
        }
        let t_in = one - (gamma - one) * half * u_in * u_in / gamma;
        let p_in = t_in.powf(gamma / (gamma - one));
        let ghost_in = Prim {
            rho: p_in / t_in,
            u: u_in,
            p: p_in,
        };
        // Outlet: supersonic exit extrapolates everything; subsonic exit
        // pins the static back pressure and extrapolates density + velocity.
        let last = prim[prim.len() - 1];
        let c_exit = (gamma * last.p / last.rho).sqrt();
        let ghost_out = if last.u >= c_exit {
            last
        } else {
            Prim {
                rho: last.rho,
                u: last.u,
                p: pb,
            }
        };
        (ghost_in, ghost_out)
    }

    /// Assemble the report: profiles, thrust coefficient, and — in the
    /// shocked regime only — the shock position.
    fn report(
        &self,
        prim: &[Prim<R>],
        x_centers: &[R],
        a_centers: &[R],
        dx: R,
    ) -> Result<Report<R>, PhysicsError> {
        let cfg = self.config;
        let gamma = cfg.gamma;
        let pb = cfg.back_pressure / cfg.p0;
        let half = R::from_f64(0.5)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
        let n = prim.len();
        let mut mach = Vec::with_capacity(n);
        let mut pressure = Vec::with_capacity(n);
        let mut max_mach = R::zero();
        for w in prim {
            let c = (gamma * w.p / w.rho).sqrt();
            let m = w.u / c;
            if m > max_mach {
                max_mach = m;
            }
            mach.push(m);
            pressure.push(w.p * cfg.p0);
        }
        let exit = prim[n - 1];
        let exit_mach = mach[n - 1];

        // Thrust coefficient C_f = (ṁ·u_e + (p_e − p_b)·A_e) / (p₀·A*). In
        // stagnation-scaled variables ρ̂û² and p̂ both carry the scale p₀, so
        // the p₀ cancels and only the area ratio remains.
        let a_exit = a_centers[n - 1];
        let a_star = cfg.profile.min_area();
        let cf = (exit.rho * exit.u * exit.u + (exit.p - pb)) * a_exit / a_star;

        let mut report = Report::new("duct_march");
        report.add_series("x", x_centers.to_vec());
        report.add_series("mach_profile", mach);
        report.add_series("pressure_profile", pressure);
        report.add_series("thrust_coefficient", vec![cf]);

        // Shocked regime: the throat ran supersonic but the exit is subsonic,
        // so a normal shock stands in the diverging section. Locate it at the
        // steepest pressure rise downstream of the area minimum. A shock-free
        // run omits the series (documented on `run`).
        if max_mach > R::one() && exit_mach < R::one() {
            let mut i_throat = 0usize;
            for (i, a) in a_centers.iter().enumerate() {
                if *a < a_centers[i_throat] {
                    i_throat = i;
                }
            }
            let mut best_i = i_throat;
            let mut best_rise = R::zero();
            for i in i_throat..(n - 1) {
                let rise = prim[i + 1].p - prim[i].p;
                if rise > best_rise {
                    best_rise = rise;
                    best_i = i;
                }
            }
            // The steepest rise sits across the face between cells i and i+1.
            let shock_x = x_centers[best_i] + half * dx;
            report.add_series("shock_position", vec![shock_x]);
        }
        Ok(report)
    }
}

/// Conserved per-unit-area state `u = (ρ̂, ρ̂û, Ê)` from the primitives.
fn conserved<R: CfdScalar>(prim: &[Prim<R>], gamma: R, half: R) -> Vec<[R; 3]> {
    prim.iter()
        .map(|w| {
            let e = w.p / (gamma - R::one()) + half * w.rho * w.u * w.u;
            [w.rho, w.rho * w.u, e]
        })
        .collect()
}

/// Recover primitives from the conserved state, enforcing positivity. The
/// pressure comes through the solver module's [`ideal_gas_pressure`]
/// primitive.
fn primitives<R: CfdScalar>(cons: &[[R; 3]], gamma: R) -> Result<Vec<Prim<R>>, PhysicsError> {
    let mut out = Vec::with_capacity(cons.len());
    for z in cons.iter() {
        let rho = z[0];
        if !(rho > R::zero() && rho.is_finite()) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "duct march: density must stay positive and finite".into(),
            ));
        }
        let u = z[1] / rho;
        let p = ideal_gas_pressure(rho, z[1], z[2], gamma);
        if !(p > R::zero() && p.is_finite()) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "duct march: pressure must stay positive and finite".into(),
            ));
        }
        out.push(Prim { rho, u, p });
    }
    Ok(out)
}

/// The Rusanov (local Lax–Friedrichs) face flux times the face area:
/// `F = ½(f(W_L) + f(W_R))·A_f − ½·s·(U_R − U_L)·A_f` with
/// `s = max(|u| + c)` over the two sides.
fn rusanov_flux<R: CfdScalar>(
    left: Prim<R>,
    right: Prim<R>,
    a_face: R,
    gamma: R,
    half: R,
) -> [R; 3] {
    let f = |w: Prim<R>| {
        let e = w.p / (gamma - R::one()) + half * w.rho * w.u * w.u;
        (
            [w.rho * w.u, w.rho * w.u * w.u + w.p, (e + w.p) * w.u],
            [w.rho, w.rho * w.u, e],
            w.u.abs() + (gamma * w.p / w.rho).sqrt(),
        )
    };
    let (fl, ul, sl) = f(left);
    let (fr, ur, sr) = f(right);
    let s = if sl > sr { sl } else { sr };
    [
        (half * (fl[0] + fr[0]) - half * s * (ur[0] - ul[0])) * a_face,
        (half * (fl[1] + fr[1]) - half * s * (ur[1] - ul[1])) * a_face,
        (half * (fl[2] + fr[2]) - half * s * (ur[2] - ul[2])) * a_face,
    ]
}
