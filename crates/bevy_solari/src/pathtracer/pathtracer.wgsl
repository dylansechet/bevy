enable wgpu_ray_query;

#import bevy_pbr::utils::rand_vec2f
#import bevy_render::view::View
#import bevy_solari::brdf::evaluate_and_sample_brdf
#import bevy_solari::scene_bindings::{trace_ray, resolve_ray_hit_full, RAY_T_MAX}

@group(1) @binding(0) var accumulation_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(1) var view_output: texture_storage_2d<rgba16float, write>;
@group(1) @binding(2) var<uniform> view: View;

@compute @workgroup_size(8, 8, 1)
fn pathtrace(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if any(global_id.xy >= vec2u(view.viewport.zw)) { return; }

    let old_color = textureLoad(accumulation_texture, global_id.xy);

    // Setup RNG
    let pixel_index = global_id.x + global_id.y * u32(view.viewport.z);
    let frame_index = u32(old_color.a) * 5782582u;
    var rng = pixel_index + frame_index;

    // Shoot primary ray from camera
    let pixel_center = vec2<f32>(global_id.xy) + 0.5;
    let jitter = rand_vec2f(&rng) - 0.5;
    let pixel_uv = (pixel_center + jitter) / view.viewport.zw;
    let pixel_ndc = pixel_uv * 2.0 - 1.0;
    let primary_ray_target = view.world_from_clip * vec4(pixel_ndc.x, -pixel_ndc.y, 1.0, 1.0);
    let ray_origin = view.world_position;
    let ray_direction = normalize(primary_ray_target.xyz / primary_ray_target.w - ray_origin);

    // White furnace environment: constant radiance in all directions.
    // Divide by exposure so the output value is 0.5 after the exposure multiply below.
    let env = vec3(0.5) / view.exposure;

    var radiance: vec3<f32>;
    let primary = trace_ray(ray_origin, ray_direction, 0.0, RAY_T_MAX, RAY_FLAG_NONE);

    if primary.kind == RAY_QUERY_INTERSECTION_NONE {
        radiance = env;
    } else {
        let hit = resolve_ray_hit_full(primary);
        let wo = -ray_direction;

        // Sample one bounce from the BRDF and assume it exits to the white environment.
        // We don't trace the bounce ray so spheres can't interact with each other.
        let s = evaluate_and_sample_brdf(wo, hit.world_normal, hit.world_tangent, hit.material, &rng);
        if s.pdf > 0.0 {
            radiance = s.throughput * env;
        }
    }

    // Camera exposure
    radiance *= view.exposure;

    // Accumulation over time via running average
    let new_color = mix(old_color.rgb, radiance, 1.0 / (old_color.a + 1.0));
    textureStore(accumulation_texture, global_id.xy, vec4(new_color, old_color.a + 1.0));
    textureStore(view_output, global_id.xy, vec4(new_color, 1.0));
}
