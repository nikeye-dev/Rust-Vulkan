use crate::graphics::vulkan::transformation::Matrix4x4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PushConstants {
    model: Matrix4x4,
}

impl PushConstants {
    pub fn new(model: Matrix4x4) -> Self {
        Self {
            model
        }
    }
}