# BHPI Data Acquisition — Windowed Multi-Reference RCA

Status: scoping note. Date: 2026-06-20.
Companion to `bhpi-groundwork.md` (this is the data front end for the §5 learner and §6 causal layer).

## 1. The problem this solves

Classical do-calculus RCA wants two synchronized datasets — a known-good baseline and a known-faulty
set — so confounders can be eliminated until a root cause remains. In practice you do not have that.
You have a **stream of metrics/telemetry that is ≥99% normal and <1% faulty**, with no clean,
time-synchronized control series. BRCD inherits this synchronicity problem.

This note defines how to construct a valid, quasi-interventional dataset from that stream using only
data every observability stack already stores.

## 2. The core reframe

Two facts make the construction sound:

1. **The fault is the intervention.** A real incident is an unplanned `do()` on some node — something
   forced off its normal mechanism. Production already performed the intervention; nothing needs to be
   injected. The faulty window is therefore *interventional data by nature*, which is what licenses
   causal (not merely associational) discovery.
2. **Known-good windows are reference environments.** Comparable normal windows are the observational
   baseline against which the intervened environment is contrasted.

Windowing thus converts a 99%-normal stream into a **multi-environment, quasi-interventional dataset**:
reference environments `e_1 … e_K` (good windows) plus one intervened environment `e_fault`.

## 3. Window construction

Given an incident of duration `D` with start/end timestamps from the alerting system:

- **Faulty window** = the incident interval extended by a margin `m` on each side
  (default `m = D/8`, i.e. ~12.5% — e.g. a 20-min incident → 2.5 min each side → 25 min total).
  The leading margin captures the onset (causally first-moving signal); the trailing margin captures
  the ramp/recovery.
- **Reference windows** = `K` known-good windows, **each of the same length as the faulty window**
  (25 min in the example), drawn from periods labelled normal by the alerting system.

The output is `K + 1` equal-length, multivariate windows over the same set of services/metrics, fed to
the learner as `K` reference environments plus one fault environment.

## 4. REQUIREMENT: K comparable normal windows (`K ≥ 1`, and `K > 1` is mandatory in practice)

This is the load-bearing requirement and is **not optional**. A single good window differs from the
fault window in many ways besides the fault (time-of-day load, deploys, batch jobs, autoscaling,
seasonality), so a single contrast confounds the fault with whatever else differed. The confounder
elimination is achieved **structurally, by invariance across multiple reference windows**: a cause is a
mechanism that is invariant across *all* `K` good windows but breaks in the fault window; any
confounder that varies across the good windows cannot masquerade as the invariant cause. More
reference environments ⇒ stronger confounder elimination. Use `K ≥ 3` where data allows.

### Why this is trivial in most deployments

The good windows do not need to be hand-curated. They are selected by **matching on operational
covariates the telemetry already carries**, so the match is a query, not a labelling project:

- **Day-of-week** — same weekday as the incident (Mon-pattern ≠ Sun-pattern).
- **Time-of-day** — same clock slot (the 3pm-peak fault matched against prior 3pm windows, never a 3am
  window), to cancel diurnal seasonality.
- **Typical load** — comparable request rate / throughput / active-flow level, so the comparison is
  like-for-like in activity, not just clock duration.
- **Build/deploy version** — same released version where possible (a deploy is itself an intervention).
- **Region / tier / shard** — same topology slice.
- **Adjacency** — the window immediately before the leading margin, plus same-slot prior periods
  (yesterday, last week), which match seasonality by construction.

A standard, low-friction selection: take the **same time-of-day slot on the `K` preceding comparable
days** (skipping any day that itself contains an alert), all at matching load tier. That yields several
diverse-yet-comparable reference windows from data the company already stores — no instrumentation
change, no fault injection, no labelled training set.

## 5. The discovery criterion (what makes this more than a noisy diff)

A magnitude diff of fault-vs-good ranks every shifted node and produces long, noisy candidate lists,
because the fault propagates and *every descendant* shifts too. The principled criterion the
invariance layer encodes:

> The root cause is the node whose **incoming mechanism breaks** in the fault window (the `do()`
> severed it from its normal parents), while its descendants' mechanisms stay intact — only their
> *inputs* shifted.

Mechanism-change localizes the root; input-change identifies descendants. Orientation comes from two
complementary signals: **within-window temporal precedence** (the root moves first; preserve the
within-window ordering, do not collapse to an aggregate) and the **invariance break** across
environments.

## 6. Failure modes to control

1. **Non-stationarity confounding (dominant risk)** — mitigated by §4: `K > 1`, diverse, covariate-
   matched, time-adjacent good windows. This is the single most important control.
2. **Recovery contamination** — the trailing margin may contain mitigation actions (failover, restart,
   throttle), themselves interventions that look causal. Handle the recovery phase separately or
   exclude it; do not conflate "what caused it" with "what fixed it."
3. **Match on activity, not just clock duration** — equal minutes ≠ equal information if load/sampling
   differ. Match comparable sample counts and operating-mode coverage.
4. **Exogenous coherent shift** — if the whole system shifts together (e.g. a traffic spike that *is*
   the exogenous cause), there may be no internal root node. The method must be able to report
   "consistent with an external driver, no internal root cause" rather than forcing a node. The
   hypergraph represents this naturally (an exogenous driver is a wide hyperedge over many outcomes).
5. **Thin data at <1% / short incidents** — a 20-min incident at 1-min resolution is ~20 samples per
   series. This is where the Bayesian, uncertainty-quantified, strength-borrowing learner beats a
   point-estimate method: it returns a *calibrated posterior over candidates*, not a false-confident
   single answer. Treated as a strength, not a weakness.

## 7. Fit with the architecture

This is the data-construction front end for the layers in `bhpi-groundwork.md`:

- §4 windowing → produces the environments `{e_1…e_K, e_fault}`.
- §6 (groundwork) invariance layer → keeps mechanisms stable across the `K` good windows; flags the
  break in the fault window.
- §5 (groundwork) learner → returns the calibrated causal hypergraph; orientation from within-window
  precedence + the invariance break.
- Output: the localized root-cause pathway(s) with calibrated uncertainty (a ranked set, not a forced
  single answer — real incidents can have multiple contributing causes, which the overlapping
  hypergraph represents directly).

## 8. Adoption

The decisive property: the inputs are a **trivial query against data the company already has**. Every
observability stack (Prometheus, Datadog, OpenTelemetry) stores time-series with alert/incident
timestamps. "Here is the incident window; here are `K` covariate-matched normal windows" is a query,
not a project — no instrumentation change, no fault injection, no labelled data. It is rare for a
causal method to require *nothing new* from the customer, and that is the immediate path to adoption.

Prior art the design sits within (defensible, not naive): Invariant Causal Prediction (Peters,
Bühlmann, Meinshausen); anomaly-as-intervention RCA (e.g. RCD, CIRCA). The novel part is the
combination with a Bayesian *overlapping-hypergraph* learner that returns calibrated uncertainty over
multiple candidate pathways — better matched to the thin-data, possibly-multi-cause reality than
existing point-estimate methods.
