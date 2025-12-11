# Feature Specification: Regge Calculus Curvature

**Target Crate:** `deep_causality_topology`
**Target Struct:** `ReggeGeometry`
**File:** `src/types/regge_geometry/mod.rs` (or `curvature.rs` extension)

## 1. Objective
Implement `calculate_ricci_curvature` to compute the spacetime curvature (deficit angles) on a simplicial complex with defined edge lengths. This enables the simulation of dynamic geometry (General Relativity) on a discrete mesh.

## 2. Theoretical Background (Regge Calculus)
In Discrete General Relativity (Regge Calculus), space is approximated by a collection of flat $n$-simplices (triangles in 2D, tetrahedra in 3D). Curvature is not smooth; it is concentrated at the "hinges" or "bones" of the mesh.

*   **Bones ($h$):** The $(n-2)$-simplices where curvature resides.
    *   In 2D (Triangulation): Bones are Vertices (0-simplices).
    *   In 3D (Tetrahedralization): Bones are Edges (1-simplices).
    *   In 4D: Bones are Triangles (2-simplices).
*   **Deficit Angle ($\delta_h$):** The measure of curvature at a bone $h$.
    $$ \delta_h = 2\pi - \sum_{s \supset h} \theta_s(h) $$
    Where $\theta_s(h)$ is the dihedral angle of the simplex $s$ at the bone $h$.
    *   $\delta_h > 0$: Positive curvature (spherical).
    *   $\delta_h < 0$: Negative curvature (hyperbolic).
    *   $\delta_h = 0$: Flat space.

## 3. API Specification

```rust
impl ReggeGeometry {
    /// Calculates the Ricci Curvature (Deficit Angles) for all bones in the complex.
    ///
    /// The resulting tensor contains the deficit angle $\delta$ for each $(n-2)$-simplex.
    ///
    /// # Arguments
    /// * `complex` - The simplicial complex defining the topology.
    ///
    /// # Returns
    /// * `Result<CausalTensor<f64>, TopologyError>` - Tensor of curvature values.
    ///   - Rank: 1
    ///   - Dimension: Number of $(n-2)$-simplices (bones).
    ///   - Index: Corresponds to the index of the bone in `complex.skeletons[n-2]`.
    pub fn calculate_ricci_curvature(
        &self, 
        complex: &SimplicialComplex
    ) -> Result<CausalTensor<f64>, TopologyError> { ... }
}
```

## 4. Implementation Logic

### Phase 1: Context Identification
1.  **Determine Dimension ($D$):** Use `complex.dim()`.
2.  **Identify Bone Dimension:** $k = D - 2$.
    *   If $D < 2$, curvature is undefined (return 0 or Error).
    *   For 3D space, $k=1$ (Edges).
    *   For 4D spacetime, $k=2$ (Triangles).

### Phase 2: Geometry Per Simplex
Iterate through all $D$-simplices to compute local dihedral angles.

1.  **Retrieve $D$-simplices:** `complex.get_simplices(D)`.
2.  **For each Simplex $S$:**
    *   Get all edge lengths from `self.edge_lengths`.
    *   **Compute Dihedral Angles:**
        *   For each bone $b \subset S$ (faces of the simplex).
        *   Use the Cayley-Menger determinant or standard formulas:
            *   **3D (Tetrahedron):** Angle between two faces meeting at edge $l$.
                $$ \cos \theta = \frac{\nabla A_1 \cdot \nabla A_2}{|A_1| |A_2|} $$
                Or using edge lengths:
                $$ \sin \theta_l = \frac{3! V l}{2 A_1 A_2} $$
                Where $V$ is volume, $A_1, A_2$ are areas of faces adjacent to edge $l$.

### Phase 3: Aggregation (The Deficit)
1.  Initialize a `curvature_map` (Vector or Hash) keyed by Bone Index.
2.  Sum the computed $\theta_s(h)$ for each bone.
3.  **Compute Deficit:** $\delta_h = 2\pi - \sum \theta(h)$.
4.  Convert to `CausalTensor`.

