use std::collections::HashSet;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;
use std::slice;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use anyhow::{anyhow, Result};
use cgmath::{Decomposed, Deg, EuclideanSpace, Euler, InnerSpace, perspective, point3, Quaternion, Rad, Rotation, SquareMatrix, Transform, vec3, vec4, Zero};
use cgmath::num_traits::real::Real;
use log::{debug, info, warn};
use vulkanalia::{Device, Entry, Instance, vk};
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::{Buffer, BufferCreateFlags, BufferCreateInfo, BufferUsageFlags, ClearColorValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferInheritanceInfo, CommandBufferLevel, CommandBufferResetFlags, CommandBufferUsageFlags, CommandPoolResetFlags, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT, DeviceCreateInfo, DeviceMemory, DeviceQueueCreateInfo, DeviceV1_0, EntryV1_0, ErrorCode, ExtDebugUtilsExtension, Fence, FenceCreateFlags, FenceCreateInfo, Handle, HasBuilder, IndexType, InstanceV1_0, KhrSwapchainExtension, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, Offset2D, PhysicalDevice, PhysicalDeviceFeatures, PipelineBindPoint, PipelineStageFlags, PresentInfoKHR, Rect2D, RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo, ShaderStageFlags, SharingMode, SubmitInfo, SubpassContents, SuccessCode, SurfaceKHR};
use vulkanalia::window as vk_window;
use vulkanalia::window::create_surface;
use winit::window::Window;

use crate::config::config::{GraphicsConfig, LogLevel};
use crate::graphics::rhi::RHI;
use crate::graphics::vulkan::atmopsheric_scattering::{AtmosphereSampleData, ScatteringMedium};
use crate::graphics::vulkan::transformation::{Matrix4x4, Transformation};
use crate::graphics::vulkan::vertex::{Vector3, Vector4, Vertex};
use crate::graphics::vulkan::view_state::ViewState;
use crate::graphics::vulkan::vulkan_data::{SyncObjects, VulkanData};
use crate::graphics::vulkan::vulkan_pipeline::PipelineDataBuilder;
use crate::graphics::vulkan::vulkan_swapchain::{SwapchainData, SwapchainDataBuilder, SwapchainSupport};
use crate::graphics::vulkan::vulkan_utils::{CompatibilityError, debug_callback, DEVICE_EXTENSIONS, INDICES, LogicalDeviceDestroy, MAX_FRAMES_IN_FLIGHT, PERSPECTIVE_CORRECTION, perspective_matrix, PORTABILITY_MACOS_VERSION, QueueFamilyIndices, VALIDATION_ENABLED, VALIDATION_LAYER, VERTICES};
use crate::utils::math::{VECTOR3_BACKWARD, VECTOR3_FORWARD};
use crate::world::transform::OwnedTransform;
use crate::world::world::World;

pub struct RHIVulkan {
    is_destroyed: bool,
    config: GraphicsConfig,
    data: VulkanData,
    frame_index: usize,
    world: Option<Arc<RwLock<World>>>
}

impl RHI for RHIVulkan {

    fn initialize(&mut self, world: Arc<RwLock<World>>) -> Result<()> {
        self.world = Some(world);
        Ok(())
    }
    fn update(&mut self) {
        todo!()
    }

