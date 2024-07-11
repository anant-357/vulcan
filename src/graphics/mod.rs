mod mandel_brot;
pub mod vertex;

use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use mandel_brot::mandel_brot_shader;
use vertex::fragment_shader;
use vertex::vertex_shader;
use vertex::CustomVertex;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::command_buffer::SubpassBeginInfo;
use vulkano::command_buffer::SubpassEndInfo;
use vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::vertex_input::VertexDefinition;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::RenderPass;
use vulkano::render_pass::Subpass;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, ClearColorImageInfo, CopyImageToBufferInfo,
        PrimaryAutoCommandBuffer,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags},
    format::ClearColorValue,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    sync::{self, GpuFuture},
    VulkanLibrary,
};

pub struct Graphics
// <T>
// where
//     T: BufferContents + Debug + Eq,
//     Rgba<T>: Pixel,
{
    width: u32,
    height: u32,
    device: Arc<Device>,
    queue: Arc<Queue>,
    mem_alloc: Arc<StandardMemoryAllocator>,
    command_buffer_alloc: Arc<StandardCommandBufferAllocator>,
    descriptor_set_alloc: Arc<StandardDescriptorSetAllocator>,
    source: Option<Subbuffer<[u8]>>,
    destination: Option<Subbuffer<[u8]>>,
    vertex: Option<Subbuffer<[CustomVertex]>>,
    command: Option<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
    image: Option<Arc<Image>>,
    compute_pipeline: Option<Arc<ComputePipeline>>,
    graphics_pipeline: Option<Arc<GraphicsPipeline>>,
    descriptor_set: Option<Arc<PersistentDescriptorSet>>,
    render_pass: Option<Arc<RenderPass>>,
    frame_buffer: Option<Arc<Framebuffer>>,
}

