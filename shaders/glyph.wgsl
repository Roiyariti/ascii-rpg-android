struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) background: f32,
};

@group(0) @binding(0) var atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex: u32,
    @location(0) origin: vec4<f32>,
    @location(1) axis_u: vec4<f32>,
    @location(2) axis_v: vec4<f32>,
    @location(3) uv_rect: vec4<f32>,
    @location(4) color: vec4<f32>,
) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0),
        vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0),
    );
    let corner = corners[vertex];
    var out: VertexOutput;
    out.position = vec4(origin.xyz + corner.x * axis_u.xyz + corner.y * axis_v.xyz, 1.0);
    out.uv = uv_rect.xy + corner * uv_rect.zw;
    out.color = color;
    out.background = origin.w;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let coverage = textureSample(atlas, atlas_sampler, input.uv).r;
    if coverage < 0.1 && input.background == 0.0 { discard; }
    let tint = mix(input.background, 1.0, coverage);
    return vec4(input.color.rgb * tint, input.color.a);
}
