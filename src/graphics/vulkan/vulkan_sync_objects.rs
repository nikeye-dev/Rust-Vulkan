use vulkanalia::vk::{DeviceV1_0, Fence, FenceCreateFlags, FenceCreateInfo, Handle, HasBuilder, Semaphore, SemaphoreCreateInfo};
use vulkanalia::{vk, Device};

use crate::graphics::vulkan::vulkan_rhi_data::VulkanRHIData;
use crate::graphics::vulkan::vulkan_swapchain::SwapchainData;
use crate::graphics::vulkan::vulkan_utils::RHIDestroy;

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

impl RHIDestroy for SyncObjects {
    fn destroy(&mut self, rhi_data: &VulkanRHIData) {
        unsafe {
            self.images_in_flight.clear();

            self.in_flight_fences.iter().for_each(|f| rhi_data.logical_device.destroy_fence(*f, None));

            self.render_finished_semaphores.iter().for_each(|s| rhi_data.logical_device.destroy_semaphore(*s, None));
            self.image_available_semaphores.iter().for_each(|s| rhi_data.logical_device.destroy_semaphore(*s, None));
        }
    }
}


