@group(0) @binding(0)
var storage_texture: texture_storage_2d<rgba16unorm, read_write>;

@group(1) @binding(0)
var <storage, read> triangles: array<Triangle>;

@group(1) @binding(1)
var <storage, read> bvh_nodes: array<Node>;

@group(1) @binding(2)
var <storage, read> materials: array<Material>;

@group(1) @binding(3)
var <storage, read> texture_data: array<u32>;

@group(1) @binding(4)
var <storage, read> texture_info: array<TextureInfo>;

@group(2) @binding(0)
var <uniform> camera: Camera;

var <immediate> renderer_info: RendererInfo;

const PI = 3.1415926535f;
const TWO_PI = 6.283185307f;
const PI_OVER_2 = 1.5707963268f;
const PI_OVER_4 = 0.7853981634f;
const EPSILON = 0.0001f;

struct RendererInfo {
    current_sample: u32,
    max_ray_depth: u32,
}

struct Camera {
    look_at: mat4x4<f32>,
    position: vec3<f32>,
}

struct TextureInfo {
    width: u32,
    height: u32,
    data_offset: u32,
}

struct Material {
    base_color: vec3<f32>,
    specular_tint: vec3<f32>,
    emission: vec3<f32>,
    transmission: f32,
    ior: f32,
    roughness: f32,
    metallic: f32,
    transparency: f32,
    base_color_tex_id: u32,
    emission_tex_id: u32,
    transparency_tex_id: u32,
    roughness_tex_id: u32,
    metallic_tex_id: u32,
}

struct Node {
    bounds_min: vec3<f32>,
    first_tri_or_child: u32,
    bounds_max: vec3<f32>,
    num_tris: u32,
}

struct Vertex {
    position: vec3<f32>,
    tex_coord_x: f32,
    normal: vec3<f32>,
    tex_coord_y: f32,
}

struct Triangle {
    vertices: array<Vertex, 3>,
    material_id: u32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitInfo {
    has_hit: bool,
    point: vec3<f32>,
    normal: vec3<f32>,
    distance: f32,
    uv: vec2<f32>,
    material_id: u32,
    front_face: bool,
    tbn: mat3x3<f32>
}

struct BSDFType {
    specular: bool,
    transmitted: bool,
    diffuse: bool
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var rng_seed = renderer_info.current_sample * 6023u + (757283u * global_id.x + 872653746u * global_id.y);
    let tex_coords = vec2<u32>(global_id.xy);

    let texture_w = f32(textureDimensions(storage_texture).x);
    let texture_h = f32(textureDimensions(storage_texture).y);
    let aspect = texture_w / texture_h;
    let screen_x = ((f32(global_id.x) / texture_w) * 2.0f - 1.0f) * aspect;
    let screen_y = (f32(u32(texture_h) - global_id.y) / texture_h) * 2.0f - 1.0f;

    var ray = Ray();
    ray.origin = camera.position;
    let jitter = vec2<f32>(rand_f32(&rng_seed) * 2.0 - 1.0, rand_f32(&rng_seed) * 2.0 - 1.0) * 0.0005;
    ray.direction = normalize(camera.look_at * vec4<f32>(-screen_x + jitter.x, screen_y + jitter.y, 1.0, 0.0)).xyz;

    let rt_color = trace(&ray, &rng_seed, renderer_info.max_ray_depth);
    let accumulation_color = textureLoad(storage_texture, tex_coords).rgb;
    let final_color = mix(accumulation_color, rt_color, 1.0f / f32(renderer_info.current_sample));

    //let final_color = debug_bvh(ray, 300.0f);

