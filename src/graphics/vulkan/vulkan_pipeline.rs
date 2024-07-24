use std::fmt::Debug;
use std::intrinsics::copy_nonoverlapping;
use std::mem::{size_of, take};
use std::ptr::slice_from_raw_parts;
use std::slice;

use anyhow::anyhow;
use cgmath::Deg;
use vulkanalia::{Device, Instance};
use vulkanalia::bytecode::Bytecode;
use vulkanalia::vk::{AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, BlendFactor, BlendOp, Buffer, BufferCopy, BufferCreateInfo, BufferUsageFlags, ClearColorValue, ClearValue, ColorComponentFlags, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferInheritanceInfo, CommandBufferLevel, CommandBufferUsageFlags, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, CopyDescriptorSet, CullModeFlags, DescriptorBufferInfo, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, DeviceMemory, DeviceSize, DeviceV1_0, Fence, Framebuffer, FramebufferCreateInfo, FrontFace, GraphicsPipelineCreateInfo, Handle, HasBuilder, ImageLayout, IndexType, InstanceV1_0, LogicOp, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, Offset2D, PhysicalDevice, Pipeline, PipelineBindPoint, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineStageFlags, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, PushConstantRange, Queue, Rect2D, RenderPass, RenderPassBeginInfo, RenderPassCreateInfo, SampleCountFlags, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags, SharingMode, SubmitInfo, SUBPASS_EXTERNAL, SubpassContents, SubpassDependency, SubpassDescription, SurfaceKHR, Viewport, WHOLE_SIZE, WriteDescriptorSet};
use winit::window::Window;
use crate::graphics::vulkan::push_constants::PushConstants;
use crate::graphics::vulkan::transform::{Matrix4x4, Transformation};

use crate::graphics::vulkan::vertex::{Vector3, Vertex};
use crate::graphics::vulkan::vulkan_swapchain::SwapchainData;
use crate::graphics::vulkan::vulkan_utils::{INDICES, LogicalDeviceDestroy, QueueFamilyIndices, VERTICES};

#[derive(Debug, Default)]
pub struct PipelineData {
    pub(crate) pipeline_layout: PipelineLayout,
    pub(crate) render_pass: RenderPass,
    pub(crate) pipeline: Pipeline,
    pub(crate) framebuffers: Vec<Framebuffer>,

    pub(crate) global_command_pool: CommandPool,

    pub(crate) command_pools: Vec<CommandPool>,
    pub(crate) command_buffers: Vec<Vec<CommandBuffer>>,

    //ToDo: Move
    pub(crate) vertex_buffer: Buffer,
    pub(crate) vertex_buffer_memory: DeviceMemory,

    //ToDo: Move
    pub(crate) index_buffer: Buffer,
    pub(crate) index_buffer_memory: DeviceMemory,

    //ToDo: Move
    pub(crate) descriptor_set_layout: DescriptorSetLayout,

    pub(crate) uniform_buffers: Vec<Buffer>,
    pub(crate) uniform_buffers_memory: Vec<DeviceMemory>,

    pub(crate) descriptor_pool: DescriptorPool,
    pub(crate) descriptor_sets: Vec<DescriptorSet>,
}

impl LogicalDeviceDestroy for PipelineData {
   fn destroy(&mut self, logical_device: &Device) {
       unsafe {
           self.command_buffers.iter().enumerate().for_each(|(i, buffers)| {
               logical_device.free_command_buffers(self.command_pools[i], &buffers);
           });

           self.command_pools.iter().for_each(|p| logical_device.destroy_command_pool(*p, None));
           logical_device.destroy_command_pool(self.global_command_pool, None);

           self.framebuffers
               .iter()
               .for_each(|fb| logical_device.destroy_framebuffer(*fb, None));

           logical_device.destroy_render_pass(self.render_pass, None);
           logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
           logical_device.destroy_pipeline(self.pipeline, None);

           logical_device.destroy_buffer(self.index_buffer, None);
           logical_device.free_memory(self.index_buffer_memory, None);

           logical_device.destroy_buffer(self.vertex_buffer, None);
           logical_device.free_memory(self.vertex_buffer_memory, None);

           logical_device.destroy_descriptor_pool(self.descriptor_pool, None);

           logical_device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

           for i in 0..self.uniform_buffers.len() {
               logical_device.destroy_buffer(self.uniform_buffers[i], None);
               logical_device.free_memory(self.uniform_buffers_memory[i], None);
           }

           self.uniform_buffers.clear();
           self.uniform_buffers_memory.clear();
       }
   }
}

