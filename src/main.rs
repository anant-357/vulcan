mod graphics;
use graphics::{vertex::CustomVertex, Graphics};

fn main() {
    let mut gfx = Graphics::init(2048 * 4, 2048 * 4).unwrap();
    println!(
        "Got device: {:#?} and queue: {:#?}",
        gfx.get_device(),
        gfx.get_queue()
    );
    gfx.set_compute_pipeline();
    gfx.create_image();
    gfx.add_descriptor_set_for_image();
    // gfx.set_source_buffer((0..64).collect());
    gfx.set_vertex_buffer(vec![
        CustomVertex::new(-0.5, -0.5),
        CustomVertex::new(0.0, 0.5),
        CustomVertex::new(0.5, -0.25),
    ]);
    gfx.set_destination_buffer(4);
    gfx.set_descriptor_set_copy_to_buffer_command_buffer();
    // gfx.set_clear_image_copy_to_buffer_command_buffer(vulkano::format::ClearColorValue::Float([
    //     1.0, 1.0, 1.0, 0.0,
    // ]));
    gfx.sync();
    // gfx.verify();
    gfx.save_image("hello4.png");
}
