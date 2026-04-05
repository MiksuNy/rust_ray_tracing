use std::ops::Mul;

use crate::math::vec::*;
use crate::math::vec3::*;

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
pub struct Mat4f {
    pub data: [[f32; 4]; 4],
}

#[allow(dead_code)]
impl Mat4f {
    pub fn new() -> Self {
        return Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
    }

    pub fn look_at(from: Vec3f, to: Vec3f, up: Vec3f) -> Self {
        let forward = (from - to).normalized();
        let right = Vec3f::cross(up, forward).normalized();
        let up = Vec3f::cross(forward, right);

        let mut m = Self::new();
        m.data[0][0] = right.x();
        m.data[0][1] = right.y();
        m.data[0][2] = right.z();
        m.data[1][0] = up.x();
        m.data[1][1] = up.y();
        m.data[1][2] = up.z();
        m.data[2][0] = forward.x();
        m.data[2][1] = forward.y();
        m.data[2][2] = forward.z();
        m.data[3][0] = from.x();
        m.data[3][1] = from.y();
        m.data[3][2] = from.z();
        return m;
    }

    // https://webgpufundamentals.org/webgpu/lessons/webgpu-cameras.html
    pub fn inverse(m: Self) -> Self {
        let mut dst = Mat4f::default();

        let m00 = m.data[0][0];
        let m01 = m.data[0][1];
        let m02 = m.data[0][2];
        let m03 = m.data[0][3];
        let m10 = m.data[1][0];
        let m11 = m.data[1][1];
        let m12 = m.data[1][2];
        let m13 = m.data[1][3];
        let m20 = m.data[2][0];
        let m21 = m.data[2][1];
        let m22 = m.data[2][2];
        let m23 = m.data[2][3];
        let m30 = m.data[3][0];
        let m31 = m.data[3][1];
        let m32 = m.data[3][2];
        let m33 = m.data[3][3];

        let tmp0 = m22 * m33;
        let tmp1 = m32 * m23;
        let tmp2 = m12 * m33;
        let tmp3 = m32 * m13;
        let tmp4 = m12 * m23;
        let tmp5 = m22 * m13;
        let tmp6 = m02 * m33;
        let tmp7 = m32 * m03;
        let tmp8 = m02 * m23;
        let tmp9 = m22 * m03;
        let tmp10 = m02 * m13;
        let tmp11 = m12 * m03;
        let tmp12 = m20 * m31;
        let tmp13 = m30 * m21;
        let tmp14 = m10 * m31;
        let tmp15 = m30 * m11;
        let tmp16 = m10 * m21;
        let tmp17 = m20 * m11;
        let tmp18 = m00 * m31;
        let tmp19 = m30 * m01;
        let tmp20 = m00 * m21;
        let tmp21 = m20 * m01;
        let tmp22 = m00 * m11;
        let tmp23 = m10 * m01;

        let t0 = (tmp0 * m11 + tmp3 * m21 + tmp4 * m31) - (tmp1 * m11 + tmp2 * m21 + tmp5 * m31);
        let t1 = (tmp1 * m01 + tmp6 * m21 + tmp9 * m31) - (tmp0 * m01 + tmp7 * m21 + tmp8 * m31);
        let t2 = (tmp2 * m01 + tmp7 * m11 + tmp10 * m31) - (tmp3 * m01 + tmp6 * m11 + tmp11 * m31);
        let t3 = (tmp5 * m01 + tmp8 * m11 + tmp11 * m21) - (tmp4 * m01 + tmp9 * m11 + tmp10 * m21);

        let d = 1.0 / (m00 * t0 + m10 * t1 + m20 * t2 + m30 * t3);

        dst.data[0][0] = d * t0;
        dst.data[0][1] = d * t1;
        dst.data[0][2] = d * t2;
        dst.data[0][3] = d * t3;

        dst.data[1][0] =
            d * ((tmp1 * m10 + tmp2 * m20 + tmp5 * m30) - (tmp0 * m10 + tmp3 * m20 + tmp4 * m30));
        dst.data[1][1] =
            d * ((tmp0 * m00 + tmp7 * m20 + tmp8 * m30) - (tmp1 * m00 + tmp6 * m20 + tmp9 * m30));
        dst.data[1][2] =
            d * ((tmp3 * m00 + tmp6 * m10 + tmp11 * m30) - (tmp2 * m00 + tmp7 * m10 + tmp10 * m30));
        dst.data[1][3] =
            d * ((tmp4 * m00 + tmp9 * m10 + tmp10 * m20) - (tmp5 * m00 + tmp8 * m10 + tmp11 * m20));

        dst.data[2][0] = d
            * ((tmp12 * m13 + tmp15 * m23 + tmp16 * m33)
                - (tmp13 * m13 + tmp14 * m23 + tmp17 * m33));
        dst.data[2][1] = d
            * ((tmp13 * m03 + tmp18 * m23 + tmp21 * m33)
                - (tmp12 * m03 + tmp19 * m23 + tmp20 * m33));
        dst.data[2][2] = d
            * ((tmp14 * m03 + tmp19 * m13 + tmp22 * m33)
                - (tmp15 * m03 + tmp18 * m13 + tmp23 * m33));
        dst.data[2][3] = d
            * ((tmp17 * m03 + tmp20 * m13 + tmp23 * m23)
                - (tmp16 * m03 + tmp21 * m13 + tmp22 * m23));

        dst.data[3][0] = d
            * ((tmp14 * m22 + tmp17 * m32 + tmp13 * m12)
                - (tmp16 * m32 + tmp12 * m12 + tmp15 * m22));
        dst.data[3][1] = d
            * ((tmp20 * m32 + tmp12 * m02 + tmp19 * m22)
                - (tmp18 * m22 + tmp21 * m32 + tmp13 * m02));
        dst.data[3][2] = d
            * ((tmp18 * m12 + tmp23 * m32 + tmp15 * m02)
                - (tmp22 * m32 + tmp14 * m02 + tmp19 * m12));
        dst.data[3][3] = d
            * ((tmp22 * m22 + tmp16 * m02 + tmp21 * m12)
                - (tmp20 * m12 + tmp23 * m22 + tmp17 * m02));

        return dst;
    }
}

impl Mul<Vec3f> for Mat4f {
    type Output = Vec3f;

    fn mul(self, rhs: Vec3f) -> Self::Output {
        let x = self.data[0][0] * rhs.x() + self.data[1][0] * rhs.y() + self.data[2][0] * rhs.z();
        let y = self.data[0][1] * rhs.x() + self.data[1][1] * rhs.y() + self.data[2][1] * rhs.z();
        let z = self.data[0][2] * rhs.x() + self.data[1][2] * rhs.y() + self.data[2][2] * rhs.z();
        return Vec3f::new(x, y, z);
    }
}
