# Electroweak Unification Example

## What This Simulation Does

This example shows how **electricity and the weak nuclear force are unified**. This is one of the biggest discoveries in
physics: the forces we thought were separate are actually unified.

Think of it like discovering that ice, water, and steam are all H₂O. At everyday energies, the electromagnetic force (
light, electricity) and the weak force (radioactive decay) look completely different. But at extremely high energies —
like those in particle accelerators or the early universe — they merge into one "electroweak" force.


## Running

```bash
cargo run --example gauge_electroweak -p physics_examples --release
```

## The Four Stages Explained

### Stage 1: Unification (Coupling Constants)

**What it does:** Calculates how strongly particles interact with the electromagnetic and weak forces.

**In plain terms:** Every force has a "strength number" that tells us how strongly it pulls or pushes. This stage
figures out those numbers (called coupling constants) and shows they're all related through a special angle called the
Weinberg angle. This proves the forces are secretly connected.

---

### Stage 2: Spontaneous Symmetry Breaking (Higgs)

**What it does:** Shows how the W and Z bosons (carriers of the weak force) get their mass.

**In plain terms:** In the early universe, all force-carrying particles were massless and moved at the speed of light.
Then the Higgs field "turned on" and gave mass to some particles. This stage calculates exactly how much mass the W
boson (~80 GeV) and Z boson (~91 GeV) received. The photon (carrier of electromagnetism) stayed massless, which is why
light still travels at... the speed of light.

---

### Stage 3: Gauge Boson Mixing

**What it does:** Verifies the mathematical relationships between particle masses.

**In plain terms:** Once we know the masses, we can check if they obey the predicted formulas. The "ρ parameter" should
equal exactly 1.0 if our theory is correct at the basic level. Any deviation tells us about more complex physics
happening behind the scenes.

---

### Stage 4: Z Resonance (Cross-Section)

**What it does:** Calculates how likely it is to create a Z boson in a particle collider.

**In plain terms:** When you smash electrons and positrons together at exactly the right energy (~91 GeV), you hit a "
sweet spot" where Z bosons are created in huge numbers. This is like tuning a radio to exactly the right frequency.
The "cross-section" (~41 nanobarns) tells us how big the target is — larger means more collisions produce Z bosons. This
was directly measured at CERN's LEP collider and matches our calculation.

## Precision Results

The simulation amatches experimental data from CERN's LEP collider within 1%.

| Metric                       | Simulation    | PDG / Experimental | Match    |
|------------------------------|---------------|--------------------|----------|
| **W Mass ($M_W$)**           | **80.25 GeV** | 80.38 GeV          | **>99%** |
| **Z Mass ($M_Z$)**           | **91.52 GeV** | 91.19 GeV          | **>99%** |
| **Total Width ($\Gamma_Z$)** | **2.513 GeV** | 2.495 GeV          | **>99%** |
| **Invisible Width**          | **0.503 GeV** | 0.499 GeV          | **>99%** |
| **Peak Cross-sec**           | **41.11 nb**  | ~41.5 nb           | **>99%** |

## Why This Matters

This simulation demonstrates that:

1. **Forces unify at high energy** — Electromagnetism and the weak force are two faces of the same coin
2. **The Higgs mechanism works** — We can correctly predict particle masses from first principles
3. **Theory matches experiment** — The calculated values match what CERN measures to within 1%

This unification was a major triumph of 20th-century physics and earned Glashow, Weinberg, and Salam the Nobel Prize in
1979.
