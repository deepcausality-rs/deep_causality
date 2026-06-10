# Research Roadmap: Unifying Causal Discovery, Model Generation, and Inference

## Vision

The Center for Dynamic Causality is building one typed algebra in which causal
discovery, model generation, and inference compose as a single operation. A user
supplies raw observational data and receives a running, auditable causal model, with
no manual step between finding the structure and reasoning over it. We call this
algebra the **Causal Arrow**.

DeepCausality rests on one axiom, effect propagation as a monadic dependency
(`m₂ = m₁ >>= f`), and both halves of the system, discovery and inference, already
run on the category theoretical mechanism that axiom provides. They do not connect to each
other yet. The open question is how to algorithmically convert discovered causal structures 
into a causal model for inference?

## Where the work stands today

**A categorical foundation.** The DeepCausality project already generalizes causality as a monadic
 dependency. Pearl's structural causal models, dynamic Bayesian networks,
Granger causality, the Rubin causal model, and conditional average treatment effects
each fall out as a specialization of the one axiom, and the
[`classical_causality_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples)
crate implements all five. The category theory under that axiom is in the code, not
on a slide. The [`deep_causality_haft`](https://docs.deepcausality.com/concepts/hkt/)
crate supplies higher-kinded types through zero-sized witness structs, so one generic
`bind` specializes per type at compile time. 

**Inference runs on the monad.** The carrier effect implements a `CausalMonad` trait
with `pure`, `bind`, and `intervene`. Counterfactual reasoning needs no abduction
step, because the world state is an explicit value wtih an explicit context.
Intervention, context substitution, and state reset are each one method call on
the same carrier. The [*Counterfactuals via the Causal
Monad*](https://docs.deepcausality.com/concepts/counterfactuals/)  document covers more details.

**Discovery runs, and it is already a monad too.** The Causal Discovery Language *CDL* walks
raw data through configure, load, clean, select, discover, analyze, and finalize. Its
stages form a typestate builder, so the compiler enforces the logical order of a data processing pipeline.
Underneath, the builder is a monadic sequence whose witness implements the same
`Functor`, `Monad`, and `Applicative` traits from `deep_causality_haft` that the
inference carrier uses. SURD and MRMR ship as discovery primitives. The same `bind`,
the same witnesses, and the same uniform surface drive both ends of the system.

## The open problem

The two ends speak the same categorical language, yet a gap sits between them. The
Causal Discovery Language emits a text report with recommendations such as "Strong
unique influence: Recommended Direct edge in `CausaloidGraph`." A person reads that
report and wires the model by hand. The
[concept documentation](https://docs.deepcausality.com/concepts/cdl/) states the
boundary directly: "The pipeline ends where the model-construction workflow begins."

This handoff is the one manual break in an otherwise continuous path from data to
action. Discovery produces a report; a human translates the report into Causaloids;
inference runs on those Causaloids. The first step and the last step already compose by
machine. The translation in the middle waits on a person. Removing that person is the
problem this roadmap sets out to solve.

## Research directions

### 1. Model generation as a typed operator

The path from data to a runnable causal model has three stages:

| stage | type | status                        |
|---|---|-------------------------------|
| discover | `Data → Decomposition` | runs today (SURD, MRMR)       |
| generate-model | `Decomposition → CausaloidModel` | the missing link              |
| infer | `CausaloidModel ⊗ Evidence → PropagatingEffect` | runs today (the causal monad) |

The middle stage is a pure function from a discovery result to an executable model.
SURD already labels each variable's relation to the target as unique, redundant, or
synergistic, and those labels carry directly into model structure: a unique cause
becomes a singleton Causaloid, a redundant set becomes a collection that fires when any
member fires, a synergistic set becomes a collection that fires only when the whole set
is active. The decomposition is a wiring diagram.

The open research question: how a discovered causal rule becomes a causal function that infers on new evidence?

### 2. The unifying interface: the Causal Arrow

The Causal Discovery Language is a monad. The Causal Monad is the heart of the inference engine. 
Both stand on the same same algebraic foundation. What they do not yet share is one
explicit causal operator, together with the generate-model
function that lets discovery's output flow into inference's input without a human in the
loop. Supplying the missing causal operator connects causal discovery and causal inference
that both were built upon the same foundational axiom.

The causal monad already subsumes five classical methods. Discovery methods reach past
what a monad alone expresses: SURD carries a lattice over variable subsets, and
a Bayesian root-cause method carries a graph and two aligned data regimes. Each fixes a
structural input before any data flows, which a monad's dynamic `bind` cannot absorb. These are not monadic operators. 
The Arrow is the interface that holds both kinds.

### 3. Generalization across carrier dimension

Because an operator in this algebra is polymorphic over its carrier objects, the same
composition runs whether a variable is a scalar series, a multivector field, or a complex geometry. 
In the DeepCausality poject, geometric-algebra and manifold code already implements the uniform monad surface
as explained in the [unified math concept guide](https://docs.deepcausality.com/concepts/uniform-math/). This, in principle, allows the exploration of causal discovey and causal inference in higher dimensional structures.

## The path forward

The program evolves around a numbe of dedicated research projects build upon the exiting infrastructure:

1. **`Endomorphism`**: a typed self-operator, proven first on the fixpoint loops in the existing 
   codebase.
2. **`Morphism`**: the typed-operator base to re-express the discovery pipeline's stages.
3. **The monoidal product on operators**: the piece that lets one operator take more than a
   single structured input.
4. **`generate-model`**: the function that compiles a discovery decomposition into an executable
   Causaloid model and closes the seam.
5. **Unified discovery, model generation, and inference.**: This last step takes all of the views together.

## Analytic operators: type-based differentiation and integration

The Causal Arrow generalizes over carrier dimension (§3): one operator runs whether a
variable is a scalar series, a multivector field, or a manifold. Two analytic operators on
those carriers, the derivative and the integral, are not bolted on as free functions — they
**are** the categorical machinery already in place: they `impl` the `Arrow` and reuse the
`Endomorphism` monoid. They live in a dedicated crate `deep_causality_calculus` that depends on
both `deep_causality_haft` (the Arrow/Endomorphism machinery) and `deep_causality_num` (the
`Dual` number), so both foundations stay self-contained — `haft` keeps its zero-dependency
status and `num` keeps only the *number* that makes differentiation possible. Two stages:

1. **`causal-arrow-calculus`** (`deep_causality_calculus`) — the analytic operators as one
   Arrow-native surface:
   - **Differentiation is the tangent functor `T`.** Its object map is exactly `Dual<A>`
     (kept in `num`); its morphism map is "run the arrow over duals". Because a concrete
     value-level `Arrow<In = f64>` cannot be lifted over `Dual` (verified: `E0308`), the
     scalar-polymorphism lives in a `DifferentiableArrow` whose `run` is generic over the
     scalar; `Diff<A>` is the derivative-arrow view, and `derivative`/`gradient` are its
     desugarings. A differentiable model is simultaneously a plain `Arrow`, so the functor
     extends the strength algebra rather than replacing it.
   - **Integration is endomorphism iteration.** `Euler`/`Rk4` build a value-level endo-arrow
     `S → S` from a rate field; evolution is the `Endomorphism` monoid's
     `iterate_n` / `iterate_to_fixpoint` / `iterate_until` — the three modes being fixed
     horizon, steady state, and integrate-until-event. (Value-level, because a capturing
     stepper is not a `fn`-pointer witness — the same reason strength realized composition at
     the value level.)
   - **Quadrature is a fold-arrow**, and the Leibniz rule (differentiate under the integral)
     is the *naturality* of `T` through that fold — `T(∫f) = ∫(Tf)` — a verified law.
   - Precision is a parameter (`Scalar = Real + Div + FromPrimitive`, with a nesting-safe
     `FromPrimitive` blanket for `Dual`): f32 / f64 / Float106, duals nesting for higher
     derivatives. `Dual`, `ε`, seeding, stepper coefficients and loops are never visible —
     a user writes a model once over `Scalar` and applies operators.

2. **`causal-arrow-application`** — applies the operators across the example suite
   (replacing hand-coded derivatives and hand-rolled Euler loops with one applied operator
   each) and adds the fluid-dynamics examples the recent CFD kernels were waiting for,
   including an avionics example verified by the Method of Manufactured Solutions. The fluid
   RHS kernels return `∂u/∂t` and demand `∇u` / `∇²u` / `∇p`: the tangent functor fills the
   inputs, the endo-arrow iteration consumes the output, and the kernel set becomes a
   runnable, verifiable solver — every piece an `Arrow` that drops into a `PropagatingProcess`
   stage.

The non-continuous carriers (discrete fields on a mesh, with no closed form) stay served by
the topology exterior-calculus surface; the tangent functor targets closed-form fields,
manufactured solutions, and parameter sensitivities. Both stages build on
`causal-arrow-foundations` (the `Dual` number, `Morphism`, `Endomorphism`) and
`causal-arrow-strength` (the value-level `Arrow`), and run on the dedicated `causal-arrow`
branch. (The earlier `causal-arrow-autodiff` / `causal-arrow-autointegration` framing, which
placed the operators as free functions in `num`, is superseded by this Arrow-native design;
`causal-arrow-autodiff`'s retained contribution is the `Dual` number reaching the chronometric
kernels via the `RealField → Real + Div` widening.)

## Scope and boundaries

Two further components live in the codebase and stay outside this phase. The [Causal State Machine](https://docs.deepcausality.com/concepts/csm/) turns an inferred effect into a proposed action. The [Effect Ethos](https://docs.deepcausality.com/concepts/effect-ethos/) evaluates every proposed action against a graph of deontic rules before it runs. Both are impplemented today, neither is yet expressed as a Monad and therefore do not compose under the propsed  **Causal Arrow**.  
Therefore, expanding the causal arrow into a causal state machine and the effect ethos is designated future work under the condition that the post causal arrow generalizes over discovery and inference.