## 5. Robustness & Edge Cases

A "Production Grade" implementation must handle real-world simulation artifacts, including degenerate geometry, boundaries, and floating-point errors.

### 5.1. Formula Validity (Metric Integrity)
Before invoking any trigonometric functions (e.g., `acos`), the implementation MUST validate the underlying geometry.

*   **Requirement:** For every simplex used in calculation, verify the **Generalized Triangle Inequalities**.
    *   In 2D (Triangle with edges $a, b, c$): $|a - b| \le c \le a + b$.
    *   In $n$-D: Use the determinative of the Cayley-Menger matrix. $vol^2 > 0$.
*   **Edge Case:** If a simplex has zero volume (degenerate) or impossible edge lengths (e.g., $1, 1, 10$):
    *   **Action:** Return `Err(TopologyError::InvalidMetric("Simplex X violates triangle inequality"))`.
    *   **Rationale:** Simulating physics on impossible geometry yields NaN/Infinity, which crashes long-running jobs (CERN/LISA). Fail fast.

### 5.2. Boundary Handling
The formula $\delta_h = 2\pi - \sum \theta$ assumes the bone $h$ is strictly **internal** to the manifold.

*   **Problem:** On a boundary, the sum of angles is naturally $<\pi$ or $<2\pi$, resulting in a massive "fake" positive curvature.
*   **Requirement:**
    1.  **Detect Boundaries:** A bone $h$ is on the boundary if any of its incident $(n-1)$-faces (wings) belongs to **only one** $n$-simplex.
    2.  **Policy:** For this implementation, **Ignore Boundary Curvature**.
        *   If `is_boundary(h)`: Set $\delta_h = 0.0$.
    *   **Rationale:** Prevents "Curvature Noise" at the simulation edges from dominating the dynamics. (Advanced users can request specific Boundary Conditions in defined specialized methods).

### 5.3. Performance & Complexity
The naive approach of iterating `Complex -> Simplices -> Bones` has $O(N_{bones} \times N_{simplices})$ complexity for lookups, which is $O(N^2)$. This is unacceptable for high-resolution meshes (LISA/Hydro).

*   **Requirement:** Use an **Inverse Look-up Cache** (Adjacency Map).
    *   **Step 1:** Pre-compute a map `HashMap<BoneIndex, Vec<SimplexIndex>>`.
        *   Iterate all $n$-simplices once ($O(N)$).
        *   For each bone in simplex, append simplex ID to the map.
    *   **Step 2:** Iterate Bones ($O(K)$).
        *   Look up incident simplices in $O(1)$.
        *   Compute Deficit.
    *   **Total Complexity:** $O(N)$.
*   **Memory Trade-off:** Requires temporary allocation of the incidence map, but ensures scale-invariant runtime.

### 5.4. Numerical Stability
*   **Clamp Input:** When computing `acos(x)`, verify $x \in [-1.0, 1.0]$.
    *   Due to accumulation errors, $x$ might be $1.0000000000004$.
    *   **Action:** Clamp strictly before calling `acos`.
*   **Singularities:** Check for divide-by-zero when normalizing vectors for angle calculation.
    *   **Action:** If denominator is close to zero, return `Err(Singularity)`.

## 6. Dependencies
*   `deep_causality_tensor` for output.
*   `deep_causality_multivector` (optional) if generalized volume/angle calculation is used.
*   New helper: `simplex_volume(edge_lengths)` and `dihedral_angle(edge_lengths)`.

## 7. Example Usage

```
let complex = SimplicialComplex::new_tetrahedron(); // 3D
let geometry = ReggeGeometry::new(lengths_tensor);

let curvature = geometry.calculate_ricci_curvature(&complex)?;
// curvature[i] is the deficit angle at Edge i.
println!("Curvature at Edge 0: {}", curvature.get(&[0]));
```
