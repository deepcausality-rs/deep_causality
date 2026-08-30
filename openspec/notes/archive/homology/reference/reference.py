#!/usr/bin/env python3
"""
Known-good reference results for the deep_causality_homology test suite.

SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

The point of this file
----------------------
A test suite is verified before it is trusted, and "verified" means its expected values were derived
independently of the implementation they check. A value read off a run of the code under test is not
an expectation; it is a transcript. Implementing against a transcript verifies the bug rather than
catching it.

Nothing here imports or runs any crate in this workspace. Run with:

    python3 reference.py

and paste the emitted constants into the suite.

Two independent checks run before anything is emitted
-----------------------------------------------------
1. Every Betti number computed here is compared against the value published for that space. The
   sources are named per space in ``SPACES`` below. If a triangulation in this file were wrong, the
   comparison fails and nothing is emitted.
2. The Euler characteristic is computed twice per space -- once from cell counts, once from Betti
   numbers -- and the two must agree. The cell counts never pass through the rank routine, so this
   is two computations agreeing rather than one rearranged.

Why RP^2 and the Klein bottle are here
--------------------------------------
Every complex the workspace ships today (T^2, T^3, cylinder, disk, 3D block) is orientable and
torsion-free, so homology over Q and over F_2 agrees at every grade of every fixture. A suite built
only on those cannot tell the two coefficient fields apart, and ``HomologyField`` would be an
untested parameter. RP^2 is the smallest space that separates them:

    beta_1(RP^2; Q) = 0        beta_1(RP^2; F_2) = 1

The Klein bottle separates them at two grades at once. Both are included for that reason, not for
coverage.
"""

from fractions import Fraction as F
from itertools import combinations, permutations

# ----------------------------------------------------------------------------- exact rank over Q


def rank_rational(rows, ncols):
    """Rank over Q by fraction-free-free Gaussian elimination on exact Fractions.

    Exact arithmetic throughout, so there is no tolerance and no pivoting heuristic to get wrong.
    """
    m = [[F(x) for x in r] for r in rows]
    rank = 0
    for col in range(ncols):
        piv = next((r for r in range(rank, len(m)) if m[r][col] != 0), None)
        if piv is None:
            continue
        m[rank], m[piv] = m[piv], m[rank]
        inv = m[rank][col]
        m[rank] = [x / inv for x in m[rank]]
        for r in range(len(m)):
            if r != rank and m[r][col] != 0:
                f = m[r][col]
                m[r] = [a - f * b for a, b in zip(m[r], m[rank])]
        rank += 1
    return rank


def rank_gf2(rows, ncols):
    """Rank over F_2 on integer bitmasks. Reduction of the entries mod 2 happens on the way in."""
    bits = []
    for r in rows:
        w = 0
        for j, x in enumerate(r):
            if x % 2:
                w |= 1 << j
        bits.append(w)
    rank = 0
    for col in range(ncols):
        mask = 1 << col
        piv = next((r for r in range(rank, len(bits)) if bits[r] & mask), None)
        if piv is None:
            continue
        bits[rank], bits[piv] = bits[piv], bits[rank]
        for r in range(len(bits)):
            if r != rank and bits[r] & mask:
                bits[r] ^= bits[rank]
        rank += 1
    return rank


# ------------------------------------------------------------------- simplicial complex machinery


