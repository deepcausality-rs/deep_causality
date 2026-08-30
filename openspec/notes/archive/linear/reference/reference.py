#!/usr/bin/env python3
"""
Known-good reference results for the deep_causality_linear test suite.

SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

The point of this file
----------------------
A test suite is verified before it is trusted, and "verified" means its expected values were derived
independently of the implementation they check. A value read off a run of the code under test is not
an expectation; it is a transcript. Implementing against a transcript verifies the bug rather than
catching it.

Everything here is computed with exact rational arithmetic where the answer is rational, and from a
closed form where it is not. Nothing imports or runs the Rust crate. Run with:

    python3 reference.py

and paste the emitted constants into the suite.
"""

from fractions import Fraction as F
import itertools
import math

# --------------------------------------------------------------------------------------- exact ops


def det(rows):
    """Determinant by the Leibniz formula: exact, and independent of any elimination."""
    n = len(rows)
    total = F(0)
    for perm in itertools.permutations(range(n)):
        p = list(perm)
        sign = 1
        for i in range(n):
            for j in range(i + 1, n):
                if p[i] > p[j]:
                    sign = -sign
        product = F(1)
        for i in range(n):
            product *= F(rows[i][perm[i]])
        total += sign * product
    return total


def rank(rows, mod=None):
    """Rank by exact elimination. `mod=2` gives the rank over 𝔽₂."""
    m = [[(F(x) if mod is None else x % mod) for x in r] for r in rows]
    R, C = len(m), len(m[0])
    r = 0
    for c in range(C):
        p = next((i for i in range(r, R) if m[i][c] != 0), None)
        if p is None:
            continue
        m[r], m[p] = m[p], m[r]
        inv = 1 if mod == 2 else F(1) / m[r][c]
        m[r] = [((x * inv) % mod if mod else x * inv) for x in m[r]]
        for i in range(R):
            if i != r and m[i][c] != 0:
                f = m[i][c]
                m[i] = [(((a - f * b) % mod) if mod else a - f * b) for a, b in zip(m[i], m[r])]
        r += 1
    return r


def solve(A, b):
    """Exact solve by Gauss-Jordan over ℚ, pivoting on magnitude."""
    n = len(A)
    M = [[F(A[i][j]) for j in range(n)] + [F(b[i])] for i in range(n)]
    for c in range(n):
        p = max(range(c, n), key=lambda i: abs(M[i][c]))
        M[c], M[p] = M[p], M[c]
        pv = M[c][c]
        M[c] = [x / pv for x in M[c]]
        for i in range(n):
            if i != c and M[i][c] != 0:
                f = M[i][c]
                M[i] = [a - f * bb for a, bb in zip(M[i], M[c])]
    return [M[i][n] for i in range(n)]


def transpose(A):
    return [[A[j][i] for j in range(len(A))] for i in range(len(A[0]))]


def matmul(A, B):
    n, k, m = len(A), len(B), len(B[0])
    return [[sum(F(A[i][t]) * F(B[t][j]) for t in range(k)) for j in range(m)] for i in range(n)]


def singular_values(A):
    """
    Singular values as the square roots of the eigenvalues of AᵀA.

    The eigenvalues come from the characteristic polynomial, solved exactly for order ≤ 2 and by
    the symmetric QR-free closed forms above that where the matrix is diagonal. Every matrix in the
    suite is small enough for one of those.
    """
    ata = matmul(transpose(A), A)
    n = len(ata)
    if n == 1:
        return [math.sqrt(float(ata[0][0]))]
    if n == 2:
        # eigenvalues of [[a,b],[b,d]] are (a+d)/2 +- sqrt(((a-d)/2)^2 + b^2)
        a, b, d = float(ata[0][0]), float(ata[0][1]), float(ata[1][1])
        mid = (a + d) / 2
        rad = math.sqrt(((a - d) / 2) ** 2 + b * b)
        vals = sorted([mid + rad, mid - rad], reverse=True)
        return [math.sqrt(max(v, 0.0)) for v in vals]
    # Diagonal AᵀA: the eigenvalues are the diagonal.
    off = any(ata[i][j] != 0 for i in range(n) for j in range(n) if i != j)
    if not off:
        return sorted((math.sqrt(float(ata[i][i])) for i in range(n)), reverse=True)
    raise NotImplementedError("add a closed form for this shape before relying on it")


def cg_reference(apply_op, b, n):
    """The exact solution of a symmetric positive-definite system, by exact elimination."""
    basis = []
    for i in range(n):
        e = [0] * n
        e[i] = 1
        basis.append(apply_op(e))
    A = transpose(basis)
    return solve(A, b)


