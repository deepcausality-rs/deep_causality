---
title: "Unified Math in Science and Engineering"
description: "How DeepCausality's unified math stack replaces glue code with three inversions: composition through higher-kinded types, conventions as parameters, and precision as a parameter. Three worked examples follow: differentiation under the integral, an electromagnetic field in geometric algebra, and Earth's mass weighed by satellite clocks."
date: 2026-10-21
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

Science and engineering rely on ever more sophisticated computer simulations to understand complex systems. Those simulations increasingly cross domains, and each domain usually rests on a different branch of mathematics. Mechanical engineering relies on calculus, differential equations, and linear algebra. Fluid dynamics leans on vector analysis and on partial differential equations derived from conservation laws. Ideally, we would treat each domain in isolation.

The automotive and avionics industries cannot. The mechanics of a car interact with its aerodynamics once the car enters the wind tunnel. The strength of an airframe cannot be separated from the drag and pressure it meets in flight, so a simulation must combine the mathematics of both disciplines. A commercial drone obeys the same aerodynamics as every other flying vehicle, and the same mathematics applies.

## Multi-Glue Math

What does it take to glue several branches of mathematics together in a program? In practice, you glue together several physics and mathematics libraries: a mesh walk in one library, a contraction in a second, a rotor in a third. Then you discover that each library encodes its own conventions: row-major or column-major, zero-based or one-based, dense or sparse, East Coast or West Coast metric. The obvious response is to convert. A lot of conversion code later, you realize that a third of the code base is glue that translates between representations and conventions.

