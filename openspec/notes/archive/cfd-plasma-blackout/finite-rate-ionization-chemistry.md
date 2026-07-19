# High-Enthalpy Air Ionization: Coupling Web, Code Divergence, and Leverage Points

Context: RAM-C II-class reentry (air, ~7 km/s, thermochemical nonequilibrium). This note covers (1) the full coupling web governing ionization chemistry, (2) where DPLR / LAURA / US3D actually diverge, and (3) the three highest-leverage points for closing the 2–3x prediction gap.

---

## Part 1 — All Known Couplings Affecting Ionization Chemistry

### The core species chain

In air (no ablation), electrons in the RAM-C regime come predominantly from **associative ionization**, not direct electron-impact ionization:

- N + O ⇌ NO⁺ + e⁻ (dominant electron source at these enthalpies)
- N + N ⇌ N₂⁺ + e⁻
- O + O ⇌ O₂⁺ + e⁻

To make electrons you first need free N and O atoms, which means you must dissociate N₂ and O₂ first. Electron density is therefore downstream of the entire dissociation cascade, which is why it is so sensitive — upstream errors compound multiplicatively by the time they reach e⁻.

### Coupling 1: Vibration–dissociation coupling (VDC / CVDV)

Dissociation does not happen from the ground vibrational state — molecules climb the vibrational ladder and dissociate preferentially from high-lying levels. So the **dissociation rate depends on the vibrational state distribution**, not just the translational temperature.

- Translational–rotational energy pumps vibrational energy (T–V transfer, Landau–Teller).
- Vibrationally excited molecules dissociate faster (preferential dissociation / vibrational favoring).
- Dissociation preferentially removes high-vibrational-energy molecules, which **cools the vibrational bath** (vibration–dissociation depletion).

A genuine two-way loop: vibrational temperature sets the dissociation rate, and dissociation reshapes the vibrational distribution. Park's two-temperature model approximates this with a geometric-mean controlling temperature √(T·Tᵥ); CVDV and Marrone–Treanor are the older closures; state-to-state kinetics resolves it explicitly. All three are approximations of the same coupling.

### Coupling 2: The multi-temperature energy ledger

Behind a strong shock, the energy modes relax at different rates and are **not equilibrated with each other**:

- Translational–rotational (fast, equilibrates in a few collisions)
- Vibrational (slow, Millikan–White relaxation times, temperature-dependent)
- Electronic excitation of atoms/molecules
- Free-electron translational temperature Tₑ

Every chemical rate depends on *which* temperature you feed it. Dissociation is governed by a T–Tᵥ combination; electron-impact processes are governed by Tₑ; associative ionization depends on heavy-particle translation. The controlling-temperature choice is a modeling decision that directly moves the electron density. This is arguably the single largest source of the 2–3x spread across DPLR/LAURA/US3D.

### Coupling 3: Electron energy balance (the Tₑ loop)

Free electrons have their own energy equation, tightly coupled both ways:

- Electrons gain energy from **elastic collisions** with hot heavy particles (slow, because of the huge mass ratio — energy transfer per collision ∝ mₑ/M).
- Electrons gain/lose energy through **inelastic** channels: electron-impact vibrational excitation (e⁻ + N₂ is a very efficient V-T bypass), electronic excitation, and especially electron-impact ionization.
- **Electron-impact ionization** consumes electron thermal energy and multiplies electrons (avalanche): e⁻ + N ⇌ N⁺ + 2e⁻. This makes Tₑ both a driver and a product of ionization.

Ionization rate depends on Tₑ, ionization consumes Tₑ, and electron production changes the electron heat capacity — a stiff self-referential loop with threshold character. If Tₑ climbs enough, electron-impact ionization takes over from associative ionization and electron density can run away.

### Coupling 4: Electron-impact vibrational coupling (the sneaky one)

Electrons are extraordinarily efficient at exciting N₂ vibration through a temporary N₂⁻ resonance:

- Free electrons pump vibrational energy far faster than heavy-particle T–V transfer would.
- That raises Tᵥ, which raises the dissociation rate (Coupling 1), which changes the atom pool, which changes associative ionization (the electron source).

So electron density feeds back on its own precursor chemistry through the vibrational bath. A small change in early ionization can accelerate vibrational excitation, accelerate dissociation, and change the whole downstream profile. Often modeled crudely or lumped, and it matters.

### Coupling 5: Radiation coupling

Minor energy term at RAM-C velocities but a real chemistry perturbation; at higher (lunar/Mars-return) speeds it becomes a major loop:

