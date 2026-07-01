/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact Riemann solver for the 1-D Euler equations (Toro, *Riemann Solvers and Numerical Methods for
//! Fluid Dynamics*, Ch. 4): Newton iteration for the star pressure, then self-similar sampling at
//! `s = (x − x₀)/t`. Used to gate the QTT Rusanov marcher on the Sod shock tube.

/// A primitive state `(ρ, u, p)`.
#[derive(Clone, Copy)]
pub struct Prim {
    pub rho: f64,
    pub u: f64,
    pub p: f64,
}

fn sound_speed(s: Prim, gamma: f64) -> f64 {
    (gamma * s.p / s.rho).sqrt()
}

/// The pressure function `f_K(p)` and its derivative for side `K` (shock if `p > p_K`, else
/// rarefaction).
fn f_k(p: f64, k: Prim, a_k: f64, gamma: f64) -> (f64, f64) {
    if p > k.p {
        let a = 2.0 / ((gamma + 1.0) * k.rho);
        let b = (gamma - 1.0) / (gamma + 1.0) * k.p;
        let q = (a / (p + b)).sqrt();
        let f = (p - k.p) * q;
        let fp = q * (1.0 - (p - k.p) / (2.0 * (b + p)));
        (f, fp)
    } else {
        let f = 2.0 * a_k / (gamma - 1.0) * ((p / k.p).powf((gamma - 1.0) / (2.0 * gamma)) - 1.0);
        let fp = 1.0 / (k.rho * a_k) * (p / k.p).powf(-(gamma + 1.0) / (2.0 * gamma));
        (f, fp)
    }
}

/// The star-region pressure `p*` and velocity `u*` (Newton iteration).
fn star_state(l: Prim, r: Prim, al: f64, ar: f64, gamma: f64) -> (f64, f64) {
    let mut p = 0.5 * (l.p + r.p).max(1e-12);
    for _ in 0..100 {
        let (fl, fpl) = f_k(p, l, al, gamma);
        let (fr, fpr) = f_k(p, r, ar, gamma);
        let f = fl + fr + (r.u - l.u);
        let fp = fpl + fpr;
        let p_new = (p - f / fp).max(1e-12);
        if (p_new - p).abs() / (0.5 * (p_new + p)) < 1e-12 {
            p = p_new;
            break;
        }
        p = p_new;
    }
    let (fl, _) = f_k(p, l, al, gamma);
    let (fr, _) = f_k(p, r, ar, gamma);
    let u = 0.5 * (l.u + r.u) + 0.5 * (fr - fl);
    (p, u)
}

/// Sample the exact solution of the Riemann problem `(l | r)` at self-similar speed `s = (x−x₀)/t`.
pub fn sample(l: Prim, r: Prim, gamma: f64, s: f64) -> Prim {
    let al = sound_speed(l, gamma);
    let ar = sound_speed(r, gamma);
    let (ps, us) = star_state(l, r, al, ar, gamma);
    let g1 = (gamma - 1.0) / (2.0 * gamma);
    let g2 = (gamma + 1.0) / (2.0 * gamma);

    if s <= us {
        // Left of the contact.
        if ps > l.p {
            // Left shock.
            let sl = l.u - al * (g2 * ps / l.p + g1).sqrt();
            if s <= sl {
                l
            } else {
                let rho = l.rho * (ps / l.p + (gamma - 1.0) / (gamma + 1.0))
                    / ((gamma - 1.0) / (gamma + 1.0) * ps / l.p + 1.0);
                Prim { rho, u: us, p: ps }
            }
        } else {
            // Left rarefaction.
            let shl = l.u - al;
            if s <= shl {
                l
            } else {
                let a_star = al * (ps / l.p).powf(g1);
                let stl = us - a_star;
                if s > stl {
                    let rho = l.rho * (ps / l.p).powf(1.0 / gamma);
                    Prim { rho, u: us, p: ps }
                } else {
                    let u = 2.0 / (gamma + 1.0) * (al + (gamma - 1.0) / 2.0 * l.u + s);
                    let fac =
                        2.0 / (gamma + 1.0) + (gamma - 1.0) / ((gamma + 1.0) * al) * (l.u - s);
                    Prim {
                        rho: l.rho * fac.powf(2.0 / (gamma - 1.0)),
                        u,
                        p: l.p * fac.powf(2.0 * gamma / (gamma - 1.0)),
                    }
                }
            }
        }
    } else {
        // Right of the contact (mirror image).
        if ps > r.p {
            // Right shock.
            let sr = r.u + ar * (g2 * ps / r.p + g1).sqrt();
            if s >= sr {
                r
            } else {
                let rho = r.rho * (ps / r.p + (gamma - 1.0) / (gamma + 1.0))
                    / ((gamma - 1.0) / (gamma + 1.0) * ps / r.p + 1.0);
                Prim { rho, u: us, p: ps }
            }
        } else {
            // Right rarefaction.
            let shr = r.u + ar;
            if s >= shr {
                r
            } else {
                let a_star = ar * (ps / r.p).powf(g1);
                let str_ = us + a_star;
                if s < str_ {
                    let rho = r.rho * (ps / r.p).powf(1.0 / gamma);
                    Prim { rho, u: us, p: ps }
                } else {
                    let u = 2.0 / (gamma + 1.0) * (-ar + (gamma - 1.0) / 2.0 * r.u + s);
                    let fac =
                        2.0 / (gamma + 1.0) - (gamma - 1.0) / ((gamma + 1.0) * ar) * (r.u - s);
                    Prim {
                        rho: r.rho * fac.powf(2.0 / (gamma - 1.0)),
                        u,
                        p: r.p * fac.powf(2.0 * gamma / (gamma - 1.0)),
                    }
                }
            }
        }
    }
}
