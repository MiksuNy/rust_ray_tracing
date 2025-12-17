@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1)
var <storage, read> triangles : array<Triangle>;

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

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_w = f32(textureDimensions(texture).x);
    let texture_h = f32(textureDimensions(texture).y);
    let aspect = texture_w / texture_h;
    let screen_x = ((f32(global_id.x) / texture_w) * 2.0 - 1.0) * aspect;
    let screen_y = (f32(u32(texture_h) - global_id.y) / texture_h) * 2.0 - 1.0;

    var ray = Ray();
    ray.origin = vec3<f32>(0.0, 0.0, 4.0);
    ray.direction = normalize(vec3<f32>(screen_x, screen_y, -1.0));

    var color = vec3<f32>(0.0, 0.0, 0.0);
    let hit_distance = trace_ray(ray);
    if hit_distance > 0.0 {
        color = vec3<f32>(1.0 / hit_distance, 0.0, 0.0);
    }

    textureStore(texture, vec2(i32(global_id.x), i32(global_id.y)), vec4(color, 1.0));
}

fn intersect_tri(ray: Ray, tri: Triangle) -> f32 {
    let v_1 = tri.vertices[0].position;
    let v_2 = tri.vertices[1].position;
    let v_3 = tri.vertices[2].position;

    let edge_1 = v_2 - v_1;
    let edge_2 = v_3 - v_1;

    let ray_cross_e2 = cross(ray.direction, edge_2);
    let det = dot(edge_1, ray_cross_e2);

    let inv_det = 1.0 / det;
    let s = ray.origin - v_1;
    let u = inv_det * dot(s, ray_cross_e2);

    let s_cross_e1 = cross(s, edge_1);
    let v = inv_det * dot(ray.direction, s_cross_e1);

    let t = inv_det * dot(edge_2, s_cross_e1);

    if t > 0.0001 && !(det < 0.0 && det > -0.0) && !(u < 0.0 || u > 1.0) && !(v < 0.0 || u + v > 1.0) {
        return t;
    } else {
        return -1.0;
    }
}

fn trace_ray(ray: Ray) -> f32 {
    let sky_distance = 10000.0f;
    var prev_hit_distance = sky_distance;
    for (var i = 0u; i < arrayLength(&triangles); i++) {
        let hit_distance = intersect_tri(ray, triangles[i]);
        if hit_distance > 0.0 && hit_distance < prev_hit_distance {
            prev_hit_distance = hit_distance;
        }
    }
    if prev_hit_distance < sky_distance {
        return prev_hit_distance;
    } else {
        return -1.0;
    }
}
