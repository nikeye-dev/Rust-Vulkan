pub type Matrix4x4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Transformation {
    view: Matrix4x4,
    projection: Matrix4x4
}

impl Transformation {
    pub fn new(view: Matrix4x4, projection: Matrix4x4) -> Self {
        Self {
            view,
            projection
        }
    }
}
