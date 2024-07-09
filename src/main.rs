mod graphics;
use graphics::Graphics;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

fn main() {
    let gfx = Graphics::init();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(gfx.get_device()));

    let data: i32 = 12;

    let buffer = Buffer::from_data(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        data,
    )
    .expect("Failed to create a buffer");

    println!("Created Buffer: {:#?}", buffer);
}
