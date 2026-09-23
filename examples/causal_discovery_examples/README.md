# Causal Discovery Examples

This crate holds the runnable examples for the
[`deep_causality_algorithms`](../../deep_causality_algorithms) and
[`deep_causality_discovery`](../../deep_causality_discovery) crates. They cover
information-theoretic decomposition (SURD), feature selection (mRMR), and the
end-to-end Causal Discovery Language (CDL) pipeline.

## Examples

| Example         | Topic                                                                 | Command                                                                  |
|-----------------|-----------------------------------------------------------------------|--------------------------------------------------------------------------|
| SURD            | Decomposing causal structure (Redundant/Unique/Synergistic + Leak)    | `cargo run -p causal_discovery_examples --example example_surd`          |
| mRMR            | Minimum-Redundancy Maximum-Relevance feature selection                | `cargo run -p causal_discovery_examples --example example_mrmr`          |
| mRMR (CDL)      | mRMR over data with missing values via the `Option<f64>` cleaner path | `cargo run -p causal_discovery_examples --example example_mrmr_cdl`      |
| CDL (SURD)      | Full SURD pipeline: load -> clean -> mRMR -> SURD -> analyze          | `cargo run -p causal_discovery_examples --example example_surd_discovery`     |
| CDL (BRCD)      | BRCD root-cause ranking with a supplied CPDAG (real Sock Shop data)  | `cargo run -p causal_discovery_examples --example example_brcd_discovery`     |
| CDL (BRCD/BOSS) | BRCD root-cause ranking, CPDAG learned from data via BOSS            | `cargo run -p causal_discovery_examples --example example_brcd_boss_discovery` |
| ML × Causal RCA | Candle anomaly detector gates the BRCD root-cause explainer through a `PropagatingProcess` ("ML detects, causality explains"); verdict checked against shipped ground truth | `cargo run -p causal_discovery_examples --example example_ml_rca` |

---

## Example: Decomposing Causal Structure with SURD-States

This example runs the `surd_states` algorithm to decompose the causal relationships in raw data from dynamic systems,
and shows how to interpret its output.

Beyond a plain "A causes B" statement, SURD-states answers four questions:

1. **What is the interaction type?** Is the causal influence from a set of variables **Redundant** (overlapping), *
   *Unique** (direct and independent), or **Synergistic** (emerging only from the combination)?
2. **How strong is the influence?** How much information, measured in bits, does a source variable provide about the
   future of a target variable?
3. **What is the role of hidden factors?** How much of the target's behavior is unexplained by the variables we can see?
   This is the **Information Leak**.
4. **How does causality change with the system's state?** Does a variable become more or less influential depending on
   its current value or the value of other variables? This is revealed by the **State-Dependent Maps**.

## How to Run This Example

```bash
cargo run -p causal_discovery_examples --example example_surd
```

The example runs four test cases, each with a different underlying causal structure, and prints a breakdown of each.

## Understanding the Output

The example analyzes a system with two source variables, `S1` and `S2`, and one target variable, `T`. The output
for each test case has two parts: the aggregate (average) causal effects and the state-dependent maps.

---

### Test Case 1: Original Example Data

A baseline case with mixed causal influences.

```
--- SURD Decomposition Result ---
Aggregate Redundant Info: {: 0.278...,: -6.66e-17}
Aggregate Synergistic Info: {: 0.121...}
Aggregate Mutual Info: {: 0.399...,: -6.66e-17,: 0.278...}
Information Leak: 0.599...
```

**Interpretation:**

* **Information Leak (~60%):** Read this number first. **60% of the target's future behavior is unexplained** by the
  source variables `S1` and `S2`, which points to unobserved factors (noise or hidden variables).
* **Aggregate Mutual Info:** `[2]: 0.278` shows that, on average, `S2` carries substantial information
  about `T`. `[1]: ~0` shows that `S1` carries almost none on its own.
* **Decomposition:** The total information from `S2` (`0.278` bits) is almost entirely **Redundant**. The `Synergistic`
  term (`0.121` bits) represents the *new* information that emerges only when we consider `S1` and `S2` together.
* **Conclusion:** `S2` is the main driver. `S1` is only useful when considered in combination with `S2`.

---

### Test Case 2: Low Information Leak (Strong, Synergistic System)

A system in which the sources largely determine the target.

```
--- SURD Decomposition Result ---
Aggregate Redundant Info: {: 0.0,: 0.0}
Aggregate Synergistic Info: {: 0.531...}
Information Leak: 0.468...
```

**Interpretation:**

* **Information Leak (47%):** The source variables explain more than half of the target's behavior.
* **Redundant & Unique Info are Zero:** `S1` alone or `S2` alone tells us nothing, the signature of an XOR-like
  relationship.
* **Synergistic Info is High:** All the causal influence (`0.531` bits) comes from the **synergy** between `S1` and
  `S2`. You *must* know both inputs to predict the output.
* **Conclusion:** The algorithm identifies a purely synergistic causal structure.

---

### Test Case 3: Medium Information Leak (Mixed System)

A system in which `S2` is the primary driver and `S1` has a small, independent effect.

```
--- SURD Decomposition Result ---
Aggregate Redundant Info: {: 0.014...,: 0.122...}
Aggregate Synergistic Info: {: 0.005...}
Information Leak: 0.857...
```

**Interpretation:**

* **Information Leak (86%):** The system is noisier; most of the target's behavior is unexplained.
* **Decomposition:** `S2` has the largest influence (`0.122` bits of **Redundant
  ** info). `S1` has a smaller, non-zero influence (`0.014` bits). The synergy is small.
* **State-Dependent Maps are Non-Empty:** The output `Causal Unique States: [[2]]` shows that the unique causal
  influence of `S2` is strong enough to be identified in specific states of the system.
* **Conclusion:** The algorithm identifies a noisy system dominated by one variable, with minor influence
  from another.

---

### Test Case 4: High Information Leak (Nearly Random System)

A system in which the target is almost independent of the sources.

```
--- SURD Decomposition Result ---
Aggregate Synergistic Info: {: 0.00115...}
Information Leak: 0.998...
```

**Interpretation:**

* **Information Leak (99.9%):** The source variables have **almost no predictive power** over the target.
* **Aggregate Info is Near Zero:** The total mutual information (`0.00115` bits) is statistical noise.
* **State-Dependent Maps are Empty:** With no causal structure present, the algorithm reports **no specific states**
  where a significant causal interaction occurs.
* **Conclusion:** SURD-states analyzes a random system and finds no causality, which shows that it separates
  signal from noise.

---

### Example: Minimum Redundancy Maximum Relevance (mRMR) Feature Selection

This example runs the `mrmr_features_selector` algorithm, which selects the features most relevant to a target variable and least redundant among themselves, and returns a normalized importance score (between 0 and 1) for each selected feature.

## How to Run This Example

```bash
cargo run -p causal_discovery_examples --example example_mrmr
```

## Understanding the Output

The example prints the selected feature indices with their normalized importance scores.

```
Selected features and their scores:
- Feature Index: 0, Importance Score: 1.0000
- Feature Index: 1, Importance Score: 0.0855
```

**Interpretation:**

*   **Feature Index:** The original column index of the selected feature in the input tensor.
*   **Importance Score:** The feature's relative importance, normalized to between 0.0 and 1.0.
    *   For the **first selected feature**, this score represents its relevance (F-statistic) to the target, normalized against the maximum score found.
    *   For **subsequent features**, this score represents its mRMR value (`Relevance / Redundancy`), also normalized.
*   A score of `1.0` marks the most important feature (or one of them, if several share the top score). Lower scores mean less importance within the selected feature set.