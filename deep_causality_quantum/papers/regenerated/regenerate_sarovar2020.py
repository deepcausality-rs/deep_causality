#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Regenerate the crosstalk simulations of Sarovar et al., Quantum 4, 321 (2020), arXiv:1908.09855,
§7, from the published error models, and detect their crosstalk graphs with pyGSTi's PC algorithm.

    python3 -m venv venv && venv/bin/pip install pygsti==0.10.2 pcalg==0.2.2
    venv/bin/python -I -B regenerate_sarovar2020.py <output directory>

pyGSTi writes `dataset_dump.txt` into the working directory, so run it from a scratch directory.

Each scenario follows the lightweight design of §6.3: per region, a bag of N_circs random depth-L
sequences of X_pi/2, Y_pi/2 and the idle; per region and bag entry, N_circs / 2 contexts whose
other regions draw from their bags, each draw replaced by the all-idle sequence with probability
p_idle; duplicate circuits dropped. Every qubit starts in |0>, every circuit is measured in the
computational basis N_rep times. Every gate, the idle included, is followed by local
depolarisation (1 - p) rho + p I / 2, the paper's (1 - p) rho + p I read so that it keeps the trace.

Scenarios, with the paper's parameters:

* one_way (§7.1.1): X_pi/2 on qubit 0 depolarises qubit 1 with p = 0.01; L = 30, N_circs = 10,
  p_idle = 0.1, N_rep = 1e4;
* zz (§7.1.2): X_pi/2 on qubit 0 is exp(-i/2 [pi/2 X(x)I + eps/2 Z(x)Z]), eps = 0.02, followed by
  qubit 1's gate; L = 30, N_circs = 10, p_idle = 0, N_rep = 1e5;
* detection (§7.1.3): the effects E_10 = (1 - p_m)|10><10| + p_m|11><11| and
  E_11 = (1 - p_m)|11><11| + p_m|10><10|, p_m = 0.01; L = 10, N_circs = 20, p_idle = 0,
  N_rep = 1e5;
* ladder (§7.2): six qubits, 0 1 2 over 5 4 3; X_pi/2 or Y_pi/2 on a bottom qubit depolarises its
  vertical neighbour with p = 0.01; gates depolarise locally at 0.01, idles at 5e-3; l = 20,
  N_circs = 10, p_idle = 0.1, N_rep = 1e4.

Two scenarios beyond the paper, with the one_way and detection designs: free, local depolarisation
alone; bath, a shared environment that flips both read-outs together with probability 0.01.

