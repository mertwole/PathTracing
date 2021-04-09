extern crate gl;
use gl::types::*;

pub struct ShaderProgram{
    gl_id : GLuint
}

impl ShaderProgram{
    pub fn new() -> ShaderProgram {
        ShaderProgram { gl_id : unsafe { gl::CreateProgram() } }
    }

    pub fn compile(&self) -> GLuint{
        unsafe {
            gl::LinkProgram(self.gl_id);

            let log_len = &mut (0 as GLsizei);
            let mut log = [0 as GLchar; 512];
            gl::GetProgramInfoLog(self.gl_id, 512 as GLsizei, log_len, &mut log[0]);
            if *log_len != 0{
                panic!("log : {}", String::from_utf8((*(&log as *const [i8; 512] as *const [u8; 512])).to_vec()).unwrap());
            }
        }

        self.gl_id
    }

    pub fn add_frag(self, source : String) -> ShaderProgram{
        unsafe { &self.add_shader(source, gl::FRAGMENT_SHADER); } self
    }

    pub fn add_vert(self, source : String) -> ShaderProgram{
        unsafe { &self.add_shader(source, gl::VERTEX_SHADER); } self
    }

    unsafe fn add_shader(&self, source : String, shader_type : GLenum) {
        let shader = gl::CreateShader(shader_type);
        let source_ptr : *const *const GLchar = &(source.as_ptr() as *const GLchar);
        gl::ShaderSource(shader, 1 as GLsizei, source_ptr, &(source.len() as GLint));
        gl::CompileShader(shader);

        let log_len = &mut (0 as GLsizei);
        let mut log = [0 as GLchar; 512];
        gl::GetShaderInfoLog(shader, 512 as GLsizei, log_len, &mut log[0]);
        if *log_len != 0{
            panic!("log : {}", String::from_utf8((*(&log as *const [i8; 512] as *const [u8; 512])).to_vec()).unwrap());
        }

        gl::AttachShader(self.gl_id, shader);
    }
}