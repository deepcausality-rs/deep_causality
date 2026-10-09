# Unified Math in Science and Engineering

Science and engineering relies on increasingly more sophisticated computer simulations to advance understanding of complex systems. Increasingly, those simulations cross multiple domains and underlying those scientific and engineering domains is usually a different category of mathematics. For example, mechanical engineering oftentimes relies on calculus, differential equations, and linear algebra whereas fluid dynamics leans on vector analysis, and partial differential equations based on fundamental conservation laws. Ideally, one would like to deal with those different domains separately and in isolation.

Unfortunately, in the automotive and avionics industry, you cannot look at these areas in isolation, because the mechanical engineering required to build the car interrelates with the fluid dynamics of the vehicle once its in the wind-tunnel.
For an airplane, you cannot separate the mechanical strength of the airframe from the drag and pressure during the flight, and therefore the mathematics of both discipline have to be considered in a simulation. And even if you don’t build an airplane, a commercial drone is subject to the same aerodynamics and physics as all flight vehicles, and the same underlying mathematics applies to a drone.

## Multi-Glue-Maths

Naturally that raises the question of what exactly does it take to glue together multiple disciplines of mathematics in programming? It turned out, you end up with multiple different physics and mathematics libraries that need to be glued together. A mesh walk in one library, a contraction in a second, a rotor in a third. And then you encounter the reality that different libraries encode different conventions: row-major against column-major, zero-based against one-based, dense against sparse, east coast metric vs west coast metric. The obvious response is to convert, and a lot of conversion code later, you realize that a third of the entire code base is glue to handle different representations and translating between conventions.

The consortium behind the Python array API standard states it plainly: the APIs of NumPy, TensorFlow, PyTorch, JAX, Dask, CuPy and MXNet "are largely similar, but with enough differences that it's quite difficult to write code that works with multiple (or all) of these libraries" (I8).
Even when the project team adheres to all engineering best practices, the mathematics is checked, the derivation is checked, but the seam between two libraries is not until someone has to debug it and finds yet another diverging convention. A convention, e.g.row major, Is nothing the compiler can statically check, and without a regression test, it is impossible to ascertain which convention applies, where and when.

Hatton's experiment (T2) showed that independent implementations of the same seismic algorithms, on the same input, agreed to six significant figures at the start and "gradually degenerated from 6 significant figures to 1 significant figure during the computation"; he attributes the disagreement "squarely at the door of software failure”.
McCullough's benchmark method found statistical packages disagreeing on the same problems, and Keeling and Pavur found across nine packages that the choice of package and of default settings changes the answer (I12). It is mildly concerning when replicating the statistics requires the exact same software because no two packages can agree on how to implement the same math formula.


## It can’t be that bad, right?

How serious could a convention mismatch be, you may ask? In one case, an in-house data-reduction program flipped the sign of anomalous differences, the protein structures came out in the wrong hand, and five papers were retracted(I2).
The Mars Climate Orbiter was lost to thruster data supplied in pound-seconds where the receiving software required newton-seconds; (I10).
In all of these cases, the program did not crash, the test suite did not fail, and the numbers looked good, maybe too good. Except they were wrong. There are countless similar cases with various degree of impact, but the main takeaway is that there is  oblivious danger in hand rolled glue code that handles  convention, dimension, and metric translation.

Computational procession deserves particular attention. ECMWF ran its operational forecast at single precision(!) with "no noticeable reduction in accuracy, and an average gain in computational efficiency by approximately 40%" (I13). That 40% was available the whole time and was spent on a decision made inside a library and remained unchallenged for far too long.
Adjacent to this specific case is the entire category of precision mismatch and integer overflow induced errors. That means precision implies a non-obvious risk factor in scientific computing besides the obvious impact on economics.

The status quo of hand-rolled glue code and randomly picked precision levels that lead to all sorts of problematic outcomes raises a number of serious questions:

The obvious first question: If the call site is the wrong place, then where is the most sensible place for a translation?

The second question: Should a library author even decide the precision level upfront?

