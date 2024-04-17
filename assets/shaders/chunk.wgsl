#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_normal_local_to_world}
#import bevy_pbr::pbr_functions::{calculate_view, prepare_world_normal}
#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::pbr_types::pbr_input_new
#import bevy_pbr::prepass_utils

struct ChunkMaterial {
    reflectance: f32,
    perceptual_roughness: f32,
    metallic: f32,
};

@group(2) @binding(0) var<uniform> chunk_material: ChunkMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) vert_data: u32,
    // @location(1) blend_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) blend_color: vec3<f32>,
    @location(3) ambient: f32,
    @location(4) instance_index: u32,
};

// struct FragmentInput {
//     // @builtin(position) clip_position: vec4<f32>,
//     @location(0) blend_color: vec3<f32>,
//     @location(1) ambient: f32,
// };

var<private> ambient_lerps: vec4<f32> = vec4<f32>(1.0,0.7,0.5,0.15);

// indexing an array has to be in some memory
// by declaring this as a var instead it works
var<private> normals: array<vec3<f32>,6> = array<vec3<f32>,6> (
	vec3<f32>(-1.0, 0.0, 0.0), // Left
	vec3<f32>(1.0, 0.0, 0.0), // Right
	vec3<f32>(0.0, -1.0, 0.0), // Down
	vec3<f32>(0.0, 1.0, 0.0), // Up
	vec3<f32>(0.0, 0.0, -1.0), // Forward
	vec3<f32>(0.0, 0.0, 1.0) // Back
);

var<private> block_color: array<vec3<f32>,3> = array<vec3<f32>,3> (
	vec3<f32>(0.0, 0.0, 0.0), // air
	vec3<f32>(0.0, 1.0, 0.0), // grass
	vec3<f32>(0.3, 0.4, 0.0), // dirt
);


fn x_positive_bits(bits: u32) -> u32{
    return (1u << bits) - 1u;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let x = f32(vertex.vert_data & x_positive_bits(6u));
    let y = f32(vertex.vert_data >> 6u & x_positive_bits(6u));
    let z = f32(vertex.vert_data >> 12u & x_positive_bits(6u));
    let ao = vertex.vert_data >> 18u & x_positive_bits(3u);
    let normal_index = vertex.vert_data >> 21u & x_positive_bits(3u);
    let block_index = vertex.vert_data >> 25u & x_positive_bits(7u);
    // let normal_index: u32 = (vertex.v_pos_6b_normal_3b_texid_8b & 1835008u) >> 18u;

    let local_position = vec4<f32>(x,y,z, 1.0);
    let world_position = get_model_matrix(vertex.instance_index) * local_position;
    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        local_position,
    );

    let ambient_lerp = ambient_lerps[ao];
    out.ambient = ambient_lerp;
    out.world_position = world_position;
    // out.world_normal = vec3<f32>(0.0,1.0,0.0);

    let normal = normals[normal_index];
    out.world_normal = mesh_normal_local_to_world(normal, vertex.instance_index);

    let s = 0.05;
    var noise = simplexNoise2(vec2<f32>(world_position.x*s, world_position.z*s));
    var k = simplexNoise2(vec2<f32>(world_position.x*s, world_position.z*s));

    // let high = vec3<f32>(0.15, 1.0, 0.2);
    // let low = vec3<f32>(0.8, 1.0, 0.45);
    let high = vec3<f32>(9.00, 6.0, 0.0);
    let low = vec3<f32>(0.8, 1.0, 0.40);
    // let low = vec3<f32>(1.0, 0.0, 0.05);
    // let h = (out.world_position.y) / 32.0;
    noise = (out.world_position.y) / 30.0;
    // noise += k*0.2;
    // noise = smoothstep(0.4, 1.0, noise);
    // noise = 0.0;
    // noise += h;
    // noise *= noise;

    // noise = max(noise, 0.00);
    // noise = 0.0;

    // out.blend_color = (low * noise) + (high * (1.0-noise));
    let fun = (low * noise) + (high * (1.0-noise));
    out.blend_color = block_color[block_index];
    out.instance_index = vertex.instance_index;
    return out;
}

// struct FragmentInput {
//     @location(0) blend_color: vec4<f32>,
// };

@fragment
// fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
// fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
fn fragment(input: VertexOutput) -> FragmentOutput {
    var pbr_input = pbr_input_new();

    pbr_input.flags = mesh[input.instance_index].flags;

    pbr_input.V = calculate_view(input.world_position, false);
    pbr_input.frag_coord = input.clip_position;
    pbr_input.world_position = input.world_position;

    pbr_input.world_normal = prepare_world_normal(
        input.world_normal,
        false,
        false,
    );
#ifdef LOAD_PREPASS_NORMALS
    pbr_input.N = prepass_utils::prepass_normal(input.clip_position, 0u);
#else
    pbr_input.N = normalize(pbr_input.world_normal);
#endif

    pbr_input.material.base_color = vec4<f32>(input.blend_color * input.ambient, 1.0);

    pbr_input.material.reflectance = chunk_material.reflectance;
    pbr_input.material.perceptual_roughness = chunk_material.perceptual_roughness;
    pbr_input.material.metallic = chunk_material.metallic;
    // pbr_input.material.metallic = 1.0;


#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
   // return vec4<f32>(input.blend_color, 1.0);
    // return vec4<f32>(input.blend_color * input.ambient, 1.0);
    // return vec4<f32>(1.0, 0.0,0.0,1.0);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing
//
fn mod289(x: vec2f) -> vec2f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3f) -> vec3f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3f) -> vec3f {
    return mod289_3(((x * 34.) + 1.) * x);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn simplexNoise2(v: vec2f) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289(i); // Avoid truncation effects in permutation

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}