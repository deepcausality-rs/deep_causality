# Example: Decomposing Causal Structure with SURD-States

This example demonstrates the `surd_states` algorithm to analyze and decompose the causal relationships from raw data of
dynamic systems. It showcases how to use the algorithm and, more importantly, how to interpret its rich, detailed
output.

The core idea is to go beyond a simple "A causes B" statement. The SURD-states algorithm provides a deep, multi-faceted
view of causality by answering four critical questions:

1. **What is the interaction type?** Is the causal influence from a set of variables **Redundant** (overlapping), *
   *Unique** (direct and independent), or **Synergistic** (emerging only from the combination)?
2. **How strong is the influence?** How much information, measured in bits, does a source variable provide about the
   future of a target variable?
3. **What is the role of hidden factors?** How much of the target's behavior is unexplained by the variables we can see?
   This is the **Information Leak**.
4. **How does causality change with the system's state?** Does a variable become more or less influential depending on
   its current value or the value of other variables? This is revealed by the **State-Dependent Maps**.

## How to Run This Example

To run the analysis, run:

```bash
cargo run --example example_surd
```

The example will run four different test cases, each with a different underlying causal structure, and print a detailed
breakdown of the causal structure.

## Understanding the Output

The example analyzes a simple system with two source variables, `S1` and `S2`, and one target variable, `T`. The output
for each test case is broken into two parts: the aggregate (average) causal effects and the detailed state-dependent
maps.

---

### Test Case 1: Original Example Data

This is a baseline case with a mix of different causal influences.

```
--- SURD Decomposition Result ---
Aggregate Redundant Info: {: 0.278...,: -6.66e-17}
Aggregate Synergistic Info: {: 0.121...}
Aggregate Mutual Info: {: 0.399...,: -6.66e-17,: 0.278...}
Information Leak: 0.599...
```

**Interpretation:**

* **Information Leak (~60%):** This is the first and most important number. It tells us that **60% of the target's future
  behavior is unexplained** by our source variables `S1` and `S2`. This suggests a significant influence from unobserved
  factors (noise or hidden variables).
* **Aggregate Mutual Info:** `[2]: 0.278` shows that, on average, `S2` provides a significant amount of information
  about `T`. `[1]: ~0` shows that `S1` provides almost no information on its own.
* **Decomposition:** The total information from `S2` (`0.278` bits) is almost entirely **Redundant**. The `Synergistic`
  term (`0.121` bits) represents the *new* information that emerges only when we consider `S1` and `S2` together.
* **Conclusion:** `S2` is the main driver. `S1` is only useful when considered in combination with `S2`.

---

### Test Case 2: Low Information Leak (Strong, Synergistic System)

This models a system where the target is almost perfectly determined by the sources.

```
--- SURD Decomposition Result ---
Aggregate Redundant Info: {: 0.0,: 0.0}
Aggregate Synergistic Info: {: 0.531...}
Information Leak: 0.468...
```

**Interpretation:**

* **Information Leak (47%):** The unexplained part is now lower. The source variables explain more than half of the
  target's behavior.
* **Redundant & Unique Info are Zero:** Looking at `S1` alone or `S2` alone tells us nothing. This is the classic
  signature of an XOR-like relationship.
* **Synergistic Info is High:** All the causal influence (`0.531` bits) comes from the **synergy** between `S1` and
  `S2`. You *must* know both inputs to predict the output.
* **Conclusion:** The algorithm has correctly identified a purely synergistic causal structure.

---

### Test Case 3: Medium Information Leak (Mixed System)

This models a system where `S2` is the primary driver, but `S1` has a small, independent effect.

```
--- SURD Decomposition Result ---
Aggregate Redundant Info: {: 0.014...,: 0.122...}
Aggregate Synergistic Info: {: 0.005...}
Information Leak: 0.857...
```

**Interpretation:**

* **Information Leak (86%):** The system is now much noisier. The majority of the target's behavior is unexplained.
* **Decomposition:** The algorithm correctly identifies that `S2` has the largest influence (`0.122` bits of **Redundant
  ** info). `S1` has a smaller, but non-zero, influence (`0.014` bits). The synergy is very small.
* **State-Dependent Maps are Non-Empty:** This is a key result. The output `Causal Unique States: [[2]]` tells us that
  the unique causal influence of `S2` is strong enough to be identified in specific states of the system.
* **Conclusion:** The algorithm has correctly identified a noisy system dominated by one variable, with minor influences
  from another.

---

### Test Case 4: High Information Leak (Nearly Random System)

This models a system where the target is almost completely independent of the sources.

```
--- SURD Decomposition Result ---
Aggregate Synergistic Info: {: 0.00115...}
Information Leak: 0.998...
```

**Interpretation:**

* **Information Leak (99.9%):** This is the immediate giveaway. The algorithm is telling you that the source variables
  you provided have **almost no predictive power** over the target.
* **Aggregate Info is Near Zero:** The total mutual information is tiny (`0.00115` bits), which is effectively
  statistical noise.
* **State-Dependent Maps are Empty:** This is the most important result. Because there is no meaningful causal structure
  to be found, the algorithm correctly reports that there are **no specific states** where a significant causal
  interaction occurs.
* **Conclusion:** This is a feature, not a bug. The SURD-states algorithm has correctly analyzed a random system and
  concluded that there is no causality to be found. It is a powerful demonstration of the algorithm's ability to
  distinguish a true signal from noise.