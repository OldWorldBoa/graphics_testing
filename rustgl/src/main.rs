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

pub mod shader;
use shader::{fragment_shader, shader_builder, vertex_shader};

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
const VDATA_LEN: i32 = 8;

static mut TEX_SHIFT: f32 = 0.0;

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
    let vertices: Vec<f32> = vec![
        -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, //v
        0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, //v
        0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, //v
        -0.5, 0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, //v
    ];
    let indices: Vec<u32> = vec![
        0, 1, 2, // t1
        0, 2, 3, // t2
    ];
    let vao = create_vertex_array(vertices);
    let ebo = create_element_buffer(indices);
    unsafe {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    }
    let stone_wall = load_img("res/stone-wall.jpg");
    let troll_face = load_img("res/troll-face.jpg");

    let shader_vcolour = shader_builder::create_shader_program(
        vertex_shader::VSHADER_CODE.as_bytes(),
        fragment_shader::FS_VERTEX.as_bytes(),
    );

    while !window.should_close() {
        // events
        // -----
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(shader_vcolour);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, stone_wall);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, troll_face);

            gl::Uniform1i(get_uniform_location(shader_vcolour, "texture1"), 0);
            gl::Uniform1i(get_uniform_location(shader_vcolour, "texture2"), 1);
            gl::Uniform1f(get_uniform_location(shader_vcolour, "tex_shift"), TEX_SHIFT);

            // Draw Triangles
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }

        // glfw: swap buffers and poll IO events (keys pressed/re leased, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn get_uniform_location(shader_program: u32, name: &str) -> i32 {
    let id = CString::new(name).unwrap();
    let casted = id.as_ptr() as *const GLchar;

    unsafe { gl::GetUniformLocation(shader_program, casted) }
}

fn load_img(file_uri: &str) -> u32 {
    let texture = ImageReader::open(file_uri).unwrap();
    let tex_dynim = texture.decode().unwrap();
    let mut tex_bytes: Vec<u16> = tex_dynim.to_rgb16().into_raw();
    tex_bytes.reverse();

    let mut tex_addr: u32 = 0;

    unsafe {
        gl::GenTextures(1, &mut tex_addr);
        gl::BindTexture(gl::TEXTURE_2D, tex_addr);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB4 as GLint,
            tex_dynim.width().try_into().unwrap(),
            tex_dynim.height().try_into().unwrap(),
            0,
            gl::RGB,
            gl::UNSIGNED_SHORT,
            tex_bytes.as_ptr().cast(),
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    tex_addr
}

fn create_element_buffer(indices: Vec<u32>) -> u32 {
    let mut ebo: u32 = 0;

    unsafe {
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &indices[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );
    }

    ebo
}

fn create_vertex_array(vertices: Vec<f32>) -> u32 {
    let mut vao: u32 = 0;
    let mut vbo: u32 = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            ((VDATA_LEN as usize) * vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        // Position
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            VDATA_LEN * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // Colour
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            VDATA_LEN * mem::size_of::<GLfloat>() as GLsizei,
            (3 * mem::size_of::<GLfloat>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        // Texture Coords
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            VDATA_LEN * mem::size_of::<GLfloat>() as GLsizei,
            (6 * mem::size_of::<GLfloat>()) as *const _,
        );
        gl::EnableVertexAttribArray(2);
    }

    vao
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
            glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => unsafe {
                TEX_SHIFT += 0.2;
                TEX_SHIFT = TEX_SHIFT.min(1.0);
            },
            glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => unsafe {
                TEX_SHIFT -= 0.2;
                TEX_SHIFT = TEX_SHIFT.max(0.0);
            },
            _ => {}
        }
    }
}
