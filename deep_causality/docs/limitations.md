[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Limitations

DeepCausality still has some more groundwork to do because of its early stage. In its current state (0.2.X),
a handful of limitations exist:

* Counterfactual reasoning is missing. DeepCausality cannot reason counter to the fact; for example, if the drone had
  not accelerated more than 50mp/h, it would not have crashed into the tree.

* Causal structural learning is missing. Right now, causal models have to be designed and built by hand. Possible areas
  of exploration for causal structural learning are:
    - Causal Reinforcement Learning ([Elias Bareinboim](https://crl.causalai.net/))
    - Deep Neuro Evolution ([Kenneth O. Stanley](https://www.uber.com/en-ID/blog/deep-neuroevolution/))

* Tooling is absent. Specifically, tools such as model server,
  dashboard, and model visualization are missing.

* Meaningful code examples and more comprehensive documentation would be welcomed.

None of these limitations is definitive, meaning with some creative work, counterfactual reasoning might become
solvable, and, likewise, causal structural learning might become feasible when adapting, for example, deep
neuroevolution to the hyper-graph structure central to DeepCausality. Tooling may require a deeper work commitment since
good tooling usually requires solid design and implementation.

