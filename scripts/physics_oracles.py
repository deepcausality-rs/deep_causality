#!/usr/bin/env python3
"""
Reference values for `deep_causality_physics`' dimensionless-number kernels.

Why this exists
---------------
The tests it feeds used to compute their own expectations by retyping the kernel's formula, which
asserts only that an expression was written the same way twice. This computes the same quantities
**independently**: in a different language, from the textbook definition rather than from the Rust
source, and in 50-digit decimal rather than `f64` — so the expectation is nearer the true value than
any `f64` evaluation can be, and the test measures the kernel's error against the mathematics
instead of against a second copy of itself.

Definitions are the standard ones. Cited to:
  * F. M. White, *Fluid Mechanics*, 7th ed., McGraw-Hill (2011), Table 5.2 — Re, Ma, Fr, We, Pr,
    Pe, St, Ec, Gr, Ra, Nu, Bo, Ca.
  * J. D. Anderson, *Modern Compressible Flow*, 3rd ed. (2003) — Ma.
  * R. B. Bird, W. E. Stewart, E. N. Lightfoot, *Transport Phenomena*, 2nd ed. (2007) — Sc, Le.
  * S. Chapman & T. G. Cowling, *The Mathematical Theory of Non-Uniform Gases* (1970) — Kn.

Usage
-----
    python3 scripts/physics_oracles.py            # emit the Rust tables on stdout
"""
from decimal import Decimal as D, getcontext

getcontext().prec = 50

# ---------------------------------------------------------------------------------------------
# The definitions. Each returns the dimensionless group from its textbook formula.
# ---------------------------------------------------------------------------------------------
def reynolds(u, L, nu):                       return u * L / nu
def mach(u, a):                               return u / a
def froude(u, g, L):                          return u / (g * L).sqrt()
def weber(rho, u, L, sigma):                  return rho * u * u * L / sigma
def prandtl(nu, alpha):                       return nu / alpha
def peclet(u, L, alpha):                      return u * L / alpha
def strouhal(f, L, u):                        return f * L / u
def knudsen(lam, L):                          return lam / L
def richardson(g, beta, dT, L, u):            return g * beta * dT * L / (u * u)
def rayleigh(g, beta, dT, L, nu, alpha):      return g * beta * dT * L**3 / (nu * alpha)
def grashof(g, beta, dT, L, nu):              return g * beta * dT * L**3 / (nu * nu)
def eckert(u, cp, dT):                        return u * u / (cp * dT)
def schmidt(nu, Dm):                          return nu / Dm
def lewis(alpha, Dm):                         return alpha / Dm
def stokes(tau, u, L):                        return tau * u / L
def capillary(mu, u, sigma):                  return mu * u / sigma
def bond(rho, g, L, sigma):                   return rho * g * L * L / sigma
def nusselt(h, L, k):                         return h * L / k

# ---------------------------------------------------------------------------------------------
# The input sets. Deliberately spread over many decades and across physical regimes rather than
# sitting on one comfortable value: creeping flow to hypersonic, microns to kilometres, liquid
# metals (Pr ~ 0.01) to heavy oils (Pr ~ 1e4).
# ---------------------------------------------------------------------------------------------
def d(*xs): return tuple(D(str(x)) for x in xs)

