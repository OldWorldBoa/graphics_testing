pub const VSHADER_CODE: &str = r#"#version 330 core
    layout (location = 0) in vec3 pos;
    layout (location = 1) in vec3 colour;
    layout (location = 2) in vec2 v_tex_coord;

    out vec4 vertex_colour;
    out vec2 tex_coord;

    void main() {
        gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
        vertex_colour = vec4(colour.r, colour.g, colour.b, 1.0);
        tex_coord = v_tex_coord;
    }
"#;
