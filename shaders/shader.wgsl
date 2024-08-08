struct VertexInput {
   @location(0) position: vec2<f32>,
}

struct Uniforms {
  transform: mat4x4<f32>,
}
@group(0) @binding(0) 
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    var pos = uniforms.transform * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    out.clip_position = pos;
    // Color based on distance from the center
    var d = length(in.position);
    out.color = vec4<f32>(d, 1.0 - d, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(
    // @builtin(position) in_position: vec4<f32>,
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return in.color;
}
 

