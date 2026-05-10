struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

struct MaterialUniform {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    _padding1: f32,
    _padding2: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> material: MaterialUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(input.position, 1.0);
    out.clip_position = camera.view_projection * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((model.normal_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let light_color = vec3<f32>(1.0, 0.98, 0.95);
    let ambient = vec3<f32>(0.15, 0.15, 0.2);

    let normal = normalize(input.world_normal);
    let view_dir = normalize(camera.camera_position.xyz - input.world_position);
    let half_dir = normalize(light_dir + view_dir);

    let ndotl = max(dot(normal, light_dir), 0.0);
    let ndoth = max(dot(normal, half_dir), 0.0);

    let base_color = material.base_color.rgb * input.color.rgb;
    let roughness = material.roughness;
    let metallic = material.metallic;

    let diffuse = base_color * (1.0 - metallic);
    let specular_strength = mix(0.04, 1.0, metallic);
    let shininess = mix(8.0, 256.0, 1.0 - roughness);
    let specular = vec3<f32>(specular_strength) * pow(ndoth, shininess);

    let color = ambient * base_color + (diffuse + specular) * light_color * ndotl;

    let tone_mapped = color / (color + vec3<f32>(1.0));
    let gamma_corrected = pow(tone_mapped, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(gamma_corrected, material.base_color.a * input.color.a);
}