class Complex:
    """A finite abstract simplicial complex, closed under taking faces.

    ``facets`` are the maximal simplices, each a tuple of vertex labels. Faces are generated, so a
    mis-stated facet list cannot silently produce a non-complex.
    """

    def __init__(self, name, facets):
        self.name = name
        faces = set()
        for f in facets:
            s = tuple(sorted(set(f)))
            if len(s) != len(f):
                raise ValueError(f"{name}: facet {f} repeats a vertex")
            for k in range(1, len(s) + 1):
                faces.update(combinations(s, k))
        self.dim = max(len(s) for s in faces) - 1
        # simplices[k] is the sorted list of k-simplices; index in that list is the column index.
        self.simplices = [
            sorted(s for s in faces if len(s) == k + 1) for k in range(self.dim + 1)
        ]
        self.index = [
            {s: i for i, s in enumerate(level)} for level in self.simplices
        ]

    def num_cells(self, k):
        return len(self.simplices[k]) if 0 <= k <= self.dim else 0

    def boundary(self, k):
        """The matrix of d_k : C_k -> C_{k-1}, as rows over the (k-1)-simplices.

        The standard alternating-sign formula: d[s] = sum_i (-1)^i * (s with vertex i dropped).
        Entries land in {-1, 0, 1} because a face of a simplex is dropped at most once.
        """
        if k == self.dim + 1:
            # d_{max+1} : 0 -> C_max has no columns, but still one row per maximal cell. Returning
            # an empty row list here would make the shape (0, 0) and break cols(d_k) == rows(d_k+1).
            return [[] for _ in range(self.num_cells(self.dim))], 0
        if k <= 0 or k > self.dim:
            return [], self.num_cells(k)
        rows = [[0] * self.num_cells(k) for _ in range(self.num_cells(k - 1))]
        for col, s in enumerate(self.simplices[k]):
            for i in range(len(s)):
                face = s[:i] + s[i + 1:]
                rows[self.index[k - 1][face]][col] += (-1) ** i
        return rows, self.num_cells(k)

    def betti(self, field):
        """beta_k = n_k - rank d_k - rank d_{k+1}, over the named field."""
        if field == "Q":
            rk = rank_rational
        elif field == "F2":
            rk = rank_gf2
        else:
            raise ValueError(f"{self.name}: unknown field {field!r}, expected 'Q' or 'F2'")
        ranks = {}
        for k in range(self.dim + 2):
            rows, ncols = self.boundary(k)
            ranks[k] = rk(rows, ncols) if rows else 0
        return [
            self.num_cells(k) - ranks[k] - ranks.get(k + 1, 0)
            for k in range(self.dim + 1)
        ]

    def euler_from_cells(self):
        return sum((-1) ** k * self.num_cells(k) for k in range(self.dim + 1))

    def dd_is_zero(self):
        """d_{k-1} . d_k = 0 for every k, over Z. The chain condition, checked not assumed."""
        for k in range(2, self.dim + 1):
            a, _ = self.boundary(k - 1)
            b, _ = self.boundary(k)
            inner = len(b)
            for i in range(len(a)):
                for j in range(len(b[0]) if b else 0):
                    if sum(a[i][t] * b[t][j] for t in range(inner)) != 0:
                        return False
        return True


# ------------------------------------------------------------------ quotients of a cubical lattice


def kuhn_simplices(corner, dim):
    """The Kuhn (Freudenthal) triangulation of one unit cube into dim! simplices.

    Each permutation of the axes gives one simplex: start at the low corner and step +1 along the
    axes in that order. The pieces meet face-to-face, so the union over a lattice is simplicial.
    """
    out = []
    for perm in permutations(range(dim)):
        v = list(corner)
        chain = [tuple(v)]
        for axis in perm:
            v[axis] += 1
            chain.append(tuple(v))
        out.append(chain)
    return out


def lattice_quotient(name, sizes, ident):
    """Triangulate a lattice of cubes and glue it by ``ident``.

    ``sizes[d]`` is the number of cubes along axis d. ``ident`` maps a lattice point to its
    canonical representative, which is what performs the gluing.
    """
    dim = len(sizes)
    facets = []
    ranges = [range(n) for n in sizes]

    def rec(prefix, d):
        if d == dim:
            for chain in kuhn_simplices(prefix, dim):
                s = tuple(sorted({ident(p) for p in chain}))
                if len(s) == dim + 1:
                    facets.append(s)
            return
        for i in ranges[d]:
            rec(prefix + (i,), d + 1)

    rec((), 0)
    return Complex(name, facets)


def torus(name, sizes):
    return lattice_quotient(name, sizes, lambda p: tuple(c % n for c, n in zip(p, sizes)))


