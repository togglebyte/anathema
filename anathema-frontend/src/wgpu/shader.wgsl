// -----------------------------------------------------------------------------
//   - Vertex -
// -----------------------------------------------------------------------------
struct SpriteData {
    @location(5) size: vec2<f32>,
    @location(6) offset: vec2<f32>,
    @location(7) c1: vec4<f32>,
    @location(8) c2: vec4<f32>,
    @location(9) c3: vec4<f32>,
    @location(10) c4: vec4<f32>,
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

    @location(1) clip_w: f32,
}

@vertex
fn vs_main(model: VertexInput, sprite: SpriteData) -> VertexOutput {
    var out: VertexOutput;


    // ignore sprite matrix/uvs:
    let world_pos = vec4<f32>(model.pos, 0.0, 1.0);
    out.clip_position = projection * world_pos;
    out.tex_coords = model.tex_coords;
    out.clip_w = out.clip_position.w;
    return out;


    // var vertex_pos = vec4<f32>(model.pos, 0.0, 1.0);
    // var sprite_mat = mat4x4<f32>(
    //     sprite.c1,
    //     sprite.c2,
    //     sprite.c3,
    //     sprite.c4,
    // );

    // let model_matrix = sprite_mat * vertex_pos;
    // out.clip_position = projection * model_matrix;

    // out.tex_coords = 
    //     // sprite coordinates from the vertex
    //     model.tex_coords 
    //     // Multiply with the tile size
    //     * sprite.size 
    //     // And set the offset for the selected tile
    //     + sprite.offset
    //     ;

    // // debug
    // out.clip_w = (projection * model_matrix).w;

    // return out;
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
    let c = clamp(in.clip_w / 1000.0, 0.0, 1.0);
    return vec4(vec3(c), 1.0);
}

//@fragment
//fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
//}
