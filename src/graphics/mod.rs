mod mandel_brot;

use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use mandel_brot::shader;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, ClearColorImageInfo, CopyImageToBufferInfo,
        PrimaryAutoCommandBuffer,
    },
    device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags},
    format::ClearColorValue,
    image::{Image, ImageCreateInfo, ImageUsage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo,
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
    device: Arc<Device>,
    queue: Arc<Queue>,
    mem_alloc: Arc<StandardMemoryAllocator>,
    command_buffer_alloc: Arc<StandardCommandBufferAllocator>,
    source: Option<Subbuffer<[u8]>>,
    destination: Option<Subbuffer<[u8]>>,
    command: Option<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
    image: Option<Arc<Image>>,
    compute_pipeline: Option<Arc<ComputePipeline>>,
}

impl Graphics
// <T>
// where
//     T: BufferContents + Debug + Eq,
//     Rgba<T>: Pixel,
{
    pub fn init() -> Result<Self, String> {
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

        Ok(Self {
            device,
            queue,
            mem_alloc,
            command_buffer_alloc,
            source: None,
            destination: None,
            command: None,
            image: None,
            compute_pipeline: None,
        })
    }

    pub fn set_source_buffer(&mut self, source_content: Vec<u8>) {
        let source = Buffer::from_iter(
            self.mem_alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            source_content,
        )
        .expect("failed to create source buffer");

        self.source = Some(source);
    }

    pub fn set_destination_buffer(&mut self, destination_content: Vec<u8>) {
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
            destination_content,
        )
        .expect("failed to create destination buffer");
        self.destination = Some(destination);
    }

    pub fn create_image(&mut self) {
        let image = Image::new(
            self.mem_alloc.clone(),
            ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: vulkano::format::Format::R8G8B8A8_UNORM,
                extent: [1024, 1024, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
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

    pub fn set_clear_image_command_buffer(&mut self, color: ClearColorValue) {
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

    pub fn set_compute_pipeline(&mut self) {
        let mb_shader = shader::load(self.device.clone()).expect("failed to create shader module");
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
        let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
        image.save(file_name).unwrap();
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }
}