The third question then becomes: How could math programming be different, without glue code and without pre-selected precision?

I am no stranger to these questions. In the DeepCausality project, I implemented a fair amount of multi-physics examples to showcase counterfactuals simulations and I haggled with all of that. Much to my dismay, mankind can standardize bananas in the EU and we have an [ISO standard (ISO 3103)](https://www.iso.org/standard/73224.html) for tea preparation, but nobody can standardize how to compose different types of math in programming without writing tons of maddening clue code half the time with hard coded precision levels that may not even reflect the actual required precision level.

In response, I rolled up the sleeves...

## Inversion Part I: GATs over Glue

Rust has no native higher-kinded types (HKTs), which is well known at this stage, but it does have Generic Associated Types a.k.a GAT. The Rust team argues that GATs give you about 80% of the flexibility one would get from HKTs minus the complexity of HKTs. I share this assessment in principle because for the vast majority of common programming tasks, HKTs would be overkill to say the least. Except, I found myself in a position of needed the other 20% when squaring together how to uniformly compose tensors, mulivectors and later topological types such as manifolds. When the hosting language does not have HKT, there is just one practical way to add HKT and that is through the witness pattern. To see how that works, lets define an arity-1 HKT

‘’’rust
pub trait HKT {
/// The Generic Associated Type (GAT) that represents the type constructor.
type Type<T>;
}
‘’’

The GAT Type<T> defines the actual type constructer over a type, hence a higher minded type. Now,  arity-1 alone is not super useful, so let’s define  arity-2 and  arity-3.

‘’’rust
pub trait HKT2<F> {
type Type<T>;
}
‘’’
Here we have generic type F fixed, and leaving type T associated. If the implementing type is `Result<(), F>` and `F` is `i32`, then `Type<T>` is `Result<T, i32>`.

Then, arity-3 represents there type parameters and that is exactly what HKT3 does:

‘’’
pub trait HKT3<F1, F2> {
type Type<T>;
}
‘’’

If the implementing type is `DiscoveryResult<(), F1, F2>`, then `Type<T>` is `DiscoveryResult<T, F1, F2>`.

How do we get those HKTs to do something uselful? We need the aformationed witness pattern which means we need a zero sized witness structure that stands in for the actual type to implement the HKT. For an arity-2 HKT over the Result type, that means


‘’’
use deep_causality_haft::{HKT2, Placeholder};

// Fix type E
pub struct ResultWitness<E>(Placeholder, E);

impl<E> HKT2<E> for ResultWitness<E> {
type Type<T> = Result<T, E>;
}

type MyResult<T> = <ResultWitness<String> as HKT2<String>>::Type<T>;

let ok_value: MyResult<i32> = Ok(20);
assert_eq!(ok_value, Ok(20));
‘’’

That is a lot of type salad and presumably the unspoken reason Rust does not have HKTs. In short, we crate the zero sizes witness type, then implement the HKT over the witness type, in which we fixate the actual type the witness stands in for, and as a result, the result from the witness type composes with the type it represents. That is obvious because the witness type only crates a typed container around the type for which it stands in.

Why bother you may ask? Turned out all the goodness of monadic composition such as Functor, Applicative, Pure, Monad and more can be implemented over those HKT witness types as a type extension and this is not possible otherwise. For example, when we implement HKT for the CausalTensor type we crate the witness and fixate the inner type to witness type.

‘’’
impl HKT for CausalTensorWitness {
type Type<T> = CausalTensor<T>;
}
‘’’

That way, you declare the witness once and implement the categorical traits for composition against it in one place. Remember, without HKTs you would have to write the glue code at every the call site. Now, you write once over the witness type. For example, for Foldable, you simply implement

‘’’
impl Foldable<CausalTensorWitness> for CausalTensorWitness {
fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
where
Func: FnMut(B, A) -> B,
{
fa.into_vec().into_iter().fold(init, f)
}
}
‘’’

Again, a ton of type salad that is a complicated way to convey to fold a tensor into just one value by applying a function e.g. sum all elements. The library author owns the HKT implementation so the API user never is going to see this and other monstrosities. I spare you the details for Monad, CoMonad, Traversable and Applicative because the type signatures are getting significantly more complex.

Instead, it’s a good time to show what the inversion from glue codes to GAT’s actually does in practice. Remember the multi glue math example from the beginning? A mesh walk in one library, a contraction in a second, a rotor in a third. This should sound scary to anyone who has done it because the glue code for handling the conversion alone is a nightmare. Now, with all those handy HKTs we write 3 lines.


‘’’
Let res = CausalTensorWitness::fmap(field, |v| rotor.geometric_product(&v)
.geometric_product(&rotor_rev))
‘’’

Done.

Conversion?  In the HKT implementation.
Conventions? In the HKT implementation.
Corrections? Made once in the HKT implementation.

When you replace the glue code with a reference transformation over a GAT in a HKT, all of a sudden, the code becomes readable again and more importantly maintainable. Implementation bugs happen, it’s a fact of life. I fixed a fair share of bugs that I crated in my life. The question boils down to how often you want to fix the same bug again? When you hand roll tensor to field mapping 12 times in a code base and you find a bug there, you fix it 12 times. Except there might be more cases that were missed, than parts remain patched while others don’t. HKTs are genuinely hard to work with and require careful evaluation, but in my experience, especially when working with complex geometric structures that already have a high base complexity, HKTs make the usage a lot less frighting because API usage reduces to combining map, bind, or fmap.


## Inversion Part II: External Conventions over Internal Divide

One of the most perplexing properties of contemporary physics is the East Coast vs. West Coast Metric Convention. The convention refers to the the choice of signs (+ and −) used in the Minkowski metric tensor for special and general relativity. [1] (https://www.math.columbia.edu/~woit/wordpress/?p=7773)
The West Coast Convention, a.k.a. "mostly minus”, defines the time component as positive, and the three spatial components as negative. Richard Feynman famously used this convention, making it popular among particle physicists.
The East Coast Convention a.k.a. "mostly plus". The time component is negative, and the three spatial components are positive. Julian Schwinger and many general relativists preferred this approach because it matches the natural positive-definite signature of pure spatial geometry

The Minkowski metric tensor is ubiquitous in relativistic physics and its also quite important in particle physics. Unfortunately, you have to pick one metric convention and stick with that. And that is where the entire house of cards falls down. If you were to pick one of those convention in one library, rest be assured somebody else has picked the other one.  As a library author, there’s absolutely nothing under the sun you can do right in this situation. Even worse, in contemporary science, particle physics and general relativity converge increasingly in a quest towards quantum gravity. Now go figure out which metric convention to use there.

In the end, I settled for declaring the metric in a dedicated metric crate, which defines the east Casto and westast Co convention, and some utility to convert one metric into the other as one central source of truth all downstream physics and math functions take in as a parameter. As it turned out, a lot of modern physics formulas are already metric agnostic and therefore not picking any metric and instead making it a parameter saves your sanity and delegates the decision on the API user.

Externalizing those conventions in one central crate has unlocked a lot more robust implementations that can be unit tests against both metric conventions when applicable and, more importantly, the correct metric type can be encoded in the type signature which causes an incorrect metric selection to trigger a compilation failure.

## Inversion Part III: Precision as a parameter,

Precision selection in engineering and even more so in scientific computing is a very profound process because fundamentally you’re trying to balance precision with the associated cost. At least that is what common sense would suggest. In reality, you have to deal with legacy libraries that somehow have stuck to F32 for some weird reason. Then everyone else has settled for F34 for general scientific computing and engineering, regardless of whether that level of precision is actually needed or not. And lastly, if you need reference class precision, you’re stuck with software emulated floating points because somehow, except for IBM, nobody has ever implemented F128 hardware acceleration in CPUs. If you work even remotely with deep learning, then you’re accustomed to at least an entire zoo of non-IEEE custom floating point number types such as BF16 from Google DeepMind or Nvidia’s custom floating point numbers that predominantly reduce the memory footprint of deep learning runs.

The problem though is that those libraries you rely on lock in their precision level and the internal libraries you are writing as part of your undertaking also locks in a specific precision level the moment you pick any primitive number type. This works until it doesn’t. In scientific supercomputing, precision is quite often reduced predominantly to conserve memory as large simulations such as whether forcast tend to consume an enormous amount of memory, and just switching from F64 to F32 cut this memory usage in half. The problem here is that you just can’t switch all downstream libraries to a different scalar type because they picked a primitive numerical type ages ago, and rewriting everything for dual precision is quite often not feasible.

In deep causality, the unified math stack has made a fundamentally different decision and declared all numerical types as the correct algebraic representation, such as real field for floating point numbers or complex field for complex numbers and then implemented the algebraic representation over the actual primitive types. As a result, precision is a parameter:

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


This example illustrates the precision as a paradigm propagates through every type, struct, function and method where instead of a named F32 you simply plug in the type alias FloatType.

If you want to determine if you gain any precision by switching from F64 to a high precision type such as Float 106, you change one line.


```rust
type FloatType = Float106; // or f32, or BF16...
```

And then you can compare the results and see if the increase in memory and CPU usage yields any meaningful gain in the results.
I did that in many cases since the vast majority of code examples in the Deep Causality project are implemented using precision as a parameter. In some cases, 13 orders of magnitude of improved precision were recorded by switching to Float 106. In other cases, switching from F64 to F32 yielded zero measurable gain in terms of precision, but memory usage dropped in half.
Some projects have reference class precision requirements, and here float 106 certainly contributes something. Other projects try to balance especially memory usage with cost, and that needs to be measured case by case in order to select the correct precision level. And other projects are much more budget conscious and therefore trying to cast in the lowest possible precision level. Sometimes to optimize for embedded devices with a limited memory budget. In those cases, BF16 may yield something because it fundamentally provides the dynamic range of F32 but only requires the memory footprint of F16. This comes at the expense of the numerical range. That means BF16 can represent a smaller number at a very high precision level at half the memory footprint of F32. In certain cases where the range of representable numbers is relatively small, even F64 can be downcasted to BF16 at no measurable loss.

Precision as a parameter allows it to make an informed choice on a per-module and sometimes even per function base and then aggregated the mixed precison via the Float106 type. Float106 becomes required for aggregation because it’s the only type that has the widest representation that can absorb all other types.

In environments where even higher precision levels are required such as Float512, this would require another custom implemented float types. The BF16 and Float106 type serve as a good template how that can be accomplished and then all existing math libraries can use the new synthetic float type because precision is a parameter.

The inversion establishes that no longer a library author decided the precision level upfront. Instead, the application programmer decides the precision level on a case by case base relative to precision, memory, and budget requirements.

The three inversions have shown that composition can be simplified, that conventions are well placed in a single central  crate as a single source of truth, and that precision can be a parameter. The last question that demands an answer is: How could math or scietific programming be different, without glue code and without pre-selected precision?


## Can you show me an example?

The DeepCausality project has an answer for that in form of the unified math stack that deeply integrate these core principles across 17 crates.

### For Starters...
...differentiate the area under an integral.

In financial modeling, whenever you have to deal with hedging, you times encounter options to hedge the stock trade. To determine the cost of hedging, you need to calculate the fair option price and compare to the ask price in the market.
An option's price is an integral of the payoff over a probability distribution of the future stock price. Traders need the price, and they need what’s called the greeks, the delta, vega and rho, which are the derivatives of that integral with respect to the spot price, the volatility and the interest rate.   In other words, you differentiate under an integral.

In pharmaceutical research, You often have to model how much of a drug that has entered the bloodstream is remaining there relative to the body's clearance curve, that itself has some parameters like the half-life. Fitting the half-life to measured blood samples means gradient descent, and gradient descent needs d(prediction)/d(half-life). The prediction is an integral, so that derivative is differentiation under the integral.

Let’s use unified math to differentiate the area under an integral. We do that to answer two questions w.r.t. to parameter theta we can tune.
1. What is the area under the integral?
2. How much does the area changes when I move theta?

```rust
type FloatType = f64;

let theta  = Dual::variable(lift::<FloatType>(1.3));      // "watch this one"

let sweep = quadrature(
    |x: Dual<FloatType>| (theta * x).sin(),   // the integrand
    Dual::constant(lift(0.0)),                // from 0
    Dual::constant(lift(1.0)),                 // to 1
    128,                                       // panels
);

let integral    = sweep.value();     // I(θ)      = 0.5634624396
let sensitivity = sweep.derivative() // dI/dθ     = 0.3077659585
```

The first step sets the working Scala type to F64. This is generally sufficient for financial modeling. Theta marks the free parameter for the integration.

Here we’re using dual number, which is a pair of a number and its own derivative.  The first slot of a dual number contains the initial value, and the second slot contains the derivative of the value in the first slot. We don’t write any chain rule or any differential calculus here. The dual type already has it on its sleeve.

A plain integrator. quadrature is Simpson's rule:: Anyone with first year calculus has encountered Simpson's rule, albeit in a more verbose way. Here the integrator is implemented over an arrow composition that is scalar invariant and thus takes in dual numbers as a parameter.

Solve the integral in one line, `sweep.value()`, and the differential, the sensitivity, in another line via `sweep.derivative()`

Full code:
examples/mathematics_examples/3_applications/differentiate_under_integral/main.rs

This is your blueprint to calculate your option pricing or your pharmaceutical drug body reaction. Anyone who has differentiated under an integral has almost likely seen much worse than that because there are only few options left. One is obviously to hand roll your integral and your differentiator. Good luck with that, because that is surprisingly hard to debug. Then you can obviously use finite differences, run the integral twice, but then you have a tuning problem. And the third option is presumably the most common solution that you pull in a full-blown Tensor framework such as TensorFlow or PyTorch to solve a plain cut calculus problem.

If you have a background in the financial industry, the default is to write those calculus frameworks yourself or reuse what is available as an internal library. There’s nothing wrong with PyTorque or TensorFlow. It’s just very hard to justify to pull in such a heavy dependency for calculating option prices in a very fast-moving market.

### For Electrical Engineers...

When you design a fast array antenna, for example, for a 5G network, you have to calculate a number of values e.g. The electric field, the magnetic field, and the consistency checked which is called a gauge condition.

Physics describes it with one function, the potential, which here is a cosine of time minus position. From that one function, everything an antenna engineer cares about follows: the electric field E, the magnetic field B, and a consistency check called the gauge condition.

In Geometric Algebra, Electric (E) and Magnetic (B) fields are unified into a single Electromagnetic Field Bivector (F) derived from a Vector Potential (A).

In unified math, we would solve that in a few steps that largely follows the physics:


```rust
type FloatType = f64;

// The wave, written once. `S` is "any scalar" derivatives.
impl DifferentiableField<2> for PlaneWavePotential {
    fn run<S: Scalar>(&self, tz: &[S; 2]) -> S {
        (self.omega.lift::<S>() * (tz[0] - tz[1])).cos()      // A_x = cos(ω(t − z))
    }
}

let metric = Metric::Minkowski(4);     // time +, space − − −

let a_x        = wave.run(&[t, z]);  // the potential at (t, z)
let [da_dt, da_dz]  = wave.gradient(&[t, z]);  // its two partials, exact

let potential_a = multivector(&[(E_X, a_x)], metric)?;
let gradient    = multivector(&[(E_T, da_dt), (E_Z, da_dz)], metric)?;

let field_f = gradient.geometric_product(&potential_a);       // F = ∇A, one product

let divergence = blade(&field_f, SCALAR)?; // ∇·A → 0, the gauge check
let e_field    = blade(&field_f, E_TX)?;    // E_x
let b_field    = blade(&field_f, E_ZX)?;    // B_y   → |E| = |B|
```

A few notes on the underlying machinery. The gradient methods comes from the DifferentiateFieldExt type extension in the calculus crate and is automatically added to PlaneWavePotential because it implements DifferentiableField, the target type of the extension. If you were to implement DifferentiableField, the  trait signature is straightforward:

```rust
	pub trait DifferentiableField<const N: usize> {
    /// Evaluate the field at working scalar `S`.
    fn run<S: Scalar>(&self, x: &[S; N]) -> S;
}
```

And the implementation for PlaneWavePotential is equally straightforward:

```rust
impl DifferentiableField<2> for PlaneWavePotential {
    fn run<S: Scalar>(&self, tz: &[S; 2]) -> S {
        (self.omega.lift::<S>() * (tz[0] - tz[1])).cos()      // A_x = cos(ω(t − z))
    }
}
```

Notice scalar here stands in for position as a parameter and takes any floating point precision you would set with the FloatType alias. If you were to need reference class results, you would use Float106, granted the noise level permits that high level of precision measuring.

First the metric. As stated earlier, metrics in physics are important and to avoid any confusion, the MultiVector requires an explicit metric in its constructor. The Metric::Minkowski(4) line tells the algebra that time and space have opposite signs, which is what spacetime physics on needs.

Getting E and B from the Vector Potential (A) means differentiating it with respect to time and with respect to z. The wave is written once as a function over "any scalar". gradient runs that same function on dual numbers, the value-plus-derivative pairs from the first walkthrough, and returns both partial derivatives exactly. This is the same mechanism as in the previous example Meaning the chain rule on calculus details are provided by the dual number type.

The code puts the potential and its derivatives into a multivector, the working object of geometric algebra. Then, `gradient.geometric_product(&potential_a)` is the entire Maxwell's field equations in one line. The geometric product of two vectors returns a scalar part and a plane part at once. The scalar part is the divergence, which physics wants to be zero. The plane parts are the fields: the time-x plane holds E, the z-x plane holds B. Three physical quantities fall out of one multiplication, and the code reads them by slot index.

This example shows only one spacetime point of an ideal plane wave in natural units covering the representation and the product. Extending the example to a real antenna simulation, you would need a mesh, boundary conditions and material properties. The Unified Math ecosystem provides all the bits and pieces for adding those components and already comes with an example for the 3D stress example in the same folder shows the mesh example.

In this particular case, I spare you how the alternative would work. However, those electrical engineers who had to code all that in, let’s say, MATLAB have my deepest sympathy, and I genuinely wish that eventually this practice will be banned on the grounds of human rights violations.

### For the crazy ones...

In general relativity, the core idea of the Einstein field equation can be separated in two different regimes. One is the strong gravity regime, also known as the strong force, where you have ultra strong gravity deformations caused by something like a black hole. Here is the Einstein field equation getting really complicated and you have to deal with approximately ten partial differential equations. However, in our solar system we don’t have any black hole and therefore this regime is not applicable.

The second regime is referred to as the weak force regime, and there you don’t have strong gravity deformations, and that accounts for a significant simplification of the mathematics where the otherwise complex equation collapse to a simpler form because space curvature approximates zero. That means relativistic effects are predominantly measured by time deformation, also known as time dilation.

The simplified form of the equation is sometimes referred to as the Einstein clock equation because it implies that it is only applicable in the weak field. When you solve the Einstein field equation forward within the weak field, it basically tells you for any amount of Gm, which is the gravitational mass constant, how much its presence deforms space-time. But remember, within the weak field limit we only measure space-time deformation by time deletion. When you invert the equation, the difference in time dilation then gives you the difference in gm. And with some clever mass you can actually determine a planetary mass solely by its relative impact on space-time deformation as measured through time dilation.

In simple words, you can put a planet on a scale and get its mass measured by time. Let’s do just that with unified math.

Every GNSS satellite carries an atomic clock, and every clock runs at a slightly different rate depending on where it is. Higher up, further from Earth's mass, a clock ticks faster. Moving faster, it ticks slower. Both effects are Einstein's, and both are about one part in ten billion. Engineers correct for them so that your phone's position comes out right. This example reads them the other way round to infer orbital mass from tiny differences in time.

```rust
type FloatType = Float106;                 // (2)32 digits,

CausalFlow::value(inputs)
    .try_step(stage_load::<FloatType>)     // (2)
    .try_step(stage_align)                 // (3)
    .try_step(stage_pair)                  // (4)
    .try_step(stage_solve_gm)              // (5)
    .try_step(stage_aggregate)             // (6)
    .run(|report| print_gm_report(&report), |err| ...);  // (7)

```

I have to admit this example uses a little bit more than the unified math. Here we’re using the causal flow domain-specific language that abstracts over the specific steps and chains together each stage through monadic composition. Under the hood, this is implemented with the exact same applied category theory as the unified math. But I digress. Let’s walk through the example.

In the first step, we have to set the working scalar type at float 106 because the signal is signal is 1e-10 of the carrier and that is the edge of what f64 can represent, therefore we have to rack up the precision and go straight to F106.

Second, we are loading one week of Galileo GNS clock data and orbital data. Seven days of public files from the Galileo satellite E14, which has a stretched orbit, so its height changes by thousands of kilometers per lap. That height swing is what makes the subtraction well-conditioned.

Clock readings arrive every 30 seconds, orbit positions every 15 minutes, so in the third step, the align stage, fits a )(tenth order Lagrange) polynomial through the orbit points to know where the satellite was at each clock tick, and takes its derivative for the speed.

In step five, we invert the clock equation per pair, solve for GM, the Gravity Mass constant, and then proceed to the next pair. The, in stage six, we aggregate, weed out extreme outliers, and eventually calculate the mean planetary mass by dividing the average GM constant by earth gravity.

In stage seven, we either propagate any error, or print the results.  Then, what do we get here?

```test
  GM_Earth  [m³/s²]    3.994180e14   recovered
                       3.986004e14   reference (JGM-3)
  Relative error:      0.2 %
```

With just one week of public domain Galileo data, we have calculated Earth planetary mass wit 0.2% error.

One obvious question is, would more data actually reduce the error? Turned out yes, that is indeed the case, but that also exposes another source of systematic error. It turned out that the lunar  gravitation has a seasonal impact on earth that needs to be accounted for when solving the reverse GM equation. If you do that and process approximately three years of data, the final result will be approximately within the official NIST standard definition of Earth a planetary matter. Am I serious here? Yes, I am because I did that. You can find the full experimental data and code in another repository.

In terms of unified math, a lot of that has been shielded behind the high level domain specific abstractions, but the TLDR is that high level abstraction does not obstruct physics or even solid performance thanks to Rusts zero cost abstraction.  A lot of the unified math examples are straightforward and without the usual boilerplate. Furthermore, precision as a parameter Facilitates a more economic view on memory, where parts of a simulation can be downcasted into, let’s say, BF16 to conserve valuable memory, which then can be used in other areas of a simulation where high precision actually yields better results.
And lastly, exploring crazy ideas such as inverting the Einstein field equations brings back the fun that would have been lost in debugging gigatons of glue code...

I could only scratch on the surface of the unified math in this post. The reality is there is an entire iceberg of unified math, and you have just encoded the tip of it. Underneath is a dense, interrelated web of abstract algebra, applied category theory, algebraic geometry, and some even weirder stuff. I don’t really wanted to go into the details of the implementation because it was a one and a half year long learning journey and some of the original ideas did survive, such as the composition via category theory, other elements such as precision as a parameter that were discovered later in the process, and a couple of really dumb ideas, such as subtype constraint within Generic Associated Types, were eventually abandoned;


The deep causality project has some downstream project in more advanced science and engineering areas such as counterfactual fluid dynamics or quantum causal models that are all built atop the unified math capabilities.

For those three readers who managed to read this blog post to the end, I sincerely appreciate your capability to digest let’s say non-mainstream ideas, and if you don’t mind, please give us a star on GitHub. 