    textureStore(storage_texture, tex_coords, vec4<f32>(final_color, 1.0f));
}

fn trace(ray: ptr<function, Ray>, rng_seed: ptr<function, u32>, max_ray_depth: u32) -> vec3<f32> {
    var ray_color = vec3<f32>(1.0f);
    var incoming_light = vec3<f32>(0.0f);

    var prev_hit_point = ray.origin;
    var transmitted_distance = 0.0f;

    var curr_ray_depth: u32 = 0u;
    while curr_ray_depth < max_ray_depth {
        let hit_info = traverse_bvh(*ray);

        if hit_info.has_hit {
            curr_ray_depth += 1u;

            let hit_material = get_material(hit_info, materials[hit_info.material_id]);
            var transmitted_distance = hit_info.distance;
            if hit_info.front_face {
                prev_hit_point = hit_info.point;
            } else {
                transmitted_distance = distance(hit_info.point, prev_hit_point);
            }

            if hit_material.transparency < rand_f32(rng_seed) {
                (*ray).origin = hit_info.point + (*ray).direction * EPSILON;
                continue;
            }

            let alpha = clamp(hit_material.roughness * hit_material.roughness, EPSILON, 1.0f);
            let sampled_normal = to_world(hit_info.tbn, sample_ggx_vndf(to_local(hit_info.tbn, -(*ray).direction), alpha, alpha, rng_seed));

            var f0 = vec3<f32>(pow(1.0f - hit_material.ior, 2) / pow(1.0f + hit_material.ior, 2));
            f0 = mix(f0, hit_material.base_color, hit_material.metallic);
            let fresnel = schlick_fresnel(dot(sampled_normal, -(*ray).direction), f0);

            let specular_dir = normalize(reflect((*ray).direction, sampled_normal));
            let transmitted_dir = normalize(refract((*ray).direction, sampled_normal, hit_material.ior));
            let diffuse_dir = normalize(to_world(hit_info.tbn, cosine_sample_hemisphere(rng_seed)));

            let bsdf_type = select_bsdf(hit_material, rng_seed);

            var new_dir: vec3<f32>;
            if length(fresnel) < rand_f32(rng_seed) && !bsdf_type.specular {
                ray_color *= hit_material.base_color;
                if bsdf_type.transmitted {
                    new_dir = transmitted_dir;
                    if dot(new_dir, hit_info.normal) > 0.0f {
                        break;
                    }
                    var absorption = vec3<f32>(1.0f);
                    if !hit_info.front_face {
                        absorption = vec3<f32>(
                            exp(-(1.0f - hit_material.base_color.r) * transmitted_distance),
                            exp(-(1.0f - hit_material.base_color.g) * transmitted_distance),
                            exp(-(1.0f - hit_material.base_color.b) * transmitted_distance),
                        );
                    }
                    ray_color *= absorption;
                } else {
                    new_dir = diffuse_dir;
                }
            } else {
                if bsdf_type.specular {
                    ray_color *= fresnel;
                }

                new_dir = specular_dir;
                if dot(new_dir, hit_info.normal) < 0.0f {
                    break;
                }
            }

            // Russian roulette
            var rr_probability = 1.0f;
            if curr_ray_depth >= 4 {
                rr_probability = max(ray_color.r, max(ray_color.b, ray_color.g));
                if rr_probability < rand_f32(rng_seed) {
                    break;
                }
            }
            ray_color /= rr_probability;

            incoming_light += hit_material.emission * ray_color;

            (*ray).origin = hit_info.point + new_dir * EPSILON;
            (*ray).direction = new_dir;
        } else {
            let sky_color = vec3<f32>(1.0f, 1.0f, 1.0f);
            let sky_strength = vec3<f32>(1.0f);

            ray_color *= sky_color;
            incoming_light += sky_strength * ray_color;

            break;
        }
    }

    if curr_ray_depth == 0u {
        return incoming_light;
    } else {
        return incoming_light / f32(curr_ray_depth);
    }
}

fn select_bsdf(material: Material, rng_seed: ptr<function, u32>) -> BSDFType {
    var bsdf_type = BSDFType();

    let specular_chance = material.metallic;
    let transmission_chance = material.transmission;
    let diffuse_chance = 1.0f - specular_chance - transmission_chance;

    let r = rand_f32(rng_seed);
    if specular_chance > r {
        bsdf_type.specular = true;
    } else if specular_chance + transmission_chance > r {
        bsdf_type.transmitted = true;
    } else {
        bsdf_type.diffuse = true;
    }

    return bsdf_type;
}

// Helper function to get actual material properties of the hit surface
fn get_material(hit_info: HitInfo, hit_material: Material) -> Material {
    var out_material = hit_material;

    var ior: f32 = hit_material.ior;
    if hit_info.front_face {
        ior = 1.0 / hit_material.ior;
    }
    out_material.ior = ior;

    var base_color = hit_material.base_color;
    if hit_material.base_color_tex_id != 0xFFFFFFFF {
        base_color = sample_texture(hit_material.base_color_tex_id, hit_info.uv).rgb;
    }
    out_material.base_color = base_color;

    var roughness = hit_material.roughness;
    if hit_material.roughness_tex_id != 0xFFFFFFFF {
        roughness = sample_texture(hit_material.roughness_tex_id, hit_info.uv).r;
    }
    out_material.roughness = roughness;

    var metallic = hit_material.metallic;
    if hit_material.metallic_tex_id != 0xFFFFFFFF {
        metallic = sample_texture(hit_material.metallic_tex_id, hit_info.uv).r;
    }
    out_material.metallic = metallic;

    var emission = hit_material.emission;
    if hit_material.emission_tex_id != 0xFFFFFFFF {
        emission = sample_texture(hit_material.emission_tex_id, hit_info.uv).rgb;
    }
    out_material.emission = emission;

    var transparency = hit_material.transparency;
    if hit_material.transparency_tex_id != 0xFFFFFFFF {
        transparency = sample_texture(hit_material.transparency_tex_id, hit_info.uv).a;
    }
    out_material.transparency = transparency;

    return out_material;
}

fn intersect_tri(ray: Ray, tri: Triangle) -> HitInfo {
    let v_1 = tri.vertices[0].position;
    let v_2 = tri.vertices[1].position;
    let v_3 = tri.vertices[2].position;

    let edge_1 = v_2 - v_1;
    let edge_2 = v_3 - v_1;

    let ray_cross_e2 = cross(ray.direction, edge_2);
    let det = dot(edge_1, ray_cross_e2);

    let inv_det = 1.0f / det;
    let s = ray.origin - v_1;
    let u = inv_det * dot(s, ray_cross_e2);

    let s_cross_e1 = cross(s, edge_1);
    let v = inv_det * dot(ray.direction, s_cross_e1);

    let t = inv_det * dot(edge_2, s_cross_e1);

    var hit_info = HitInfo();

    hit_info.has_hit = t > 0.0f && !(det < 0.0f && det > -0.0f) && !(u < 0.0f || u > 1.0f) && !(v < 0.0f || u + v > 1.0f);
    hit_info.point = fma(ray.direction, vec3<f32>(t), ray.origin);
    hit_info.distance = t;

    let front_face = det > 0.0f;
    hit_info.front_face = front_face;

    let n_0 = tri.vertices[0].normal;
    let n_1 = tri.vertices[1].normal;
    let n_2 = tri.vertices[2].normal;
    let normal = n_0 * (1.0f - u - v) + (n_1 * u) + (n_2 * v);
    hit_info.normal = normalize(select(-normal, normal, front_face));

    let t_0 = vec2<f32>(tri.vertices[0].tex_coord_x, tri.vertices[0].tex_coord_y);
    let t_1 = vec2<f32>(tri.vertices[1].tex_coord_x, tri.vertices[1].tex_coord_y);
    let t_2 = vec2<f32>(tri.vertices[2].tex_coord_x, tri.vertices[2].tex_coord_y);
    hit_info.uv = t_0 * (1.0f - u - v) + (t_1 * u) + (t_2 * v);

    hit_info.material_id = tri.material_id;

    var tangent: vec3<f32>;
    var bitangent: vec3<f32>;
    build_orthonormal_basis(hit_info.normal, &tangent, &bitangent);
    hit_info.tbn = mat3x3<f32>(tangent, bitangent, hit_info.normal);

    return hit_info;
}

fn intersect_node(ray: Ray, node: Node, max_distance: f32) -> f32 {
    let t_min = (node.bounds_min - ray.origin) / ray.direction;
    let t_max = (node.bounds_max - ray.origin) / ray.direction;
    let t_1 = min(t_min, t_max);
    let t_2 = max(t_min, t_max);
    let t_near = max(max(t_1.x, t_1.y), t_1.z);
    let t_far = min(min(t_2.x, t_2.y), t_2.z);
    return select(1e30f, t_near, t_near <= t_far && t_near < max_distance && t_far > 0.0f);
}

fn traverse_bvh(ray: Ray) -> HitInfo {
    var hit_info = HitInfo();
    hit_info.distance = 1e30f;

    var stack = array<Node, 16u>();
    var node = bvh_nodes[0u];
    var stack_ptr: u32 = 0u;

    loop {
        if node.num_tris > 0u {
            for (var i = 0u; i < node.num_tris; i++) {
                let temp_hit_info = intersect_tri(ray, triangles[node.first_tri_or_child + i]);
                if temp_hit_info.has_hit && temp_hit_info.distance < hit_info.distance {
                    hit_info = temp_hit_info;
                }
            }
            if stack_ptr == 0u {
                break;
            } else {
                stack_ptr--;
                node = stack[stack_ptr];
            }
            continue;
        }

        var child_1 = bvh_nodes[node.first_tri_or_child];
        var child_2 = bvh_nodes[node.first_tri_or_child + 1u];
        var dist_1 = intersect_node(ray, child_1, hit_info.distance);
        var dist_2 = intersect_node(ray, child_2, hit_info.distance);

        if dist_1 > dist_2 {
            let temp_dist = dist_1;
            dist_1 = dist_2;
            dist_2 = temp_dist;

            let temp_child = child_1;
            child_1 = child_2;
            child_2 = temp_child;
        }

        if dist_1 == 1e30f {
            if stack_ptr == 0u {
                break;
            } else {
                stack_ptr--;
                node = stack[stack_ptr];
            }
        } else {
            node = child_1;
            if dist_2 < 1e30f {
                stack[stack_ptr] = child_2;
                stack_ptr++;
            }
        }
    }

    return hit_info;
}

fn debug_bvh(ray: Ray, factor: f32) -> vec3<f32> {
    var stack = array<Node, 16u>();
    var node = bvh_nodes[0u];
    var stack_ptr: u32 = 0u;

    var debug_value = 0.0f;
    loop {
        debug_value += 1.0f;
        if node.num_tris > 0u {
            for (var i = 0u; i < node.num_tris; i++) {
                debug_value += 1.1f;
            }
            if stack_ptr == 0u {
                break;
            } else {
                stack_ptr--;
                node = stack[stack_ptr];
            }
            continue;
        }

        var child_1 = bvh_nodes[node.first_tri_or_child];
        var child_2 = bvh_nodes[node.first_tri_or_child + 1u];
        var dist_1 = intersect_node(ray, child_1, 1e30f);
        var dist_2 = intersect_node(ray, child_2, 1e30f);

        if dist_1 > dist_2 {
            let temp_dist = dist_1;
            dist_1 = dist_2;
            dist_2 = temp_dist;

            let temp_child = child_1;
            child_1 = child_2;
            child_2 = temp_child;
        }

        if dist_1 == 1e30f {
            if stack_ptr == 0u {
                break;
            } else {
                stack_ptr--;
                node = stack[stack_ptr];
            }
        } else {
            node = child_1;
            if dist_2 < 1e30f {
                stack[stack_ptr] = child_2;
                stack_ptr++;
            }
        }
    }

    debug_value /= factor;

    return turbo_colormap(&debug_value);
}

// https://research.google/blog/turbo-an-improved-rainbow-colormap-for-visualization/
fn turbo_colormap(x_ptr: ptr<function, f32>) -> vec3<f32> {
    let kRedVec4 = vec4<f32>(0.13572138, 4.61539260, -42.66032258, 132.13108234);
    let kGreenVec4 = vec4<f32>(0.09140261, 2.19418839, 4.84296658, -14.18503333);
    let kBlueVec4 = vec4<f32>(0.10667330, 12.64194608, -60.58204836, 110.36276771);
    let kRedVec2 = vec2<f32>(-152.94239396, 59.28637943);
    let kGreenVec2 = vec2<f32>(4.27729857, 2.82956604);
    let kBlueVec2 = vec2<f32>(-89.90310912, 27.34824973);

    var x = *x_ptr;
    x = clamp(x, 0.0f, 1.0f);
    let v4 = vec4<f32>(1.0, x, x * x, x * x * x);
    let v2 = v4.zw * v4.z;
    return vec3<f32>(
        dot(v4, kRedVec4)   + dot(v2, kRedVec2),
        dot(v4, kGreenVec4) + dot(v2, kGreenVec2),
        dot(v4, kBlueVec4)  + dot(v2, kBlueVec2)
    );
}

fn xor_shift(input: ptr<function, u32>) -> u32 {
    var x = *input;
    x = x ^ (x << 13);
    x = x ^ (x >> 17);
    x = x ^ (x << 5);
    *input = x;
    return x;
}

fn rand_f32(input: ptr<function, u32>) -> f32 {
    return f32(xor_shift(input)) / f32(0xFFFFFFFF);
}

fn sample_texture(texture_index: u32, uv: vec2<f32>) -> vec4<f32> {
    let info = texture_info[texture_index];
    let i: i32 = i32(uv.x * f32(info.width));
    let j: i32 = i32(uv.y * f32(info.height));
    let data_len = i32(info.width * info.height);
    let index: i32 = i + j * i32(info.width) % data_len + i32(info.data_offset);
    return unpack4x8unorm(texture_data[u32(index)]);
}

fn sample_ggx_vndf(ve: vec3<f32>, ax: f32, ay: f32, rng_seed: ptr<function, u32>) -> vec3<f32> {
    let u1 = rand_f32(rng_seed);
    let u2 = rand_f32(rng_seed);

    let Vh = normalize(vec3<f32>(ax * ve.x, ay * ve.y, ve.z));

    let lensq = Vh.x * Vh.x + Vh.y * Vh.y;
    let T1 = select(vec3<f32>(1.0f, 0.0f, 0.0f), vec3<f32>(-Vh.y, Vh.x, 0.0f) * inverseSqrt(lensq), lensq > 0.0f);
    let T2 = cross(Vh, T1);

    let r = sqrt(u1);
    let phi = 2.0f * PI * u2;
    let t1 = r * cos(phi);
    var t2 = r * sin(phi);
    let s = 0.5f * (1.0f + Vh.z);
    t2 = (1.0f - s) * sqrt(1.0f - t1 * t1) + s * t2;

    let Nh = t1 * T1 + t2 * T2 + sqrt(max(0.0f, 1.0f - t1 * t1 - t2 * t2)) * Vh;

    let Ne = normalize(vec3<f32>(ax * Nh.x, ay * Nh.y, max(0.0f, Nh.z)));
    return Ne;
}

// https://www.pbr-book.org/3ed-2018/Monte_Carlo_Integration/2D_Sampling_with_Multidimensional_Transformations#ConcentricSampleDisk
fn concentric_sample_disk(u: vec2<f32>) -> vec2<f32> {
    let u_offset = 2.0f * u - vec2<f32>(1.0f);
    if u_offset.x == 0.0f && u_offset.y == 0.0f {
        return vec2<f32>(0.0f);
    }
    var theta: f32;
    var r: f32;
    if abs(u_offset.x) > abs(u_offset.y) {
        r = u_offset.x;
        theta = PI_OVER_4 * (u_offset.y / u_offset.x);
    } else {
        r = u_offset.y;
        theta = PI_OVER_2 - PI_OVER_4 * (u_offset.x / u_offset.y);
    }
    return r * vec2<f32>(cos(theta), sin(theta));
}

// https://www.pbr-book.org/3ed-2018/Monte_Carlo_Integration/2D_Sampling_with_Multidimensional_Transformations#CosineSampleHemisphere
fn cosine_sample_hemisphere(rng_seed: ptr<function, u32>) -> vec3<f32> {
    let u = vec2<f32>(rand_f32(rng_seed), rand_f32(rng_seed));
    let d = concentric_sample_disk(u);
    let z = sqrt(max(0.0f, 1.0f - d.x * d.x - d.y * d.y));
    return vec3<f32>(d.x, d.y, z);
}

fn schlick_fresnel(n_dot_v: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0f - f0) * pow(1.0f - n_dot_v, 5);
}

fn to_local(tbn: mat3x3<f32>, world: vec3<f32>) -> vec3<f32> {
    return transpose(tbn) * world;
}

fn to_world(tbn: mat3x3<f32>, local: vec3<f32>) -> vec3<f32> {
    return tbn * local;
}

fn build_orthonormal_basis(normal: vec3<f32>, tangent: ptr<function, vec3<f32>>, bitangent: ptr<function, vec3<f32>>) {
    let up = select(vec3<f32>(1.0f, 0.0f, 0.0f), vec3<f32>(0.0f, 0.0f, 1.0f), abs(normal.z) < 0.9999999f);
    *tangent = normalize(cross(up, normal));
    *bitangent = cross(normal, *tangent);
}
