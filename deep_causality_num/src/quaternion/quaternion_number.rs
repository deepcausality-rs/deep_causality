use crate::float::Float;
use crate::quaternion::Quaternion;

impl<F> Quaternion<F>
where
    F: Float,
{
    pub fn conjugate(&self) -> Self {
        Quaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub fn norm_sqr(&self) -> F {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn norm(&self) -> F {
        self.norm_sqr().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n.is_zero() { *self } else { *self / n }
    }

    pub fn inverse(&self) -> Self {
        let n_sqr = self.norm_sqr();
        if n_sqr.is_zero() {
            Quaternion::new(F::nan(), F::nan(), F::nan(), F::nan())
        } else {
            self.conjugate() / n_sqr
        }
    }

    pub fn dot(&self, other: &Self) -> F {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }
}
