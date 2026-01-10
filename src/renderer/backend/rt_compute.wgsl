@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1)
var <storage, read> triangles: array<Triangle>;

@group(0) @binding(2)
var <storage, read> bvh_nodes: array<Node>;

@group(0) @binding(3)
var <storage, read> materials: array<Material>;

@group(0) @binding(4)
var <storage, read> texture_data: array<u32>;

@group(0) @binding(5)
var <storage, read> texture_info: array<TextureInfo>;

@group(1) @binding(0)
var <uniform> camera_look_at : mat4x4<f32>;

@group(1) @binding(1)
var <uniform> camera_position: vec3<f32>;

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
    base_color_tex_id: u32,
    emission_tex_id: u32,
}

struct Node {
    bounds_min: vec3<f32>,
    first_tri_or_child: u32,
    bounds_max: vec3<f32>,
    num_tris: u32,
}

struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    tex_coord: vec2<f32>,
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
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var rng_seed = 22235u + (757283u * global_id.x + 872653746u * global_id.y);

    let texture_w = f32(textureDimensions(texture).x);
    let texture_h = f32(textureDimensions(texture).y);
    let aspect = texture_w / texture_h;
    let screen_x = ((f32(global_id.x) / texture_w) * 2.0f - 1.0f) * aspect;
    let screen_y = (f32(u32(texture_h) - global_id.y) / texture_h) * 2.0f - 1.0f;

    // TODO: This breaks silently at high sample counts.
    // The loop should probably be done on the CPU and on the GPU we just read from a texture containing the previous sample's pixel data.
    let samples = 10u;
    var final_color = vec3<f32>(0.0);
    for (var i = 0u; i < samples; i++) {
        var ray = Ray();
        ray.origin = camera_position;
        let jitter = vec2<f32>(rand_f32(&rng_seed) * 2.0 - 1.0, rand_f32(&rng_seed) * 2.0 - 1.0) * 0.0005;
        ray.direction = normalize(camera_look_at * vec4<f32>(-screen_x + jitter.x, screen_y + jitter.y, 1.0, 0.0)).xyz;

        final_color += trace(&ray, &rng_seed, 1u);
    }
    final_color /= f32(samples);
    final_color = linear_to_srgb(final_color);

    //var ray = Ray();
    //ray.origin = camera_position;
    //ray.direction = normalize(camera_look_at * vec4<f32>(-screen_x, screen_y, 1.0, 0.0)).xyz;
    //let debug_color = debug_bvh(ray, 300.0f);

    textureStore(texture, vec2<i32>(i32(global_id.x), i32(global_id.y)), vec4<f32>(final_color, 1.0));
}

fn trace(ray: ptr<function, Ray>, rng_seed: ptr<function, u32>, max_depth: u32) -> vec3<f32> {
    var ray_color = vec3<f32>(1.0f);
    var incoming_light = vec3<f32>(0.0f);
    var emitted_light = vec3<f32>(0.0f);

    var curr_depth: u32 = 0u;
    while curr_depth < max_depth {
        let hit_info = traverse_bvh(*ray);

        if hit_info.has_hit {
            let hit_material = materials[hit_info.material_id];

            if hit_material.base_color_tex_id != 0xFFFFFFFF {
                return sample_texture(hit_material.base_color_tex_id, hit_info.uv).xyz;
            } else {
                return hit_material.base_color;
            }
            emitted_light += hit_material.emission;
            incoming_light += emitted_light * ray_color;

            let new_dir = normalize(hit_info.normal + rand_in_unit_sphere(rng_seed));
            (*ray).origin = hit_info.point + new_dir * 0.0001f;
            (*ray).direction = new_dir;

            curr_depth += 1u;
        } else {
            let sky_color = vec3<f32>(0.99f, 0.97f, 0.98f);
            let sky_strength = vec3<f32>(1.0f);

            ray_color *= sky_color;
            emitted_light += sky_strength;
            incoming_light += emitted_light * ray_color;

            break;
        }
    }

    if curr_depth == 0u {
        return incoming_light;
    } else {
        return incoming_light / f32(curr_depth);
    }
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
    hit_info.point = ray.origin + (ray.direction * t);
    hit_info.distance = t;

    let front_face = det > 0.0f;
    hit_info.front_face = front_face;

    let n_0 = tri.vertices[0].normal;
    let n_1 = tri.vertices[1].normal;
    let n_2 = tri.vertices[2].normal;
    var normal = n_0 * (1.0f - u - v) + (n_1 * u) + (n_2 * v);
    hit_info.normal = select(-normal, normal, front_face);

    let t_0 = tri.vertices[0].tex_coord;
    let t_1 = tri.vertices[1].tex_coord;
    let t_2 = tri.vertices[2].tex_coord;
    hit_info.uv = t_0 * (1.0f - u - v) + (t_1 * u) + (t_2 * v);

    hit_info.material_id = tri.material_id;

    return hit_info;
}

