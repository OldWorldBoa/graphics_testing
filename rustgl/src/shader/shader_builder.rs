use gl::types::{GLchar, GLint};
use std::ffi::CString;
use std::ptr;

pub fn create_shader_program(v_code: &[u8], f_code: &[u8]) -> u32 {
    unsafe {
        // Create Vertex Shader
        let vshader_id = build_shader(gl::VERTEX_SHADER, v_code);
        let fshader_id = build_shader(gl::FRAGMENT_SHADER, f_code);

        // Link Shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vshader_id);
        gl::AttachShader(shader_program, fshader_id);
        gl::LinkProgram(shader_program);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);

        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                info_log.as_ptr() as *mut GLchar,
            );

            log_result("Unable to link program", info_log);
        }

        gl::DeleteShader(vshader_id);
        gl::DeleteShader(fshader_id);

        shader_program
    }
}

fn build_shader(shader_type: u32, code: &[u8]) -> u32 {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let shader_src = CString::new(code).unwrap();
        gl::ShaderSource(shader, 1, &shader_src.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut success = gl::FALSE as GLint;
        let mut info_log: Vec<u8> = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );

            log_result("Unable to copmile shader", info_log);
        }

        shader
    }
}

fn log_result(info_title: &str, info_log: Vec<u8>) {
    match info_log.iter().position(|&r| r == 0) {
        Some(idx) => match str::from_utf8(&info_log[0..idx]) {
            Ok(v) => println!("{info_title}: {v}"),
            Err(e) => println!("Unable to get vertex shader error: {e}"),
        },
        None => println!("There is no null terminator"),
    };
}