The consortium behind the Python array API standard states it plainly: the APIs of NumPy, TensorFlow, PyTorch, JAX, Dask, CuPy, and MXNet "are largely similar, but with enough differences that it's quite difficult to write code that works with multiple (or all) of these libraries" ([Python array API standard](https://data-apis.org/array-api/2024.12/purpose_and_scope.html)).

Even a team that follows every engineering best practice checks the mathematics and the derivation, but not the seam between two libraries, until someone debugs it and finds yet another diverging convention. The compiler cannot check a convention such as row-major order, and without a regression test, nobody can tell which convention applies where.

In Hatton's experiment T2, independent implementations of the same seismic algorithms, run on the same input, "gradually degenerated from 6 significant figures to 1 significant figure during the computation." Hatton lays the disagreement "squarely at the door of software failure" ([Hatton, 1997](https://www.leshatton.org/Documents/Texp_ICSE297.pdf)). McCullough's benchmark method found statistical packages disagreeing on the same problems ([McCullough, 1998](https://www.tandfonline.com/doi/abs/10.1080/00031305.1998.10480597)), and Keeling and Pavur found across nine packages that the choice of package and of default settings changes the answer ([Keeling & Pavur, 2007](https://www.sciencedirect.com/science/article/abs/pii/S0167947306000491)). When replicating a statistical result requires the exact same software, because no two packages implement the same formula alike, something is wrong.

## It Can't Be That Bad, Right?

How serious can a convention mismatch be? In one case, an in-house data-reduction program flipped the sign of anomalous differences, the protein structures came out with the wrong handedness, and five papers were retracted ([Miller, 2006](https://www.science.org/doi/full/10.1126/science.314.5807.1856)). NASA lost the Mars Climate Orbiter because one piece of software supplied thruster data in pound-seconds where the receiving software expected newton-seconds ([Mishap Investigation Board, 1999](https://llis.nasa.gov/llis_lib/pdf/1009464main1_0641-mr.pdf)).

In both cases, the program did not crash, no test failed, and the numbers looked good, maybe too good. They were wrong. Countless similar cases exist, with varying impact, and they share one lesson: hand-rolled glue code that translates conventions, dimensions, and metrics hides a danger nobody sees.

Computational precision deserves particular attention. ECMWF ran its forecast model at single precision with "no noticeable reduction in accuracy, and an average gain in computational efficiency by approximately 40%" ([Váňa et al., 2017](https://journals.ametsoc.org/view/journals/mwre/145/2/mwr-d-16-0228.1.xml)). Those 40% were available all along, spent on a precision decision that went unchallenged for far too long. Next to this case sits the whole category of errors caused by precision mismatch and integer overflow. Precision is a hidden risk factor in scientific computing, on top of its obvious cost.

Hand-rolled glue code and arbitrarily chosen precision raise three questions:

1. If the call site is the wrong place for a translation, where is the right place?
2. Should a library author decide the precision upfront?
3. How could math programming work without glue code and without preselected precision?

I know these questions firsthand. For the DeepCausality project, I implemented many multi-physics examples to showcase counterfactual simulations, and I wrestled with all of the above. The EU standardizes bananas, and an [ISO standard (ISO 3103)](https://www.iso.org/standard/73224.html) governs how to brew tea, yet nobody has standardized how to compose different kinds of math in a program. Instead, we write maddening glue code, often with hard-coded precision that may not match the precision the problem requires.

So I rolled up my sleeves.

## Inversion I: GATs over Glue

Rust has no native higher-kinded types (HKTs), but it does have generic associated types (GATs). The Rust team argues that GATs deliver about 80% of the flexibility of HKTs without the complexity. I agree in principle: for most programming tasks, HKTs would be overkill. However, I needed the other 20% when I tried to compose tensors, multivectors, and later topological types such as manifolds without loosing my mind. When the host language lacks HKTs, the witness pattern is the one practical way to add them back. To see how it works, let's define an arity-1 HKT:

```rust
pub trait HKT {
    /// The Generic Associated Type (GAT) that represents the type constructor.
    type Type<T>;
}
```

The GAT `Type<T>` is the type constructor: a type parameterized over a type, hence higher-kinded. Arity 1 alone is of limited use, so let's define arity 2 and arity 3:

```rust
pub trait HKT2<F> {
    type Type<T>;
}
```

Here the generic type `F` is fixed and `T` remains open. If the implementing type is `Result<(), F>` and `F` is `i32`, then `Type<T>` is `Result<T, i32>`.

Arity 3 fixes two type parameters, which is exactly what `HKT3` does:

```rust
pub trait HKT3<F1, F2> {
    type Type<T>;
}
```

If the implementing type is `DiscoveryResult<(), F1, F2>`, then `Type<T>` is `DiscoveryResult<T, F1, F2>`.

How do we make these HKTs useful? Through the witness pattern: a zero-sized witness struct stands in for the actual type and implements the HKT. For an arity-2 HKT over `Result`:

```rust
use deep_causality_haft::{HKT2, Placeholder};

// Fix type E
pub struct ResultWitness<E>(Placeholder, E);

impl<E> HKT2<E> for ResultWitness<E> {
    type Type<T> = Result<T, E>;
}

type MyResult<T> = <ResultWitness<String> as HKT2<String>>::Type<T>;

let ok_value: MyResult<i32> = Ok(20);
assert_eq!(ok_value, Ok(20));
```

That is a lot of type salad, and presumably the unspoken reason Rust has no HKTs. In short: we create a zero-sized witness type, implement the HKT for it, and fix inside that implementation the type the witness stands for. The witness then composes with the type it represents, because it only wraps that type in a typed container.

Why bother? Because Functor, Applicative, Monad, and the rest of monadic composition can be implemented over these witness types as type extensions, which is impossible otherwise. For example, the HKT implementation for `CausalTensor` declares a witness and binds the tensor type to it:

```rust
impl HKT for CausalTensorWitness {
    type Type<T> = CausalTensor<T>;
}
```

You declare the witness once and implement the categorical traits against it in one place. Without HKTs, you would write the glue code at every call site. With them, you write it once, over the witness type. `Foldable`, for example:

```rust
impl Foldable<CausalTensorWitness> for CausalTensorWitness {
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.into_vec().into_iter().fold(init, f)
    }
}
```

Again, a ton of type salad that says something simple: fold a tensor into one value by applying a function, such as summing all elements. The library author owns the HKT implementation, so the API user never sees this or the other monstrosities. I'll spare you `Monad`, `CoMonad`, `Traversable`, and `Applicative`; their signatures get considerably worse.

Instead, let's see what replacing glue code with GATs does in practice. Recall the example from the beginning: a mesh walk in one library, a contraction in a second, a rotor in a third. That should scare anyone who has done it, because the conversion glue alone is a nightmare. With HKTs, it takes two lines:

```rust
let res = CausalTensorWitness::fmap(field, |v| {
    rotor.geometric_product(&v).geometric_product(&rotor_rev)
});
```

Done.

* Conversion? In the HKT implementation.
* Conventions? In the HKT implementation.
* Corrections? Made once, in the HKT implementation.

Replace glue code with a reference transformation over a GAT, and the code becomes readable and, more importantly, maintainable. Bugs are a fact of life; I have fixed plenty of my own. The question is how often you want to fix the same bug. Hand-roll a tensor-to-field mapping twelve times, find a bug in it, and you fix it twelve times. Miss one copy, and part of the code base stays broken while the rest is patched. HKTs are genuinely hard to work with and demand careful evaluation. In my experience, though, especially with geometric structures that are complex to begin with, HKTs reduce API usage to `map`, `bind`, or `fmap`. That beats hand-rolled index mapping.

## Inversion II: External Conventions over Internal Divides

One of the most perplexing quirks of contemporary physics is the [East Coast versus West Coast metric convention](https://www.math.columbia.edu/~woit/wordpress/?p=7773): the choice of signs (+ and −) in the Minkowski metric tensor of special and general relativity.

* The **West Coast** convention, or "mostly minus," makes the time component positive and the three spatial components negative. Richard Feynman used it, which made it popular among particle physicists.
* The **East Coast** convention, or "mostly plus," makes the time component negative and the three spatial components positive. Julian Schwinger and many general relativists preferred it because it matches the positive-definite signature of purely spatial geometry.

The Minkowski metric is ubiquitous in relativistic physics and central to particle physics. Unfortunately, you must pick one convention and stick with it, and that is where the house of cards collapses. Pick one convention in your library, and rest assured, someone else picked the other. As a library author, you cannot get this right. Worse, particle physics and general relativity increasingly converge in the quest for quantum gravity. Now go figure out which convention to use there.

In the end, I declared the metric in a dedicated metric crate. It defines both the East Coast and the West Coast convention, plus utilities to convert one into the other, and serves as the single source of truth that every downstream physics and math function takes as a parameter. Many modern physics formulas turn out to be metric-agnostic anyway, so not picking a metric, and making it a parameter instead, saves your sanity and delegates the decision to the API user.

Moving the conventions into one central crate made the implementations more robust. Where it applies, a function is unit-tested against both conventions. More importantly, the metric can be encoded in the type signature, so choosing the wrong metric fails at compile time.

## Inversion III: Precision as a Parameter

Choosing precision in engineering, and even more so in scientific computing, should be a deliberate trade-off between accuracy and cost. That is what common sense suggests. In reality, you deal with legacy libraries that stuck with `f32` for obscure reasons. Everyone else settled on `f64` for scientific computing and engineering, whether or not the problem needs it. If you need reference-class precision, you are stuck with software-emulated floating point, because apart from IBM, no CPU vendor has implemented `f128` in hardware. If you work anywhere near deep learning, you know the zoo of non-IEEE formats, such as BF16 from Google Brain or NVIDIA's custom formats, which mainly shrink the memory footprint of training runs.

The problem: the libraries you depend on lock in their precision, and so does your own code the moment you pick a primitive number type. That works until it doesn't. In supercomputing, teams often reduce precision to save memory, because large simulations such as weather forecasts consume enormous amounts of it, and switching from `f64` to `f32` halves that consumption. But you cannot switch the downstream libraries to a different scalar type, because they picked a primitive type ages ago, and rewriting everything for two precisions is rarely feasible.

DeepCausality's unified math stack made a different decision. It declares every numerical type by its algebraic structure, such as a real field for floating-point numbers or a complex field for complex numbers, and implements that structure for the concrete primitive types. As a result, precision becomes a parameter:

```rust
type FloatType = f64; // or f32, or Float106, or BF16...

pub struct Epoch {
    /// Seconds since the first epoch.
    t_sec: FloatType,
    /// Δt to the next epoch
    dt: FloatType,
}

pub fn advance(
    value: CausalEffect<FloatType>,
    mut state: NavState,
) -> NavProcess {
   //...
}
```

The `FloatType` alias propagates through every type, struct, function, and method. Where code would name `f32`, it names `FloatType`.

To find out whether a higher-precision type such as `Float106` gains you anything over `f64`, you change one line:

```rust
type FloatType = Float106; // or f32, or BF16...
```

Then you compare the results and check whether the extra memory and CPU time buy a meaningful improvement. I have done this often, since most code examples in the DeepCausality project use precision as a parameter. In some cases, switching to `Float106` improved precision by 13 orders of magnitude. In others, switching from `f64` to `f32` cost no measurable precision and halved memory usage.

Some projects require reference-class precision, and there `Float106` contributes. Others balance memory against cost and must measure case by case to choose the right precision. Budget-conscious projects, sometimes targeting embedded devices with little memory, want the lowest precision that works. There, BF16 may help: it keeps the dynamic range of `f32` in the memory footprint of `f16`, at the expense of precision, since it carries only about three significant decimal digits. Where a quantity needs only that many digits, even `f64` data can be downcast to BF16 without measurable loss.

Precision as a parameter lets you choose per module, sometimes per function, and then aggregate the mixed-precision results in `Float106`. Aggregation requires `Float106` because it is the widest type available and can absorb every other type.

Environments that need still more precision, such as a hypothetical `Float512`, would need another custom float type. `BF16` and `Float106` serve as templates for building one, and every existing math library in the stack can use the new type, because precision is a parameter.

With this inversion, the library author no longer decides the precision upfront. The application programmer decides, case by case, based on precision, memory, and budget requirements.

The three inversions show that composition can be simplified, that conventions belong in a single central crate as the single source of truth, and that precision can be a parameter. One question remains: how could mathematical and scientific programming work without glue code and without preselected precision?

## Can You Show Me an Example?

DeepCausality answers with its unified math stack, which integrates these principles across 17 crates.

### For Starters: Differentiation under the Integral

In financial modeling, hedging a stock trade often involves options. To determine the cost of a hedge, you calculate the fair option price and compare it with the market's ask price. An option's price is an integral of the payoff over a probability distribution of the future stock price. Traders need the price, and they need the Greeks (delta, vega, and rho), the derivatives of that integral with respect to the spot price, the volatility, and the interest rate. In other words, you differentiate under the integral.

In pharmaceutical research, you model how much of a drug remains in the bloodstream against the body's clearance curve, which itself depends on parameters such as the half-life. Fitting the half-life to measured blood samples means gradient descent, and gradient descent needs d(prediction)/d(half-life). The prediction is an integral, so that derivative, again, is differentiation under the integral.

Let's differentiate under the integral with unified math. We answer two questions about a tunable parameter θ:

1. What is the area under the curve?
2. How much does the area change when θ moves?

```rust
type FloatType = f64;

let theta = Dual::variable(lift::<FloatType>(1.3)); // "watch this one"

let sweep = quadrature(
    |x: Dual<FloatType>| (theta * x).sin(), // the integrand
    Dual::constant(lift(0.0)),              // from 0
    Dual::constant(lift(1.0)),              // to 1
    128,                                    // panels
);

let integral = sweep.value();         // I(θ)  = 0.5634624396
let sensitivity = sweep.derivative(); // dI/dθ = 0.3077659585
```

The first line sets the working scalar type to `f64`, which generally suffices for financial modeling. `theta` marks the free parameter.

`theta` is a dual number: a pair of a value and its derivative. The first slot holds the value; the second holds the derivative. We write no chain rule and no differential calculus; the dual type carries it.

`quadrature` is a plain integrator implementing Simpson's rule, which anyone with first-year calculus has met in a more verbose form. It is built as an arrow composition that is generic over the scalar type, so it accepts dual numbers.

One line, `sweep.value()`, returns the integral; another, `sweep.derivative()`, returns its sensitivity.

Full code: [examples/mathematics_examples/3_applications/differentiate_under_integral](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples/3_applications/differentiate_under_integral)

This is your blueprint for pricing an option or modeling a drug's clearance. Anyone who has differentiated under an integral has almost certainly seen far worse, because the alternatives are few. You can hand-roll both the integrator and the differentiator; good luck, because that is surprisingly hard to debug. You can use finite differences and run the integral twice, but then you have a step-size tuning problem. Or, most commonly, you pull in a full-blown tensor framework such as TensorFlow or PyTorch to solve a plain calculus problem.

In the financial industry, the default is to write such calculus code yourself or to reuse an internal library. Nothing is wrong with PyTorch or TensorFlow, but a dependency that heavy is hard to justify for pricing options in a fast-moving market.

### For Electrical Engineers: An Electromagnetic Wave in One Product

Designing a phased-array antenna, for example for a 5G network, requires several quantities: the electric field, the magnetic field, and a consistency check called the gauge condition.

Physics describes the wave with one function, the potential, here a cosine of time minus position. Everything an antenna engineer cares about follows from it. In geometric algebra, the electric field E and the magnetic field B unify into a single electromagnetic field bivector F, derived from the vector potential A.

In unified math, the solution follows the physics step by step:

```rust
type FloatType = f64;

// The wave, written once. `S` is "any scalar".
impl DifferentiableField<2> for PlaneWavePotential {
    fn run<S: Scalar>(&self, tz: &[S; 2]) -> S {
        (self.omega.lift::<S>() * (tz[0] - tz[1])).cos() // A_x = cos(ω(t − z))
    }
}

let metric = Metric::Minkowski(4); // time +, space − − −

let a_x = wave.run(&[t, z]);                  // the potential at (t, z)
let [da_dt, da_dz] = wave.gradient(&[t, z]);  // its two partials, exact

let potential_a = multivector(&[(E_X, a_x)], metric)?;
let gradient = multivector(&[(E_T, da_dt), (E_Z, da_dz)], metric)?;

let field_f = gradient.geometric_product(&potential_a); // F = ∇A, one product

let divergence = blade(&field_f, SCALAR)?; // ∇·A → 0, the gauge check
let e_field = blade(&field_f, E_TX)?;      // E_x
let b_field = blade(&field_f, E_ZX)?;      // B_y → |E| = |B|
```

A few notes on the machinery. The `gradient` method comes from the `DifferentiateFieldExt` extension in the calculus crate. `PlaneWavePotential` gets it automatically because it implements `DifferentiableField`, the trait the extension targets. Implementing `DifferentiableField` is straightforward:

```rust
pub trait DifferentiableField<const N: usize> {
    /// Evaluate the field at working scalar `S`.
    fn run<S: Scalar>(&self, x: &[S; N]) -> S;
}
```

`PlaneWavePotential` implements it in three lines, shown above. Note that `S: Scalar` is precision as a parameter: it accepts whatever floating-point type you set with the `FloatType` alias. For reference-class results you would use `Float106`, provided the measurement noise justifies that precision.

First, the metric. As discussed earlier, the metric matters in physics, so to avoid any confusion, the multivector constructor requires it explicitly. `Metric::Minkowski(4)` tells the algebra that time and space carry opposite signs, as spacetime physics requires.

Getting E and B from the vector potential A means differentiating it with respect to t and to z. The wave is written once as a function over "any scalar." `gradient` runs that function on dual numbers, the value-plus-derivative pairs from the first example, and returns both partial derivatives exactly. The mechanism is the same as before: the dual number type supplies the chain rule.

The code places the potential and its derivatives into multivectors, the working objects of geometric algebra. Then `gradient.geometric_product(&potential_a)` computes the field F = ∇A in one line. The geometric product of two vectors returns a scalar part and a bivector (plane) part at once. The scalar part is the divergence, which the gauge condition requires to vanish. The plane parts are the fields: the t–x plane holds E, and the z–x plane holds B. One multiplication yields three physical quantities, and the code reads each by its blade index.

This example evaluates one spacetime point of an ideal plane wave in natural units; it covers the representation and the product. A real antenna simulation would add a mesh, boundary conditions, and material properties. The unified math stack provides those components, and the [structural stress example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples/3_applications/structural_stress_on_mesh) in the same folder shows a mesh-based computation.

Full code: [examples/mathematics_examples/3_applications/electromagnetic_field](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples/3_applications/electromagnetic_field)

I'll spare you the alternative. Electrical engineers who had to code all this in, say, MATLAB have my deepest sympathy, and I sincerely hope the practice will one day be banned as a human rights violation.

### For the Crazy Ones: Weighing Earth with Satellite Clocks

General relativity distinguishes two regimes of the Einstein field equations. In the strong-field regime, extreme masses such as black holes deform spacetime drastically, and you face ten coupled partial differential equations in full. Our solar system has no black hole, so that regime does not apply here.

In the weak-field regime, gravity deforms spacetime only slightly, and the equations collapse into a much simpler form because spatial curvature is nearly zero. Relativistic effects then show up mainly as a deformation of time, known as time dilation.

The simplified relation is sometimes called the Einstein clock equation, a name that signals it holds only in the weak field. Solved forward, it tells you how much a body with gravitational parameter GM deforms spacetime, and in the weak field we measure that deformation through time dilation alone. Invert it, and differences in time dilation give you GM. With some clever math, you can determine a planet's mass solely from its effect on time.

Put simply, you can weigh a planet with a clock. Let's do that with unified math.

Every GNSS satellite carries an atomic clock, and each clock runs at a slightly different rate depending on where it is. Higher up, farther from Earth's mass, a clock ticks faster. Moving faster, it ticks slower. Both effects come from Einstein, and both are about one part in ten billion. Engineers correct for them so that your phone computes the right position. This example reads them in reverse and infers Earth's gravitational parameter from tiny differences in time.

```rust
type FloatType = Float106;                     // (1) ~32 significant digits

CausalFlow::value(inputs)
    .try_step(stage_load::<FloatType>)         // (2)
    .try_step(stage_align)                     // (3)
    .try_step(stage_pair)                      // (4)
    .try_step(stage_solve_gm)                  // (5)
    .try_step(stage_aggregate)                 // (6)
    .run(|report| print_gm_report(&report), |err| ...); // (7)
```

I admit this example uses more than unified math. It uses the `CausalFlow` domain-specific language, which abstracts over the individual steps and chains the stages through monadic composition. Under the hood, it rests on the same applied category theory as unified math. But I digress. Let's walk through the stages.

1. Set the working scalar type to `Float106`. The signal is about 10⁻¹⁰ of the carrier, which sits at the edge of what `f64` resolves, so we go straight to `Float106`.
2. Load one week of Galileo GNSS clock and orbit data: seven days of public files for satellite E14. E14 flies an eccentric orbit, so its altitude changes by thousands of kilometers per revolution. That altitude swing keeps the subtraction well-conditioned.
3. Align the data. Clock readings arrive every 30 seconds, orbit positions every 15 minutes, so this stage fits a tenth-order Lagrange polynomial through the orbit points to locate the satellite at each clock tick, and differentiates it for the velocity.
4. Pair the aligned epochs.
5. Invert the clock equation for each pair and solve for GM, the gravitational parameter.
6. Aggregate: reject extreme outliers, average GM, and divide by the Newtonian gravitational constant G to obtain Earth's mass.
7. Propagate any error, or print the results.

What do we get?

```text
  GM_Earth  [m³/s²]    3.994180e14   recovered
                       3.986004e14   reference (JGM-3)
  Relative error:      0.2 %
```

From one week of public Galileo data, we recovered Earth's mass to within 0.2%.

Would more data reduce the error? Yes, but more data also exposes another systematic error: lunar gravity exerts a periodic influence on Earth that the inverse GM equation must account for. Do that, process about three years of data, and the result lands close to the official reference value for Earth's mass. Am I serious? Yes; I did it. The full experimental data and code live in a separate repository.

Full code for the one-week example: [examples/physics_examples/chronometric_gm_recovery](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/chronometric_gm_recovery)


Much of this example hides behind high-level, domain-specific abstractions. The short version: thanks to Rust's zero-cost abstractions, high-level abstraction obstructs neither the physics nor performance. Most unified math examples read straightforwardly, without the usual boilerplate. Precision as a parameter also makes memory an economic choice: parts of a simulation can drop to, say, BF16, freeing memory for the parts where higher precision improves the results. And exploring crazy ideas, such as inverting the Einstein field equations, brings back the fun that debugging gigatons of glue code would have killed.

## Conclusion

This post only scratches the surface. Unified math is an iceberg, and you have seen its tip. Underneath lies a dense web of abstract algebra, applied category theory, algebraic geometry, and some even stranger material. I skipped the implementation details because they span a year and a half of learning. DeepCausality's downstream projects in advanced science and engineering, such as counterfactual fluid dynamics and quantum causal models, are all built on the unified math foundation.

* Explore [the code and examples on GitHub](https://github.com/deepcausality-rs/deep_causality).
* [Join the community](https://www.deepcausality.com/community).

## About

DeepCausality is a dynamic-causality framework that enables fast and deterministic context-aware
causal reasoning in Rust. The DeepCausality project is hosted at the Linux Foundation for Artificial Intelligence and
Data (LF AI & Data). 

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open-source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all
members of the community. For more information, please visit lfaidata.foundation.

Please give us a [star on GitHub.](https://github.com/deepcausality-rs/deep_causality)
