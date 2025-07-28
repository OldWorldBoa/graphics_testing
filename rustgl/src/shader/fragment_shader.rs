pub const FS_VERTEX: &str = r#"#version 330 core
    in vec4 vertex_colour;
    in vec2 tex_coord;

    out vec4 frag_colour;

    uniform vec4 colour_shift;
    uniform sampler2D texture_data;

    void main() {
        frag_colour = texture(texture_data, tex_coord) * vertex_colour;
    }
"#;
