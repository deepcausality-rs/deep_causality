# Geometric TCAS (Traffic Collision Avoidance)

## Avionics Background
Traffic Alert and Collision Avoidance Systems (TCAS) are the last line of defense in aviation. As the industry moves toward **NextGen** and **SESAR** airspace management, and with the rise of **Urban Air Mobility (UAM)**, the airspace is becoming crowded with drones flying complex, high-dynamic 3D trajectories.
Traditional TCAS relies on calculating "Tau" ($\tau = r / \dot{r}$), the time to closest point of approach. This scalar-based method works for linear, head-on encounters but degrades computationally and numerically when dealing with complex multi-agent 3D geometries or grazing encounters.

## The Challenge
The core engineering challenge is to robustly and efficiently calculate the **Closest Point of Approach (CPA)** in 3D.
*   **Singularities**: Standard math struggles when relative velocity $\dot{r} \to 0$ or when paths are parallel.
*   **Computational Cost**: Trig-heavy geometric calculations (matrices, Euler angles) consume valuable cycles in embedded avionics.
*   **Ambiguity**: Knowing *that* you will collide is easy; deciding *which way* to turn (Resolution Advisory) in 3D without inducing secondary conflicts is hard.

## The DeepCausality Solution
This example leverages **Geometric Algebra (Clifford Algebra)** via the `deep_causality_multivector` crate to provide a "coordinate-free" solution:

### 1. The Power of the Bivector
Instead of analyzing raw positions, we compute the **Moment Bivector** $M$ of the relative motion:
$$ M = P_{rel} \wedge V_{rel} $$
This single object (a 3-component bivector in 3D) encodes the entire "plane of collision" and the "angular momentum" of the encounter.

### 2. Robust CPA Formulation
The minimum pass distance (impact parameter) $d$ is derived directly from the bivector magnitude, avoiding many trigonometric singularities:
$$ d_{min} = \frac{|P_{rel} \wedge V_{rel}|}{|V_{rel}|} $$
This calculation is numerically stable even for grazing encounters.

### 3. Causal Decision Logic (`PropagatingEffect`)
The safety logic is encapsulated in a causal workflow:
1.  **Monitor**: Continuously computes Geometric CPA ($d$) and Time-to-CPA ($\tau$).
2.  **Evaluate**: If $d < \text{Threshold}$ AND $0 < \tau < 60s$, a causal link triggers.
3.  **Resolve**: The system propagates a **Resolution Advisory (RA)** (e.g., `CLIMB`, `DESCEND`).
    *   *Implementation Note*: The example uses a simplified heuristic (vertical preference), but GA allows calculating the optimal avoidance vector by simply rotating $V_{rel}$ in the plane defined by $M$.

### 4. Automatic Intervention via `Intervenable`
The system utilizes the **`Intervenable` trait**, a core mechanism of the `deep_causality_core` library (based on Pearl's Causal Hierarchy, specifically Layer 2: Intervention), to implement a **Closed Loop Safety Interlock**:

*   **Universal Mechanism**: The intervention logic is not a hacked "if-statement" but a formal operation on the causal chain. `effect.intervene(new_value)` produces a distinct causal history, separating "what naturally happened" from "what was forced to happen".
*   **Safety-Critical Implications**: This allows for a **Dual-Path Architecture**:
    1.  **Prediction Path**: Calculate the collision course (Factual).
    2.  **Intervention Path**: Calculate the avoidance maneuver (Counterfactual).
    3.  **Execution**: The system can seamlessly switch to the intervention path if the factual path leads to a catastrophic state (e.g., pilot non-response).
*   **Scenario**: 
    - If a detected collision persists for > 2.5 seconds without pilot response (simulating hypoxia/incapacitation).
    - The system **Intervenes**: Effectively rewriting the velocity vector in the causal graph.
    - **Recovery**: The example shows `D_CPA` increasing from a dangerous 50m to a safe 374m as the automation takes over.


The same TCAS system can be adapted for autonomous drones to prevent in-flight collisions with other drones that may not have an active TCAS system. 

## No-Std Support

The underlying core crate already compiles to a `no_std` environment. However, the `multivector` crate would need some refactoring to become `no_std` compatible. This is possible but requires targeted effort for a future update.

## Running the Example
```bash
cargo run -p avionics_examples --example geometric_tcas
```

## Breakdown of the Results
The simulation output demonstrates highly realistic physics and logic:

1.  **Kinematics (Closing Speed)**
    *   **Scenario**: Ownship is flying North at 200 m/s (~390 kts). Intruder is flying South at 200 m/s.
    *   **Physics**: This creates a **closing speed of 400 m/s** (Mach 1.2 encounter).
    *   **Math**: `Range (8000m) / Probability Closing Speed (400m/s) = 20.0 seconds`.
    *   **Output**: The table starts exactly at `20.0s` Time-to-CPA and counts down linearly.

2.  **Geometry (The 50m CPA)**
    *   **Scenario**: The aircraft are head-on but separated vertically by **50m** (Ownship at 10,000m, Intruder at 10,050m).
    *   **Physics**: Since their horizontal paths overlap exactly, the closest they will ever get is that vertical difference.
    *   **Output**: `D_CPA` stays locked at `50.0m`. This confirms the Geometric Algebra calculation is correctly identifying the "impact parameter" even miles away.

3.  **Auto-Intervention (Closed Loop)**
    *   At T=3.0s, the **Automatic Recovery** kicks in (`[AUTO INTERVENE]`).
    *   The aircraft dives (-20 m/s).
    *   Result: `D_CPA` rapidly increases to **374.5m**, demonstrating a successful collision avoidance maneuver.