//ToDo: Restructure
#[derive(Debug, Default)]
pub struct PipelineDataBuilder<'a> {
    value: PipelineData,
    window: Option<&'a Window>,
    instance: Option<&'a Instance>,
    physical_device: PhysicalDevice,
    logical_device: Option<&'a Device>,
    surface: SurfaceKHR,
    swapchain_data: Option<&'a SwapchainData>,
    graphics_queue: Queue
}

impl<'a> PipelineDataBuilder<'a> {
    pub fn swapchain_data(mut self, swapchain_data: &'a SwapchainData) -> Self {
        self.swapchain_data = Some(swapchain_data);
        self
    }

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

    pub fn graphics_queue(mut self, graphics_queue: Queue) -> Self {
        self.graphics_queue = graphics_queue;
        self
    }

    pub fn build(&mut self) -> anyhow::Result<PipelineData> {
        assert!(self.window.is_some());
        assert!(self.instance.is_some());
        assert!(self.logical_device.is_some());
        assert!(self.swapchain_data.is_some());
        assert!(!self.graphics_queue.is_null());

        self.create_descriptor_set_layout();
        self.create_pipeline();
        self.create_framebuffers();
        self.create_command_pools();

        self.create_vertex_buffer();
        self.create_index_buffer();
        self.create_uniform_buffers();
        self.create_descriptor_pool();
        self.create_descriptor_sets();
        self.create_command_buffers();

        Ok(take(&mut self.value))
    }

    fn create_pipeline(&mut self) {
        let logical_device = self.logical_device.unwrap();

        //ToDo: Make dynamic
        let vert = include_bytes!("../../../resources/shaders/test_vert.spv");

        let vert_module = self.create_shader_module(&vert[..]).unwrap();

        let vert_stage = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::VERTEX)
            .module(vert_module)
            .name(b"main\0");

