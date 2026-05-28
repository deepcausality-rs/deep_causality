---
title: Literature
description: The main scholarly works DeepCausality and the Effect Propagation Process build upon, from process philosophy and monadic composition to Pearl's SCM and Hardy's causaloid.
sidebar:
  order: 5
---

This page collects the main publications that shaped DeepCausality, grouped by the contribution each one made to the framework. It is adapted from the Precedent section of the [EPP monograph](https://github.com/deepcausality-rs/deep_causality/tree/main/papers).

## Whitehead and Bergson: process philosophy

The EPP's primary departure point is a rejection of the classical Newtonian conception of a static, absolute background spacetime. This move is rooted in the tradition of process philosophy, which argues that reality is not composed of enduring, static substances but is a dynamic flow of interconnected events. The idea finds its clearest expression in the work of Alfred North Whitehead, who posited a universe of "actual occasions" ([Whitehead, *Process and Reality*](#ref-whitehead)), and Henri Bergson, who described reality as a continuous "creative evolution" ([Bergson, *Creative Evolution*](#ref-bergson)). Their shared insight of reality as a process inspires the EPP's redefinition of causality itself, shifting from a static, happens-before relation to a dynamic process of effect propagation.

## Moggi and Wadler: monadic composition

The carrier's `CausalMonad` trait and its `bind` operation inherit a programming discipline whose theoretical move belongs to Eugenio Moggi and whose practical articulation belongs to Philip Wadler. Moggi proposed in 1989 that monads from category theory provide a useful structuring tool for the denotational semantics of programming languages ([Moggi, *Computational lambda-calculus and monads*](#ref-moggi)). His key move was distinguishing a value type `a` from a computation type `M a`, and showing that monads can encapsulate features such as state, exception handling, and continuations within a single uniform interface. Wadler then demonstrated in 1992 that this insight transfers directly into a programming discipline ([Wadler, *The essence of functional programming*](#ref-wadler)). A function of type `a -> b` can be lifted into monadic form `a -> M b`, and the resulting programs can gain error handling, state, output, or non-deterministic choice by changing the monad while leaving the program structure essentially intact.

The EPP adopts this discipline for causal composition. The monadic axiom `m₂ = m₁ >>= f` is the Kleisli composition of a monad whose context parameters carry the state, configuration, error condition, and audit log of the causal process. The flexibility Wadler showed for lifting plain functions into a context-aware monadic form is the same flexibility with which the carrier effect, which implements the `CausalMonad` trait, interoperates with the Causaloid through the shared `PropagatingEffect` type.

## Russell: a critique of causality

Bertrand Russell's critique of causality, formulated in his 1912 essay *On the Notion of Cause* ([Russell, 1912](#ref-russell)), led to the realization that the central issue is not necessarily causality itself, but the underlying assumption of time asymmetry. That assumption sits at odds with Russell's observation that most successful theories in physics are based on time symmetry. While physics routinely models dynamic change in complex systems, computational causality consistently struggles to capture dynamic causality. There is truth to causal invariance, yet dynamic systems also emit different causal structures depending on dynamic change, and that is where computational causality is at odds with physics and, to an extent, with reality. From there, it became clear that for causality to handle dynamics, it requires a new foundation of causality itself.

## Hardy: the causaloid

Lucien Hardy introduced the "causaloid" ([Hardy, 2005](#ref-hardy)), a concept that encapsulates a spatial region and the causal connections within it, as a foundation for his work on a theory of quantum gravity. Unlike all prior forms of causality, Hardy's causaloid is spacetime-agnostic because it folds cause and effect into one entity and removes the need for temporal order. His seminal work *Probability Theories with Dynamic Causal Structure* had a three-fold impact on the EPP. First, his causaloid formalism proved instrumental in the formation of isomorphic-recursive causal data structures. Second, his insight that the formalism puts deterministic and probabilistic structures on equal footing led directly to the multi-modal reasoning of the EPP. Third, his demonstration that fundamental differences of theoretical foundations are contained in a causaloid informed the representation of causal relations as a causal function, which resulted in the single axiomatic formulation of the EPP.

## Pearl: the structural causal model

Judea Pearl, with his Structural Causal Model, established the foundation upon which the entire field of computational causality was subsequently built. His work in *Causality: Models, Reasoning, and Inference* ([Pearl, 2000](#ref-pearl-causality)) was as influential as his later critique in *Theoretical Impediments to Machine Learning with Seven Sparks from the Causal Revolution* ([Pearl, 2018](#ref-pearl-theoretical)). His contribution to the algorithmization of counterfactuals proved instrumental for the development of contextual counterfactuals in the EPP.

## Bareinboim: transportability of causal effects

Bareinboim's calculus of transportability ([Bareinboim and Pearl, 2012](#ref-bareinboim)) and his subsequent work on data fusion formalize the very problem of contextual variance that the EPP's explicit Context is designed to manage at a computational level. Where Bareinboim provides the logical framework for reasoning about moving causality between discrete contexts, the EPP provides the computational primitive, a dynamic, queryable, multi-modal Context, to operationalize that reasoning.

## Forbus: a defeasible deontic calculus

Kenneth Forbus's work on formalizing deontic calculus ([Olson, Salas-Damian, and Forbus, 2024](#ref-forbus)) proved invaluable for the problem of conflicting norms in the Effect Ethos. In practice, it is rarely possible to write conflict-free norms, and a recurring theme during the development of the Effect Ethos was the acceptance of that reality. The search for a solution led to the adoption of the Defeasible Deontic Calculus as the primary means to resolve normative conflicts.

## Bornholt: an uncertain type

The contribution of Bornholt and colleagues in *Uncertain&lt;T&gt;: A First-Order Type for Uncertain Data* ([Bornholt, Mytkowicz, and McKinley, 2014](#ref-bornholt)) informed the unification of deterministic and probabilistic reasoning in the EPP. Instead of representing a value with a single number, the Uncertain type represents a value with a full probability distribution, or even a computation graph that produces one. The EPP reasoning logic can lift simpler deterministic and probabilistic effects into the Uncertain distribution, aggregate the distributions, and infer a logical combination of all inputs without loss of information. The final output collapses rich uncertainty into a single value at the last moment while preserving second-order properties such as the standard deviation or confidence level. Decisions, and crucially deontic reasoning, become more robust under uncertainty.

## Zhao et al.: feature selection for discovery

The Causal Discovery Language draws its feature-selection stage from Maximum Relevance and Minimum Redundancy (mRMR), the filter method Zhao, Anand, and Wang developed for Uber's marketing machine-learning platform ([Zhao, Anand, and Wang, 2019](#ref-mrmr)). mRMR selects the features most relevant to a target while controlling redundancy among the selected set. That balance is what lets the CDL reduce a large observational feature space to a compact, informative subset before any causal structure is inferred, which keeps the downstream discovery step both tractable and interpretable.

## Martínez-Sánchez et al.: causality by states

The causal-discovery algorithms build on SURD, the decomposition of causality into synergistic, unique, and redundant components by Martínez-Sánchez and Lozano-Durán ([Martínez-Sánchez and Lozano-Durán, 2025](#ref-surd)). Their state-and-interaction-type formulation quantifies causal influence as a function of system state and separates redundant from synergistic interactions, rather than reporting a single average causal strength. The `deep_causality_algorithms` crate implements this as a discovery method, which suits dynamic causality directly: causal structure that varies with the state of the system is exactly what the EPP is built to represent.

## Liu et al.: hypergraph analytics

UltraGraph, the two-phase hypergraph data structure that backs the Causaloid Graph and the Context Hypergraph, is built on the hypergraph-analytics work of Liu, Firoz, Gebremedhin, and Lumsdaine ([Liu et al., 2022](#ref-nwhy)). Their NWHy framework for hypergraph representations, data structures, and algorithms informed UltraGraph's separation of a mutable construction phase from an immutable, cache-friendly analysis phase. That split is what makes performance-constrained hypergraph composition practical inside the EPP, wherever the Causaloid Graph or the Context grows large.

## References

- <span id="ref-whitehead"></span>Whitehead, A. N. *Process and Reality*. Simon and Schuster, 2010 (originally 1929). [PDF](https://www.palmyreoomen.nl/uploads/pdf%27s/A.N.Whitehead_Process-and-Reality.pdf)
- <span id="ref-bergson"></span>Bergson, H. *Creative Evolution*. Routledge, 2022.
- <span id="ref-moggi"></span>Moggi, E. "Computational lambda-calculus and monads." *Proceedings of the Fourth Annual Symposium on Logic in Computer Science*, IEEE Computer Society, 1989, pp. 14–23. [ACM](https://dl.acm.org/doi/10.5555/77350.77353) · [PDF](https://www.cs.cmu.edu/~crary/819-f09/Moggi89.pdf)
- <span id="ref-wadler"></span>Wadler, P. "The essence of functional programming." *Proceedings of the 19th ACM SIGPLAN-SIGACT Symposium on Principles of Programming Languages*, ACM, 1992, pp. 1–14. [doi.org/10.1145/143165.143169](https://doi.org/10.1145/143165.143169)
- <span id="ref-pearl-causality"></span>Pearl, J. *Causality: Models, Reasoning, and Inference*. Cambridge University Press, 2000. [PDF](https://archive.illc.uva.nl/cil/uploaded_files/inlineitem/Pearl_2009_Causality.pdf)
- <span id="ref-pearl-theoretical"></span>Pearl, J. "Theoretical Impediments to Machine Learning with Seven Sparks from the Causal Revolution." 2018. [arxiv.org/abs/1801.04016](https://arxiv.org/abs/1801.04016)
- <span id="ref-bareinboim"></span>Bareinboim, E., and Pearl, J. "Transportability of Causal Effects: Completeness Results." *Proceedings of the AAAI Conference on Artificial Intelligence*, vol. 26, no. 1, 2012, pp. 698–704. [ACM](https://dl.acm.org/doi/10.5555/2900728.2900828) · [PDF](https://apps.dtic.mil/sti/tr/pdf/ADA557446.pdf)
- <span id="ref-forbus"></span>Olson, T., Salas-Damian, R., and Forbus, K. D. "A Defeasible Deontic Calculus for Resolving Norm Conflicts." 2024. [arxiv.org/abs/2407.04869](https://arxiv.org/abs/2407.04869)
- <span id="ref-russell"></span>Russell, B. "On the Notion of Cause." *Proceedings of the Aristotelian Society*, vol. 13, 1912, pp. 1–26. [doi.org/10.1093/aristotelian/13.1.1](https://doi.org/10.1093/aristotelian/13.1.1)
- <span id="ref-hardy"></span>Hardy, L. "Probability Theories with Dynamic Causal Structure: A New Framework for Quantum Gravity." 2005. [arxiv.org/abs/gr-qc/0509120](https://arxiv.org/abs/gr-qc/0509120)
- <span id="ref-bornholt"></span>Bornholt, J., Mytkowicz, T., and McKinley, K. S. "Uncertain&lt;T&gt;: A First-Order Type for Uncertain Data." 2014. [microsoft.com/en-us/research](https://www.microsoft.com/en-us/research/publication/uncertaint-a-first-order-type-for-uncertain-data-2/)
- <span id="ref-mrmr"></span>Zhao, Z., Anand, R., and Wang, M. "Maximum Relevance and Minimum Redundancy Feature Selection Methods for a Marketing Machine Learning Platform." *2019 IEEE International Conference on Data Science and Advanced Analytics (DSAA)*, 2019. [IEEE](https://ieeexplore.ieee.org/document/8964172) · [arXiv](https://arxiv.org/abs/1908.05376)
- <span id="ref-surd"></span>Martínez-Sánchez, Á., and Lozano-Durán, A. "Observational causality by states and interaction type for scientific discovery." 2025. [arxiv.org/abs/2505.10878](https://arxiv.org/abs/2505.10878)
- <span id="ref-nwhy"></span>Liu, X. T., Firoz, J., Gebremedhin, A. H., and Lumsdaine, A. "NWHy: A Framework for Hypergraph Analytics: Representations, Data Structures, and Algorithms." *2022 IEEE International Parallel and Distributed Processing Symposium Workshops (IPDPSW)*, 2022, pp. 275–284. [IEEE](https://ieeexplore.ieee.org/document/9835472)
