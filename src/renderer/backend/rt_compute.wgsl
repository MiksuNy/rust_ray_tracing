@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1)
var <storage, read> triangles : array<Triangle>;

@group(0) @binding(2)
var <storage, read> bvh_nodes : array<Node>;

struct Node {
    bounds_min: vec3<f32>,
    first_tri_or_child: u32,
    bounds_max: vec3<f32>,
    num_tris: u32,
}

// TODO: Make this fit into 32 bytes
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
    let texture_w = f32(textureDimensions(texture).x);
    let texture_h = f32(textureDimensions(texture).y);
    let aspect = texture_w / texture_h;
    let screen_x = ((f32(global_id.x) / texture_w) * 2.0f - 1.0f) * aspect;
    let screen_y = (f32(u32(texture_h) - global_id.y) / texture_h) * 2.0f - 1.0f;

    var ray = Ray();
    ray.origin = vec3<f32>(0.0f, 0.0f, 5.0f);
    ray.direction = normalize(vec3<f32>(screen_x, screen_y, -2.0f));

    let hit_info = traverse_bvh(ray);
    let color = vec3<f32>(clamp(dot(normalize(vec3<f32>(1.0f, 1.0f, 1.0f)), hit_info.normal), 0.0, 1.0));

    textureStore(texture, vec2(i32(global_id.x), i32(global_id.y)), vec4(color, 1.0));
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

    hit_info.has_hit = t > 0.0001f && !(det < 0.0f && det > -0.0f) && !(u < 0.0f || u > 1.0f) && !(v < 0.0f || u + v > 1.0f);
    hit_info.point = ray.origin + (ray.direction * t);
    hit_info.distance = t;

    let front_face = det > 0.0f;
    hit_info.front_face = front_face;

    let n_0 = tri.vertices[0].normal;
    let n_1 = tri.vertices[0].normal;
    let n_2 = tri.vertices[0].normal;
    var normal = n_0 * (1.0 - u - v) + (n_1 * u) + (n_2 * v);
    if !front_face {
        normal = normal * -1.0f;
    }
    hit_info.normal = normal;

    let t_0 = tri.vertices[0].tex_coord;
    let t_1 = tri.vertices[1].tex_coord;
    let t_2 = tri.vertices[2].tex_coord;
    hit_info.uv = t_0 * (1.0 - u - v) + (t_1 * u) + (t_2 * v);

    hit_info.material_id = tri.material_id;

    return hit_info;
}

fn intersect_node(ray: Ray, node: Node) -> f32 {
    let t_min = (node.bounds_min - ray.origin) / ray.direction;
    let t_max = (node.bounds_max - ray.origin) / ray.direction;
    let t_1 = min(t_min, t_max) - vec3<f32>(0.0001f, 0.0001f, 0.0001f);
    let t_2 = max(t_min, t_max) + vec3<f32>(0.0001f, 0.0001f, 0.0001f);
    let t_near = max(max(t_1.x, t_1.y), t_1.z);
    let t_far = min(min(t_2.x, t_2.y), t_2.z);

    return select(1e30f, t_near, t_near < t_far && t_far > 0.0f);
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