        //Create vertex input
        let binding_descriptions = &[Vertex::binding_description()];
        let attribute_descriptions = Vertex::attribute_descriptions();
        let vertex_input_state = PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions)
            ;

        let frag = include_bytes!("../../../resources/shaders/test_frag.spv");
        let frag_module = self.create_shader_module(&frag[..]).unwrap();

        let frag_stage = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(b"main\0");

        let input_assembly_state = PipelineInputAssemblyStateCreateInfo::builder()
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.swapchain_data.unwrap().swapchain_extent.width as f32)
            .height(self.swapchain_data.unwrap().swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            ;

        let scissor = Rect2D::builder()
            .offset(Offset2D {x: 0, y: 0})
            .extent(self.swapchain_data.unwrap().swapchain_extent)
            ;

        let viewports = &[viewport];
        let scissors = &[scissor];
        let viewport_state = PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors)
            ;

        let rasterization_state = PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            ;

        let multisample_state = PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::_1)
            ;

        // let depth_stencil_state = PipelineDepthStencilStateCreateInfo::builder();

        let color_blend_attachment = PipelineColorBlendAttachmentState::builder()
            .color_write_mask(ColorComponentFlags::all())
            .blend_enable(false)
            .src_color_blend_factor(BlendFactor::ONE)
            .dst_color_blend_factor(BlendFactor::ZERO)
            .color_blend_op(BlendOp::ADD)
            .src_alpha_blend_factor(BlendFactor::ONE)
            .dst_alpha_blend_factor(BlendFactor::ZERO)
            .alpha_blend_op(BlendOp::ADD)
            ;

        let attachments = &[color_blend_attachment];
        let color_blend_state = PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            ;

        let vert_push_constant_range = PushConstantRange::builder()
            .stage_flags(ShaderStageFlags::VERTEX)
            .offset(0)
            .size(size_of::<PushConstants>() as u32)
        ;

        let layouts = &[self.value.descriptor_set_layout];
        let push_constant_ranges = &[vert_push_constant_range];
        let layout_info = PipelineLayoutCreateInfo::builder()
            .set_layouts(layouts)
            .push_constant_ranges(push_constant_ranges)
        ;

        self.value.pipeline_layout = unsafe { logical_device.create_pipeline_layout(&layout_info, None) }.unwrap();

        self.create_render_pass();

        let stages = &[vert_stage, frag_stage];
        let pipeline_info = GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(self.value.pipeline_layout)
            .render_pass(self.value.render_pass)
            .subpass(0)
            .base_pipeline_handle(Pipeline::null())
            .base_pipeline_index(-1)
            ;

        self.value.pipeline = unsafe { logical_device.create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None) }.unwrap().0[0];

        unsafe {
            logical_device.destroy_shader_module(vert_module, None);
            logical_device.destroy_shader_module(frag_module, None);
        }
    }

    fn create_shader_module(&self, bytecode: &[u8]) -> anyhow::Result<ShaderModule> {
        let bytecode = Bytecode::new(bytecode).unwrap();

        let shader_info = ShaderModuleCreateInfo::builder()
            .code_size(bytecode.code_size())
            .code(bytecode.code())
            ;

        Ok(unsafe { self.logical_device.unwrap().create_shader_module(&shader_info, None) }.unwrap())
    }

    fn create_render_pass(&mut self) {
        let swapchain_data = self.swapchain_data.unwrap();
        let color_attachment = AttachmentDescription::builder()
            .format(swapchain_data.swapchain_format)
            .samples(SampleCountFlags::_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR)
            ;

        let color_attachment_ref = AttachmentReference::builder()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            ;

        let color_attachments = &[color_attachment_ref];
        let subpass = SubpassDescription::builder()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments)
            ;

        let dependency = SubpassDependency::builder()
            .src_subpass(SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(AccessFlags::empty())
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)
            ;

        let attachments = &[color_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let render_pass_info = RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies)
            ;

        self.value.render_pass = unsafe { self.logical_device.unwrap().create_render_pass(&render_pass_info, None) }.unwrap();
    }

    fn create_framebuffers(&mut self) {
        let swapchain_data = self.swapchain_data.unwrap();

        let framebuffers = swapchain_data.swapchain_image_views
            .iter()
            .map(|iv| {
                let attachments = &[*iv];
                let create_info = FramebufferCreateInfo::builder()
                    .attachments(attachments)
                    .render_pass(self.value.render_pass)
                    .width(swapchain_data.swapchain_extent.width)
                    .height(swapchain_data.swapchain_extent.height)
                    .layers(1)
                    ;

                unsafe { self.logical_device.unwrap().create_framebuffer(&create_info, None) }.unwrap()
            })
            .collect::<Vec<_>>();

        self.value.framebuffers = framebuffers;
    }

    fn create_command_pool(&self) -> CommandPool {
        let indices = QueueFamilyIndices::get(self.instance.unwrap(), self.physical_device, self.surface).unwrap();

        let create_info = CommandPoolCreateInfo::builder()
            .queue_family_index(indices.graphics)
            .flags(CommandPoolCreateFlags::TRANSIENT)
            ;

        unsafe { self.logical_device.unwrap().create_command_pool(&create_info, None) }.unwrap()
    }

    fn create_command_pools(&mut self) {
        self.value.global_command_pool = self.create_command_pool();

        for _ in 0..self.swapchain_data.unwrap().swapchain_images.len() {
            let command_pool = self.create_command_pool();
            self.value.command_pools.push(command_pool);
        }
    }

    fn create_command_buffers(&mut self) {
        let logical_device = self.logical_device.unwrap();

        for image_index in 0..self.swapchain_data.unwrap().swapchain_images.len() {
            let allocate_info = CommandBufferAllocateInfo::builder()
                .level(CommandBufferLevel::PRIMARY)
                .command_pool(self.value.command_pools[image_index])
                .command_buffer_count(1)
                ;

            let command_buffers = unsafe { logical_device.allocate_command_buffers(&allocate_info) }.unwrap();
            self.value.command_buffers.push(command_buffers);
        }
    }

    //Vertex buffer
    fn create_vertex_buffer(&mut self) {
        let logical_device = self.logical_device.unwrap();

        let size = (size_of::<Vertex>() * VERTICES.len()) as u64;
        let (staging_buffer, staging_buffer_memory) = self.create_buffer(size, BufferUsageFlags::TRANSFER_SRC, MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE).unwrap();

        unsafe { logical_device.bind_buffer_memory(staging_buffer, staging_buffer_memory, 0) }.unwrap();

        let app_memory = unsafe { logical_device.map_memory(staging_buffer_memory, 0, size, MemoryMapFlags::empty()) }.unwrap();

        unsafe {
            copy_nonoverlapping(VERTICES.as_ptr(), app_memory.cast(), VERTICES.len());
            logical_device.unmap_memory(staging_buffer_memory);
        }

        let (buffer, buffer_memory) = self.create_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER, MemoryPropertyFlags::DEVICE_LOCAL).unwrap();

        self.value.vertex_buffer = buffer;
        self.value.vertex_buffer_memory = buffer_memory;

        self.copy_buffer(staging_buffer, self.value.vertex_buffer, size);

        unsafe {
            logical_device.destroy_buffer(staging_buffer, None);
            logical_device.free_memory(staging_buffer_memory, None);
        }
    }

    fn get_memory_type_index(&self, properties: MemoryPropertyFlags, requirements: MemoryRequirements) -> anyhow::Result<u32> {
        let instance = self.instance.unwrap();
        let physical_device = self.physical_device;

        let memory = unsafe { instance.get_physical_device_memory_properties(physical_device) };

        (0..memory.memory_type_count)
            .find(|i| {
                let suitable = (requirements.memory_type_bits & (1u32 << i)) != 0;
                let memory_type = memory.memory_types[*i as usize];
                suitable && memory_type.property_flags.contains(properties)
            })
            .ok_or_else(|| anyhow!("Failed to find suitable memory type"))
    }

    fn create_buffer(&self, size: DeviceSize, usage: BufferUsageFlags, properties: MemoryPropertyFlags) -> anyhow::Result<(Buffer, DeviceMemory)> {
        let logical_device = self.logical_device.unwrap();

        let buffer_info = BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE)
        ;

        let buffer = unsafe { logical_device.create_buffer(&buffer_info, None) }?;
        let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };

        let memory_info = MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(self.get_memory_type_index(properties, requirements).unwrap());

        let buffer_memory = unsafe { logical_device.allocate_memory(&memory_info, None) }?;

        unsafe { logical_device.bind_buffer_memory(buffer, buffer_memory, 0) }.unwrap();

        Ok((buffer, buffer_memory))
    }

    fn copy_buffer(&self, src: Buffer, dst: Buffer, size: DeviceSize) {
        let logical_device = self.logical_device.unwrap();

        let info = CommandBufferAllocateInfo::builder()
            .level(CommandBufferLevel::PRIMARY)
            .command_pool(self.value.global_command_pool)
            .command_buffer_count(1)
        ;

        let command_buffer = unsafe { logical_device.allocate_command_buffers(&info) }.unwrap()[0];
        let begin_info = CommandBufferBeginInfo::builder()
            .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { logical_device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();

            let regions = BufferCopy::builder().size(size);
        unsafe { logical_device.cmd_copy_buffer(command_buffer, src, dst, &[regions]); }

        unsafe { logical_device.end_command_buffer(command_buffer) }.unwrap();

            let command_buffers = &[command_buffer];
            let info = SubmitInfo::builder()
                .command_buffers(command_buffers)
            ;

        unsafe { logical_device.queue_submit(self.graphics_queue, &[info], Fence::null()) }.unwrap();
        unsafe { logical_device.queue_wait_idle(self.graphics_queue)}.unwrap();

        unsafe { logical_device.free_command_buffers(self.value.global_command_pool, &[command_buffer]); }
    }

    //Index buffer
    //ToDo: Unify with Vertex buffer creation
    fn create_index_buffer(&mut self) {
        let logical_device = self.logical_device.unwrap();

        let size = (size_of::<u16>() * INDICES.len()) as u64;
        let (staging_buffer, staging_buffer_memory) = self.create_buffer(size, BufferUsageFlags::TRANSFER_SRC, MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE).unwrap();

        unsafe { logical_device.bind_buffer_memory(staging_buffer, staging_buffer_memory, 0) }.unwrap();

        let app_memory = unsafe { logical_device.map_memory(staging_buffer_memory, 0, size, MemoryMapFlags::empty()) }.unwrap();

        unsafe {
            copy_nonoverlapping(INDICES.as_ptr(), app_memory.cast(), INDICES.len());
            logical_device.unmap_memory(staging_buffer_memory);
        }

        let (buffer, buffer_memory) = self.create_buffer(size, BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER, MemoryPropertyFlags::DEVICE_LOCAL).unwrap();

        self.value.index_buffer = buffer;
        self.value.index_buffer_memory = buffer_memory;

        self.copy_buffer(staging_buffer, self.value.index_buffer, size);

        unsafe {
            logical_device.destroy_buffer(staging_buffer, None);
            logical_device.free_memory(staging_buffer_memory, None);
        }
    }

    //Uniform Buffers
    fn create_descriptor_set_layout(&mut self) {
        let ubo_binding = DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(ShaderStageFlags::VERTEX);

        let bindings = &[ubo_binding];
        let info = DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
        ;

        self.value.descriptor_set_layout = unsafe { self.logical_device.unwrap().create_descriptor_set_layout(&info, None) }.unwrap();
    }

    fn destroy_uniform_buffers(&mut self) {
        unsafe {
            for i in 0..self.value.uniform_buffers.len() {
                self.logical_device.unwrap().destroy_buffer(self.value.uniform_buffers[i], None);
                self.logical_device.unwrap().free_memory(self.value.uniform_buffers_memory[i], None);
            }
        }


        self.value.uniform_buffers.clear();
        self.value.uniform_buffers_memory.clear();
    }

    fn create_uniform_buffers(&mut self) {
        self.destroy_uniform_buffers();

        self.swapchain_data.unwrap().swapchain_images.iter().for_each(|_| {
            let size = size_of::<Transformation>() as u64;
            let (uniform_buffer, uniform_buffer_memory) = self.create_buffer(size, BufferUsageFlags::UNIFORM_BUFFER,
                                                                             MemoryPropertyFlags::HOST_COHERENT | MemoryPropertyFlags::HOST_VISIBLE).unwrap();

            self.value.uniform_buffers.push(uniform_buffer);
            self.value.uniform_buffers_memory.push(uniform_buffer_memory);
        });
    }

    fn create_descriptor_pool(&mut self) {
        let pool_size = DescriptorPoolSize::builder()
            .type_(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(self.swapchain_data.unwrap().swapchain_images.len() as u32)
        ;

        let pool_sizes = &[pool_size];
        let info = DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(self.swapchain_data.unwrap().swapchain_images.len() as u32);

        self.value.descriptor_pool = unsafe { self.logical_device.unwrap().create_descriptor_pool(&info, None) }.unwrap();
    }

    fn create_descriptor_sets(&mut self) {
        let layouts = vec![self.value.descriptor_set_layout; self.swapchain_data.unwrap().swapchain_images.len()];
        let info = DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.value.descriptor_pool)
            .set_layouts(&layouts)
        ;

        self.value.descriptor_sets = unsafe { self.logical_device.unwrap().allocate_descriptor_sets(&info) }.unwrap();

        for i in 0..self.swapchain_data.unwrap().swapchain_images.len() {
            let info = DescriptorBufferInfo::builder()
                .buffer(self.value.uniform_buffers[i])
                .offset(0)
                .range(size_of::<Transformation>() as u64)
            ;

            let buffer_infos = &[info];
            let write_info = WriteDescriptorSet::builder()
                .dst_set(self.value.descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_infos)
            ;

            unsafe { self.logical_device.unwrap().update_descriptor_sets(&[write_info], &[] as &[CopyDescriptorSet]) }
        }
    }
}