extern crate glfw;
use gl::GenBuffers;

use self::glfw::{Action, Context, Key};

extern crate gl;

use std::sync::mpsc::Receiver;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

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

    // render loop
    // -----------
    while !window.should_close() {
        // events
        // -----
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Setup buffers
            let mut vao = 0; // Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            assert_ne!(vao, 0);

            let mut vbo = 0; // Vertex Buffer Object
            gl::GenBuffers(1, &mut vbo);
            assert_ne!(vbo, 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Setup Geometry
            type Vertex = [f32; 3];
            const VERTICES: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&VERTICES) as isize,
                VERTICES.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>().try_into().unwrap(),
                0 as *const _,
            );
            gl::EnableVertexAttribArray(0);

            // Create Shaders
            attach_vertex_shader();
            attach_fragment_shader();
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn attach_vertex_shader() {
    unsafe {
        let vshader = gl::CreateShader(gl::VERTEX_SHADER);
        assert_ne!(vshader, 0);
        const vshader_code: &str = r#"#version 330 core
                layout (location = 0) in vec3 pos;;

                void main() {
                    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
                }
            "#;

        gl::ShaderSource(
            vshader,
            1,
            &(vshader_code.as_bytes().as_ptr().cast()),
            &(vshader_code.len().try_into().unwrap()),
        );

        gl::CompileShader(vshader);

        let mut success = 0;
        gl::GetShaderiv(vshader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(vshader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }
    }
}

fn attach_fragment_shader() {
    unsafe {
        let fshader = gl::CreateShader(gl::FRAGMENT_SHADER);
        assert_ne!(fshader, 0);
        const fshader_code: &str = r#"#version 330 core
                out vec4 final_color;

                void main() {
                    final_color = vec4(1.0, 0.5, 0.2, 1.0);
                }
            "#;

        gl::ShaderSource(
            fshader,
            1,
            &(fshader_code.as_bytes().as_ptr().cast()),
            &(fshader_code.len().try_into().unwrap()),
        );

        gl::CompileShader(fshader);

        let mut success = 0;
        gl::GetShaderiv(fshader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(fshader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }
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