def klein_bottle(m, n):
    """Wrap in x with no flip; wrap in y with a flip in x. That is the Klein bottle's gluing."""

    def ident(p):
        x, y = p
        q, yy = divmod(y, n)
        if q % 2:
            x = m - x
        return (x % m, yy)

    return lattice_quotient("klein_bottle", (m, n), ident)


def cylinder(m, n):
    """Wrap in x only. The y edges stay free, which is what makes it a boundary."""
    return lattice_quotient("cylinder", (m, n), lambda p: (p[0] % m, p[1]))


def mobius_band(m, n):
    """Wrap in x with a flip in y, leaving y free. One boundary circle, non-orientable."""

    def ident(p):
        x, y = p
        q, xx = divmod(x, m)
        if q % 2:
            y = n - y
        return (xx, y)

    return lattice_quotient("mobius_band", (m, n), ident)


# --------------------------------------------------------------------------------- the space list
#
# Each entry is (complex, beta over Q, beta over F_2, citation). The Betti numbers are the PUBLISHED
# values; the script checks its own triangulations against them rather than reporting whatever they
# happen to produce.

SPACES = [
    (
        Complex("point", [(0,)]),
        [1],
        [1],
        "Hatcher, Algebraic Topology, Section 2.1: H_0 of a point is Z, H_i = 0 for i > 0.",
    ),
    (
        Complex("interval", [(0, 1)]),
        [1, 0],
        [1, 0],
        "Contractible, so the homology of a point (Hatcher, Corollary 2.11).",
    ),
    (
        Complex("circle", [(0, 1), (1, 2), (0, 2)]),
        [1, 1],
        [1, 1],
        "Hatcher, Example 2.13 (boundary of a 2-simplex): H_0 = H_1 = Z.",
    ),
    (
        Complex("sphere_2", [(0, 1, 2), (0, 1, 3), (0, 2, 3), (1, 2, 3)]),
        [1, 0, 1],
        [1, 0, 1],
        "Hatcher, Corollary 2.14: H_i(S^n) = Z for i in {0, n}, else 0.",
    ),
    (
        torus("torus_2", (3, 3)),
        [1, 2, 1],
        [1, 2, 1],
        "Hatcher, Example 2.36: H_*(T^2) = (Z, Z^2, Z). Torsion-free, so Q and F_2 agree.",
    ),
    (
        torus("torus_3", (3, 3, 3)),
        [1, 3, 3, 1],
        [1, 3, 3, 1],
        "Kunneth over the torsion-free H_*(S^1): beta_k(T^n) = C(n, k). Hatcher, Example 3.16.",
    ),
    (
        cylinder(3, 1),
        [1, 1, 0],
        [1, 1, 0],
        "Deformation retracts to S^1 (Hatcher, Section 0), so H_* = H_*(S^1).",
    ),
    (
        mobius_band(3, 1),
        [1, 1, 0],
        [1, 1, 0],
        "Deformation retracts to its core circle (Hatcher, Section 0), so H_* = H_*(S^1).",
    ),
    (
        Complex(
            "real_projective_plane",
            [
                (1, 2, 3), (1, 3, 4), (1, 4, 5), (1, 5, 6), (1, 2, 6),
                (2, 3, 5), (3, 4, 6), (2, 4, 5), (3, 5, 6), (2, 4, 6),
            ],
        ),
        [1, 0, 0],
        [1, 1, 1],
        "Hatcher, Example 2.42: H_i(RP^n; Z/2) = Z/2 for 0 <= i <= n. Over Z, "
        "H_*(RP^2) = (Z, Z/2, 0), so beta(Q) = (1, 0, 0).",
    ),
    (
        klein_bottle(4, 4),
        [1, 1, 0],
        [1, 2, 1],
        "Hatcher, Example 2.47: H_*(K) = (Z, Z + Z/2, 0). The universal coefficient theorem then "
        "gives H_*(K; F_2) = (F_2, F_2^2, F_2).",
    ),
]


# ------------------------------------------------------------------------------- the two checks