impl Graphics
// <T>
// where
//     T: BufferContents + Debug + Eq,
//     Rgba<T>: Pixel,
{
    pub fn init(width: u32, height: u32) -> Result<Self, String> {
        let lib = VulkanLibrary::new().expect("Failed to find local vulkan!");
        let instance =
            Instance::new(lib, InstanceCreateInfo::default()).expect("Failed to create instance");

        let physical_device = instance
            .enumerate_physical_devices()
            .expect("Couldn't find any device")
            .next()
            .expect("No device");

        let graphical_queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_, queue_family_properties)| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .expect("No graphical queue family") as u32;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: graphical_queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("Failed to create device");
        let queue = queues.next().unwrap();
        let mem_alloc = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_alloc = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));
        let descriptor_set_alloc = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Ok(Self {
            width,
            height,
            device,
            queue,
            mem_alloc,
            command_buffer_alloc,
            descriptor_set_alloc,
            source: None,
            destination: None,
            vertex: None,
            command: None,
            image: None,
            compute_pipeline: None,
            graphics_pipeline: None,
            descriptor_set: None,
            render_pass: None,
            frame_buffer: None,
        })
    }

    pub fn set_destination_buffer(&mut self, destination_content: u32) {
        let destination = Buffer::from_iter(
            self.mem_alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            (0..self.width * self.height * destination_content).map(|_| 0u8),
        )
        .expect("failed to create destination buffer");
        self.destination = Some(destination);
    }

    pub fn set_vertex_buffer(&mut self, vertex_content: Vec<CustomVertex>) {
        let buffer = Buffer::from_iter(
            self.mem_alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertex_content,
        )
        .unwrap();

        self.vertex = Some(buffer);
    }

    pub fn create_image(&mut self) {
        let image = Image::new(
            self.mem_alloc.clone(),
            ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: vulkano::format::Format::R8G8B8A8_UNORM,
                extent: [self.width, self.height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT
                    | ImageUsage::STORAGE
                    | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();

        self.image = Some(image);
    }

    pub fn set_descriptor_set_copy_to_buffer_command_buffer(&mut self) {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_alloc,
            self.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .bind_pipeline_compute(self.compute_pipeline.clone().unwrap())
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute,
                self.compute_pipeline.clone().unwrap().layout().clone(),
                0,
                self.descriptor_set.clone().unwrap(),
            )
            .unwrap()
            .dispatch([self.width / 8, self.height / 8, 1])
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                self.image.clone().unwrap(),
                self.destination.clone().unwrap(),
            ))
            .unwrap();

        self.command = Some(builder.build().unwrap());
    }

    pub fn set_clear_image_copy_to_buffer_command_buffer(&mut self, color: ClearColorValue) {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_alloc,
            self.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .clear_color_image(ClearColorImageInfo {
                clear_value: color,
                ..ClearColorImageInfo::image(self.image.clone().unwrap().clone())
            })
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                self.image.clone().unwrap().clone(),
                self.destination.clone().unwrap().clone(),
            ))
            .unwrap();
        self.command = Some(builder.build().unwrap());
    }

    pub fn set_draw_image_copy_to_buffer_command(&mut self) {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_alloc,
            self.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(self.frame_buffer.clone().unwrap())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .bind_pipeline_graphics(self.graphics_pipeline.clone().unwrap())
            .unwrap()
            .bind_vertex_buffers(0, self.vertex.clone().unwrap())
            .unwrap()
            .draw(3, 1, 0, 0)
            .unwrap()
            .end_render_pass(SubpassEndInfo::default())
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                self.image.clone().unwrap(),
                self.destination.clone().unwrap(),
            ))
            .unwrap();

        self.command = Some(builder.build().unwrap());
    }

    pub fn set_compute_pipeline(&mut self) {
        let mb_shader =
            mandel_brot_shader::load(self.device.clone()).expect("failed to create shader module");
        let cs = mb_shader.entry_point("main").unwrap();
        let stage = PipelineShaderStageCreateInfo::new(cs);
        let layout = PipelineLayout::new(
            self.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(self.device.clone())
                .unwrap(),
        )
        .expect("failed to create pipeline layout");

        let compute_pipeline = ComputePipeline::new(
            self.device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .expect("failed to create compute pipeline");
        self.compute_pipeline = Some(compute_pipeline);
    }

    pub fn set_graphics_pipeline(&mut self) {
        let vs = vertex_shader::load(self.device.clone())
            .expect("failed to create vertex shader")
            .entry_point("main")
            .unwrap();
        let fs = fragment_shader::load(self.device.clone())
            .expect("failed to create fragment shader")
            .entry_point("main")
            .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [self.width as f32, self.height as f32],
            depth_range: 0.0..=1.0,
        };

        let vertex_input_state = CustomVertex::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            self.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.device.clone())
                .unwrap(),
        )
        .unwrap();

        let subpass = Subpass::from(self.render_pass.clone().unwrap(), 0).unwrap();
        let graphics_pipeline = GraphicsPipeline::new(
            self.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap();

        self.graphics_pipeline = Some(graphics_pipeline);
    }

    pub fn add_descriptor_set_for_image(&mut self) {
        let view = ImageView::new_default(self.image.clone().unwrap()).unwrap();
        let binding = self.compute_pipeline.clone().unwrap();
        let layout = binding.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_alloc,
            layout.clone(),
            [WriteDescriptorSet::image_view(0, view.clone())],
            [],
        )
        .unwrap();

        self.descriptor_set = Some(set);
    }

    pub fn add_render_pass(&mut self) {
        let render_pass = vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: vulkano::format::Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .unwrap();

        self.render_pass = Some(render_pass);
    }

    pub fn add_framebuffer(&mut self) {
        let view = ImageView::new_default(self.image.clone().unwrap()).unwrap();
        let frame_buffer = Framebuffer::new(
            self.render_pass.clone().unwrap(),
            vulkano::render_pass::FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },
        )
        .unwrap();
        self.frame_buffer = Some(frame_buffer);
    }

    pub fn sync(&self) {
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), self.command.clone().unwrap())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
    }

    pub fn verify(&self) {
        assert_eq!(
            &*self.source.clone().unwrap().read().unwrap(),
            &*self.destination.clone().unwrap().read().unwrap()
        );
        println!("Everything succeeded!");
    }

    pub fn save_image(&mut self, file_name: &str) {
        let binding = self.destination.clone().unwrap();
        let buffer_content = binding.read().unwrap();
        let image =
            ImageBuffer::<Rgba<u8>, _>::from_raw(self.width, self.height, &buffer_content[..])
                .unwrap();
        image.save(file_name).unwrap();
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }
}
