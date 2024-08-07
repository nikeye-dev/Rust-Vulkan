use vulkanalia::{Device, Entry, Instance, vk};
use vulkanalia::vk::{DebugUtilsMessengerEXT, DeviceV1_0, ExtDebugUtilsExtension, Fence, FenceCreateFlags, FenceCreateInfo, Handle, HasBuilder, InstanceV1_0, KhrSurfaceExtension, PhysicalDevice, Queue, Semaphore, SemaphoreCreateInfo, SurfaceKHR};

use crate::graphics::vulkan::vulkan_pipeline::PipelineData;
use crate::graphics::vulkan::vulkan_swapchain::SwapchainData;
use crate::graphics::vulkan::vulkan_utils::LogicalDeviceDestroy;

pub struct VulkanData {
    pub(crate) entry: Entry,
    pub(crate) instance: Instance,
    pub(crate) messenger: DebugUtilsMessengerEXT,
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) logical_device: Device,
    pub(crate) graphics_queue: Queue,
    pub(crate) surface: SurfaceKHR,
    pub(crate) present_queue: Queue,
    pub(crate) swapchain_data: SwapchainData,

    pub(crate) pipeline_data: PipelineData,

    pub(crate) sync_objects: SyncObjects,
}

impl VulkanData {
    pub fn destroy(&mut self) {
        unsafe {
            self.logical_device.device_wait_idle().unwrap();

            self.sync_objects.destroy(&self.logical_device);
            self.pipeline_data.destroy(&self.logical_device);

            self.swapchain_data.destroy(&self.logical_device);

            self.logical_device.destroy_device(None);
            self.instance.destroy_debug_utils_messenger_ext(self.messenger, None);
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

pub(crate) struct SyncObjects {
    pub(crate) image_available_semaphores: Vec<vk::Semaphore>,
    pub(crate) render_finished_semaphores: Vec<vk::Semaphore>,

    pub(crate) in_flight_fences: Vec<Fence>,
    pub(crate) images_in_flight: Vec<Fence>
}

impl SyncObjects {
    pub(crate) fn create(logical_device: &Device, swapchain_data: &SwapchainData, max_frames: usize) -> Self {
        let create_info = SemaphoreCreateInfo::builder();
        let fence_info = FenceCreateInfo::builder()
            .flags(FenceCreateFlags::SIGNALED)
            ;

        let mut image_semaphores = Vec::<Semaphore>::with_capacity(max_frames);
        let mut render_semaphores = Vec::<Semaphore>::with_capacity(max_frames);
        let mut fences = Vec::<Fence>::with_capacity(max_frames);

        for _ in 0..max_frames {
            unsafe {
                image_semaphores.push(logical_device.create_semaphore(&create_info, None).unwrap());
                render_semaphores.push(logical_device.create_semaphore(&create_info, None).unwrap());

                fences.push(logical_device.create_fence(&fence_info, None).unwrap());
            };
        }

        let image_fences = swapchain_data.swapchain_images.iter().map(|_| Fence::null()).collect::<Vec<Fence>>();

        Self {
            image_available_semaphores: image_semaphores,
            render_finished_semaphores: render_semaphores,
            in_flight_fences: fences,
            images_in_flight: image_fences
        }
    }

    pub(crate) fn set_image_fence(&mut self, image_index: usize, fence: Fence) {
        self.images_in_flight[image_index] = fence;
    }
}

impl LogicalDeviceDestroy for SyncObjects {
    fn destroy(&mut self, logical_device: &Device) {
        unsafe {
            self.images_in_flight.clear();

            self.in_flight_fences.iter().for_each(|f| logical_device.destroy_fence(*f, None));

            self.render_finished_semaphores.iter().for_each(|s| logical_device.destroy_semaphore(*s, None));
            self.image_available_semaphores.iter().for_each(|s| logical_device.destroy_semaphore(*s, None));
        }
    }
}