def check_spaces():
    print("# --- spaces: computed vs published -----------------------------------------------")
    for cx, beta_q, beta_f2, cite in SPACES:
        got_q = cx.betti("Q")
        got_f2 = cx.betti("F2")
        cells = [cx.num_cells(k) for k in range(cx.dim + 1)]

        assert cx.dd_is_zero(), f"{cx.name}: d.d != 0, the triangulation is not a complex"
        assert got_q == beta_q, f"{cx.name}: beta(Q) computed {got_q}, published {beta_q}"
        assert got_f2 == beta_f2, f"{cx.name}: beta(F2) computed {got_f2}, published {beta_f2}"

        chi_cells = cx.euler_from_cells()
        chi_betti = sum((-1) ** k * b for k, b in enumerate(got_q))
        assert chi_cells == chi_betti, (
            f"{cx.name}: chi from cells {chi_cells}, from Betti {chi_betti}"
        )

        split = " <- Q and F_2 DIFFER" if beta_q != beta_f2 else ""
        print(f"#   {cx.name:24s} cells={cells} chi={chi_cells}{split}")
        print(f"#     beta(Q)  = {got_q}")
        print(f"#     beta(F2) = {got_f2}")
        print(f"#     {cite}")
    print()


def emit_spaces():
    print("# --- paste into the Rust suite ---------------------------------------------------")
    for cx, beta_q, beta_f2, _ in SPACES:
        cells = [cx.num_cells(k) for k in range(cx.dim + 1)]
        name = cx.name.upper()
        print(f"// {cx.name}")
        print(f"pub const {name}_CELLS: &[usize] = &{cells!r}".replace("[", "[").replace("'", "") + ";")
        print(f"pub const {name}_BETTI_RATIONAL: &[usize] = &{beta_q!r};")
        print(f"pub const {name}_BETTI_GF2: &[usize] = &{beta_f2!r};")
    print()


def emit_boundaries():
    """The boundary matrices themselves, for the shape and d.d = 0 assertions."""
    print("# --- boundary operator shapes ----------------------------------------------------")
    for cx, _, _, _ in SPACES:
        shapes = []
        for k in range(cx.dim + 2):
            rows, ncols = cx.boundary(k)
            shapes.append((len(rows), ncols))
        print(f"#   {cx.name:24s} (rows, cols) by grade 0..{cx.dim + 1}: {shapes}")
    print("#   Grade 0 is (0, n_0) and grade max+1 is (n_max, 0): the degenerate grades carry the")
    print("#   shape their dimension implies, so d_1 . d_0 and d_{max+1} compose by shape.")
    print()


# --------------------------------------------------- F_2 matrices with rank known in closed form
#
# These are the shapes for the kernel/image basis tests. Every rank here is derived from the
# matrix's structure, not measured: an elimination is never consulted to produce an expectation.
# Sizes cross the 64-bit word boundary in both directions.
#
# What is pinned, and what is not. A kernel basis is not unique -- any invertible change of basis
# gives another correct answer -- so pinning particular basis vectors would test the elimination's
# pivot order rather than its result. What IS unique is the dimension. So the reference fixes the
# rank and the nullity, and the Rust suite checks the vectors themselves by the two properties that
# hold for every valid basis: A.v = 0 for each kernel vector, and the vectors are independent.

WORD_SIZES = [63, 64, 65, 129]


