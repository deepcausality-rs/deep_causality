---
title: Views on machine learning in Rust
description: This post briefly summarizes my current thoughts on machine learning in Rust. 
date: 2023-08-31
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

{{< toc >}}

This post briefly summarizes my current thoughts on machine learning in Rust. I wonder how these may change five years
from now.

## Why Rust for machine learning

Those companies that moved their ML production to Rust see more benefits
than [just a 25X speedup](https://www.lpalmieri.com/posts/2019-12-01-taking-ml-to-production-with-rust-a-25x-speedup/).
Others do the
obvious: [interfacing Rust with C++ code ](https://www.reddit.com/r/rust/comments/fvehyq/d_who_is_using_rust_for_machine_learning_in/?rdt=37685&onetap_auto=true)
since most ML libraries are still solid C++ implementations.

## The Good

Rust as a language provides all the critical bits required to write performant and reliable machine-learning libraries.
When computation is known to be heavy, performance is as important as reliability. Explicit mutability and optional
interior mutability give enough flexibility to design public APIs well. I might cling to a minority position, but I like
lifetime annotation for enabling static memory management verified by the compiler. Code performance boils down to two
challenges: One, programming all the different memory tiers efficiently, and two, avoiding unnecessary computation
altogether. Rust is making the first part a lot less error-prone via verified lifelines and the second part a lot more
convenient via its zero-cost abstraction of high-level syntactic sugar paired with various tricks, i.e., memoization.
We already see new numeric Rust libraries, such as [pola-rs](https://www.pola.rs/), with good performance and, for the
most part, ergonomic API
design.

## The Not-So-Great

As for mature machine libraries, Rust still wraps around large C++ code bases, as it does for TensorFlow and PyTorch. In
all fairness, rewriting these massive libraries from scratch isn't economically feasible and probably not even necessary
after years of battle-tested production deployments. While the number of ML crates
has [increased significantly](https://lib.rs/science/ml), the
large number of Rust wrappers around existing libraries confirms that there isn't a high demand for reinventing the
wheel in a different language.

## Python

Historically, companies build all their R&D in Python, and when data scientists came up with something of commercial
value, the engineers ported it to C++ and plugged it into the production system. With the advent of ML servers, the ML
pipeline became much more straightforward by just training and testing models and loading them into a model server for
gradual roll-out into production. Also, with the advent
of [Mojo, a language that is more of a compiler for Python](https://www.modular.com/mojo),
the slow performance of Python might become a lot less of an issue than it used to be.
If you can write your data science work in Python with all the existing great ML libraries while still getting good
performance by compiling Python with Mojo, why would you rewrite anything in Rust?

## The Ugly

If there is just one ugly thing about machine learning in Rust, it’s the complete lack of academic research in Rust
machine learning. In all fairness, the same argument applies to any other non-Python programming language since all
machine learning research in Industry and academia uses Python by default. There are many good things to be said about
having agreed on one common language. Sharing research is easy for once, and hiring talent at least leans on fairly
standardized skills regardless of talent shortage. On the flip side, a lot is missing that would otherwise be possible
when relying more on recent advancements in programming languages. For example, Swift protocols have led to the advent
of protocol-based differentiable
programming [pioneered by Google Research](https://www.tensorflow.org/swift/guide/overview), which has led to an ongoing
effort to [bring
differentiable programming for gradient-based machine learning into the Swift compiler](https://forums.swift.org/t/differentiable-programming-for-gradient-based-machine-learning/42147).

When deciding on a language for differentiable programming, Google argued against Rust:

> "We believe that Rust supports all the ingredients necessary to implement the techniques in this paper:
> it has a strong static side, and its traits system supports zero-cost abstractions which can be provably eliminated
> by the compiler. […] A concern with using Rust is that a strong goal of this project is to appeal to the entire
> TensorFlow community, which is currently pervasively Python-based. We love Rust, but it has a steep learning curve
> that may exclude data scientists and other non-expert programmers who frequently use TensorFlow.
> The ownership model is really great but mostly irrelevant to the problems faced by today's machine learning code
> implemented in Python."
>
> -- [Why Swift for TensorFlow? ](https://github.com/tensorflow/swift/blob/main/docs/WhySwiftForTensorFlow.md#why-swift-for-tensorflow)

A Rust pre-RFC proposing the addition of differential programming
support[ went boldly nowhere](https://internals.rust-lang.org/t/pre-rfc-differential-programming-support/11429/20) but
at least concluded
that [it's all feasible with Rust](https://internals.rust-lang.org/t/pre-rfc-differential-programming-support/11429/20).
Unfortunately, without the backing of a company like Google, there is little
chance to implement these kind of advanced features into any programming language. However, Generic Automatic
Differentiation (GAD) became later available as a [separate
crate](https://docs.rs/gad/latest/gad/).

## Conclusion

About a year ago, when I had to decide which language to use for my project DeepCausality, I chose Rust for almost the
opposite reasons Google chose. For once, traits in Rust are as powerful as protocols in Swift. I know that because I’ve
implemented the entire core of DeepCausality with just a handful of carefully crafted protocols with default
implementation. Then, the steep learning curve, while true, is worth taking as every meaningful challenge makes an
engineer better. Indeed, working my way through Rust certainly made me a better engineer on many levels, but I still
have much to learn. While irrelevant for data scientists, the ownership system is highly relevant for the SRE team
running a library cost-efficiently in production.

However, I fully agree that everyone in the machine learning community works with Python and possibly Mojo if things go
well. However, there is still the possibility to wrap a Rust machine learning crate in Python to get all the Rust
goodness in production while still having Python in R&D. After all, companies have done the same with complex C++
codebases for decades, so this migh be the least controversial path forward for machine learning in Rust.

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware
causal reasoning in Rust. Please give us a [star on GitHub.](https://github.com/deepcausality-rs/deep_causality)