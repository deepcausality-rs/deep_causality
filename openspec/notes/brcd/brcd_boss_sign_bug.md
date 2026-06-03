# BRCD bug report — BOSS local score is sign-inverted, so structure learning fails

**Component:** `BRCD/boss.py` + `BRCD/LocalScoreFunction.py` — `local_score_BIC_from_cov`
**Affects:** the no-CPDAG / Petshop path (`brcd_helper(cpdag=None)`, `main-petshop.py`),
and the `BRCD-M` ancestral-knowledge variant — everything that learns the CPDAG with
the **vendored** BOSS rather than a supplied service map.
**Severity:** High. The learned CPDAG is structurally wrong (often empty), so every
downstream BRCD posterior on the Petshop/bootstrap path is computed against a
near-meaningless graph. The supplied-CPDAG path (OB, Sock Shop) is unaffected.

## Summary

The vendored `local_score_BIC_from_cov` returns the **negated** linear-Gaussian
BIC (lower = better fit):

```python
# BRCD/LocalScoreFunction.py
if len(PAi) == 0:
    return n * np.log(cov[i, i])
...
H = np.log(cov[i, i] - yX @ inv_XX @ yX.T)
return n * H + np.log(n) * len(PAi) * lambda_value      # <-- lower is better
```

But the BOSS search it feeds **maximizes** this score. `gst.GSTNode.grow` keeps a
parent only when `score(with) > score(without)`, `shrink` removes a parent only when
`score(without) > score(with)`, and `boss.better_mutation` takes `np.nanargmax`.
With a *lower-is-better* score and a *maximizing* search, the two disagree in sign,
so the search is driven the wrong way:

- **grow** adds a parent only when it **fails** to reduce the residual variance
  (an *independent* parent → no fit gain, only the penalty term, which the negated
  score *rewards*); and
- it **rejects** a parent that *does* reduce the variance (a *real* edge → a fit
  gain, which the negated score now *penalizes*).

Net effect: the learner **connects independent variables and disconnects dependent
ones** — the exact inverse of structure learning.

For comparison, causal-learn's own `local_score_BIC_from_cov` is documented
"**higher is better**" (`-0.5*n*(1+log σ²) - λ*(|Pa|+1)*log n`) — the opposite sign.
The ICML 2026 BRCD paper states (Appendix D, "Real-world data experiment") that it
uses "the **default setting** of … BOSS **from causal-learn**", i.e. the
higher-is-better score — so the vendored copy diverges from what the paper describes
and would not reproduce the paper's Petshop numbers.

## Evidence — learned edges, wrong sign vs correct sign

Two generators, 600 samples, seeds fixed:
- **chain** `X → Y → Z`: `X~N(0,1)`, `Y = X + N(0,1)`, `Z = Y + N(0,1)`.
- **collider** `X → Z ← Y`: `X,Y ~ N(0,1)` independent, `Z = X + Y + noise·N(0,1)`.

Running the **vendored** BOSS (wrong sign) vs **causal-learn**'s BOSS (correct sign)
on identical data:

| data              | WRONG sign (vendored `boss.py`) | CORRECT sign (causal-learn) |
|-------------------|---------------------------------|-----------------------------|
| chain             | `[]`  (empty graph)             | `X — Y`, `Y — Z`  ✓ chain   |
| collider noise=0.2| `X — Y`  (one *wrong* edge)     | `X → Z`, `Y → Z`  ✓ collider|
| collider noise=1.0| `X — Y`                         | `X → Z`, `Y → Z`  ✓         |
| collider noise=3.0| `X — Y`                         | `X → Z`, `Y → Z`  ✓         |

The wrong sign learns the **empty** graph on a clean chain, and on every collider it
asserts the single edge between the **independent** pair `X — Y` while **dropping**
both real edges `X — Z`, `Y — Z`.

That `X — Y` edge is provably spurious — `X` and `Y` are generated independently:

| noise | corr(X, Y) | corr(X, Z) | corr(Y, Z) | partial corr(X, Y │ Z) |
|-------|-----------:|-----------:|-----------:|-----------------------:|
| 0.2   | **−0.040** | +0.693     | +0.676     | −0.958                 |
| 1.0   | **−0.040** | +0.559     | +0.556     | −0.510                 |
| 3.0   | **−0.040** | +0.302     | +0.314     | −0.149                 |

`corr(X, Y) ≈ 0` at every noise level (no marginal X–Y relation), while `corr(X, Z)`
and `corr(Y, Z)` are substantial (the real edges). The strongly-negative
`partial corr(X, Y │ Z)` is the collider signature — `X ⫫ Y` but `X ̸⫫ Y │ Z` — so the
correct structure is unambiguously the collider, and the vendored output is the
inverse of it.

## Root cause

A sign mismatch between the score and the search direction. The local score returns
`n·ln(σ²) + ln(n)·|PA|·λ` (proportional to `+`BIC, **lower** is better), while BOSS's
GST grow/shrink and `better_mutation` are written to **maximize**. They should agree:
either negate the score (higher is better, matching causal-learn) or invert the
search's comparisons.

## Fix (one line)

Return the higher-is-better score the search expects — i.e. negate it (equivalently,
adopt causal-learn's form):

```python
# BRCD/LocalScoreFunction.py  — higher is better, matching the maximizing search
if len(PAi) == 0:
    return -0.5 * n * np.log(cov[i, i]) - 0.5 * np.log(n) * lambda_value
H = np.log(cov[i, i] - yX @ inv_XX @ yX.T)
return -0.5 * n * H - 0.5 * np.log(n) * (len(PAi) + 1) * lambda_value
```

(Per-node-constant terms do not affect the `argmax` over parent sets, so the
proportional `-n·ln(σ²) - ln(n)·|PA|·λ` is equivalent for the search.)

## How to reproduce

The vendored `boss.py` uses relative imports; load its modules under a synthetic
`BRCD` package, then run both BOSS variants on the data above:

```python
import importlib.util, sys, types, os, numpy as np, random
BRCD_DIR = ".../ctx/next/brcd/BRCD"
pkg = types.ModuleType("BRCD"); pkg.__path__ = [BRCD_DIR]; sys.modules["BRCD"] = pkg
def load(m, f):
    spec = importlib.util.spec_from_file_location(f"BRCD.{m}", os.path.join(BRCD_DIR, f))
    mod = importlib.util.module_from_spec(spec); sys.modules[f"BRCD.{m}"] = mod
    spec.loader.exec_module(mod); return mod
load("LocalScoreFunction", "LocalScoreFunction.py")
load("LocalScoreFunctionClass", "LocalScoreFunctionClass.py")
repo_boss = load("boss", "boss.py").boss                          # vendored, WRONG sign
from causallearn.search.PermutationBased.BOSS import boss as cl_boss  # CORRECT sign

def collider(n, s, noise):
    r = np.random.RandomState(s)
    x = r.normal(0,1,n); y = r.normal(0,1,n); z = x + y + noise*r.normal(0,1,n)
    return np.column_stack([x, y, z])

X = collider(600, 1, 1.0)
random.seed(0); np.random.seed(0); print(repo_boss(X, verbose=False).graph)  # X—Y only
random.seed(0); np.random.seed(0); print(cl_boss(X, verbose=False).graph)    # X→Z, Y→Z
```

To confirm the diagnosis end-to-end, monkey-patch the vendored score to return its
negation (preserving `__name__ = "local_score_BIC_from_cov"` so the covariance branch
still fires); the **same** `boss.py` algorithm then recovers the chain and the
collider (at identifiable noise), confirming the sign is the sole defect.

## Note — a second, independent issue surfaced while reproducing

The vendored `BRCD/LocalScoreFunctionClass.score_nocache` passes the **raw data**
matrix to `local_score_BIC_from_cov` (whose covariance branch keys off
`self.local_score_fun.__name__`). It works only because the function is named
`local_score_BIC_from_cov`; any wrapper/rename silently routes to the wrong branch
and unpacks `cov, n = Data` from the raw array. Worth tightening, but separate from
the sign bug above.