- Electronic states are populated by collisions and **depopulated by radiative emission**, so the electronic-state distribution is not purely collisional (non-Boltzmann electronic states).
- Radiation can escape (optically thin) or be reabsorbed (optically thick), so the local energy balance depends on the whole flowfield's radiative transport — a **nonlocal coupling**.
- Photoionization and radiative recombination (e⁻ + NO⁺ → N + O + hν, and inverse) are direct electron source/sink terms tied to the radiation field.

Radiation–flowfield coupling is nonlocal, which breaks the otherwise-local structure of the chemistry and makes the problem harder to solve and validate.

### Coupling 6: Ambipolar diffusion and charge separation

The plasma is quasi-neutral but the species diffuse:

- Electrons want to diffuse much faster than ions, but charge separation sets up an electric field that drags ions along and holds electrons back — **ambipolar diffusion**.
- This transports electrons and ions across the boundary layer and shock layer, coupling *local* production chemistry to *spatial* transport. Peak electron density and its location depend on this.
- Different ion species have different mobilities, so multicomponent ambipolar diffusion couples all charged species' spatial distributions together.

This is where the chemistry stops being a box model and becomes a coupled reacting-transport PDE.

### Coupling 7: Surface / wall coupling

The wall is not passive:

- **Catalytic recombination** at the surface (N + N → N₂, O + O → O₂, and ion neutralization) removes atoms and charges near the wall, changing near-wall electron density and heat flux. Catalycity ranges from fully catalytic to inert, and the real value is uncertain — another factor-of-2 knob.
- Wall temperature sets the near-wall thermodynamic state and thus local rates.

### Coupling 8: Ablation coupling

RAM-C partially avoided this (beryllium/teflon configuration to keep the air chemistry clean), but it is central for most real vehicles:

- Ablation injects carbon, hydrogen, and other species with **low ionization potentials**. Trace ablation products (e.g. alkali contaminants, carbon-bearing radicals) can **dominate** electron density even at small mass fractions.
- Adds an entire second chemistry network (CN, CO, C₂, hydrocarbons) coupled to the air network and to the surface mass-loss rate, which itself depends on heat flux, which depends on the chemistry.

### How it closes into the hard loop

Trace one cycle:

> shock heats translation → T–V transfer raises Tᵥ → high-Tᵥ molecules dissociate (VDC) → free N, O atoms appear → associative ionization makes NO⁺ + e⁻ → free electrons pump N₂ vibration via the resonance (electron-impact V excitation) → *raises Tᵥ again* → more dissociation → more atoms → more electrons → electron-impact ionization avalanche once Tₑ is high enough → all while ambipolar diffusion moves the charges spatially, radiation bleeds/adds energy nonlocally, and the wall eats atoms and charges catalytically.

Every arrow is a rate coefficient with real uncertainty, evaluated at a temperature whose very definition is a modeling choice, over state distributions that are not Boltzmann. There is no clean separation of timescales to exploit — the vibrational relaxation time, dissociation time, ionization time, and flow residence time are all comparable in the RAM-C regime (the definition of thermochemical nonequilibrium). When timescales are comparable, no sub-process can be approximated as instantaneous or frozen, so all couplings stay live simultaneously.

That is why the honest deviation is 2–3x: not one uncertain number, but a multiplicatively-compounding network of uncertain rates and modeling choices, closed into feedback loops, with no clean regime to isolate any single link — validated against a handful of flights.

---

## Part 2 — Where DPLR / LAURA / US3D Diverge Most

The three codes solve the same governing equations; the spread comes from **closure choices**, not numerics. Biggest divergences, in rough order of impact on predicted electron density:

**1. The controlling-temperature closure for reaction rates.**
All three descend from Park's two-temperature framework, but they do not all use the same controlling temperature for the same reactions, and the exponents/averaging differ. Dissociation is typically governed by Tₐ = T^q · Tᵥ^(1−q), and the choice of q (Park's classic 0.7, or 0.5 for the geometric mean, or other values) directly scales the dissociation rate — which sets the atom pool feeding associative ionization. Because electron density sits at the end of that chain, a modest difference in q shows up amplified in e⁻. "DPLR vs LAURA" is often really "this input deck vs that deck."

**2. The rate coefficient set itself.**
Park 1985 vs Park 1990/93 vs Park 2001 vs newer ab-initio-informed sets (e.g. NASA Ames rates, or legacy Bortner / Dunn-Kang sets). LAURA has traditionally defaulted toward Dunn-Kang / Park variants; DPLR carries Park sets with Ames modifications; US3D users frequently plug in their own. The associative ionization rates (N+O→NO⁺+e⁻ and its reverse dissociative recombination) vary by ~2x across these sets, and that reaction is *the* electron source, so this is a first-order divider.

