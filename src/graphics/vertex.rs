use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct CustomVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

impl CustomVertex {
    pub fn new(x: f32, y: f32) -> Self {
        Self { position: [x, y] }
    }
}

pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
        #version 460

        layout(location = 0) in vec2 position;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }

        "
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
        #version 460

        layout(location = 0) out vec4 f_color;

        void main() {
            f_color = vec4(1.0, 0.0, 0.0, 1.0);
        }
        "
    }
}
