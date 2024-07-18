use std::mem::size_of;
use vulkanalia::vk::{Format, HasBuilder, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate};

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: Vector3,
    color: Vector4
}

impl Vertex {
    pub const fn new(pos: Vector3, color: Vector4) -> Self {
        Self {
            position: pos,
            color
        }
    }

    pub fn with_pos(pos: Vector3) -> Self {
        Vertex::new(pos, Vector4::new(0., 0., 0., 1.))
    }

    pub fn with_pos_raw(x: f32, y: f32, z: f32) -> Self {
        Vertex::new(Vector3::new(x, y, z), Vector4::new(0., 0., 0., 1.))
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

        let color_attribute = VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(Format::R32G32B32A32_SFLOAT)
            .offset(size_of::<Vector3>() as u32)
            .build();

        Vec::from([position_attribute, color_attribute])
    }
}