**3. The electron energy equation and Tₑ treatment.**
How each code handles free-electron energy — elastic e-heavy exchange, electron-impact ionization energy sink, e⁻-N₂ vibrational coupling, and whether Tₑ = Tᵥ (single vibrational-electronic temperature) or Tₑ is tracked separately — differs. The common simplification Tₑ = Tᵥ is convenient but wrong when electron-impact ionization is active. This directly governs whether the electron-impact avalanche turns on.

**4. Vibration–dissociation coupling model.**
Straight two-temperature (T·Tᵥ favoring) vs CVDV vs Marrone–Treanor with different U parameters. Reshapes which molecules dissociate and cools the vibrational bath differently, moving the atom pool again.

**5. Surface catalycity and diffusion model.**
Fully-catalytic vs finite-rate wall, and multicomponent vs approximate (e.g. constant-Schmidt-number, SCEBD) diffusion. Matters more for near-wall electron density and heat flux than for the peak, but a real code-to-code difference — LAURA and DPLR handle wall catalysis and ambipolar diffusion with different defaults.

**Through-line:** the divergence is concentrated in the chemistry-thermodynamics closure, and within that, in the handful of parameters that set the atom pool and the electron source/sink. Two of these codes with matched rate sets, matched controlling-temperature exponents, and matched electron-energy treatment converge much more tightly than their defaults suggest — the numerics are not the problem.

---

## Part 3 — Top Three Highest-Leverage Points

Ranked by leverage per unit effort for closing the 2–3x gap:

### Lever 1 — Nail the associative ionization / dissociative recombination rate pair (N + O ⇌ NO⁺ + e⁻)

This single reaction pair is the dominant electron source/sink in air at RAM-C enthalpies, and its rate is uncertain by ~2x, which nearly *is* the observed deviation. Everything downstream inherits this uncertainty multiplicatively, and nothing upstream can compensate for getting it wrong. Leverage: one reaction (tractable) that gates the entire quantity of interest. Attack with modern ab initio potential surfaces + quantum/quasi-classical scattering to get a temperature-dependent (ideally state-resolved) rate, validated against shock-tube electron-onset data. Highest ratio of "impact on e⁻" to "number of quantities you must pin down."

### Lever 2 — Replace the controlling-temperature closure with a state-to-state (or state-informed) dissociation model for N₂ and O₂

The controlling-temperature trick is the largest *modeling* (as opposed to *parameter*) uncertainty, and it is the thing the three codes most disagree on. Moving from "pick an exponent q" to a rovibrational state-to-state treatment removes the free parameter entirely and captures the real vibration-dissociation coupling and electron-impact vibrational pumping in one framework. Expensive (thousands of state-resolved rates, stiff systems), which is why it is not the default — but it is where the accuracy *ceiling* lifts rather than just shifts. A reduced/binned (coarse-grained) state-to-state model captures most of the benefit at a fraction of the cost — the pragmatic version of this lever.

### Lever 3 — Track a separate free-electron temperature Tₑ with a proper electron energy budget (drop Tₑ = Tᵥ)

Decisive for whether electron-impact ionization avalanches, and cheap relative to Lever 2 — one extra energy equation plus a set of electron-collision source terms. Getting Tₑ right (elastic exchange + the e⁻-N₂ resonance V-excitation + the ionization energy sink) determines the high-density tail of the electron profile that the reflectometers actually measured. Codes that lump Tₑ into Tᵥ systematically misplace the ionization onset. High leverage precisely because it is a structural fix at modest cost.

### Why these three and not the others

Radiation, ablation, and wall catalysis are real couplings, but for the *air, RAM-C-class* problem they are second-order to the atom-pool → associative-ionization → electron-energy chain. Levers 1–3 sit exactly on that chain: one fixes the dominant source rate (parameter), one fixes the dominant modeling approximation feeding it (dissociation closure), and one fixes the feedback loop that sets the peak (electron energy). Pull those three and you have addressed the multiplicative core; the rest are refinements on an already-converged answer.

### Caveat

Levers 1 and 2 are validated against the same sparse shock-tube and flight data, so there is residual circularity — you can reduce the *modeling* uncertainty faster than you can reduce the *ground-truth* uncertainty. Closing the last bit ultimately needs a new clean flight or a genuinely contamination-free high-enthalpy air experiment. That is the resource constraint, not the physics constraint.