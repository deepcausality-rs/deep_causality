# Composite RCA — a multi-resolution causal diagnostic stack for CDL

**Status:** Forward-looking design hypothesis. Not committed work; not yet validated. Records a research direction that emerged from the BRCD investigation ([BRCD.md](BRCD.md)). Goal: capture the idea, a realistic use case, the architecture, and the honest risks — before any prototyping.

**One line:** Combine a regime-aware *fault localizer* (the clean essence of BRCD) with SURD's information-theoretic *contribution decomposition* so CDL can answer both **"which node broke"** and **"which of that node's underlying variables caused it"** — a multi-resolution diagnosis neither tool delivers alone.

---

## 1. The gap each tool leaves

| | Resolution | Consumes | Answers | Blind spot |
|---|---|---|---|---|
| **SURD** (have it) | sub-node / variable | one dataset, a target + its sources | how sources drive a target: synergistic / unique / redundant | cannot *localize* a fault across a network; combinatorially explodes with #sources, so can't run fleet-wide |
| **BRCD / F-node localizer** (the essence worth porting) | network / node | two datasets (normal + anomalous) + a graph | *which node's* conditional mechanism changed under the failure | treats each node as a black box — no insight into the node's internal drivers |

- **SURD is too low-level** to see a system-wide cascading failure: it needs you to already point at a target and its sources.
- **BRCD is too high-level** to see the contributing variables: it pins the node but says nothing about *why* that node changed.

They are not substitutes — they operate at different resolutions on different data regimes, and each covers the other's weakness.

---

## 2. The hypothesis: a diagnostic stack with the graph as the interface

```
CDL (causal graph: learned or supplied)          ← the shared substrate / interface
   │
   ├── BRCD-essence localizer  (two-regime, F-node, conditional-shift scoring)
   │      → "node R is where the mechanism changed" (the root, not the collateral)
   │
   └── SURD on R's neighborhood / internal variables
          → "R's change was driven by a UNIQUE cause C, with SYNERGY between {A,B}"
```

The common artifact is **the graph**: the localizer ranks nodes *in* the graph; SURD decomposes contributions *for a node* in the graph. CDL already unifies graph + data + reasoning, so it is the natural host. (This also strengthens the case in [BRCD.md §13](BRCD.md) for a **shared Tier-B causal-graph layer** — it is the common substrate of the whole stack, not BRCD plumbing.)

### 2.1 The key refinement — *differential* SURD

BRCD is inherently two-regime; SURD is single-dataset. The powerful move is to run SURD on **both** regimes and diff:

> SURD(normal) vs SURD(anomalous) on the localized node → "under the failure, the synergistic contribution from {A,B} collapsed and a new unique dependence on C emerged."

This reveals *how the contribution structure changed*, which is strictly more than either tool alone produces. This differential decomposition is the actual prize.

### 2.2 Two valid SURD targets (disambiguated)

1. **SURD on the root-cause node R itself** → during the fault, R should *decouple* from its normal parents (it is now driven by the exogenous fault, not its parents). A collapsing unique/synergistic input *confirms* R is genuinely intervened-on, not merely propagating.
2. **SURD on a downstream effect X that R feeds** → attributes how R-and-siblings propagated into X.

So the composition is naturally a **recursive drill-down**: localize → attribute → if a contributor is itself suspect, localize again.

### 2.3 The mutual benefit (why this is more than a pipeline)

- **Localization makes SURD tractable.** SURD's redundancy/synergy lattice grows ~`2^n` in the number of sources, so it cannot run fleet-wide. The localizer restricts it to *one node and its modest variable set* — exactly where SURD is computationally sane.
- **SURD gives the localizer its missing "why."** The localizer points at the node; SURD explains the mechanism. Each covers the other's blind spot.

---

## 3. Worked use case — network cascading-failure root cause

**Setting.** A network of devices (routers, switches), each continuously streaming high-rate health telemetry (a *gazillion* variables per device: CPU, line-card temperature, PSU rail voltages, ingress/egress queue depth, packet drops, BGP session state, fan RPM, …). The **network topology graph** (which device depends on / connects to which) is known and maintained.

**Incident.** A router goes offline and takes down a whole network segment. Because the failure **cascades**, every downstream device in the segment simultaneously trips threshold alarms — so a naive monitor lights up everywhere and cannot distinguish the **root** from the **collateral**.

**Diagnosis with the stack:**

1. **Structure (CDL).** The topology graph is the device-level causal/dependency structure (oriented along dependency direction — note the call-graph-style direction convention; see [BRCD.md §14/§15]).
2. **Localize (BRCD-essence).** Take a *normal* window and a *failure* window of device-level telemetry. The two-regime conditional-shift test asks, for each device, *did its conditional behavior change independently of its parents?* The collateral devices' anomalies are **explained by their dependence on the dead router** (no independent mechanism change) — so they score low. The offline router scores high: its mechanism changed exogenously. → **Root = router R7**, not the dozens of collateral devices.
3. **Attribute (differential SURD).** Pull **R7's own telemetry stream** (normal vs failure window) and run SURD over its internal variables. The differential decomposition reveals which variable(s), and in what mode, drove R7's state change — e.g. *"unique cause: PSU rail voltage sag; synergistic: ingress-queue depth × line-card temperature."*

**Result:** "Segment outage root cause = router **R7**; R7 failed due to **PSU voltage collapse** (unique cause), amplified by a **thermal × buffer-pressure synergy**." Actionable down to the variable — the maintenance ticket writes itself.

**Why neither tool alone:** BRCD says "R7" but treats R7 as a black box (no *why*). SURD over the whole fleet can't localize and would drown in the `N×M` variable explosion. Localization narrows SURD to R7's `M` variables; SURD turns "R7 is down" into "R7's PSU sagged." The cascade — the thing that defeats threshold monitors — is exactly what the conditional-shift test is built to cut through (intervention vs propagation).

