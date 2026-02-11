struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn main(
    @builtin(vertex_index) vertexIndex: u32
) -> VertexOutput {
    
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),   // Top center
        vec2<f32>(-0.5, -0.5), // Bottom left
        vec2<f32>(0.5, -0.5)   // Bottom right
    );

    var out: VertexOutput;
    out.position = vec4<f32>(pos[vertexIndex], 0.0, 1.0);
    out.color = vec3<f32>(1.0, 0.0, 0.0);
    
    return out;
}