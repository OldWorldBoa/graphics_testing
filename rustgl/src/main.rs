extern crate glfw;

use self::glfw::{Action, Context, Key};
use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr};

extern crate gl;

use image::ImageReader;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::sync::mpsc::Receiver;
use std::time::Instant;

pub mod shader;
use shader::{fragment_shader, shader_builder, vertex_shader};

type Triangle = [i32; 3];
type Vertex = [f32; 8];

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

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // Vertices are 3f Position, 3f Colour, 2f Texture Coordinate
    let vertices: Vec<Vertex> = vec![
        [-0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        [0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
        [0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0],
        [-0.5, 0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0],
    ];
    let indices: Vec<Triangle> = vec![[0, 1, 2], [0, 2, 3]];

    let texture = load_img();

    let shader_vcolour = shader_builder::create_shader_program(
        vertex_shader::VSHADER_CODE.as_bytes(),
        fragment_shader::FS_VERTEX.as_bytes(),
    );

    let start = Instant::now();
    while !window.should_close() {
        // events
        // -----
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(shader_vcolour);

            let clr_dta = CString::new("colour_shift").unwrap();
            let clr_str: *const GLchar = clr_dta.as_ptr() as *const GLchar;
            let colour = (start.elapsed().as_secs_f32().sin() / 2f32) + 0.5;
            let colour_shift = gl::GetUniformLocation(shader_vcolour, clr_str);
            gl::Uniform4f(colour_shift, 0.0, colour, 1.0, 1.0);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            // Draw Triangles
            for vao in vaos.iter() {
                gl::BindVertexArray(*vao);
                gl::DrawArrays(gl::TRIANGLES, 0, 3);
            }
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn load_img() -> u32 {
    let texture = ImageReader::open("res/stone-wall.jpg").unwrap();
    let tex_dynim = texture.decode().unwrap();
    let tex_bytes: Vec<u8> = tex_dynim.to_rgb8().into_raw();

    let mut tex_addr: u32 = 0;

    unsafe {
        gl::GenTextures(1, &mut tex_addr);
        gl::BindTexture(gl::TEXTURE_2D, tex_addr);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB8 as GLint,
            tex_dynim.width().try_into().unwrap(),
            tex_dynim.height().try_into().unwrap(),
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            tex_bytes.as_ptr().cast(),
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    tex_addr
}

fn create_element_buffer(indices: Vec<Triangle>) -> u32 {
    let mut ebo: u32 = 0;

    unsafe {
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &indices[0],
            gl::STATIC_DRAW,
        );
    }

    ebo
}

fn gen_vaos(vertices: Vec<Vertex>) -> Vec<u32> {
    let mut vaos = vec![];
    let (mut vao, mut vbo) = (0, 0);

    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    }

    for vertex in vertices.iter() {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertex[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );

            // Position
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Colour
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<GLfloat>() as GLsizei,
                (3 * mem::size_of::<GLfloat>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // Texture Coords
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<GLfloat>() as GLsizei,
                (6 * mem::size_of::<GLfloat>()) as *const _,
            );
            gl::EnableVertexAttribArray(2);
        }

        vaos.push(vao);
    }

    vaos
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
