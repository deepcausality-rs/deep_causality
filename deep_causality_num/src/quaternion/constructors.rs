use crate::alias::Vector3;
use crate::float::Float;
use crate::quaternion::Quaternion;

impl<F> Quaternion<F>
where
    F: Float,
{
    pub fn new(w: F, x: F, y: F, z: F) -> Self {
        Quaternion { w, x, y, z }
    }

    pub fn identity() -> Self {
        Quaternion {
            w: F::one(),
            x: F::zero(),
            y: F::zero(),
            z: F::zero(),
        }
    }

    pub fn from_axis_angle(axis: Vector3<F>, angle: F) -> Self {
        let half_angle = angle / (F::one() + F::one());
        let s = half_angle.sin();
        let c = half_angle.cos();
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if len.is_zero() {
            return Quaternion::identity();
        }
        let inv_len = F::one() / len;
        Quaternion {
            w: c,
            x: axis[0] * inv_len * s,
            y: axis[1] * inv_len * s,
            z: axis[2] * inv_len * s,
        }
    }

    pub fn from_euler_angles(roll: F, pitch: F, yaw: F) -> Self {
        let cy = (yaw * F::from(0.5).unwrap()).cos();
        let sy = (yaw * F::from(0.5).unwrap()).sin();
        let cp = (pitch * F::from(0.5).unwrap()).cos();
        let sp = (pitch * F::from(0.5).unwrap()).sin();
        let cr = (roll * F::from(0.5).unwrap()).cos();
        let sr = (roll * F::from(0.5).unwrap()).sin();

        Quaternion {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }
}
