extern crate glfw;

use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr};

use self::glfw::{Action, Context, Key};

extern crate gl;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::sync::mpsc::Receiver;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const VSHADER_CODE: &str = r#"#version 330 core
    layout (location = 0) in vec3 pos;;

    void main() {
        gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    }
"#;

const FSHADER_CODE: &str = r#"#version 330 core
    out vec4 final_color;

    void main() {
        final_color = vec4(1.0, 0.5, 0.2, 1.0);
    }
"#;

pub fn main() {
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "RustGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader_program, vao) = unsafe {
        // Create Vertex Shader
        let vshader = gl::CreateShader(gl::VERTEX_SHADER);
        let vshader_src = CString::new(VSHADER_CODE.as_bytes()).unwrap();
        gl::ShaderSource(vshader, 1, &vshader_src.as_ptr(), ptr::null());
        gl::CompileShader(vshader);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderiv(vshader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                vshader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "Error compiling vertex shader:{}",
                str::from_utf8(&info_log).unwrap()
            );
        }

        // Create Fragment Shader
        let fshader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fshader_src = CString::new(FSHADER_CODE.as_bytes()).unwrap();
        gl::ShaderSource(fshader, 1, &fshader_src.as_ptr(), ptr::null());
        gl::CompileShader(fshader);
        gl::GetShaderiv(fshader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                fshader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "Error compiling fragment shader:{}",
                str::from_utf8(&info_log).unwrap()
            );
        }

        // Link Shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vshader);
        gl::AttachShader(shader_program, fshader);
        gl::LinkProgram(shader_program);

        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "Error linking shader program:{}",
                str::from_utf8(&info_log).unwrap()
            );
        }
        gl::DeleteShader(vshader);
        gl::DeleteShader(fshader);
        gl::DeleteProgram(shader_program);

        // Create Geometry
        let vertices: [f32; 18] = [
            0.0, 0.0, 0.0, -0.1, 0.0, 0.0, 0.0, -0.1, 0.0, // triangle 1
            0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1, 0.0, // triangle 2
        ];
        let (mut vao, mut vbo) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenVertexArrays(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        (shader_program, vao)
    };

    // render loop
    // -----------
    while !window.should_close() {
        // events
        // -----
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw Triangles
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(vao);
            gl::UseProgram(shader_program);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

// NOTE: not the same version as in common.rs!
fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
