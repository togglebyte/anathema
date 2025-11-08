// -----------------------------------------------------------------------------
//   - Vertex -
// -----------------------------------------------------------------------------
struct SpriteData {
    @location(4) index: i32,
    @location(5) c1: vec4<f32>,
    @location(6) c2: vec4<f32>,
    @location(7) c3: vec4<f32>,
    @location(8) c4: vec4<f32>,
    @location(9) size: vec2<f32>,
    @location(10) offset: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> projection: mat4x4<f32>;

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
fn vs_main(model: VertexInput, sprite: SpriteData) -> VertexOutput {
    var out: VertexOutput;

    var vertex_pos = vec4<f32>(model.pos, 1.0, 1.0);
    var sprite_mat = mat4x4<f32>(
        sprite.c1,
        sprite.c2,
        sprite.c3,
        sprite.c4,
    );

    let model_matrix = sprite_mat * vertex_pos;
    out.clip_position = projection * model_matrix;

    out.tex_coords = 
        // sprite coordinates from the vertex
        model.tex_coords 
        // Multiply with the tile size
        * sprite.size 
        // And set the offset for the selected tile
        + sprite.offset
        ;

    out.idx = sprite.index;

    return out;
}

// -----------------------------------------------------------------------------
//   - Fragment  -
// -----------------------------------------------------------------------------
@group(0) @binding(0)
var t_diffuse: binding_array<texture_2d<f32>>;
// var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let index: i32 = in.idx;
    return textureSample(t_diffuse[index], s_diffuse, in.tex_coords);
}
