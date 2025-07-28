pub const FS_VERTEX: &str = r#"#version 330 core
    in vec4 vertex_colour;
    in vec2 tex_coord;

    out vec4 frag_colour;

    uniform float tex_shift;

    uniform sampler2D texture1;
    uniform sampler2D texture2;

    void main() {
        frag_colour = mix(
            texture(texture1, tex_coord),
            texture(texture2, vec2(1.0 - tex_coord.x, 1.0 - tex_coord.y)),
            tex_shift
        ) * vertex_colour;
    }
"#;
