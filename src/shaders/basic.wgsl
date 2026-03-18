struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    base_color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_pos;
    // Use the inverse-transpose (normal_matrix) for correct normals under non-uniform scaling
    out.world_normal = normalize((uniforms.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_position = world_pos.xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // When alpha < 1.0, output flat unlit color (used for wireframe overlay)
    if uniforms.base_color.a < 0.99 {
        return vec4<f32>(uniforms.base_color.rgb, 1.0);
    }
    // Simple directional lighting
    let sun_dir = normalize(vec3<f32>(0.5, 0.8, 1.0));
    let ambient = 0.15;
    let diffuse = max(dot(in.world_normal, sun_dir), 0.0);
    let light = ambient + diffuse * 0.85;
    let color = uniforms.base_color.rgb * light;
    return vec4<f32>(color, 1.0);
}
