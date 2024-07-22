pub type Matrix4x4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Transformation {
    model: Matrix4x4,
    view: Matrix4x4,
    projection: Matrix4x4
}

impl Transformation {
    pub fn new(model: Matrix4x4, view: Matrix4x4, projection: Matrix4x4) -> Self {
        Self {
            model,
            view,
            projection
        }
    }
}