def laplacian_1d(v):
    n = len(v)
    return [2 * v[i] - (v[i - 1] if i > 0 else 0) - (v[i + 1] if i + 1 < n else 0) for i in range(n)]


# ------------------------------------------------------------------------------------- the fixtures

FIXTURES = {
    "rank_deficient_3x3": [[1, 2, 3], [4, 5, 6], [5, 7, 9]],
    "unit_determinant_3x3": [[1, 2, 3], [0, 1, 4], [0, 0, 1]],
    "zero_leading_entry_3x3": [[0, 1, 0], [1, 0, 0], [0, 0, 1]],
    "singular_2x2": [[1, 2], [2, 4]],
    "boundary_alphabet_3x3": [[1, -1, 0], [0, 1, -1], [1, 0, -1]],
    "ranks_disagree_3x3": [[1, 1, 0], [0, 1, 1], [1, 0, 1]],
    "integer_determinant_4x4": [[2, 1, 0, 0], [1, 2, 1, 0], [0, 1, 2, 1], [0, 0, 1, 2]],
    "tetrahedron_cm_5x5": [
        [0, 1, 1, 1, 1],
        [1, 0, 1, 1, 1],
        [1, 1, 0, 1, 1],
        [1, 1, 1, 0, 1],
        [1, 1, 1, 1, 0],
    ],
    "right_triangle_cm_4x4": [[0, 1, 1, 1], [1, 0, 1, 1], [1, 1, 0, 2], [1, 1, 2, 0]],
}


def main():
    print("=== determinants (exact, Leibniz) ===")
    for name, m in FIXTURES.items():
        if len(m) == len(m[0]):
            print(f"  {name:26s} det = {det(m)}")

    print("\n=== ranks (exact elimination over Q, and over F2) ===")
    for name, m in FIXTURES.items():
        r_q = rank(m)
        r_2 = rank(m, mod=2)
        note = "  <-- they differ" if r_q != r_2 else ""
        print(f"  {name:26s} rank_Q = {r_q}   rank_F2 = {r_2}{note}")

    print("\n=== simplex content from the Cayley-Menger determinants ===")
    d_tet = float(det(FIXTURES["tetrahedron_cm_5x5"]))
    vol = math.sqrt(d_tet / 288.0)
    print(f"  tetrahedron  det = {d_tet}  vol^2 = {d_tet / 288.0!r}  vol = {vol!r}")
    print(f"  sqrt(2)/12                                        = {math.sqrt(2) / 12!r}")
    d_tri = float(det(FIXTURES["right_triangle_cm_4x4"]))
    print(f"  right triangle det = {d_tri}  area = {math.sqrt(d_tri / -16.0)!r}")

    print("\n=== singular values (sqrt of the eigenvalues of A^T A) ===")
    for name, A in [
        ("identity_3x3", [[1, 0, 0], [0, 1, 0], [0, 0, 1]]),
        ("diag_1_3", [[1, 0], [0, 3]]),
        ("rank_one_2x2", [[1, 2], [2, 4]]),
        ("identity_4x4", [[1 if i == j else 0 for j in range(4)] for i in range(4)]),
    ]:
        print(f"  {name:16s} {[repr(v) for v in singular_values(A)]}")

    print("\n=== solves (exact over Q) ===")
    for name, A, b in [
        ("well_conditioned_2x2", [[2, 1], [1, 3]], [5, 10]),
        ("permutation_3x3", [[0, 1, 0], [1, 0, 0], [0, 0, 1]], [1, 2, 3]),
        ("lower_triangular_2x2", [[1, 0], [2, 1]], [1, 4]),
        ("upper_unitriangular_3x3", [[1, 2, 3], [0, 1, 4], [0, 0, 1]], [1, 2, 3]),
    ]:
        x = solve(A, b)
        print(f"  {name:26s} x = {[str(v) for v in x]}   as f64 {[float(v) for v in x]}")

    print("\n=== conjugate gradient: the exact answer the iteration must approach ===")
    for name, b in [("laplacian_3_rhs_101", [1, 0, 1])]:
        x = cg_reference(laplacian_1d, b, len(b))
        print(f"  {name:26s} x = {[str(v) for v in x]}   as f64 {[float(v) for v in x]}")

    print("\n=== eigenvalues of the symmetric fixtures ===")
    print(f"  diag(2,5)                  eigenvalues = [2, 5]")

    print("\n=== GF(2) kernel and image dimensions ===")
    m = FIXTURES["ranks_disagree_3x3"]
    r2 = rank(m, mod=2)
    print(f"  ranks_disagree_3x3         rank_F2 = {r2}  kernel dim = {len(m[0]) - r2}  image dim = {r2}")


if __name__ == "__main__":
    main()
