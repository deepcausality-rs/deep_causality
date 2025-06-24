use crate::prelude::AdjustableTangentSpacetime;

impl AdjustableTangentSpacetime {
    /// Returns position as [t, x, y, z]
    pub fn position(&self) -> [f64; 4] {
        [self.t, self.x, self.y, self.z]
    }

    /// Returns velocity as [dt, dx, dy, dz]
    pub fn velocity(&self) -> [f64; 4] {
        [self.dt, self.dx, self.dy, self.dz]
    }

    /// Returns the coordinate-time velocity (∂t/∂τ)
    pub fn time_velocity(&self) -> f64 {
        self.dt
    }

    /// Computes spatial velocity magnitude (ignoring dt)
    pub fn spatial_velocity(&self) -> f64 {
        (self.dx.powi(2) + self.dy.powi(2) + self.dz.powi(2)).sqrt()
    }

    /// Returns 3D velocity vector
    pub fn velocity_vector(&self) -> [f64; 3] {
        [self.dx, self.dy, self.dz]
    }

    /// Computes Euclidean spatial distance to another point
    pub fn euclidean_distance(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt()
    }
}
