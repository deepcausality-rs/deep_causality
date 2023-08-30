[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Heilmeier Questions

Original: [The Heilmeier Catechism](https://www.darpa.mil/work-with-us/heilmeier-catechism)

## What are you trying to do?

I am working on hypergeometric computational causality to solve deep learning problems that are subject to the
independent data distribution assumption, frequent contextual changes, and require full explainability.
Specifically, the main motivation comes from making AI models:

1) Distribution and scale invariant
2) Fully explainable & easier to verify for correctness.
3) Context aware

## How does this get done at present?

Currently, all research in computational causality is relying on libraries written in Python. All major projects are
already free of the independent data distribution assumption. However, to the best of my knowledge, there is no real
answer to the question of how to contextualize causal models.

## Who does it and how?

* [Judea Pearl](http://bayes.cs.ucla.edu/jp_home.html) at UCLA
* [Ilya Shpitser](https://www.cs.jhu.edu/~ilyas/) at Johns Hopkins University
* [Miguel Hernan](https://www.hsph.harvard.edu/miguel-hernan/), [Causal Lab](https://causalab.sph.harvard.edu/) at
  Harvard University
* [Elias Bareinboim](https://causalai.net/) at Columbia University
* [Causality and Machine Learning](https://www.microsoft.com/en-us/research/group/causal-inference/) at Microsoft
  Research

## What are the limitations of current practices?

While all the projects listed above provide important contributions and produce highly valuable knowledge, I would name
main limitations as the absence of model contextualization and the lack of production grade tooling. Specifically, all
projects mentioned above share that there are written in Python (common choice in research)
and that they are not well optimized for industry grade performance requirements.

However, to the best of my knowledge, there is no ongoing work to answer the question of how to contextualize causal
models.

## Are you aware of the state-of-the-art and have you thoroughly thought through all the options?

I read through the large corpus of foundational work of Judea Pearl, I investigated the fascinating work of Elias
Bareinboim and his PhD students, I read through the work of Miguel Hernan and I looked into the comprehensive work
contributed by Microsoft research.

While looking at the source code of all of those projects, I noticed the limitations mentioned above. I emphasize the
enormous value each of the existing projects added to my discovery process. Eventually, I concluded there is a need to
build a production grade causality inference system.

## What is new about your approach?

1) I’ve written all software in Rust with production grade safety, reliability, and performance in mind.

2) I’ve invented isomorphic, recursive causal data-structures that enable concise expression of arbitrary complex causal
   structures.

3) I’ve invented context aware causality reasoning across data-like, time-like, space-like, spacetime-like entities
   stored within (multiple) context hyper-graphs.

## Why do you think you can be successful?

1) The underlying research has already been concluded and the resulting software technology has already been written.
   While the causality learning remains an open topic warranting further investigation, the remaining technology has
   reached a point at which practical application becomes necessary to advance further.

2) Reliability and (inference) performance, two of the main design goals, have been thoroughly tested, benchmarked, and
   evaluated with an overall positive assessment.

3) With the increased adoption of artificial intelligence in all areas of life, it is time to provide solutions that are
   reliable and explainable to ensure trust in those systems.

## Who cares?

If you are looking for innovation at the forefront of contemporary technology solving hard problems,
this might be for you.

## If you succeed, what difference will it make?

1) Simplified modelling of complex tempo-spatial patterns.

2) Simplified reasoning and understanding of the task at hand due to explainability.

3) Accelerated ideation to production process since the underlying library is already written with production
   requirements in mind.

## What does first, second, and third-order analysis of your approach reveal?

### First order:

The geometric causality modelling combined with ability to add multiple contexts to a causality model leads to a
significant reduction of complexity, faster run-time performance, and faster inference.

Reasoning performance for basic causality functions is micro-second for graphs below 1k nodes and still
sub-seconds for graphs below 10k nodes. Only complex graphs with well over 10k nodes induce notable latency. However,
inference measured on simple causality collections i.e. vector or array is multiple orders of magnitudes faster than
graph
structures for large number of causes i.e. 1 million or more. When dealing with very large models, memory is more of a
concern
than inference speed.

### Second order:

Because of the reduced complexity and better performance, the primary area of application might be in embedded or
real-time control systems that require certain performance limits while also maintaining full explainability.

### Third order:

Because of the centrality of explainability at all stages, one potential higher order effect of this invention
is the advancement of understanding evolving complex systems.
Specifically, when the underlying causal model changes, so does the line of reasoning and that allows for deliberate
model enquiry under simulated (adverse) evolution to assess model robustness in a changing environment ahead of
deployment.

## What are the risks?

One central risk, as with all software technology, remains complexity.
Specifically, when software projects become too complex, chances of failure increase.

## How do you mitigate those risks?

Addressing the risk of undue complexity, the project leans heavily on
Rust's module system and separates disjoint modules into dedicated crates.
That way, developer only deal with the complexity within each crate while leaving the composition
of crates as a separate problem. 