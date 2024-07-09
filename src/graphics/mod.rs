use std::sync::Arc;

use vulkano::{
    device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags},
    instance::{Instance, InstanceCreateInfo},
    VulkanLibrary,
};

pub struct Graphics {
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl Graphics {
    pub fn init() -> Self {
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

        Self { device, queue }
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }
}
