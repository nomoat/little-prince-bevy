// One 16-byte-aligned block shared by all environmental materials (WebGL2).
struct Environment {
    motion: vec4<f32>, // kind: vegetation=0, pond=1, brook=2, waterfall=3; time; wind amplitude; height
    sun: vec4<f32>, // direction toward sun; daylight amount
    tint: vec4<f32>, // transmitted light / water shallow tint; strength
    flow: vec4<f32>, // direction XZ; flow speed; roughness
    actor: vec4<f32>, // player position; interaction radius
};
@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> environment: Environment;

fn wind_offset(p: vec3<f32>, weight: f32) -> vec3<f32> {
    let t = environment.motion.y;
    let phase = dot(p.xz, vec2(0.34, 0.23));
    let gust = sin(phase - t * 1.35) * 0.65 + sin(phase * 2.7 - t * 2.1) * 0.25;
    let flutter = sin(dot(p.xz, vec2(5.1, 3.7)) + t * 7.0) * 0.1;
    let bend = (0.35 + gust + flutter) * environment.motion.z * weight * weight;
    let away = p.xz - environment.actor.xz;
    let distance = length(away);
    let nearby_height = 1.0 - smoothstep(0.4, 1.5, abs(p.y - environment.actor.y));
    let push = (1.0 - smoothstep(0.15, environment.actor.w, distance)) * weight * 0.2 * nearby_height;
    let offset = environment.flow.xy * bend + away / max(distance, 0.05) * push;
    return vec3(offset.x, -length(offset) * 0.15, offset.y);
}
