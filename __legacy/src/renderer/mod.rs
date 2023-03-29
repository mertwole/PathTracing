use std::sync::Arc;

use gl::types::*;

mod shader_program;
use shader_program::ShaderProgram;

pub mod window;
use window::Window;

use crate::Framebuffer;

pub struct Renderer {
    width: u32,
    height: u32,
    window: Window,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Renderer {
        let window = Window::open(width, height);

        let frag_bytes = include_bytes!("shaders/screen_image.frag");
        let vert_bytes = include_bytes!("shaders/screen_image.vert");

        let frag_src = String::from_utf8_lossy(frag_bytes).to_string();
        let vert_src = String::from_utf8_lossy(vert_bytes).to_string();

        let program = ShaderProgram::new()
            .add_frag(frag_src)
            .add_vert(vert_src)
            .compile();

        let mut vao = 0 as GLuint;
        let mut vbo = 0 as GLuint;
        let mut screen_tex = 0 as GLuint;

        let quad_verts: [f32; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0];

        unsafe {
            gl::UseProgram(program);

            gl::GenVertexArrays(1, &mut vao as *mut GLuint);
            gl::GenBuffers(1, &mut vbo as *mut GLuint);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (quad_verts.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                quad_verts.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (2 * std::mem::size_of::<f32>()) as GLsizei,
                0 as *const GLvoid,
            );
            gl::EnableVertexAttribArray(0);

            gl::GenTextures(1, &mut screen_tex as *mut GLuint);
            gl::BindTexture(gl::TEXTURE_2D, screen_tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB32F as GLint,
                width as GLsizei,
                height as GLsizei,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
        }

        Renderer {
            window,
            width,
            height,
        }
    }

    pub fn render_from_raw(&mut self, pixels: *const u32) {
        unsafe {
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.width as GLsizei,
                self.height as GLsizei,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels as *const GLvoid,
            );
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }

        self.window.swap_buffers();
    }

    pub fn enter_rendering_loop(mut self, framebuffer: Arc<Framebuffer>) {
        loop {
            let _ = self.window.process_events();
            if self.window.should_close() {
                break;
            }

            if let Some(image_pointer) = framebuffer.get_image_pointer() {
                self.render_from_raw(image_pointer);
            }
        }
    }
}
