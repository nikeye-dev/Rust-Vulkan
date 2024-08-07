use std::ffi::CStr;
use std::os::raw::c_void;

use anyhow::anyhow;
use log::{debug, error, trace, warn};
use thiserror::Error;
use vulkanalia::{Device, Instance, Version, vk};
use vulkanalia::vk::{ExtensionName, InstanceV1_0, KHR_SWAPCHAIN_EXTENSION, KhrSurfaceExtension, PhysicalDevice, QueueFlags, SurfaceKHR};

use crate::graphics::vulkan::transformation::Matrix4x4;
use crate::graphics::vulkan::vertex::{Vector3, Vector4, Vertex};

pub(crate) const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

pub(crate) const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub(crate) const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

pub(crate) const DEVICE_EXTENSIONS: &[ExtensionName] = &[KHR_SWAPCHAIN_EXTENSION.name];

pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Debug, Error)]
#[error("Suitability Error: {0}.")]
pub struct CompatibilityError(pub &'static str);

// pub static VERTICES: [Vertex; 8] = [
//     Vertex::new(Vector3::new(-1.0, -1.0, -1.0), Vector4::new(1.0, 0.0, 0.0, 1.0)),
//     Vertex::new(Vector3::new(1.0, -1.0, -1.0), Vector4::new(0.0, 1.0, 0.0, 1.0)),
//     Vertex::new(Vector3::new(1.0, 1.0, -1.0), Vector4::new(0.0, 0.0, 1.0, 1.0)),
//     Vertex::new(Vector3::new(-1.0, 1.0, -1.0), Vector4::new(1.0, 0.0, 0.0, 1.0)),
//     Vertex::new(Vector3::new(-1.0, -1.0, 1.0), Vector4::new(0.0, 1.0, 0.0, 1.0)),
//     Vertex::new(Vector3::new(1.0, -1.0, 1.0), Vector4::new(0.0, 0.0, 1.0, 1.0)),
//     Vertex::new(Vector3::new(1.0, 1.0, 1.0), Vector4::new(1.0, 0.0, 0.0, 1.0)),
//     Vertex::new(Vector3::new(-1.0, 1.0, 1.0), Vector4::new(0.0, 1.0, 0.0, 1.0)),
// ];
//
//
// pub static INDICES: &[u16] = &[
//     0, 1, 3, 3, 1, 2,
//     1, 5, 2, 2, 5, 6,
//     5, 4, 6, 6, 4, 7,
//     4, 0, 7, 7, 0, 3,
//     3, 2, 7, 7, 2, 6,
//     4, 5, 0, 0, 5, 1
// ];

pub static VERTICES: [Vertex; 3] = [
    Vertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector4::new(0.0, 0.0, 1.0, 1.0)),
    Vertex::new(Vector3::new(0.0, 0.5, 0.0), Vector4::new(0.0, 1.0, 0.0, 1.0)),
    Vertex::new(Vector3::new(0.5, -0.5, 0.0), Vector4::new(1.0, 0.0, 0.0, 1.0)),
];

pub static INDICES: &[u16] = &[0, 1, 2];

pub const PERSPECTIVE_CORRECTION: Matrix4x4 = Matrix4x4::new(
    1.0,  0.0,       0.0, 0.0,
    0.0, -1.0,       0.0, 0.0,
    0.0,  0.0, 1.0 / 2.0, 0.0,
    0.0,  0.0, 1.0 / 2.0, 1.0,
);

pub extern "system" fn debug_callback(severity: vk::DebugUtilsMessageSeverityFlagsEXT, message_type: vk::DebugUtilsMessageTypeFlagsEXT,
                                  data: *const vk::DebugUtilsMessengerCallbackDataEXT, _: *mut c_void)
-> vk::Bool32
{
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message).to_string_lossy() };

    debug!("Hello from callback");

    match severity
    {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => trace!("({:?}) {}", message_type, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => debug!("({:?}) {}", message_type, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("({:?}) {}", message_type, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("({:?}) {}", message_type, message),
        _ => unreachable!()
    }

    vk::FALSE
}

pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32
}

impl QueueFamilyIndices {
    const QUEUE_FLAGS: QueueFlags = QueueFlags::GRAPHICS;

    pub fn get(instance: &Instance, physical_device: PhysicalDevice, surface: SurfaceKHR) -> anyhow::Result<QueueFamilyIndices> {
        let properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let maybe_index = properties.iter().enumerate()
            .position(|(i, p)| p.queue_flags.contains(Self::QUEUE_FLAGS) && unsafe { instance.get_physical_device_surface_support_khr(physical_device, i as u32, surface) }.unwrap())
            .map(|i| i as u32);

        if let Some(i) = maybe_index {
            Ok(QueueFamilyIndices { graphics: i, present: i })
        } else {
            Err(anyhow!(CompatibilityError("Missing required queue families")))
        }
    }
}

//ToDo: Something more sensible
pub trait LogicalDeviceDestroy {
    fn destroy(&mut self, logical_device: &Device);
}
