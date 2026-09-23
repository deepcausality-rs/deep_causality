# Tissue classification by topology

This example tells a solid tumour from one with a necrotic core by counting holes. A tumour that
outgrows its blood supply dies from the inside out and leaves a shell of living cells around a
dead centre. On an MRI slice that shell reads as a ring, and a ring instead of a solid mass marks
the tumour as aggressive.

```bash
cargo run -p medicine_examples --example tissue_classification
```

## What the run prints

Two samples of twenty-four voxel centres each, a filled disc and a ring, triangulated the same way
and read the same way.

```text
Sample A, solid mass
  complex             24 vertices, 55 edges, 34 triangles, 2 tetrahedra
  Euler characteristic  24 - 55 + 34 - 2 = 1
  neighbour count     3 to 7   (varies, the sample has an interior)
  reading             solid, the complex fills in with no void

Sample B, ring
  complex             24 vertices, 48 edges, 24 triangles, 0 tetrahedra
  Euler characteristic  24 - 48 + 24 - 0 = 0
  neighbour count     4 to 4   (uniform, the sample is all rim)
  reading             a void is enclosed, consistent with a necrotic core
```

## The mathematics

**The Vietoris-Rips complex.** Every pair of voxels closer than the radius `0.62` is joined by an
edge, every three mutually joined voxels close a triangle, and every four close a tetrahedron. The
point cloud becomes a simplicial complex, and the complex carries the sample's connectivity.

**The Euler characteristic.** `χ = V − E + F − T` is the alternating sum of the cell counts by
grade. It counts a shape's connected pieces against the holes in them, and it holds under any
deformation that leaves the connectivity alone.

| Shape | χ | Reading |
|---|---|---|
| filled disc | 1 | solid, the complex fills in |
| ring | 0 | one void enclosed, a necrotic core |
| two separate pieces | 2 | the sample is too sparse to read |

**The local density.** Each voxel counts its neighbours within the same radius. A disc has an
interior and a rim, so the counts span a range; a ring is rim everywhere, so they sit at one value.
The range corroborates the Euler reading from a different direction.

## What the code demonstrates

| Operation | Acts on | For |
|---|---|---|
| `PointCloud::triangulate` | the voxel centres | the Vietoris-Rips complex |
| `BaseTopology::euler_characteristic` | the complex | the full alternating sum over every grade |
| `extend` | the cloud, focused on one voxel at a time | the neighbour count at each voxel |
| `fold` | the density map | the range those counts span |

`extend` hands its closure the cloud focused on one voxel, so the closure reads that voxel's
coordinates and the whole cloud together. `fold` reduces the map it returns to its lowest and
highest count.

## Precision is a parameter

One alias in `main.rs` sets the working scalar for the whole program.

```rust
pub type FloatType = Float106;
```

The coordinates, the distances and the density map recompute at that scalar. `f32`, `f64`,
`BFloat16` and `Float106` all return χ = 1 for the disc and χ = 0 for the ring: the readings are
integers, and a radius test with the voxels this far from the threshold resolves the same way at
every precision.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias and the classification of both samples |
| `model.rs` | the sample geometries, the complex, and the two topological readings |
| `utils_print.rs` | the presentation, and the only `lower` calls in the example |

## Adaptation

- **Real MRI data.** Replace the two generated samples with voxel centres from a scan; the radius
  scales with the voxel pitch.
- **A radius sweep.** Run the classification across radii to find the scale at which the two
  readings separate most cleanly.
- **Betti numbers.** Persistent homology names which holes the Euler characteristic counts, at
  which scales they appear, and at which they close.