    fn render(&mut self, window: &Window) -> Result<()> {
        let fence = self.data.sync_objects.in_flight_fences[self.frame_index];

        unsafe {
            self.data.logical_device.wait_for_fences(&[fence], true, u64::MAX).unwrap();
        }

        let next_image_result = unsafe {
            self.data.logical_device.acquire_next_image_khr(self.data.swapchain_data.swapchain, u64::MAX, self.data.sync_objects.image_available_semaphores[self.frame_index], Fence::null())
        };

        let image_index = match next_image_result {
            Ok((image_index, _)) => image_index,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e))
        };

        if !self.data.sync_objects.images_in_flight[image_index as usize].is_null() {
            unsafe {
                self.data.logical_device.wait_for_fences(&[self.data.sync_objects.images_in_flight[image_index as usize]], true, u64::MAX).unwrap();
            }
        }

        self.data.sync_objects.set_image_fence(image_index as usize, fence);

        self.update_command_buffers(image_index as usize);
        self.update_uniform_buffers(image_index as usize);

        let wait_semaphores = &[self.data.sync_objects.image_available_semaphores[self.frame_index]];
        let wait_stages = &[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.pipeline_data.primary_command_buffers[image_index as usize]];
        let signal_semaphores = &[self.data.sync_objects.render_finished_semaphores[self.frame_index]];
        let submit_info = SubmitInfo::builder()
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores)
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
        ;

        unsafe {
            self.data.logical_device.reset_fences(&[fence]).unwrap();
            self.data.logical_device.queue_submit(self.data.graphics_queue, &[submit_info], self.data.sync_objects.in_flight_fences[self.frame_index]).unwrap();
        }

        let swapchains = &[self.data.swapchain_data.swapchain];
        let image_indices = &[image_index];
        let present_info = PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices)
        ;

        let present_result = unsafe { self.data.logical_device.queue_present_khr(self.data.present_queue, &present_info) };

        //ToDo: Handle swapchain invalidation better - resize, minimize etc
        if present_result == Ok(SuccessCode::SUBOPTIMAL_KHR) || present_result == Err(ErrorCode::OUT_OF_DATE_KHR) {
            self.recreate_swapchain(window).unwrap();
        } else if let Err(e) = present_result {
            return Err(anyhow!(e));
        }

        self.frame_index = (self.frame_index + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    fn destroy(&mut self) {
        self.is_destroyed = true;
        self.data.destroy();
    }

    fn get_width(&self) -> u32 {
        todo!()
    }

    fn get_height(&self) -> u32 {
        todo!()
    }
}

impl RHIVulkan {
    pub fn new(window: &Window, config: GraphicsConfig) -> Self {
        let loader = unsafe { LibloadingLoader::new(LIBRARY) }.unwrap();
        let entry = unsafe { Entry::new(loader) }.unwrap();
        let instance = Self::create_instance(window, &entry, &config).unwrap();

        let mut messenger = DebugUtilsMessengerEXT::default();
        if VALIDATION_ENABLED {
            let severity = match config.log_level {
                LogLevel::Verbose => DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
                LogLevel::Info => DebugUtilsMessageSeverityFlagsEXT::INFO,
                LogLevel::Warning => DebugUtilsMessageSeverityFlagsEXT::WARNING,
                LogLevel::Error => DebugUtilsMessageSeverityFlagsEXT::ERROR
            };

            let debug_info = DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(severity)
                .message_type(DebugUtilsMessageTypeFlagsEXT::all())
                .user_callback(Some(debug_callback));

            messenger = unsafe { instance.create_debug_utils_messenger_ext(&debug_info, None) }.unwrap();
        }

        let surface = unsafe { create_surface(&instance, &window, &window) }.unwrap();

        let physical_device = Self::pick_physical_device(&instance, surface).unwrap();
        let logical_device = Self::create_logical_device(&instance, physical_device, surface);

        let queue_family_indices = QueueFamilyIndices::get(&instance, physical_device, surface).unwrap();

        let graphics_queue = unsafe { logical_device.get_device_queue(queue_family_indices.graphics, 0) };
        let present_queue = unsafe { logical_device.get_device_queue(queue_family_indices.present, 0) };

        let swapchain_data = SwapchainDataBuilder::default()
            .window(window)
            .instance(&instance)
            .surface(surface)
            .physical_device(physical_device)
            .logical_device(&logical_device)
            .build()
            .unwrap();

        let pipeline_data = PipelineDataBuilder::default()
            .window(window)
            .instance(&instance)
            .surface(surface)
            .physical_device(physical_device)
            .logical_device(&logical_device)
            .swapchain_data(&swapchain_data)
            .graphics_queue(graphics_queue)
            .build()
            .unwrap();

        let sync_objects = SyncObjects::create(&logical_device, &swapchain_data, MAX_FRAMES_IN_FLIGHT);

        let data = VulkanData{
            entry,
            instance,
            messenger,
            surface,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            swapchain_data,
            pipeline_data,
            sync_objects,

        };

        Self {
            is_destroyed: false,
            config,
            data,
            frame_index: 0,
            world: None
        }
    }

