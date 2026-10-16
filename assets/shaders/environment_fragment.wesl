import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
};
import super::environment_types::environment;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in, front);
    pbr.material.base_color = alpha_discard(pbr.material.flags, pbr.material.alpha_cutoff, pbr.material.base_color);
    let day = environment.sun.w;
    if environment.motion.x < 0.5 {
        // Thin leaves transmit light from behind and receive a broad sky fill.
        // This avoids treating grass cards as opaque, black-backed surfaces.
        let backlight = pow(max(dot(-pbr.V, environment.sun.xyz), 0.0), 3.0);
        let wrap = clamp(dot(pbr.N, environment.sun.xyz) * 0.5 + 0.5, 0.0, 1.0);
        let base = pbr.material.base_color.rgb;
        pbr.material.emissive = vec4(base * (0.10 + 0.16 * wrap + environment.tint.a * backlight) * day * environment.tint.rgb, 1.0);
        pbr.material.perceptual_roughness = 0.92;
    } else if environment.motion.x > 2.5 {
        // Vertical sheets use falling streaks instead of horizontal pond waves.
        let t = environment.motion.y * environment.flow.z;
        let p = in.world_position;
        let streak = pow(abs(sin((p.x + p.z) * 18.0 + sin(p.y * 1.3 + t) * 0.35)), 6.0);
        let churn = pow(max(sin(p.y * 6.0 + t * 4.0 + streak * 2.0), 0.0), 4.0);
        pbr.N = normalize(pbr.N + vec3(sin(p.y * 8.0 + t) * 0.12, 0.06 * streak, 0.0));
        pbr.material.base_color = vec4(mix(environment.tint.rgb * 0.65, vec3(0.92, 0.96, 1.0), streak * 0.5 + churn * 0.3), 1.0);
        pbr.material.emissive = vec4(environment.tint.rgb * (0.06 + streak * 0.12) * day, 1.0);
        pbr.material.perceptual_roughness = 0.12;
    } else {
        // World-space waves remain continuous across the original pond/brook meshes.
        let t = environment.motion.y * environment.flow.z;
        let p = in.world_position.xz;
        let a = dot(p, vec2(2.1, 1.4)) - t * 1.4;
        let b = dot(p, vec2(-1.3, 2.6)) - t * 0.9;
        var ripple = vec2(cos(a) * 0.09 - cos(b) * 0.05, cos(a) * 0.06 + cos(b) * 0.10);
        // Short waves break up broad specular bands; spatial phases make the
        // highlights travel with the flow instead of pulsing the whole pond.
        ripple += vec2(sin(dot(p, vec2(27.0, 19.0)) - t * 5.0),
                       cos(dot(p, vec2(-18.0, 31.0)) - t * 4.2)) * 0.015;
        if environment.motion.x < 1.5 {
            // _SpringCenter / _RingRadius / _RingWavelength in M_B612_Pond.
            let radial = p - vec2(-10.709165, 1.0481554);
            let distance = length(radial);
            let packet = pow(0.5 + 0.5 * cos(distance * 18.48 - environment.motion.y * 4.25), 2.0);
            let ring = cos(distance * 52.35988 - t * 52.35988) * exp(-distance * distance / 1.69) * packet * 0.03;
            ripple += radial / max(distance, 0.01) * ring;
        }
        pbr.N = normalize(pbr.N + vec3(ripple.x, 0.0, ripple.y));
        pbr.material.perceptual_roughness = environment.flow.w;
        let fresnel = pow(1.0 - max(dot(pbr.N, pbr.V), 0.0), 5.0);
        let caustic = pow(max(sin(a * 1.7) * sin(b * 1.4), 0.0), 8.0);
        pbr.material.base_color = vec4(mix(pbr.material.base_color.rgb, environment.tint.rgb, 0.2 + fresnel * 0.45), pbr.material.base_color.a);
        let reflected = reflect(-pbr.V, pbr.N);
        let sky_reflection = mix(vec3(0.32, 0.46, 0.50), vec3(0.12, 0.35, 0.58), clamp(reflected.y, 0.0, 1.0));
        pbr.material.emissive = vec4((sky_reflection * (0.025 + fresnel * 0.5) + vec3(caustic * 0.04)) * day, 1.0);
    }
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    out.color = main_pass_post_lighting_processing(pbr, out.color);
    return out;
}
