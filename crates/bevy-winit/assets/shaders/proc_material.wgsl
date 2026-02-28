#import bevy_pbr::forward_io::VertexOutput

struct MyMaterial {
    frequency: f32,
    time: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: MyMaterial;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.0, 0.0);

    let uv = mesh.uv;

    let d = distance(uv, center);

    let r = 0.5 + 0.5 * sin(uv.x * material.frequency + material.time);
    let g = 0.5 + 0.5 * cos(uv.y * material.frequency + material.time);
    let b = 0.5 + 0.5 * tanh(d * material.frequency + material.time);

    return vec4<f32>(r, g, b, 1.0);
}
