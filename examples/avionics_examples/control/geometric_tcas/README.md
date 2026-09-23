# Geometric TCAS (Traffic Collision Avoidance)

## Avionics Background
This example detects a converging aircraft with Geometric Algebra, issues traffic and resolution advisories, and forces an automatic descent when the resolution advisory goes unanswered.

Traffic Alert and Collision Avoidance Systems (TCAS) are the last line of defense in aviation. With **NextGen** and **SESAR** airspace management and the rise of **Urban Air Mobility (UAM)**, drones flying high-dynamic 3D trajectories crowd the airspace.
Traditional TCAS computes "Tau" ($\tau = r / \dot{r}$), the time to closest point of approach. This scalar method works for linear, head-on encounters but degrades computationally and numerically in multi-agent 3D geometries and grazing encounters.

## The Challenge
The task is to compute the **Closest Point of Approach (CPA)** in 3D reliably and cheaply.
*   **Singularities**: Standard formulations break down when relative velocity $\dot{r} \to 0$ or when paths are parallel.
*   **Computational Cost**: Trig-heavy geometric calculations (matrices, Euler angles) consume cycles in embedded avionics.
*   **Ambiguity**: Detecting a collision course is easy; deciding *which way* to turn (Resolution Advisory) in 3D without inducing secondary conflicts is hard.

## The DeepCausality Solution
The example uses **Geometric Algebra (Clifford Algebra)** from the `deep_causality_multivector` crate for a coordinate-free solution:

### 1. The Power of the Bivector
Instead of analyzing raw positions, the example computes the **Moment Bivector** $M$ of the relative motion:
$$ M = P_{rel} \wedge V_{rel} $$
This single object (a 3-component bivector in 3D) encodes the "plane of collision" and the "angular momentum" of the encounter.

### 2. Robust CPA Formulation
The minimum pass distance (impact parameter) $d$ follows directly from the bivector magnitude, avoiding many trigonometric singularities:
$$ d_{min} = \frac{|P_{rel} \wedge V_{rel}|}{|V_{rel}|} $$
The calculation stays numerically stable for grazing encounters.

### 3. Causal Decision Logic (`PropagatingEffect`)
A `PropagatingEffect` bind-chain encapsulates the safety logic:
1.  **Monitor**: Compute Geometric CPA ($d$) and Time-to-CPA ($\tau$) every tick.
2.  **Evaluate**: If $d < 500\,m$ AND $0 \le \tau < 45s$, issue a Traffic Advisory (TA); below $\tau = 20s$, a Resolution Advisory (RA).
3.  **Resolve**: The RA carries a direction (`CLIMB` or `DESCEND`).
    *   *Implementation Note*: The example uses a vertical-preference heuristic (descend if the intruder is above, climb if below). GA would also allow the optimal avoidance vector: rotate $V_{rel}$ in the plane defined by $M$.

### 4. Automatic Intervention via `CausalFlow::branch`
Each 0.5 s tick of the safety loop is one `CausalFlow`: `assess -> intervene? -> output -> integrate`, and `iterate_n` runs 30 ticks. The auto-pilot takeover is a `branch` on the carried value, so the override step runs only when the **Closed Loop Safety Interlock** fires:

*   **Interlock**: A `DESCEND` RA that persists for > 2.5 seconds without pilot response (simulating hypoxia or incapacitation) sets the trigger.
*   **Intervention**: The `intervene` step rewrites the ownship velocity, steepening the descent by 5 m/s per tick down to -20 m/s, and records the override.
*   **Recovery**: `D_CPA` grows from a dangerous 50m to a safe 374m as the automation takes over.

The same TCAS logic applies to autonomous drones avoiding other drones that carry no active TCAS.

## No-Std Support

`deep_causality_core` and `deep_causality_multivector` both compile to a `no_std` environment through their `no-std` features. The example binary itself uses `std` for console output.

## Running the Example
```bash
cargo run -p avionics_examples --example geometric_tcas
```

## Breakdown of the Results
The simulation output breaks down as follows:

1.  **Kinematics (Closing Speed)**
    *   **Scenario**: Ownship is flying North at 200 m/s (~390 kts). Intruder is flying South at 200 m/s.
    *   **Physics**: This creates a **closing speed of 400 m/s** (Mach 1.2 encounter).
    *   **Math**: `Range (8000m) / Probability Closing Speed (400m/s) = 20.0 seconds`.
    *   **Output**: The table starts exactly at `20.0s` Time-to-CPA and counts down linearly.

2.  **Geometry (The 50m CPA)**
    *   **Scenario**: The aircraft are head-on but separated vertically by **50m** (Ownship at 10,000m, Intruder at 10,050m).
    *   **Physics**: Since their horizontal paths overlap exactly, the closest they will ever get is that vertical difference.
    *   **Output**: `D_CPA` stays at `50.0m`: the Geometric Algebra calculation identifies the "impact parameter" even miles away.

3.  **Auto-Intervention (Closed Loop)**
    *   At T=3.0s, the **Automatic Recovery** engages (`[AUTO INTERVENE]`).
    *   The aircraft dives (-20 m/s).
    *   Result: `D_CPA` increases to **374.5m**, and the collision is avoided.
