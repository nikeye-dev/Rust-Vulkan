use cgmath::Zero;

use crate::graphics::vulkan::vertex::Vector4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ViewState {
    pub world_camera_origin : Vector4,
    pub atmosphere_light_direction : Vector4,
    pub atmosphere_light_illuminance_outer_space : Vector4,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            world_camera_origin: Vector4::zero(),
            atmosphere_light_direction: Vector4::zero(),
            atmosphere_light_illuminance_outer_space: Vector4::zero()
        }
    }
}