    fn create_instance(window: &Window, entry: &Entry, config: &GraphicsConfig) -> Result<Instance> {
        let app_info = vk::ApplicationInfo::builder()
            .application_version(vk::make_version(0, 1, 0))
            .api_version(vk::make_version(1, 0, 0))
            .engine_version(vk::make_version(1, 0, 0))
            .application_name(b"Test Name")
            .engine_name(b"Test Engine")
            .build();

        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        //Enable compatibility extensions
        // Required by Vulkan SDK on macOS since 1.3.216.
        let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION
        {
            info!("Enabling extensions for macOS portability.");
            extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
            extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };
        //

        //Validation layers
        let available_layers = unsafe { entry.enumerate_instance_layer_properties() }
            .unwrap()
            .iter()
            .map(|l| l.layer_name)
            .collect::<HashSet<_>>();

        if VALIDATION_ENABLED {
            if !available_layers.contains(&VALIDATION_LAYER) {
                return Err(anyhow!("Validation layer requested but not supported."));
            }

            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        let layers = if VALIDATION_ENABLED {
            vec![VALIDATION_LAYER.as_ptr()]
        } else {
            Vec::new()
        };
        //

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .flags(flags);

        //Debug
        let mut debug_info = DebugUtilsMessengerCreateInfoEXT::builder();
        if VALIDATION_ENABLED {
            let severity = match config.log_level {
                LogLevel::Verbose => DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
                LogLevel::Info => DebugUtilsMessageSeverityFlagsEXT::INFO,
                LogLevel::Warning => DebugUtilsMessageSeverityFlagsEXT::WARNING,
                LogLevel::Error => DebugUtilsMessageSeverityFlagsEXT::ERROR
            };
            println!("Severity: {:?}", severity);
            debug_info.message_severity(severity)
                      .message_type(DebugUtilsMessageTypeFlagsEXT::all())
                      .user_callback(Some(debug_callback));

            instance_info.push_next(&mut debug_info);

            info!("Added debug callback to Vulkan with level {:?}", severity);
        }
        //

        let result = unsafe { entry.create_instance(&instance_info, None) }.unwrap();
        Ok(result)
    }

    fn pick_physical_device(instance: &Instance, surface: SurfaceKHR) -> Result<PhysicalDevice> {
        for physical_device in unsafe {instance.enumerate_physical_devices()?} {
            let properties = unsafe { instance.get_physical_device_properties(physical_device) };

            match Self::check_physical_device_compatibility(instance, physical_device, surface) {
                Ok(_) => {
                    info!("Selected physical device (`{}`).", properties.device_name);
                    return Ok(physical_device);
                }
                Err(error) => warn!("Skipping physical device (`{}`): {}", properties.device_name, error)
            }
        }

        Err(anyhow!(CompatibilityError("Failed to find compatible physical device")))
    }

    fn check_physical_device_compatibility(instance: &Instance, physical_device: PhysicalDevice, surface: SurfaceKHR) -> Result<()> {
        let _ = QueueFamilyIndices::get(instance, physical_device, surface)?;
        let _ = Self::check_physical_device_extensions(instance, physical_device)?;

        let support = SwapchainSupport::get(instance, physical_device, surface).unwrap();
        if support.formats.is_empty() || support.present_modes.is_empty() {
            return Err(anyhow!(CompatibilityError("Insufficient swapchain support.")));
        }

        Ok(())
    }

    fn check_physical_device_extensions(instance: &Instance, physical_device: PhysicalDevice) -> Result<()> {
        let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device, None).unwrap().iter().map(|e| e.extension_name).collect::<HashSet<_>>() };
        //Check for graphics commands
        let is_supported = DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e));
        if is_supported {
            Ok(())
        }
        else {
            Err(anyhow!(CompatibilityError("Missing required queue family extensions.")))
        }
    }

    //ToDo: Maybe instance method - e.g. initialize?
    fn create_logical_device(instance: &Instance, physical_device: PhysicalDevice, surface: SurfaceKHR) -> Device {
        let queue_family_indices = QueueFamilyIndices::get(instance, physical_device, surface).unwrap();
        let mut indices = HashSet::new();

        indices.insert(queue_family_indices.graphics);
        indices.insert(queue_family_indices.present);

        let layers = if VALIDATION_ENABLED {
            vec![VALIDATION_LAYER.as_ptr()]
        } else {
            vec![]
        };

        let extensions = DEVICE_EXTENSIONS
            .iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<_>>();

        let features = PhysicalDeviceFeatures::builder();

        let queue_priorities = &[1.0];
        let queue_infos =
            indices.iter().map(|i| {
                DeviceQueueCreateInfo::builder()
                    .queue_family_index(*i)
                    .queue_priorities(queue_priorities)
            })
                .collect::<Vec<_>>();

        let device_info = DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .enabled_features(&features);

        let device = unsafe { instance.create_device(physical_device, &device_info, None) }.unwrap();

        device
    }

    fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        unsafe {
            self.data.logical_device.device_wait_idle()?;
        }

        self.data.swapchain_data.destroy(&self.data.logical_device);

        //ToDo: Reuse what can be reused (e.g. command buffers)
        self.data.pipeline_data.destroy(&self.data.logical_device);

        self.data.sync_objects.destroy(&self.data.logical_device);

        self.data.swapchain_data = SwapchainDataBuilder::default()
            .window(window)
            .instance(&self.data.instance)
            .surface(self.data.surface)
            .physical_device(self.data.physical_device)
            .logical_device(&self.data.logical_device)
            .build()
            .unwrap();

        self.data.pipeline_data = PipelineDataBuilder::default()
            .window(window)
            .instance(&self.data.instance)
            .surface(self.data.surface)
            .physical_device(self.data.physical_device)
            .logical_device(&self.data.logical_device)
            .swapchain_data(&self.data.swapchain_data)
            .graphics_queue(self.data.graphics_queue)
            .build()
            .unwrap();

        self.data.sync_objects = SyncObjects::create(&self.data.logical_device, &self.data.swapchain_data, MAX_FRAMES_IN_FLIGHT);

        Ok(())
    }

    //ToDo: Add transforms and move from here
    fn update_uniform_buffers(&self, image_index: usize) {
        let world = self.world.as_ref().unwrap().read().unwrap();
        let camera = world.active_camera();
        let view = camera.view_matrix();

        let camera_pos = camera.transform().location();
        let projection = PERSPECTIVE_CORRECTION * perspective_matrix(camera.view().fov,
                                                        self.data.swapchain_data.swapchain_extent.width as f32,
                                                        self.data.swapchain_data.swapchain_extent.height as f32,
                                                        camera.view().near,
                                                        camera.view().far);

        let transformation = Transformation::new(view, projection);

        unsafe {
            let buffer_memory = self.data.pipeline_data.uniform_buffers_memory[image_index][0];
            let memory = self.data.logical_device.map_memory(
                buffer_memory,
                0,
                size_of::<Transformation>() as u64,
                MemoryMapFlags::empty())
                .unwrap();

            copy_nonoverlapping(&transformation, memory.cast(), 1);
            self.data.logical_device.unmap_memory(buffer_memory)
        };

        let light_pos = vec4(0.0f32, 100., 1000., 0.);
        let light_rot = Quaternion::from(Euler {
            x: Deg(-65.0),
            y: Deg(25.0),
            z: Deg(0.0)
        });

        let light_dir =  light_rot.rotate_vector(VECTOR3_FORWARD).extend(0.0);//(light_pos - Vector4::zero()).normalize();
        let light_illuminance_outer_space = vec4(1., 1., 1., 1.) * 100.0;

        let view_state = ViewState {
            world_camera_origin: camera_pos.extend(0.0),
            atmosphere_light_direction: light_dir,
            atmosphere_light_illuminance_outer_space: light_illuminance_outer_space
        };

        unsafe {
            let buffer_memory = self.data.pipeline_data.uniform_buffers_memory[image_index][1];
            let memory = self.data.logical_device.map_memory(
                buffer_memory,
                0,
                size_of::<ViewState>() as u64,
                MemoryMapFlags::empty())
                .unwrap();

            copy_nonoverlapping(&view_state, memory.cast(), 1);
            self.data.logical_device.unmap_memory(buffer_memory)
        };

        let unit_scale = 0.2;
        let scattering_ray = vec3(0.175287, 0.409607, 1.0);
        let medium = ScatteringMedium::new(0.2, scattering_ray);

        unsafe {
            let buffer_memory = self.data.pipeline_data.uniform_buffers_memory[image_index][2];
            let memory = self.data.logical_device.map_memory(
                buffer_memory,
                0,
                size_of::<ScatteringMedium>() as u64,
                MemoryMapFlags::empty())
                .unwrap();

            copy_nonoverlapping(&medium, memory.cast(), 1);
            self.data.logical_device.unmap_memory(buffer_memory)
        };

        let atmospheric_sample_data = AtmosphereSampleData {
            planet_pos: vec3(0.0, 0.0, 0.0).extend(0.0),
            planet_radius: 6.3710,
            atmosphere_thickness: 0.0600,
            sample_count: 100.0,
            sample_count_light: 15.0,
            unit_scale,
            light_dir,
            light_intensity: light_illuminance_outer_space,

            pad: [0.0, 0.0, 0.0]
        };

        unsafe {
            let buffer_memory = self.data.pipeline_data.uniform_buffers_memory[image_index][3];
            let memory = self.data.logical_device.map_memory(
                buffer_memory,
                0,
                size_of::<AtmosphereSampleData>() as u64,
                MemoryMapFlags::empty())
                .unwrap();

            copy_nonoverlapping(&atmospheric_sample_data, memory.cast(), 1);
            self.data.logical_device.unmap_memory(buffer_memory)
        };
    }

    fn update_command_buffers(&mut self, image_index: usize) {
        let command_pool = self.data.pipeline_data.command_pools[image_index];
        unsafe { self.data.logical_device.reset_command_pool(command_pool, CommandPoolResetFlags::empty()) }.unwrap();

        let command_buffer = self.data.pipeline_data.primary_command_buffers.get(image_index).unwrap();
        self.update_command_buffer(image_index, *command_buffer);
    }

    fn update_command_buffer(&mut self, image_index: usize, command_buffer: CommandBuffer) {
        let command_buffer_inheritance_info = CommandBufferInheritanceInfo::builder();

        let command_buffer_begin_info = CommandBufferBeginInfo::builder()
            .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .inheritance_info(&command_buffer_inheritance_info);

        let logical_device = &self.data.logical_device;


        let render_area = Rect2D::builder()
            .extent(self.data.swapchain_data.swapchain_extent)
            .offset(Offset2D::default())
            ;

        let color_clear_value = ClearValue {
            color: ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0]
            }
        };

        let depth_clear_value = ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let render_pass_begin_info = RenderPassBeginInfo::builder()
            .render_pass(self.data.pipeline_data.render_pass)
            .framebuffer(self.data.pipeline_data.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values)
            ;

        unsafe {
            logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
            logical_device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, SubpassContents::SECONDARY_COMMAND_BUFFERS);
        }

        let secondary_command_buffer = self.data.pipeline_data.get_or_allocate_secondary_buffer(image_index, 0, &self.data.logical_device);
        self.update_secondary_command_buffer(secondary_command_buffer, image_index);

        unsafe {
            logical_device.cmd_execute_commands(command_buffer, &[secondary_command_buffer]);

            logical_device.cmd_end_render_pass(command_buffer);
            logical_device.end_command_buffer(command_buffer).unwrap();
        }
    }

    //ToDo: Make async and parallelize
    fn update_secondary_command_buffer(&self, command_buffer: CommandBuffer, image_index: usize) {
        //ToDo:
        let world = self.world.as_ref().unwrap().read().unwrap();
        let entities = world.get_entities();

        let model = entities[0].transform.matrix();
        let model_bytes = unsafe { slice::from_raw_parts(&model as *const Matrix4x4 as *const u8, size_of::<Matrix4x4>()) };

        // let command_buffer = self.get_or_add_secondary_buffer(&image_index, buffer_index);

        let inheritance_info = CommandBufferInheritanceInfo::builder()
            .render_pass(self.data.pipeline_data.render_pass)
            .subpass(0)
            .framebuffer(self.data.pipeline_data.framebuffers[image_index])
            ;

        let info = CommandBufferBeginInfo::builder()
            .flags(CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info)
            ;

        let logical_device = &self.data.logical_device;
        unsafe {
            logical_device.begin_command_buffer(command_buffer, &info).unwrap();
            logical_device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, self.data.pipeline_data.pipeline);

            logical_device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.data.pipeline_data.vertex_buffer], &[0]);
            logical_device.cmd_bind_index_buffer(command_buffer, self.data.pipeline_data.index_buffer, 0, IndexType::UINT16);

            logical_device.cmd_bind_descriptor_sets(command_buffer, PipelineBindPoint::GRAPHICS, self.data.pipeline_data.pipeline_layout, 0, &[self.data.pipeline_data.descriptor_sets[image_index]], &[]);

            logical_device.cmd_push_constants(command_buffer, self.data.pipeline_data.pipeline_layout, ShaderStageFlags::VERTEX, 0, model_bytes);

            logical_device.cmd_draw_indexed(command_buffer, INDICES.len() as u32, 1, 0, 0, 0);

            logical_device.end_command_buffer(command_buffer).unwrap()
        }
    }
}

impl Drop for RHIVulkan {
    fn drop(&mut self) {
        if !self.is_destroyed {
            self.destroy();
        }
    }
}
