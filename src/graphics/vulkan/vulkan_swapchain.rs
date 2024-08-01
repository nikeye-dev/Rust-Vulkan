use std::cmp::min;
use log::debug;
use vulkanalia::{Device, Instance, vk};
use vulkanalia::vk::{ColorSpaceKHR, CompositeAlphaFlagsKHR, DeviceV1_0, Extent2D, Format, Handle, HasBuilder, Image, ImageUsageFlags, ImageView, KhrSurfaceExtension, KhrSwapchainExtension, PhysicalDevice, PresentModeKHR, SharingMode, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR};
use winit::window::Window;
use crate::graphics::vulkan::vulkan_utils::LogicalDeviceDestroy;

#[derive(Debug, Default)]
pub struct SwapchainData {
    pub swapchain: SwapchainKHR,
    pub swapchain_images: Vec<Image>,
    pub swapchain_format: Format,
    pub swapchain_extent: Extent2D,
    pub swapchain_image_views: Vec<ImageView>,
}

impl LogicalDeviceDestroy for SwapchainData {
    fn destroy(&mut self, logical_device: &Device) {
        unsafe {
            self.swapchain_image_views
                .iter()
                .for_each(|v| logical_device.destroy_image_view(*v, None));

            logical_device.destroy_swapchain_khr(self.swapchain, None);
        }
    }
}

#[derive(Default)]
pub struct SwapchainDataBuilder<'a> {
    window: Option<&'a Window>,
    instance: Option<&'a Instance>,
    physical_device: PhysicalDevice,
    logical_device: Option<&'a Device>,
    surface: SurfaceKHR
}

impl<'a> SwapchainDataBuilder<'a> {
    pub fn window(mut self, window: &'a Window) -> Self {
        self.window = Some(window);
        self
    }

    pub fn instance(mut self, instance: &'a Instance) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn physical_device(mut self, physical_device: PhysicalDevice) -> Self {
        self.physical_device = physical_device;
        self
    }

    pub fn surface(mut self, surface: SurfaceKHR) -> Self {
        self.surface = surface;
        self
    }

    pub fn logical_device(mut self, logical_device: &'a Device) -> Self {
        self.logical_device = Some(logical_device);
        self
    }

    pub fn build(&self) -> anyhow::Result<SwapchainData> {
        assert!(self.window.is_some());
        assert!(self.instance.is_some());
        assert!(self.logical_device.is_some());

        let SwapchainSupport { capabilities, formats, present_modes } = SwapchainSupport::get(self.instance.unwrap(), self.physical_device, self.surface).unwrap();

        let swapchain = self.create_swapchain(&formats, &present_modes, capabilities);

        //ToDo: Do we need swapchain_images when we are going to only use them through image views?
        let swapchain_images = unsafe { self.logical_device.unwrap().get_swapchain_images_khr(swapchain) }.unwrap();

        let surface_format = Self::get_swapchain_surface_format(&formats).format;
        let swapchain_image_views = Self::create_swapchain_image_views(&swapchain_images, self.logical_device.unwrap(), surface_format);

        let surface_extent = Self::get_swapchain_extent(self.window.unwrap(), capabilities);

        Ok(SwapchainData {
            swapchain,
            swapchain_format: surface_format,
            swapchain_extent: surface_extent,
            swapchain_images,
            swapchain_image_views,
        }
        )
    }

    fn create_swapchain(&self, formats: &[SurfaceFormatKHR], present_modes: &[PresentModeKHR], capabilities: SurfaceCapabilitiesKHR) -> SwapchainKHR {
        let surface_format = Self::get_swapchain_surface_format(formats);
        let present_mode = Self::get_swapchain_present_mode(present_modes);
        let extent = Self::get_swapchain_extent(self.window.unwrap(), capabilities);

        let image_count = min(capabilities.min_image_count + 1, capabilities.max_image_count);

        debug!("Image count will be: {:?}", image_count);

        //Sharing mode between graphics and presentation queue. We rely on them being the same one, so we use Exclusive
        let image_sharing_mode = SharingMode::EXCLUSIVE;

        let info = SwapchainCreateInfoKHR::builder()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(SwapchainKHR::null())
            ;

        unsafe { self.logical_device.unwrap().create_swapchain_khr(&info, None) }.unwrap()
    }

    fn create_swapchain_image_views(swapchain_images: &[Image], logical_device: &Device, swapchain_format: Format) -> Vec<ImageView> {
        let components = vk::ComponentMapping::builder()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY);

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        swapchain_images
            .iter()
            .map(|i| {
                let info = vk::ImageViewCreateInfo::builder()
                    .image(*i)
                    .view_type(vk::ImageViewType::_2D)
                    .format(swapchain_format)
                    .components(components)
                    .subresource_range(subresource_range);

                unsafe { logical_device.create_image_view(&info, None) }.unwrap()
            })
            .collect::<Vec<ImageView>>()
    }

    fn get_swapchain_surface_format(formats: &[SurfaceFormatKHR]) -> SurfaceFormatKHR {
        formats
            .iter()
            .find(|f| {
                f.format == Format::B8G8R8A8_SRGB && f.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| &formats[0])
            .clone()
    }

    fn get_swapchain_present_mode(present_modes: &[PresentModeKHR]) -> PresentModeKHR {
        present_modes
            .iter()
            .find(|m| **m == PresentModeKHR::MAILBOX)
            .unwrap_or(&PresentModeKHR::FIFO)
            .clone()
    }

    fn get_swapchain_extent(window: &Window, capabilities: SurfaceCapabilitiesKHR) -> Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            Extent2D::builder()
                .width(window.inner_size().width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(window.inner_size().height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
                .build()
        }
    }
}

pub struct SwapchainSupport {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn get(
        instance: &Instance,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
    ) -> anyhow::Result<Self> {
        let capabilities = unsafe { instance.get_physical_device_surface_capabilities_khr(physical_device, surface) }.unwrap();
        let formats = unsafe { instance.get_physical_device_surface_formats_khr(physical_device, surface) }.unwrap();
        let present_modes = unsafe { instance.get_physical_device_surface_present_modes_khr(physical_device, surface) }.unwrap();

        Ok(Self {
            capabilities,
            formats,
            present_modes,
        })
    }
}