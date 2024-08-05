struct VertexInput {
   @location(0) position: vec2<f32>,
   @location(1) color: vec4<f32>,
}

struct Uniforms {
    offset: vec2<f32>,
    zoom: f32,
}
@group(0) @binding(0) 
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    var pos = in.position * uniforms.zoom + uniforms.offset;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

 

