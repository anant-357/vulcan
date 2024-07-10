mod graphics;
use graphics::Graphics;

fn main() {
    let mut gfx = Graphics::<u8>::init().unwrap();
    println!(
        "Got device: {:#?} and queue: {:#?}",
        gfx.get_device(),
        gfx.get_queue()
    );
    // gfx.set_source_buffer((0..64).collect());
    gfx.set_destination_buffer((0..1024 * 1024 * 4).map(|_| 0u8).collect());
    gfx.create_image();
    gfx.set_clear_image_copy_to_buffer_command_buffer(vulkano::format::ClearColorValue::Float([
        0.0, 0.0, 1.0, 1.0,
    ]));
    gfx.sync();
    gfx.verify();
}
