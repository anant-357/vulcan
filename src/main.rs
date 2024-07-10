mod graphics;
use graphics::Graphics;

fn main() {
    let mut gfx = Graphics::init(1920, 1080).unwrap();
    println!(
        "Got device: {:#?} and queue: {:#?}",
        gfx.get_device(),
        gfx.get_queue()
    );
    gfx.set_compute_pipeline();
    gfx.create_image();
    gfx.add_descriptor_set_for_image();
    // gfx.set_source_buffer((0..64).collect());
    gfx.set_destination_buffer(4);
    gfx.set_descriptor_set_copy_to_buffer_command_buffer();
    // gfx.set_clear_image_copy_to_buffer_command_buffer(vulkano::format::ClearColorValue::Float([
    //     1.0, 1.0, 1.0, 0.0,
    // ]));
    gfx.sync();
    // gfx.verify();
    gfx.save_image("hello4.png");
}
