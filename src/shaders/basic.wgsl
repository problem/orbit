struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    base_color: vec4<f32>,
    light_view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var shadow_map: texture_depth_2d;
@group(1) @binding(1) var shadow_sampler: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) light_space_pos: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_pos;
    out.world_normal = normalize((uniforms.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_position = world_pos.xyz;
    out.light_space_pos = uniforms.light_view_proj * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Flat unlit mode for wireframe overlay
    if uniforms.base_color.a < 0.99 {
        return vec4<f32>(uniforms.base_color.rgb, 1.0);
    }

    // Directional lighting
    let sun_dir = normalize(vec3<f32>(0.5, 0.8, 1.0));
    let ambient = 0.2;
    let diffuse = max(dot(in.world_normal, sun_dir), 0.0);

    // Shadow calculation
    let light_ndc = in.light_space_pos.xyz / in.light_space_pos.w;
    // Convert from NDC [-1,1] to texture coords [0,1]
    let shadow_uv = vec2<f32>(light_ndc.x * 0.5 + 0.5, -light_ndc.y * 0.5 + 0.5);
    let shadow_depth = light_ndc.z;

    var shadow = 1.0;
    // Only apply shadow within the shadow map bounds
    if shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 && shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0 && shadow_depth >= 0.0 && shadow_depth <= 1.0 {
        shadow = textureSampleCompare(shadow_map, shadow_sampler, shadow_uv, shadow_depth);
    }

    let light = ambient + diffuse * 0.8 * shadow;
    let color = uniforms.base_color.rgb * light;
    return vec4<f32>(color, 1.0);
}
