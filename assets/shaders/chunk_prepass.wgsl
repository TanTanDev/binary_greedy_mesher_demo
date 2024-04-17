#import bevy_pbr::{
    prepass_bindings,
    mesh_functions,
    prepass_io::{FragmentOutput},
    skinning,
    morph,
    mesh_view_bindings::{view, previous_view_proj},
}

#ifdef DEFERRED_PREPASS
#import bevy_pbr::rgb9e5
#endif

#import bevy_pbr::mesh_functions::{mesh_normal_local_to_world}
#import bevy_render::instance_index::{get_instance_index}


struct ChunkMaterial {
    reflectance: f32,
    perceptual_roughness: f32,
    metallic: f32,
    // _padding: f32,
};

@group(2) @binding(0) var<uniform> material: ChunkMaterial;

fn x_positive_bits(bits: u32) -> u32{
    return (1u << bits) - 1u;
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) vert_data: u32,
    // @location(0) position: vec3<f32>,
    // @location(0) vert_data: u32,
    // @location(1) blend_color: vec4<f32>,
};

struct MyVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    // @location(1) world_position: vec4<f32>,
    // @location(4) blend_color: vec3<f32>,
    // @location(5) ambient: f32,
};

// indexing an array has to be in some memory
// by declaring this as a var instead it works
var<private> normals: array<vec3<f32>,6> = array<vec3<f32>,6> (
	vec3<f32>(-1.0, 0.0, 0.0), // Left
	vec3<f32>(1.0, 0.0, 0.0), // Right
	vec3<f32>(0.0, -1.0, 0.0), // Down
	vec3<f32>(0.0, 1.0, 0.0), // Up
	vec3<f32>(0.0, 0.0, -1.0), // Back
	vec3<f32>(0.0, 0.0, 1.0) // Forward
);

@vertex
fn vertex(vertex: Vertex) -> MyVertexOutput {
    var out: MyVertexOutput;

    let x = f32((vertex.vert_data & 63u));
    let y = f32((vertex.vert_data & 4032u) >> 6u);
    let z = f32((vertex.vert_data & 258048u) >> 12u);
    let ao = u32((vertex.vert_data & (3u << 18u)) >> 18u);
    let normal_index = vertex.vert_data >> 21u & x_positive_bits(3u);


    // let ambient_lerp = ambient_lerps[ao];
    // out.ambient = ambient_lerp;
    // out.blend_color = vec3<f32>(1.0,0.0,0.0);

    let normal = normals[normal_index];
    // out.world_normal = mesh_normal_local_to_world(normal, vertex.instance_index);
    out.world_normal = mesh_normal_local_to_world(normal, vertex.instance_index);

    let local_position = vec4<f32>(x,y,z, 1.0);

    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    var model = mesh_functions::get_model_matrix(vertex.instance_index);

    let world_position = model* local_position;
    out.position = mesh_functions::mesh_position_local_to_clip(model, local_position);

    // out.world_position = mesh_functions::mesh_position_local_to_world(model, local_position);

    return out;
}

#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(in: MyVertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.frag_depth = in.position.z;
#ifdef NORMAL_PREPASS
    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
#endif

#ifdef DEFERRED_PREPASS
    // There isn't any material info available for this default prepass shader so we are just writing 
    // emissive magenta out to the deferred gbuffer to be rendered by the first deferred lighting pass layer.
    // This is here so if the default prepass fragment is used for deferred magenta will be rendered, and also
    // as an example to show that a user could write to the deferred gbuffer if they were to start from this shader.
    out.deferred = vec4(0u, bevy_pbr::rgb9e5::vec3_to_rgb9e5_(vec3(1.0, 0.0, 1.0)), 0u, 0u);
    out.deferred_lighting_pass_id = 1u;
#endif

    return out;
}
#endif // PREPASS_FRAGMENT
