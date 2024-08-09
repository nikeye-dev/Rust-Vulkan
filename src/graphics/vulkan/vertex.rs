use std::mem::size_of;

use vulkanalia::vk::{Format, HasBuilder, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate};
use crate::utils::math::Zero;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;

//For uniform buffer data prefer over Vector3.
//Vector3s are memory aligned as Vector4s in shader but not in C/Rust
//and alignment needs to be manually padded if Vector3 is used
pub type Vector4 = cgmath::Vector4<f32>;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: Vector4,
    normal: Vector4,
    color: Vector4,
}

impl Vertex {
    pub const fn new(pos: Vector3, normal: Vector3, color: Vector4) -> Self {
        Self {
            position: Vector4::new(pos.x, pos.y, pos.z, 1.0),
            normal: Vector4::new(normal.x, normal.y, normal.z, 0.0),
            color,
        }
    }

    pub fn with_pos(pos: Vector3) -> Self {
        Vertex::new(pos, Vector3::zero(), Vector4::new(0., 0., 0., 1.))
    }

    pub fn with_pos_raw(x: f32, y: f32, z: f32) -> Self {
        Vertex::new(Vector3::new(x, y, z), Vector3::zero(), Vector4::new(0., 0., 0., 1.))
    }

    //Vulkan specific
    pub fn binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> Vec<VertexInputAttributeDescription> {
        let position_attribute = VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();

        let normal_attribute = VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vector4>() as u32)
            .build();

        let color_attribute = VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(Format::R32G32B32A32_SFLOAT)
            .offset(size_of::<Vector4>() as u32)
            .build();

        Vec::from([position_attribute, normal_attribute, color_attribute])
    }
}