Each scenario writes `sarovar2020_<scenario>.csv`: one row per circuit, the bag index each region
ran (N_circs for the all-idle sequence), then the count of every outcome, qubit 0 the most
significant bit; and `sarovar2020_<scenario>_bags.csv`: each region's bag, one sequence per row in
X, Y and I. `sarovar2020_edges.csv` lists, per scenario, the cross-region edges of the PC skeleton
at alpha = 0.01 on the written draw, the edge set the paper publishes (for free, none; for bath,
R0-R1), and in how many of ten further draws, seeds 1000 to 1009, PC finds that set.
"""

import csv
import sys
import types
from functools import reduce

import networkx as nx
import numpy as np
import pcalg
import pygsti.extras.crosstalk as crosstalk
from scipy.stats import chi2

SEED = 20200907
GATES = "XYI"
I2 = np.eye(2, dtype=complex)
X = np.array([[0, 1], [1, 0]], dtype=complex)
Y = np.array([[0, -1j], [1j, 0]], dtype=complex)
Z = np.diag([1, -1]).astype(complex)
UNITARY = {"X": (I2 - 1j * X) / np.sqrt(2), "Y": (I2 - 1j * Y) / np.sqrt(2), "I": I2}

SCENARIOS = {
    "one_way": dict(qubits=2, L=30, n_circs=10, p_idle=0.1, n_rep=10_000),
    "zz": dict(qubits=2, L=30, n_circs=10, p_idle=0.0, n_rep=100_000),
    "detection": dict(qubits=2, L=10, n_circs=20, p_idle=0.0, n_rep=100_000),
    "ladder": dict(qubits=6, L=20, n_circs=10, p_idle=0.1, n_rep=10_000),
    "free": dict(qubits=2, L=30, n_circs=10, p_idle=0.1, n_rep=10_000),
    "bath": dict(qubits=2, L=10, n_circs=20, p_idle=0.0, n_rep=100_000),
}
P_LOCAL, P_IDLE_LOCAL_LADDER, P_CROSS, EPSILON, P_M, P_BATH = 0.01, 5e-3, 0.01, 0.02, 0.01, 0.01
VERTICAL = {3: 2, 4: 1, 5: 0}
EXPECTED = {"one_way": "R1-S0", "zz": "R0-R1 R0-S1 R1-S0", "detection": "R0-R1",
            "ladder": "R0-S5 R1-S4 R2-S3", "free": "", "bath": "R0-R1"}
FURTHER_SEEDS = range(1000, 1010)


def design(rng, qubits, L, n_circs, p_idle, **_):
    bags = [["".join(rng.choice(list(GATES), L)) for _ in range(n_circs)] for _ in range(qubits)]
    circuits, seen = [], set()
    for region in range(qubits):
        for index in range(n_circs):
            for _ in range(n_circs // 2):
                settings = [index if r == region else
                            (n_circs if rng.random() < p_idle else int(rng.integers(n_circs)))
                            for r in range(qubits)]
                if tuple(settings) not in seen:
                    seen.add(tuple(settings))
                    circuits.append(settings)
    return [bag + ["I" * L] for bag in bags], circuits


def depolarise(rho, p):
    return (1 - p) * rho + p * np.trace(rho) * I2 / 2


def single_qubit_marginals(scenario, sequences):
    """Per-qubit states for the product-preserving models: one_way, free and ladder."""
    states = [np.diag([1, 0]).astype(complex) for _ in sequences]
    for layer in zip(*sequences):
        nxt = []
        for q, gate in enumerate(layer):
            u = UNITARY[gate]
            rho = depolarise(u @ states[q] @ u.conj().T,
                             P_IDLE_LOCAL_LADDER if scenario == "ladder" and gate == "I" else P_LOCAL)
            if scenario == "one_way" and q == 1 and layer[0] == "X":
                rho = depolarise(rho, P_CROSS)
            if scenario == "ladder" and q in VERTICAL.values():
                source = next(s for s, t in VERTICAL.items() if t == q)
                if layer[source] != "I":
                    rho = depolarise(rho, P_CROSS)
            nxt.append(rho)
        states = nxt
    return [state[1, 1].real for state in states]


def two_qubit_distribution(scenario, sequences):
    """The outcome distribution over 00, 01, 10, 11 for the zz, detection and bath models."""
    rho = np.zeros((4, 4), dtype=complex)
    rho[0, 0] = 1
    zz = np.kron(Z, Z)
    for g0, g1 in zip(*sequences):
        if scenario == "zz" and g0 == "X":
            h = np.pi / 2 * np.kron(X, I2) + EPSILON / 2 * zz
            w, v = np.linalg.eigh(h)
            u = np.kron(I2, UNITARY[g1]) @ (v @ np.diag(np.exp(-0.5j * w)) @ v.conj().T)
        else:
            u = np.kron(UNITARY[g0], UNITARY[g1])
        rho = u @ rho @ u.conj().T
        for q in range(2):
            kraus = [np.sqrt(1 - 3 * P_LOCAL / 4) * I2] + [np.sqrt(P_LOCAL / 4) * P for P in (X, Y, Z)]
            ops = [np.kron(k, I2) if q == 0 else np.kron(I2, k) for k in kraus]
            rho = sum(k @ rho @ k.conj().T for k in ops)
    p = np.clip(np.diag(rho).real, 0, None)
    if scenario == "detection":
        p = np.array([p[0], p[1], (1 - P_M) * p[2] + P_M * p[3], (1 - P_M) * p[3] + P_M * p[2]])
    if scenario == "bath":
        p = (1 - P_BATH) * p + P_BATH * p[::-1]
    return p / p.sum()


def distribution(scenario, qubits, sequences):
    if scenario in ("zz", "detection", "bath"):
        return two_qubit_distribution(scenario, sequences)
    marginals = single_qubit_marginals(scenario, sequences)
    return reduce(np.kron, [np.array([1 - m, m]) for m in marginals])


def weighted_g_square(weights):
    """gsq's G-square test of x and y given s, over distinct rows each counted `weights` times.

    It forms the contingency table gsq's g_square_dis forms row by row, with the same degrees of
    freedom, the same ten-samples-per-degree requirement and the same p-value.
    """
    def test(dm, x, y, s, **_):
        levels = np.amax(dm, axis=0) + 1
        s = sorted(s)
        dof = (levels[x] - 1) * (levels[y] - 1) * int(np.prod([levels[z] for z in s]))
        if weights.sum() < 10 * dof:
            return 1
        k = (np.unique(np.ravel_multi_index(tuple(dm[:, z] for z in s), tuple(levels[z] for z in s)),
                       return_inverse=True)[1] if s else np.zeros(len(dm), dtype=int))
        nijk = np.zeros((levels[x], levels[y], k.max() + 1))
        np.add.at(nijk, (dm[:, x], dm[:, y], k), weights)
        nik, njk = nijk.sum(axis=1), nijk.sum(axis=0)
        with np.errstate(divide="ignore", invalid="ignore"):
            g2 = np.nansum(2 * nijk * np.log(nijk * njk.sum(axis=0) / (nik[:, None, :] * njk[None, :, :])))
        return 1 if dof == 0 else chi2.sf(g2, dof)
    return test


def estimate_skeleton(test, data, alpha, ignore_edges):
    """pcalg's skeleton search from the complete graph less `ignore_edges`.

    pyGSTi 0.10.2 passes the edges between settings, which its design randomises independently, as
    a fourth argument; the pcalg on PyPI takes the starting graph instead.
    """
    graph = nx.complete_graph(data.shape[1])
    graph.remove_edges_from(ignore_edges)
    return pcalg.estimate_skeleton(test, data, alpha, init_graph=graph)


crosstalk.core.pcalg = types.SimpleNamespace(estimate_skeleton=estimate_skeleton,
                                             estimate_cpdag=pcalg.estimate_cpdag)


def edges(qubits, circuits, counts):
    """The cross-region edges of pyGSTi's PC skeleton at alpha = 0.01, its tests counting each
    distinct shot once with its multiplicity."""
    rows, weights = [], []
    for settings, row in zip(circuits, counts):
        for outcome, n in enumerate(row):
            if n:
                rows.append([(outcome >> (qubits - 1 - q)) & 1 for q in range(qubits)] + list(settings))
                weights.append(n)
    crosstalk.core.ci_test_dis = weighted_g_square(np.asarray(weights, dtype=float))
    result = crosstalk.do_basic_crosstalk_detection(np.asarray(rows, dtype=np.int64), qubits,
                                                    [1] * qubits, confidence=0.99, verbosity=0)
    label = lambda node: f"R{node}" if node < qubits else f"S{node - qubits}"
    region = lambda node: node % qubits
    return sorted("-".join(sorted((label(a), label(b)))) for a, b in result.skel.edges()
                  if region(a) != region(b) and not (a >= qubits and b >= qubits))


def draw(rng, scenario, spec):
    bags, circuits = design(rng, **spec)
    counts = [rng.multinomial(spec["n_rep"],
                              distribution(scenario, spec["qubits"], [bags[r][s] for r, s in enumerate(c)]))
              for c in circuits]
    return bags, circuits, counts


def main(out_dir):
    rng = np.random.default_rng(SEED)
    found = []
    for scenario, spec in SCENARIOS.items():
        bags, circuits, counts = draw(rng, scenario, spec)
        q = spec["qubits"]
        with open(f"{out_dir}/sarovar2020_{scenario}_bags.csv", "w", newline="") as f:
            out = csv.writer(f, lineterminator="\n")
            out.writerow(["region", "index", "sequence"])
            for r, bag in enumerate(bags):
                out.writerows([r, i, s] for i, s in enumerate(bag))
        with open(f"{out_dir}/sarovar2020_{scenario}.csv", "w", newline="") as f:
            out = csv.writer(f, lineterminator="\n")
            out.writerow([f"s{r}" for r in range(q)] + [f"n{o:0{q}b}" for o in range(2 ** q)])
            out.writerows(list(c) + list(n) for c, n in zip(circuits, counts))
        found.append((scenario, " ".join(edges(q, circuits, counts))))
        print(scenario, len(circuits), "circuits;", found[-1][1])
    recovered = {s: 0 for s in SCENARIOS}
    for seed in FURTHER_SEEDS:
        further = np.random.default_rng(seed)
        for scenario, spec in SCENARIOS.items():
            _, circuits, counts = draw(further, scenario, spec)
            recovered[scenario] += " ".join(edges(spec["qubits"], circuits, counts)) == EXPECTED[scenario]
    with open(f"{out_dir}/sarovar2020_edges.csv", "w", newline="") as f:
        out = csv.writer(f, lineterminator="\n")
        out.writerow(["scenario", "edges", "published", "recovered_of_ten"])
        out.writerows([s, e, EXPECTED[s], recovered[s]] for s, e in found)


if __name__ == "__main__":
    main(sys.argv[1])
