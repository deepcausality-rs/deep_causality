# BRCD bug report — posterior ranking underflows, corrupting secondary root-cause ranks

**Component:** `brcd.py` — `brcd_update` (posterior assembly) + `brcd_helper` (ranking)
**Affects:** both the standalone `brcd.py` and `RCAEval/e2e/brcd.py` (identical code path)
**Severity:** High for any top-k / multi-root evaluation; the **top-1** result is unaffected.

## What the paper specifies (golden reference)

BRCD ranks candidate root-cause sets `R` by the **posterior** `p(R | D)`
(ICML paper, §4):

> **Eq. (3):**  `p(R | D) = p(D | R)·p(R) / Σ_{R'} p(D | R')·p(R')`
> **Eq. (4):**  `p(D | R) = Σ_{G ∈ [G*]} p(D | G, R)·p(G | R)`
> **Algorithm 2, line 7–8:** `p(R|D) ← p(D|R)p(R) / (Σ_{R'} p(D|R')p(R'))`; `return p(R|D)`
> **Figure 1:** "Rank by Posterior → Top Root Causes."

The denominator of Eq. (3) is **constant across `R`**, so ranking by `p(R | D)` is
order-identical to ranking by the numerator `p(D | R)·p(R)`, i.e. by the
**log-posterior** `log p(D|R) + log p(R)`. That is the quantity the algorithm
already computes internally (`log_posterior`).

Crucially, the paper proves the posterior **concentrates on the true cause**:
**Theorem 4.3** gives `p(G*, R* | D) → 1` as `n → ∞`, and Figure 1 notes "as the
number of anomaly samples grows, the posterior of `R=Y` will get closer to 1."
So on any real failure the top candidate's log-posterior dominates the rest by a
large margin **by design** — which is exactly the regime that breaks the
implementation's `exp` shortcut below.

## Summary

The implementation does **not** rank by `p(R | D)` (Eq. 3) directly. Instead it
**exponentiates** the per-candidate log-posterior with a max-shift and sorts that:

```python
# brcd_update (brcd.py ~L1854–1856 ; RCAEval/e2e/brcd.py ~L2218–2220)
log_posterior = log_likelihood + np.log(prior)
posterior     = np.exp(log_posterior - log_posterior.max())   # <-- underflows

# brcd_helper (brcd.py ~L1931 ; RCAEval/e2e/brcd.py ~L2312)
sorted_indices = np.argsort(-posterior)                        # <-- ties → index order
```

When one candidate dominates (a real fault summed over hundreds of rows easily
exceeds the top by **> ~709 nats**), `np.exp(log_posterior − max)` **underflows to
`0.0`** for every other candidate. `np.argsort(-posterior)` then sees a sea of
exact `0.0`s and falls back to **ascending index order**. So every rank below #1
is a tie-break artifact, not a score — the secondary ranking carries no signal.

## Evidence — Online Boutique, `adservice_cpu` case

For this case the supplied CPDAG is fully directed and the candidate variables are
parentless, so BRCD scores each variable by its **between-regime mechanism shift**
(per-regime vs pooled Gaussian). Ranking by the *log*-posterior (no underflow)
recovers exactly that; ranking by the exponentiated posterior does not.

`shift(σ)` = `|mean_anom − mean_normal| / within-regime-std`. **`Rust#`** ranks on
the log-posterior (correct); **`Py#`** is the current `expected.txt` (exp-then-argsort).

| col | metric | shift (σ) | correct rank (log-posterior) | BRCD `expected.txt` |
|----:|--------|----------:|----------------:|-----------:|
| 0 | adservice_cpu (the injected fault) | 9.22 | **1** | **1** |
| 12 | adservice_mem | 12.52 | 2 | **36** |
| 44 | time.1 | 3.46 | 4 | 45 |
| 18 | main_mem | 2.98 | 5 | 42 |
| 19 | paymentservice_mem | 2.71 | 6 | 43 |
| 13 | cartservice_mem | 1.74 | 8 | 37 |

…and the variables BRCD currently ranks **#2–#6** have essentially **zero** shift:

| col | metric | shift (σ) | correct rank | BRCD `expected.txt` |
|----:|--------|----------:|-------------:|-----------:|
| 24 | adservice_load | 0.02 | 39 | **2** |
| 25 | cartservice_load | 0.07 | 36 | **3** |
| 26 | checkoutservice_load | 0.04 | 34 | **4** |
| 27 | currencyservice_load | 0.06 | 37 | **5** |
| 28 | emailservice_load | 0.06 | 41 | **6** |

So BRCD reports five *unchanged* variables (≤ 0.07σ) at ranks 2–6 and buries the
clearly-changed `*_mem` variables (3–12σ) at ranks 36–43. The tell-tale signature
in `expected.txt` is **runs of consecutive indices** (`…,24,25,26,…,42,…` then
`…,1,2,3,…,21,…`) — classic `argsort` index-tie-breaking over underflowed `0.0`s.

(Sock Shop cases are unaffected: no single candidate dominates, nothing
underflows, and the full top-k is correct.)

## Root cause

The ranking key is the underflow-prone `exp(lp − max)` instead of the posterior
`p(R | D)` of Eq. (3) (equivalently, the log-posterior). `exp` is monotonic, so
`argsort(-exp(lp − max))` *would* match `argsort(-lp)` — **except** where `exp`
underflows many distinct `lp` values to the same `0.0`, which destroys their
order. By **Theorem 4.3** the posterior concentrates on the true cause as samples
grow, so on a real failure the gap from the top candidate to the rest reliably
exceeds `ln(f64::MAX) ≈ 709`, and the underflow is not an edge case — it is the
expected regime.

## Fix (one line)

Rank by the posterior of Eq. (3) directly. Since its denominator is constant
across `R`, this is just ranking on the log-posterior:

```python
# brcd_helper — rank by p(R|D) ∝ p(D|R)p(R), i.e. by log_posterior
sorted_indices = np.argsort(-log_posterior)     # full resolution, no underflow
```

i.e. return / thread `log_posterior` through to the sort instead of the
max-shifted `posterior`. (If the normalized `p(R | D)` of Eq. (3) is wanted as a
reported weight, compute it with a log-sum-exp normalization —
`p = exp(lp − logsumexp(lp))` — rather than `exp(lp − max)`, and still sort on
`lp`.) This changes nothing for top-1 and restores the paper's intended ordering
for every lower rank.

## How to reproduce / verify

Run `brcd_helper` on the committed Online Boutique `adservice_cpu` inputs
(`normal.csv`, `anomalous.csv`, `cpdag.txt`); the returned `ranks` place
`adservice_load` (no shift) at #2. With `argsort(-log_posterior)` (Eq. 3 ordering)
the `*_mem` / `*_cpu` variables (real shifts) move to the top and the ranking
tracks the per-variable mechanism shift. (Independently confirmed by a
from-scratch BRCD reimplementation that follows the paper — Algorithm 1/2 and
Eq. 3 — ranking on `p(R|D)` via the log-posterior; it reproduces Sock Shop's full
top-k exactly and disagrees with OB's `expected.txt` only on the
underflow-corrupted tail.)
