use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    VulkanLibrary,
};

pub struct Graphics<T>
where
    T: BufferContents,
{
    device: Arc<Device>,
    queue: Arc<Queue>,
    mem_alloc: Arc<StandardMemoryAllocator>,
    source: Option<Subbuffer<[T]>>,
    destination: Option<Subbuffer<[T]>>,
}

impl<T> Graphics<T>
where
    T: BufferContents,
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
            .expect("No graphical queue family");

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: graphical_queue_family_index as u32,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("Failed to create device");
        let queue = queues.next().unwrap();
        let mem_alloc = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        Ok(Self {
            device,
            queue,
            mem_alloc,
            source: None,
            destination: None,
        })
    }

    pub fn set_source_buffer(&mut self, source_content: Vec<T>) {
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

    pub fn set_destination_buffer(&mut self, destination_content: Vec<T>) {
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

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }
}