**Domain bonus:** high-rate streaming telemetry *mitigates* the worst seam below — the failure window can still accumulate many samples quickly, easing SURD's anomalous-side sample hunger.

---

## 4. Honest seams / risks (must validate, do not oversell)

1. **Anomalous-side sample starvation.** SURD needs enough samples to estimate information quantities; failure windows are short. SURD(normal) will be solid; SURD(anomalous) is the binding constraint. High-rate domains (networking, sensors) help; slow/rare faults hurt. **Confirmed by the SURD-states paper itself — see §7.**
2. **Representation impedance.** SURD is information-theoretic (wants discretized/binned states); the localizer's Gaussian path is continuous. The stack needs one coherent discretization story, and the binning choice affects SURD's decomposition.
3. **Shared graph-quality dependency.** Both stand on the structure. A wrong graph makes both wrong *together*, confidently. The stack inherits, not fixes, structure correctness.
4. **Interpretability discipline.** "A synergistic cause broke" means a *joint* term changed; which member is culpable still needs further localization. Don't let the decomposition imply more attribution than it delivers — this is what the recursive drill-down (§2.2) is for.

---

## 5. What to build, and first experiment

- **Build the localizer essence, not the BRCD reference.** The clean two-regime / F-node / conditional-shift ranker over a supplied-or-learned graph (the [BRCD.md §14](BRCD.md) spec) — deterministic, in-repo numerics, no cliquepicking/KDE/BOSS apparatus. It is the middle layer of this stack.
- **Reuse SURD as-is** for the attribution layer (already in `deep_causality_algorithms`).
- **CDL orchestrates**: holds the graph, routes the two-regime data to the localizer and the per-node windows to SURD, composes the report.
- **First validation experiment** (the make-or-break): differential-SURD-on-a-localized-node on a system where the fault is *controlled* (inject a known device/variable fault, confirm the stack recovers both the node and the driving variable). Networking telemetry or a simulated cascading failure is ideal because the high sample rate addresses seam #1. Prove the differential decomposition survives the anomalous-window sample budget before committing this as a CDL headline.

---

## 6. Relationship to the BRCD effort

This reframes the BRCD work ([BRCD.md](BRCD.md)): the value is **not** a faithful port of an incremental, messy reference, but extracting its **localizer essence** as the middle layer of this stack. The composite — regime-aware localization + information-theoretic differential attribution, with localization making attribution tractable — is arguably a more novel and more useful capability than BRCD's own delta over RCD, and it is one CDL is uniquely positioned to offer. Treat this note as the *why*; treat BRCD.md §14 as the *what to build* for the localizer.

---

## 7. Substantiation from the SURD-states paper

**Reference:** Á. Martínez-Sánchez & A. Lozano-Durán, *"Observational causality by states and interaction type for scientific discovery,"* arXiv 2505.10878v2 (2025) — `docs/papers/surd-state.pdf`. This is the **state-dependent** SURD, and per the CDL README it is exactly the method CDL already implements as `surd_states_cdl`. ⇒ the stack's attribution layer needs **no new algorithm — only orchestration.**

**How it substantiates the composite:**

1. **State-dependent by construction → differential attribution is native, not a bolt-on.** SURD-states characterizes causal influence *as a function of system state* (Figs 2–5 are 2D maps of causal contribution over source-state × target-state, e.g. "this causality is active only when `q₁>0`"). The composite's "how did the contribution structure change under the failure?" is precisely a contrast across state-regions — the method is *built* to show that. (Refines §2.1: you may not even need two separate runs; the state map already concentrates causal contribution into the anomalous state-region.)

2. **U/S/R + causality-leak is exactly the attribution the stack needs.** The decomposition is redundant / unique / synergistic *plus* a **causality leak** `ΔI_leak` — information about the target left unexplained by the observed variables. The leak is a free bonus for RCA: a **high leak on the localized node means the true driver is unobserved** — your telemetry doesn't capture the cause (→ instrument more). Neither BRCD nor naive SURD surfaces that.

3. **Time-series-native, with a forecast horizon `ΔT`** (forward-in-time information propagation, `ΔT>0`). Fits streaming telemetry directly: sources = the node's past internal channels, target = its future health/output channel.

4. **It is rigorous and well-validated** — controlled benchmarks with known ground-truth direction + two real physical systems (wall turbulence inner/outer-layer coupling; Pacific Walker circulation). This is the **trustworthy anchor** of the stack, in sharp contrast to the BRCD reference code. The owner's read ("quite good for what it is") holds up.

**It also independently confirms the composite's main risk (§4 seam #1).** In the Walker-circulation application (780 monthly samples) the authors state the data *"was insufficient to reliably estimate synergistic causalities, which are therefore not included,"* and restrict to *"at most two variables."* So **synergy is the most sample-hungry component and the first to fail under limited data** — exactly the anomalous-window-starvation seam. Concrete implication: in a short failure window the differential SURD will reliably yield **redundant + unique** contributions, but **synergistic** attribution may be unavailable unless the regime is sample-rich. High-rate telemetry (networking, sensors) keeps synergy in reach; slow/rare faults force a redundant+unique-only attribution. State this up front, don't discover it late.

**Refinement to the attribution layer (§5):** specify the SURD invocation explicitly — **target** = the node channel BRCD flagged as shifted (or the node's designated health/output channel); **sources** = the node's other internal channels; **`ΔT`** = forecast horizon (chosen, as in the paper, to maximize the cross-induced unique causality); run on the normal window and — sample-permitting — the anomalous window, then contrast. Because `surd_states_cdl` already produces the per-state decomposition, the differential is a comparison over its existing output, not new math.