def structured_gf2_cases():
    cases = []
    for n in WORD_SIZES:
        cases.append((
            f"identity_{n}", n, n,
            [[1 if i == j else 0 for j in range(n)] for i in range(n)],
            n, 0, n,
            "The identity has full rank by definition, so its kernel is trivial.",
        ))
        cases.append((
            f"zero_{n}x{n}", n, n,
            [[0] * n for _ in range(n)],
            0, n, 0,
            "The zero map has rank 0, so its kernel is everything.",
        ))
        cases.append((
            f"all_ones_{n}x{n}", n, n,
            [[1] * n for _ in range(n)],
            1, n - 1, 1,
            "Every row is equal, so the row space is one dimensional.",
        ))
        cases.append((
            f"upper_unitriangular_{n}", n, n,
            [[1 if j >= i else 0 for j in range(n)] for i in range(n)],
            n, 0, n,
            "Unitriangular: n pivots on the diagonal, so full rank over any field.",
        ))
        cases.append((
            f"wide_{n}x{2 * n}", n, 2 * n,
            [[1 if (j % n) == i else 0 for j in range(2 * n)] for i in range(n)],
            n, n, n,
            "Two identity blocks side by side: rank n, and the kernel is the antidiagonal "
            "of paired columns, of dimension n.",
        ))
        cases.append((
            f"tall_{2 * n}x{n}", 2 * n, n,
            [[1 if (i % n) == j else 0 for j in range(n)] for i in range(2 * n)],
            n, 0, n,
            "Two identity blocks stacked: rank n, and no kernel since the columns are "
            "independent.",
        ))
    # The cases where the field matters. The circulant with ones at (i, i) and (i, i+1 mod n) has
    # determinant 1 - (-1)^n, which is 2 for odd n and 0 for even n. So for ODD n it is invertible
    # over Q and singular over F_2: every row sums into the all-ones vector, and n odd makes that
    # dependency invisible to Q. An implementation that silently works over the wrong field gets
    # these two ranks the wrong way round.
    #
    # n = 64 is included next to them precisely because it does NOT separate: an even circulant is
    # rank n-1 over both fields. A suite containing only the even case would report agreement and
    # conclude the field parameter is untested.
    for n in [63, 64, 65, 129]:
        rows = [[1 if j == i or j == (i + 1) % n else 0 for j in range(n)] for i in range(n)]
        rank_f2 = n - 1
        rank_q = n if n % 2 else n - 1
        cases.append((
            f"circulant_{n}", n, n, rows, rank_f2, n - rank_f2, rank_q,
            f"det = 1 - (-1)^{n} = {1 - (-1) ** n}. "
            + (
                "Odd n: invertible over Q, singular over F_2, so the two fields disagree."
                if n % 2
                else "Even n: singular over both, so this one does NOT separate the fields."
            ),
        ))
    return cases


def check_gf2_cases():
    print("# --- F_2 shapes: closed-form rank vs elimination ---------------------------------")
    separating = 0
    for name, m, n, rows, rank, nullity, rank_q, why in structured_gf2_cases():
        assert len(rows) == m, f"{name}: {len(rows)} rows, expected {m}"
        assert all(len(r) == n for r in rows), f"{name}: a row is not {n} wide"
        got = rank_gf2(rows, n)
        assert got == rank, f"{name}: elimination says F_2 rank {got}, closed form says {rank}"
        got_q = rank_rational(rows, n)
        assert got_q == rank_q, f"{name}: elimination says Q rank {got_q}, closed form says {rank_q}"
        assert rank + nullity == n, f"{name}: rank {rank} + nullity {nullity} != {n} columns"
        assert rank_q >= rank, f"{name}: F_2 rank exceeds Q rank, which cannot happen"
        split = "  <- Q and F_2 DIFFER" if rank_q != rank else ""
        separating += rank_q != rank
        print(
            f"#   {name:28s} {m:4d}x{n:<4d} rank_F2={rank:<4d} nullity={nullity:<4d}"
            f" rank_Q={rank_q:<4d}{split}"
        )
        print(f"#     {why}")
    assert separating >= 2, "no shape distinguishes Q from F_2; the field parameter is untested"
    print(f"#   {separating} of these shapes separate the two coefficient fields.")
    print()


def emit_gf2_cases():
    print("# --- paste into the Rust suite ---------------------------------------------------")
    print("// (name, rows, cols, rank_gf2, nullity_gf2, rank_rational). The matrices are rebuilt")
    print("// in Rust from the same closed-form description, so no packed data crosses over.")
    print("pub const GF2_SHAPES: &[(&str, usize, usize, usize, usize, usize)] = &[")
    for name, m, n, _, rank, nullity, rank_q, _ in structured_gf2_cases():
        print(f'    ("{name}", {m}, {n}, {rank}, {nullity}, {rank_q}),')
    print("];")
    print()


if __name__ == "__main__":
    check_spaces()
    check_gf2_cases()
    emit_spaces()
    emit_boundaries()
    emit_gf2_cases()