CASES = {
    "REYNOLDS": ("u, L, nu", reynolds, [
        d(1e-6, 1e-6, 1e-6),      # creeping flow, micron scale
        d(1e-3, 1e-3, 1.5e-5),    # microfluidic air
        d(1.0, 0.01, 1.0e-6),     # water, centimetre
        d(10.0, 2.5, 1.5e-5),     # air over a car
        d(250.0, 60.0, 1.46e-5),  # airliner cruise
        d(7800.0, 10.0, 1.0e-4),  # re-entry scale
        d(1e5, 1e3, 1e-7),        # extreme, still finite
        d(3.7e-2, 8.9e-3, 2.3e-6),
    ]),
    "MACH": ("u, a", mach, [
        d(1e-3, 340.29), d(34.029, 340.29), d(340.29, 340.29),
        d(680.58, 340.29), d(1701.45, 340.29), d(7800.0, 295.0), d(1e4, 1e-2),
    ]),
    "FROUDE": ("u, g, L", froude, [
        d(1e-3, 9.80665, 1e-3), d(0.5, 9.80665, 0.1), d(10.0, 9.80665, 2.5),
        d(3.0, 1.62, 100.0),      # lunar gravity
        d(15.0, 24.79, 1.0),      # Jovian gravity
        d(1e3, 9.80665, 1e6), d(2.7, 3.72, 0.35),
    ]),
    "WEBER": ("rho, u, L, sigma", weber, [
        d(1000.0, 2.0, 0.001, 0.072),   # water droplet, air
        d(1.225, 100.0, 0.05, 0.072),
        d(13546.0, 0.5, 0.002, 0.4865), # mercury
        d(789.0, 10.0, 1e-4, 0.0223),   # ethanol spray
        d(1000.0, 1e-3, 1e-6, 0.072),
        d(1.0, 1e4, 1e3, 1e-3),
    ]),
    "PRANDTL": ("nu, alpha", prandtl, [
        d(1.5e-5, 2.2e-5),   # air, ~0.7
        d(1.0e-6, 1.43e-7),  # water, ~7
        d(1.1e-7, 4.3e-5),   # liquid sodium, ~0.0026
        d(9.0e-4, 8.6e-8),   # engine oil, ~1e4
        d(1e-9, 1e-9), d(1e3, 1e-3),
    ]),
    "PECLET": ("u, L, alpha", peclet, [
        d(1e-6, 1e-6, 1e-7), d(1.0, 0.01, 1.43e-7), d(10.0, 2.5, 2.2e-5),
        d(250.0, 60.0, 2.2e-5), d(1e4, 1e3, 1e-8),
    ]),
    "STROUHAL": ("f, L, u", strouhal, [
        d(0.2, 1.0, 1.0), d(120.0, 0.01, 10.0), d(1e-3, 1e3, 1.0),
        d(5.0, 0.05, 1.2), d(1e6, 1e-6, 1e-3),
    ]),
    "KNUDSEN": ("lam, L", knudsen, [
        d(6.8e-8, 1.0), d(6.8e-8, 1e-6), d(1e-3, 1e-3), d(1.0, 1e-9), d(1e-12, 1e3),
    ]),
    "RICHARDSON": ("g, beta, dT, L, u", richardson, [
        d(9.80665, 3.4e-3, 10.0, 1.0, 1.0), d(9.80665, 2.07e-4, 50.0, 0.1, 0.01),
        d(1.62, 1e-3, 100.0, 10.0, 5.0), d(9.80665, 1e-6, 1e-3, 1e-3, 1e3),
    ]),
    "RAYLEIGH": ("g, beta, dT, L, nu, alpha", rayleigh, [
        d(9.80665, 3.4e-3, 10.0, 0.1, 1.5e-5, 2.2e-5),
        d(9.80665, 2.07e-4, 20.0, 0.05, 1.0e-6, 1.43e-7),
        d(1.62, 1e-3, 5.0, 2.0, 1e-4, 1e-5),
        d(9.80665, 1e-5, 1e-2, 1e-3, 1e-3, 1e-3),
    ]),
    "GRASHOF": ("g, beta, dT, L, nu", grashof, [
        d(9.80665, 3.4e-3, 10.0, 0.1, 1.5e-5),
        d(9.80665, 2.07e-4, 20.0, 0.05, 1.0e-6),
        d(1.62, 1e-3, 5.0, 2.0, 1e-4), d(24.79, 1e-4, 1e3, 1e2, 1e-2),
    ]),
    "ECKERT": ("u, cp, dT", eckert, [
        d(2000.0, 1004.0, 3000.0), d(10.0, 4184.0, 1.0), d(1e-3, 1e3, 1e3),
        d(7800.0, 1004.0, 20000.0), d(1e4, 1e-3, 1e-3),
    ]),
    "SCHMIDT": ("nu, Dm", schmidt, [
        d(1.5e-5, 2.0e-5), d(1.0e-6, 1.5e-9), d(1e-9, 1e-3), d(9.0e-4, 1e-10),
    ]),
    "LEWIS": ("alpha, Dm", lewis, [
        d(2.2e-5, 2.0e-5), d(1.43e-7, 1.5e-9), d(1e-3, 1e-9), d(1e-9, 1e-3),
    ]),
    "STOKES": ("tau, u, L", stokes, [
        d(1e-3, 10.0, 0.1), d(1e-6, 1.0, 1e-3), d(1.0, 1e3, 1e-2), d(1e-9, 1e-3, 1e3),
    ]),
    "CAPILLARY": ("mu, u, sigma", capillary, [
        d(1.0e-3, 0.01, 0.072), d(1.5e-5, 100.0, 0.072), d(1.0, 1e-6, 0.0223),
        d(0.1, 1.0, 0.4865), d(1e-6, 1e6, 1e-3),
    ]),
    "BOND": ("rho, g, L, sigma", bond, [
        d(1000.0, 9.80665, 0.01, 0.072), d(1000.0, 9.80665, 0.001, 0.072),
        d(13546.0, 9.80665, 0.005, 0.4865), d(789.0, 1.62, 0.02, 0.0223),
        d(1.0, 24.79, 1e3, 1e-3),
    ]),
    "NUSSELT": ("h, L, k", nusselt, [
        d(10.0, 1.0, 0.0257), d(1000.0, 0.05, 0.6), d(1e-3, 1e-3, 1e3),
        d(1e5, 10.0, 401.0), d(25.0, 0.3, 0.14),
    ]),
}

def rs(x):
    """Shortest representation that round-trips to the same `f64`, as a Rust float literal."""
    t = repr(x)
    if "." not in t and "e" not in t and "E" not in t:
        t += ".0"
    return t


def emit():
    print("// GENERATED by scripts/physics_oracles.py -- do not edit by hand.")
    print("// Reference values in 50-digit decimal from the textbook definitions; see the script")
    print("// header for the citations.\n")
    for name, (params, fn, rows) in CASES.items():
        arity = len(rows[0]) + 1
        print(f"/// `({params}, expected)`")
        print(f"pub const {name}: &[[f64; {arity}]] = &[")
        for r in rows:
            want = fn(*r)
            vals = ", ".join(rs(float(x)) for x in r)
            print(f"    [{vals}, {rs(float(want))}],")
        print("];\n")

if __name__ == "__main__":
    emit()
