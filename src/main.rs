mod graphics;
use graphics::{vertex::CustomVertex, Graphics};

fn main() {
    let mut gfx = Graphics::init(2048 * 4, 2048 * 4).unwrap();
    println!(
        "Got device: {:#?} and queue: {:#?}",
        gfx.get_device(),
        gfx.get_queue()
    );
    gfx.create_image();
    gfx.add_render_pass();
    gfx.add_framebuffer();
    gfx.set_graphics_pipeline();
    // gfx.add_descriptor_set_for_image();
    // gfx.set_source_buffer((0..64).collect());
    gfx.set_vertex_buffer(vec![
        CustomVertex::new(-0.5, -0.5),
        CustomVertex::new(0.0, 0.5),
        CustomVertex::new(0.5, -0.25),
    ]);
    gfx.set_destination_buffer(4);
    gfx.set_draw_image_copy_to_buffer_command();
    // gfx.set_descriptor_set_copy_to_buffer_command_buffer();
    // gfx.set_clear_image_copy_to_buffer_command_buffer(vulkano::format::ClearColorValue::Float([
    //     1.0, 1.0, 1.0, 0.0,
    // ]));
    gfx.sync();
    // gfx.verify();
    gfx.save_image("triangle.png");
}
