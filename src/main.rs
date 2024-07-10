mod graphics;
use graphics::Graphics;

fn main() {
    let mut gfx = Graphics::<i32>::init().unwrap();
    println!(
        "Got device: {:#?} and queue: {:#?}",
        gfx.get_device(),
        gfx.get_queue()
    );
    gfx.set_source_buffer((0..64).collect());
    gfx.set_destination_buffer((0..64).map(|_| 0).collect());
    gfx.set_copy_command_buffer();
    gfx.create_image();
    gfx.sync();
    gfx.verify();
}
