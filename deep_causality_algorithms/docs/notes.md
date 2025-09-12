
The detailed decomposition provided by SURD-states maps directly to the core components of the EPP.

1. Mapping Causal Links to `CausaloidGraph` Structure
   The aggregate SURD results tell you which variables influence others. This directly informs how you build the CausaloidGraph.

* A strong unique influence from Q1 to Q2 suggests a simple directed edge in the graph: Causaloid(Q1) -> Causaloid(Q2).
* A strong synergistic influence from Q1 and Q3 onto Q2 suggests a many-to-one connection: Causaloid(Q1) and Causaloid(Q3) would both have
  edges pointing to Causaloid(Q2). The logic inside the Q2 Causaloid would then know to expect inputs from both.
* A high information leak for a variable suggests its Causaloid should model a high degree of internal randomness or dependency on an
  unobserved Context.

2. Mapping State-Dependency to `Causaloid` Logic
   This is the most powerful connection. The state-dependent maps produced by SURD-states provide the exact conditional logic that you would
   program into the causal_fn of a Causaloid.

For example, if the SURD analysis shows that:
* When q1 > 0, Q1 has a strong unique effect on Q2.
* When q1 <= 0, Q2 and Q1 have a synergistic effect on Q2.

The SURD-states algorithm is perfectly suited for detecting multiple causes, and the Causaloid Collection in the EPP is
the specific architectural primitive designed to model exactly that discovery.

Here is a detailed explanation of how they work together:

1. Detecting Multiple Causes with SURD-States

The core purpose of the SURD-states algorithm is to take a target variable (X) and a set of potential source variables (e.g., Q1, Q2, Q3)
and decompose the total causal influence. The output explicitly quantifies the influence of individual variables and combinations of
variables.

Therefore, SURD-states detects multiple causes for variable X if it finds that:
* More than one variable has a non-zero Unique contribution.
* A Redundant or Synergistic contribution exists for any group of two or more variables.

If the analysis shows that Q1, Q2, and Q3 all have some form of causal influence (Unique, Redundant, or Synergistic) on X, you have
empirically detected a multi-causal relationship.

2. Modeling Multiple Causes with a Causaloid Collection

As described in the EPP monograph, a Causaloid Collection is a container for a set of individual Causaloids that are evaluated together
using a specific Aggregate Logic. This is the EPP's formal mechanism for modeling the "interplay of multiple factors."

The output of the SURD-states analysis provides the critical evidence needed to choose the correct Aggregate Logic for the collection,
leading to a more accurate and faithful causal model.

Here’s how the SURD results would guide your modeling decision:

* If SURD detects strong SYNERGY:
    * Example: A chemical reaction requires both reagent A and reagent B to occur. The SURD analysis would show a very large synergistic
      value for the pair (A, B) -> Reaction.
    * EPP Model: You would model this with a Causaloid Collection containing Causaloid(A) and Causaloid(B) and set the Aggregate Logic to
      `All` (Conjunction). The collection only becomes active if both causaloids are active.

* If SURD detects strong UNIQUE or REDUNDANT influences:
    * Example: A patient has a fever that could be caused by a virus OR a bacterial infection. The SURD analysis would show strong unique
      contributions from both Virus -> Fever and Bacteria -> Fever.
    * EPP Model: You would model this with a Causaloid Collection containing Causaloid(Virus) and Causaloid(Bacteria) and set the
      Aggregate Logic to `Any` (Disjunction). The collection becomes active if at least one of the causaloids is active.

* If SURD detects complex, mixed influences:
    * Example: A server fails if any two of its three power supplies fail. The SURD analysis would likely show strong synergistic effects
      for all pairs (PS1, PS2), (PS1, PS3), and (PS2, PS3).
    * EPP Model: This maps perfectly to a Causaloid Collection with an Aggregate Logic of `Some(k)` (Threshold). In this case, Some(2),
      meaning the collection is active if at least two of the three power supply causaloids are active.

In summary, the SURD-states algorithm provides the data-driven evidence to identify multi-causal structures, and the Causaloid Collection
provides the formal mechanism within DeepCausality to build an executable model of that precise structure.