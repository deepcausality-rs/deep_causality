# Deep Causality

## About 

Deep Causality is a hyper-geometric computational causality library that enables fast context
aware causal reasoning across arbitrary complex causality models. Deep Causality induces only
minimal overhead and thus is suitable for deployment on low power (IoT) devices 
or for real-time applications. 

**Why?**

> "ANSR hypothesizes that several of the limitations in ML today are a consequence of the inability to incorporate contextual
> and background knowledge, and treating each data set as an independent, uncorrelated input.
> In the real world, observations are often correlated and a product of an underlying causal mechanism,
> which can be modeled and understood" - [ANSR](https://www.darpa.mil/program/assured-neuro-symbolic-learning-and-reasoning)

For more details, see the [motivation document](/doc/motivation.md). 

**How is deep causality different from deep learning?**

* Free of the [independent and identically distributed data (IID) assumption.](https://towardsdatascience.com/independent-and-identically-distributed-ce250ad1bfa8)
* Deterministic and explainable causal reasoning. 
* Reasoning over causal collection, graph, or hyper-graph structure supported.
* Recursive causal data-structures enable concise expression of arbitrary complex causal structures.
* Context aware causality reasoning across data-like, time-like, space-like, spacetime-like entities stored within (multiple) context-hyper-graphs

**When to use deep causality and when to use  deep learning?**

**Use deep learning (PyTorch, Tensorflow, Keras, MXNEet etc):**
* Explanations are not needed.
* Language models (ChatGPT etc)
* Generative models (Midjourney etc.)
* Object / Image classification ([ImageNet](https://paperswithcode.com/sota/image-classification-on-imagenet) etc.)

Whenever it is expected that the data the model sees in production have the 
same or similar distribution and characteristics as the data used during the 
model training, deep learning will be very successful. Also, when it is acceptable 
that decisions made by the model remain an unexplainable blackbox or an explanation
is fundamentally not needed (image classification), then deep learning is perhaps the best solution
available for the time being. Last but not least, use deep learning whenever the data distribution 
used in a model remains largely unaffected whenever context may change. 
Otherwise, you would have to retrain a model quite frequently. 

**Use deep causality:**

* Explanations are required
* Determinism is required
* Multi-causal, contextual multi-stage reasoning is required
* Complex event processing in real-time is required

Whenever an inferred decision must be deterministic and explainable regardless 
of data distribution or characteristics, use deep causality. Furthermore,
when data feed into the model are affected by contextual changes, then deep causality
allows you to model the context and account for those changes before reasoning occurs.
Deep causality is inherently deterministic and therefore suited for applications with
high reliability and robustness standards.

**When to use both, deep causality and deep learning:**

In practice, there is a broad and wide space for coexistence because of the 
largely complementary properties. For example, data from a sensor array may
use deep learning for object classification and use then a deep causality model
to derive contextualized decisions in real-time. Or in another setting, a deep causality
model monitors multiple IoT sensor arrays in real-time for anomalies and only when an anomaly
has been detected triggers the engagement of a drone with a more sophisticated Ai model 
to discern what kind of anomaly may occurs. On a more fundamental level, when multiple causal models 
haven been developed, [hyper geometric deep learning ](https://deephypergraph.com/) may help to find 
more or novel causality graph representations. 
These are just a few examples, but in reality endless possibilities emerge from combining deep learning with deep causality. 

## Install

```Bash
cargo add deep_causality
```

## Usage:

See:

* [Benchmark](benches/benchmarks)
* [Example](examples/smoking/run.rs)
* [Test](tests)

## Cargo & Make

Cargo works as expected, but in addition to cargo, a makefile exists
that abstracts over a number of additional tools you may have to install
before all make scripts work:

* [nextest](https://nexte.st/)
* [outdated](https://github.com/kbknapp/cargo-outdated)
* [udeps](https://crates.io/crates/cargo-udeps)
* [audit](https://crates.io/crates/cargo-audit)

```bash 
    make build          Builds the code base incrementally (fast).
    make bench          Runs all benchmarks across all crates.
    make check          Checks the code base for security vulnerabilities.
    make example        Runs the default example: Smoking.
    make fix            Fixes linting issues as reported by cargo
    make test           Runs all tests across all crates.
```

## Author

* Marvin Hansen
* Github key ID: 4AEE18F83AFDEB23
* GPG key ID: 210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC

## Licence

* [MIT Licence](LICENSE)
* Software is "as is" without any warranty. 