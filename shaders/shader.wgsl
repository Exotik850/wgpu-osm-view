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
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.transform * vec4<f32>(in.position, 0.0, 1.0);
    // out.color = in.color;
    return out;
}

@fragment
fn fs_main(
    @builtin(position) in_position: vec4<f32>
) -> @location(0) vec4<f32> {
    return vec4<f32>(in_position.xy, 0.0, 1.0);
}
 

