// -----------------------------------------------------------------------------
//   - Projection matrix -
// -----------------------------------------------------------------------------
@group(1) @binding(0)
var<uniform> projection: mat4x4<f32>;

// -----------------------------------------------------------------------------
//   - Font uniform -
// -----------------------------------------------------------------------------
struct Font {
    @location(0) transformation: mat4x4<f32>;
    @location(1) size: vec2<f32>;
}

@group(2) @binding(0)
var<uniform> font Font;

// -----------------------------------------------------------------------------
//   - Character -
// -----------------------------------------------------------------------------
struct Character {
    @location(0) pos: vec3<f32>,
    @location(1) offset: vec2<f32>,
}

// -----------------------------------------------------------------------------
//   - Vertex -
// -----------------------------------------------------------------------------
struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) idx: i32,
}

@vertex
fn vs_main(model: VertexInput, char: Character) -> VertexOutput {
    var out: VertexOutput;

    var vertex_pos = vec4<f32>(model.pos, 1.0, 1.0);
    var sprite_mat = font.transformation

    let model_matrix = sprite_mat * vertex_pos; // where do we add char.pos
    out.clip_position = projection * model_matrix;

    out.tex_coords = 
        // sprite coordinates from the vertex
        model.tex_coords 
        // And set the offset for the selected tile
        + char.offset
        ;

    return out;
}

// -----------------------------------------------------------------------------
//   - Fragment  -
// -----------------------------------------------------------------------------
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