fn intersect_node(ray: Ray, node: Node) -> f32 {
    let t_min = (node.bounds_min - ray.origin) / ray.direction;
    let t_max = (node.bounds_max - ray.origin) / ray.direction;
    let t_1 = min(t_min, t_max);
    let t_2 = max(t_min, t_max);
    let t_near = max(max(t_1.x, t_1.y), t_1.z);
    let t_far = min(min(t_2.x, t_2.y), t_2.z);

    return select(1e30f, t_near, t_near <= t_far && t_far > 0.0f);
}

fn traverse_bvh(ray: Ray) -> HitInfo {
    var hit_info = HitInfo();
    hit_info.distance = 1e30f;

    var stack = array<Node, 32u>();
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
        var dist_1 = intersect_node(ray, child_1);
        var dist_2 = intersect_node(ray, child_2);
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
    var stack = array<Node, 32u>();
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
        var dist_1 = intersect_node(ray, child_1);
        var dist_2 = intersect_node(ray, child_2);
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
    let v4 = vec4<f32>( 1.0, x, x * x, x * x * x);
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

fn rand_f32_nd(input: ptr<function, u32>) -> f32 {
    let theta = 6.283185f * rand_f32(input);
    let rho = sqrt(-2.0f * log(rand_f32(input)));
    return rho * cos(theta);
}

fn rand_in_unit_sphere(input: ptr<function, u32>) -> vec3<f32> {
    return normalize(vec3<f32>(
        rand_f32_nd(input),
        rand_f32_nd(input),
        rand_f32_nd(input)
    ));
}

fn rand_in_unit_hemisphere(input: ptr<function, u32>, normal: vec3<f32>) -> vec3<f32> {
    let unit_sphere = rand_in_unit_sphere(input);
    return faceForward(-unit_sphere, unit_sphere, normal);
}

// https://gamedev.stackexchange.com/a/194038
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(f32(linear.r < 0.0031308f), f32(linear.g < 0.0031308f), f32(linear.b < 0.0031308f));
    let higher = vec3<f32>(1.055) * pow(linear, vec3<f32>(1.0/2.4)) - vec3<f32>(0.055);
    let lower = linear * vec3<f32>(12.92);
    return mix(higher, lower, cutoff);
}

fn schlick_fresnel(n_dot_v: f32, ior: f32) -> f32 {
    let f_0 = pow(ior - 1.0, 2) / pow(ior + 1.0, 2);
    return f_0 + (1.0 - f_0) * pow(1.0 - n_dot_v, 5);
}

fn sample_texture(texture_index: u32, uv: vec2<f32>) -> vec4<f32> {
    if texture_index >= arrayLength(&texture_info) {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    }
    let info = texture_info[texture_index];
    let i: i32 = i32(uv.x * f32(info.width));
    let j: i32 = i32(uv.y * f32(info.height));
    var index: i32 = i + i32(j * i32(info.width));
    let data_len = i32(info.width * info.height);
    while index > data_len - 1 {
        index -= data_len - 1;
    }
    while index < 0 {
        index += data_len - 1;
    }
    index += i32(info.data_offset);
    return unpack4x8unorm(texture_data[u32(index)]);
}
