mod graphics;
use graphics::Graphics;

fn main() {
    let mut gfx = Graphics::<i32>::init().unwrap();
    gfx.set_source_buffer((0..64).collect());
    gfx.set_destination_buffer((0..64).map(|_| 0).collect());
}
