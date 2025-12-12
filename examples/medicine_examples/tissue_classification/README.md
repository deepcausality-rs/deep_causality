# MRI Tissue Classification via Topological Data Analysis

This example demonstrates using Topological Data Analysis (TDA) to classify tissue samples as healthy or pathological based on topological features.

## How to Run

```bash
cargo run -p medicine_examples --example mri_tissue_classification
```

---

## Engineering Value

In medical imaging (MRI/CT), distinguishing between healthy tissue and pathological structures is critical:

- **Tumor Detection**: Some tumors have similar density to healthy tissue but distinct *structure*
- **Necrotic Core Identification**: Dead tissue forms "holes" inside tumors
- **Robust Classification**: Topology is invariant to noise and deformation

### Why Topology?

Traditional methods use pixel intensity (density). TDA adds structural analysis:

| Feature | Intensity-Based | Topology-Based |
|---------|-----------------|----------------|
| Solid tumor | High density | χ = 1 (no holes) |
| Necrotic tumor | Similar density | χ ≤ 0 (has holes) |
| Healthy tissue | Normal density | χ = 1 (contractible) |

---

## Key Concept: Euler Characteristic

The Euler Characteristic (χ) counts topological features:

```
χ = V - E + F = (connected components) - (holes)
```

| Topology | χ Value | Meaning |
|----------|---------|---------|
| Solid disk | 1 | Healthy tissue (contractible) |
| Ring/Torus | 0 | Has a hole (necrotic core) |
| Two disks | 2 | Disconnected (fragmented) |

---

## Causal Pipeline

```text
[Step 1] Ingest Points     → Load 3D point cloud from simulated MRI
            ↓
[Step 2] Triangulate       → Build Vietoris-Rips complex (radius=0.6)
            ↓
[Step 3] Compute χ         → Calculate Euler Characteristic
            ↓
[Step 4] Diagnose          → Classify based on topology
```

Each step is composed monadically via `PropagatingEffect::bind()`.

---

## Output Interpretation

```
Sample A (Healthy):  χ = 1  → HEALTHY (no holes)
Sample B (Tumor):    χ = 0  → PATHOLOGICAL (has hole/void)
```

---

## Adapting This Example

1. **Real MRI Data**: Replace simulated points with actual scan data
2. **Different radii**: Tune triangulation radius for your data scale
3. **3D Visualization**: Export simplicial complex for rendering
4. **Persistent Homology**: Use Betti numbers for richer topological features
5. **Multi-scale Analysis**: Sweep radius to find optimal classification

---

## Key APIs Used

- `PropagatingEffect::bind()` - Monadic composition of analysis steps
- `PointCloud::triangulate()` - Vietoris-Rips complex construction
- `BaseTopology::num_elements_at_grade()` - Count simplices at each dimension
- `CausalTensor` - Store point cloud coordinates
