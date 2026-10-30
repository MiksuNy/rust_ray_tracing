/* References:
    [Sampling the GGX Distribution of Visible Normals] https://jcgt.org/published/0007/04/01/paper.pdf
    [knightcrawler25's Disney BSDF GLSL implementation] https://github.com/knightcrawler25/GLSL-PathTracer/blob/master/src/shaders/common/disney.glsl
*/

enable wgpu_binding_array;

@group(0) @binding(0)
var rt_texture: texture_storage_2d<rgba32float, read_write>;

@group(1) @binding(0)
var <storage, read> triangles: array<Triangle>;

@group(1) @binding(1)
var <storage, read> emissive_triangles: array<u32>;

@group(1) @binding(2)
var <storage, read> bvh_nodes: array<Node>;

@group(1) @binding(3)
var <storage, read> materials: array<Material>;

@group(1) @binding(4)
var textures: binding_array<texture_2d<f32>, 128>;

@group(1) @binding(5)
var textures_array_sampler: sampler;

@group(2) @binding(0)
var <uniform> camera: Camera;

var <immediate> renderer_info: RendererInfo;

const PI = 3.1415926535f;
const INV_PI = 0.3183098862f;
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

struct Material {
    base_color: vec3<f32>,
    transmission: f32,
    specular_tint: vec3<f32>,
    ior: f32,
    emission: vec3<f32>,
    roughness: f32,
    metallic: f32,
    transparency: f32,
    anisotropy: f32,
    clearcoat: f32,
    clearcoat_roughness: f32,
    a_x: f32,
    a_y: f32,
    base_color_tex_id: u32,
    transparency_tex_id: u32,
    roughness_tex_id: u32,
    metallic_tex_id: u32,
    emission_tex_id: u32,
    normal_tex_id: u32,
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
    triangle_id: u32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitInfo {
    tbn: mat3x3<f32>,
    point: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    has_hit: bool,
    front_face: bool,
    material_id: u32,
    triangle_id: u32,
    distance: f32,
    eta: f32,
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var rng_seed = renderer_info.current_sample * 6023u + (757283u * global_id.x + 872653746u * global_id.y);
    let tex_coords = vec2<u32>(global_id.xy);

    let texture_dimensions = vec2<f32>(f32(textureDimensions(rt_texture).x), f32(textureDimensions(rt_texture).y));
    let aspect = texture_dimensions.x / texture_dimensions.y;
    let screen_coords = vec2<f32>(
        ((f32(global_id.x) / texture_dimensions.x) * 2.0f - 1.0f) * aspect,
        (f32(u32(texture_dimensions.y) - global_id.y) / texture_dimensions.y) * 2.0f - 1.0f
    );

    var ray = Ray();
    ray.origin = camera.position;
    let jitter = vec2<f32>(rand_f32(&rng_seed) * 2.0f - 1.0f, rand_f32(&rng_seed) * 2.0f - 1.0f) * 0.0005f;
    ray.direction = (camera.look_at * normalize(vec4<f32>(-screen_coords.x + jitter.x, screen_coords.y + jitter.y, 1.0f, 0.0f))).xyz;

    let rt_color = trace(&ray, &rng_seed, renderer_info.max_ray_depth);
    let accumulation_color = textureLoad(rt_texture, tex_coords).rgb;
    let final_color = mix(accumulation_color, rt_color, 1.0f / f32(renderer_info.current_sample));

    //let final_color = debug_bvh(ray, 300.0f);

    textureStore(rt_texture, tex_coords, vec4<f32>(final_color, 1.0f));
}

fn trace(ray: ptr<function, Ray>, rng_seed: ptr<function, u32>, max_ray_depth: u32) -> vec3<f32> {
    var ray_albedo = vec3<f32>(1.0f);
    var ray_throughput = vec3<f32>(0.0f);

    var curr_ray_depth: u32 = 0u;
    while curr_ray_depth < max_ray_depth {
        var hit_info = traverse_bvh(*ray);

        if hit_info.has_hit {
            var material = materials[hit_info.material_id];
            set_surface_properties((*ray).direction, &hit_info, &material);

            if material.transparency < rand_f32(rng_seed) {
                (*ray).origin = hit_info.point + (*ray).direction * EPSILON;
                continue;
            }

            var l = vec3<f32>(0.0f);
            var pdf: f32;
            let f = sample_disney(hit_info, material, -(*ray).direction, hit_info.normal, &l, &pdf, rng_seed);

            ray_throughput += material.emission * ray_albedo;

            if pdf > 0.0f {
                ray_albedo *= f / pdf;
            } else {
                break;
            }

            // Russian roulette
            if curr_ray_depth >= 2u {
                let q = min(max(ray_albedo.r, max(ray_albedo.g, ray_albedo.b)) + 0.001f, 0.95f);
                if rand_f32(rng_seed) > q {
                    break;
                }
                ray_albedo /= q;
            }

            (*ray).origin = hit_info.point + l * EPSILON;
            (*ray).direction = l;

            curr_ray_depth += 1u;
        } else {
            let sky_color = vec3<f32>(1.0f, 1.0f, 1.0f);
            let sky_strength = vec3<f32>(1.0f);

            ray_albedo *= sky_color;
            ray_throughput += sky_strength * ray_albedo;

            break;
        }
    }

    return select(ray_throughput / f32(curr_ray_depth), ray_throughput, curr_ray_depth == 0u);
}

fn tint_colors(material: Material, eta: f32, f_0: ptr<function, f32>, c_spec_0: ptr<function, vec3<f32>>) {
    let lum = luminance(material.base_color);
    let c_tint = select(vec3<f32>(1.0f), material.base_color / lum, lum > 0.0f);

    *f_0 = (1.0f - eta) / (1.0f + eta);
    *f_0 *= *f_0;

    *c_spec_0 = *f_0 * mix(vec3<f32>(1.0f), c_tint, material.specular_tint);
}

fn sample_disney(hit_info: HitInfo, material: Material, v_in: vec3<f32>, n: vec3<f32>, l: ptr<function, vec3<f32>>, pdf: ptr<function, f32>, rng_seed: ptr<function, u32>) -> vec3<f32> {
    *pdf = 0.0f;

    var v = to_local(hit_info.tbn, v_in);

    var f_0: f32;
    var c_spec_0: vec3<f32>;
    tint_colors(material, hit_info.eta, &f_0, &c_spec_0);

    let dielectric_wt = (1.0f - material.metallic) * (1.0f - material.transmission);
    let metal_wt = material.metallic;
    let glass_wt = (1.0f - material.metallic) * material.transmission;

    let schlick_wt = schlick_weight(v.z);

    var diffuse_pr = dielectric_wt * luminance(material.base_color);
    var dielectric_pr = dielectric_wt * luminance(mix(c_spec_0, vec3<f32>(1.0f), schlick_wt));
    var metal_pr = metal_wt * luminance(mix(material.base_color, vec3<f32>(1.0f), schlick_wt));
    var glass_pr = glass_wt;
    var clearcoat_pr = 0.25f * material.clearcoat;

    let inv_total_wt = 1.0f / (diffuse_pr + dielectric_pr + metal_pr + glass_pr + clearcoat_pr);
    diffuse_pr *= inv_total_wt;
    dielectric_pr *= inv_total_wt;
    metal_pr *= inv_total_wt;
    glass_pr *= inv_total_wt;
    clearcoat_pr *= inv_total_wt;

    var cdf = array<f32, 5u>();
    cdf[0] = diffuse_pr;
    cdf[1] = cdf[0] + dielectric_pr;
    cdf[2] = cdf[1] + metal_pr;
    cdf[3] = cdf[2] + glass_pr;
    cdf[4] = cdf[3] + clearcoat_pr;

    var r = rand_f32(rng_seed);
    if r < cdf[0] { // Diffuse
        *l = cosine_sample_hemisphere(rng_seed);
    } else if r < cdf[2] {
        var h = sample_ggx_vndf(v, material.a_x, material.a_y, rng_seed);

        if h.z < 0.0f {
            h = -h;
        }

        *l = normalize(reflect(-v, h));
    } else if r < cdf[3] { // Specular
        var h = sample_ggx_vndf(v, material.a_x, material.a_y, rng_seed);
        let f = dielectric_fresnel(abs(dot(v, h)), hit_info.eta);

        if h.z < 0.0f {
            h = -h;
        }

        r = (r - cdf[2]) / (cdf[3] - cdf[2]);

        if r < f {
            *l = normalize(reflect(-v, h));
        } else {
            *l = normalize(refract(-v, h, hit_info.eta));
        }
    } else { // Clearcoat
        var h = sample_gtr_1(material.clearcoat_roughness, rng_seed);

        if h.z < 0.0f {
            h = -h;
        }

        *l = normalize(reflect(-v, h));
    }

    *l = normalize(to_world(hit_info.tbn, *l));
    v = normalize(to_world(hit_info.tbn, v));

    return eval_disney(hit_info, material, v, n, *l, pdf);
}

fn eval_disney(hit_info: HitInfo, material: Material, v_in: vec3<f32>, n: vec3<f32>, l_in: vec3<f32>, pdf: ptr<function, f32>) -> vec3<f32> {
    *pdf = 0.0f;
    var f = vec3<f32>(0.0f);

    let l = to_local(hit_info.tbn, l_in);
    let v = to_local(hit_info.tbn, v_in);

    var h: vec3<f32>;
    if l.z > 0.0f {
        h = normalize(l + v);
    } else {
        h = normalize(l + v * hit_info.eta);
    }

    if h.z < 0.0f {
        h = -h;
    }

    var f_0: f32;
    var c_spec_0: vec3<f32>;
    tint_colors(material, hit_info.eta, &f_0, &c_spec_0);

    let dielectric_wt = (1.0f - material.metallic) * (1.0f - material.transmission);
    let metal_wt = material.metallic;
    let glass_wt = (1.0f - material.metallic) * material.transmission;

    let schlick_wt = schlick_weight(v.z);

    var diffuse_pr = dielectric_wt * luminance(material.base_color);
    var dielectric_pr = dielectric_wt * luminance(mix(c_spec_0, vec3<f32>(1.0f), schlick_wt));
    var metal_pr = metal_wt * luminance(mix(material.base_color, vec3<f32>(1.0f), schlick_wt));
    var glass_pr = glass_wt;
    var clearcoat_pr = 0.25f * material.clearcoat;

    let inv_total_wt = 1.0f / (diffuse_pr + dielectric_pr + metal_pr + glass_pr + clearcoat_pr);
    diffuse_pr *= inv_total_wt;
    dielectric_pr *= inv_total_wt;
    metal_pr *= inv_total_wt;
    glass_pr *= inv_total_wt;
    clearcoat_pr *= inv_total_wt;

    let should_reflect = l.z * v.z > 0.0f;

    var tmp_pdf = 0.0f;
    let v_dot_h = abs(dot(v, h));

    if diffuse_pr > 0.0f && should_reflect {
        f += eval_disney_diffuse(l, v, h, material, &tmp_pdf) * dielectric_wt;
        *pdf += tmp_pdf * diffuse_pr;
    }

    if dielectric_pr > 0.0f && should_reflect {
        let F = (dielectric_fresnel(v_dot_h, 1.0f / material.ior) - f_0) / (1.0f - f_0);

        f += eval_disney_microfacet_reflection(l, v, h, mix(c_spec_0, vec3<f32>(1.0f), F), material, hit_info, &tmp_pdf) * dielectric_wt;
        *pdf += tmp_pdf * dielectric_pr;
    }

    if metal_pr > 0.0f && should_reflect {
        let F = mix(material.base_color, vec3<f32>(1.0f), schlick_weight(v_dot_h));

        f += eval_disney_microfacet_reflection(l, v, h, F, material, hit_info, &tmp_pdf) * metal_wt;
        *pdf += tmp_pdf * metal_pr;
    }

    if glass_pr > 0.0f {
        let F = dielectric_fresnel(v_dot_h, hit_info.eta);

        if should_reflect {
            f += eval_disney_microfacet_reflection(l, v, h, vec3<f32>(F), material, hit_info, &tmp_pdf) * glass_wt;
            *pdf += tmp_pdf * glass_pr * F;
        } else {
            f += eval_disney_microfacet_refraction(l, v, h, vec3<f32>(F), material, hit_info, &tmp_pdf) * glass_wt;
            *pdf += tmp_pdf * glass_pr * (1.0f - F);
        }
    }

    if clearcoat_pr > 0.0f && should_reflect {
        f += eval_disney_clearcoat(l, v, h, material, &tmp_pdf) * 0.25f * material.clearcoat;
        *pdf += tmp_pdf * clearcoat_pr;
    }

    return f * abs(l.z);
}

fn dielectric_fresnel(cos_theta_i: f32, eta: f32) -> f32 {
    let sin_theta_t_sq = eta * eta * (1.0f - cos_theta_i * cos_theta_i);

    if sin_theta_t_sq > 1.0f {
        return 1.0f;
    }

    let cos_theta_t = sqrt(max(1.0f - sin_theta_t_sq, 0.0f));

    let r_s = (eta * cos_theta_t - cos_theta_i) / (eta * cos_theta_t + cos_theta_i);
    let r_p = (eta * cos_theta_i - cos_theta_t) / (eta * cos_theta_i + cos_theta_t);

    return 0.5f * (r_s * r_s + r_p * r_p);
}

// TODO: Missing subsurface and sheen
fn eval_disney_diffuse(l: vec3<f32>, v: vec3<f32>, h: vec3<f32>, material: Material, pdf: ptr<function, f32>) -> vec3<f32> {
    *pdf = 0.0f;
    if l.z <= 0.0f {
        return vec3<f32>(0.0f);
    }

    let l_dot_h = dot(l, h);

    let rr = 2.0f * material.roughness * l_dot_h * l_dot_h;

    let f_l = schlick_weight(l.z);
    let f_v = schlick_weight(v.z);
    let f_retro = rr * (f_l + f_v + f_l * f_v * (rr - 1.0f));
    let f_d = (1.0f - 0.5f * f_l) * (1.0f - 0.5f * f_v);

    *pdf = l.z * INV_PI;
    return INV_PI * material.base_color * (f_d + f_retro);
}

fn eval_disney_clearcoat(l: vec3<f32>, v: vec3<f32>, h: vec3<f32>, material: Material, pdf: ptr<function, f32>) -> vec3<f32> {
    *pdf = 0.0f;
    if l.z <= 0.0f {
        return vec3<f32>(0.0f);
    }

    let v_dot_h = dot(v, h);

    let f = mix(0.04f, 1.0f, schlick_weight(v_dot_h));
    let d = gtr_1(h.z, material.clearcoat_roughness);
    let g = smith_g(l.z, 0.25f) * smith_g(v.z, 0.25f);
    let jacobian = 1.0f / (4.0f * v_dot_h);

    *pdf = d * h.z * jacobian;
    return vec3<f32>(f) * d * g;
}

fn eval_disney_microfacet_reflection(l: vec3<f32>, v: vec3<f32>, h: vec3<f32>, f: vec3<f32>, material: Material, hit_info: HitInfo, pdf: ptr<function, f32>) -> vec3<f32> {
    *pdf = 0.0f;
    if l.z <= 0.0f {
        return vec3<f32>(0.0f);
    }

    let d = gtr_2_aniso(h.z, h.x, h.y, material.a_x, material.a_y);
    let g_1 = smith_g_aniso(abs(v.z), v.x, v.y, material.a_x, material.a_y);
    let g_2 = g_1 * smith_g_aniso(abs(l.z), l.x, l.y, material.a_x, material.a_y);

    *pdf = g_1 * d / (4.0f * v.z);
    return f * d * g_2 / (4.0f * l.z * v.z);
}

fn eval_disney_microfacet_refraction(l: vec3<f32>, v: vec3<f32>, h: vec3<f32>, f: vec3<f32>, material: Material, hit_info: HitInfo, pdf: ptr<function, f32>) -> vec3<f32> {
    *pdf = 0.0f;
    if l.z >= 0.0f {
        return vec3<f32>(0.0f);
    }

    let l_dot_h = dot(l, h);
    let v_dot_h = dot(v, h);

    let d = gtr_2_aniso(h.z, h.x, h.y, material.a_x, material.a_y);
    let g_1 = smith_g_aniso(abs(v.z), v.x, v.y, material.a_x, material.a_y);
    let g_2 = g_1 * smith_g_aniso(abs(l.z), l.x, l.y, material.a_x, material.a_y);
    var denom = l_dot_h + v_dot_h * hit_info.eta;
    denom *= denom;
    let eta_2 = hit_info.eta * hit_info.eta;
    let jacobian = abs(l_dot_h) / denom;

    *pdf = g_1 * max(0.0f, v_dot_h) * d * jacobian / v.z;
    return pow(material.base_color, vec3<f32>(0.5f)) * (vec3<f32>(1.0f) - f) * d * g_2 * abs(v_dot_h) * jacobian * eta_2 / abs(l.z * v.z);
}

fn sqr(x: f32) -> f32 {
    return x * x;
}

fn gtr_1(n_dot_h: f32, a: f32) -> f32 {
    if a >= 1.0f {
        return INV_PI;
    }
    let a_2 = a * a;
    let t = 1.0f + (a_2 - 1.0f) * n_dot_h * n_dot_h;
    return (a_2 - 1.0f) / (PI * log(a_2) * t);
}

fn gtr_2_aniso(n_dot_h: f32, h_dot_x: f32, h_dot_y: f32, a_x: f32, a_y: f32) -> f32 {
    let a = h_dot_x / a_x;
    let b = h_dot_y / a_y;
    let c = a * a + b * b + n_dot_h * n_dot_h;
    return 1.0f / (PI * a_x * a_y * c * c);
}

fn smith_g(n_dot_v: f32, alpha_g: f32) -> f32 {
    let a = alpha_g * alpha_g;
    let b = n_dot_v * n_dot_v;
    return (2.0f * n_dot_v) / (n_dot_v + sqrt(a + b - a * b));
}

fn smith_g_aniso(n_dot_v: f32, v_dot_x: f32, v_dot_y: f32, a_x: f32, a_y: f32) -> f32 {
    let a = v_dot_x * a_x;
    let b = v_dot_y * a_y;
    let c = n_dot_v;
    return (2.0f * n_dot_v) / (n_dot_v + sqrt(a * a + b * b + c * c));
}

fn schlick_weight(u: f32) -> f32 {
    let m = clamp(1.0f - u, 0.0f, 1.0f);
    let m_2 = m * m;
    return m_2 * m_2 * m;
}

// Helper function to set actual material properties and other parameters of the hit surface
fn set_surface_properties(ray_dir: vec3<f32>, hit_info: ptr<function, HitInfo>, material: ptr<function, Material>) {
    hit_info.eta = select(material.ior, 1.0f / material.ior, hit_info.front_face);

    (*material).clearcoat_roughness = max(0.001f, material.clearcoat_roughness);

    // Base color
    if material.base_color_tex_id != 0xFFFFFFFF {
        (*material).base_color = pow(sample_texture(material.base_color_tex_id, hit_info.uv).rgb, vec3<f32>(2.2f));
    }

    // Transparency
    if material.transparency_tex_id != 0xFFFFFFFF {
        (*material).transparency = sample_texture(material.transparency_tex_id, hit_info.uv).a;
    }

    // Roughness
    if material.roughness_tex_id != 0xFFFFFFFF {
        let roughness = sample_texture(material.roughness_tex_id, hit_info.uv).g;
        (*material).roughness = roughness;
    }

    let aspect = sqrt(1.0f - material.anisotropy * 0.9f);
    (*material).a_x = max(0.001f, sqr(material.roughness) / aspect);
    (*material).a_y = max(0.001f, sqr(material.roughness) * aspect);

    // Metallic
    if material.metallic_tex_id != 0xFFFFFFFF {
        (*material).metallic = sample_texture(material.metallic_tex_id, hit_info.uv).b;
    }

    // Emission
    if material.emission_tex_id != 0xFFFFFFFF {
        (*material).emission = pow(sample_texture(material.emission_tex_id, hit_info.uv).rgb, vec3<f32>(2.2f));
    }

    // Build ONB from geometric normal
    var tangent: vec3<f32>;
    var bitangent: vec3<f32>;
    build_orthonormal_basis((*hit_info).normal, &tangent, &bitangent);
    (*hit_info).tbn = mat3x3<f32>(tangent, bitangent, (*hit_info).normal);

    // FIXME: Can cause a situation where a ray is reflected through a surface because this overrides the geometric normal
    if material.normal_tex_id != 0xFFFFFFFF {
        var texture_normal = normalize(sample_texture(material.normal_tex_id, (*hit_info).uv).xyz * 2.0f - 1.0f);

        let orig_normal = hit_info.normal;
        (*hit_info).normal = normalize(to_world(hit_info.tbn, texture_normal));

        // Rebuild ONB from texture normal
        build_orthonormal_basis((*hit_info).normal, &tangent, &bitangent);
        (*hit_info).tbn = mat3x3<f32>(tangent, bitangent, (*hit_info).normal);
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
    hit_info.point = fma(ray.direction, vec3<f32>(t), ray.origin);
    hit_info.distance = t;

    let front_face = det > 0.0f;
    hit_info.front_face = front_face;

    let n_0 = tri.vertices[0].normal;
    let n_1 = tri.vertices[1].normal;
    let n_2 = tri.vertices[2].normal;
    let normal = normalize(n_0 * (1.0f - u - v) + (n_1 * u) + (n_2 * v));
    hit_info.normal = select(-normal, normal, front_face);

    let t_0 = vec2<f32>(tri.vertices[0].tex_coord_x, tri.vertices[0].tex_coord_y);
    let t_1 = vec2<f32>(tri.vertices[1].tex_coord_x, tri.vertices[1].tex_coord_y);
    let t_2 = vec2<f32>(tri.vertices[2].tex_coord_x, tri.vertices[2].tex_coord_y);
    hit_info.uv = t_0 * (1.0f - u - v) + (t_1 * u) + (t_2 * v);

    hit_info.material_id = tri.material_id;
    hit_info.triangle_id = tri.triangle_id;

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

    // Consider: Testing root here outside

    var stack = array<u32, 16u>();
    var stack_ptr: u32 = 0u;
    var node_idx: u32 = 1u;

    loop {
        let left_node = bvh_nodes[node_idx];
        let right_node = bvh_nodes[node_idx + 1u];

        let dist_left = intersect_node(ray, left_node, hit_info.distance);
        let dist_right = intersect_node(ray, right_node, hit_info.distance);

        let hit_left = dist_left < 1e30f;
        let hit_right = dist_right < 1e30f;

        let intersect_left = hit_left && left_node.num_tris > 0u;
        let intersect_right = hit_right && right_node.num_tris > 0u;

        if intersect_left || intersect_right {
            let first = select(right_node.first_tri_or_child, left_node.first_tri_or_child, intersect_left);
            var end: u32;
            if !intersect_right {
                end = left_node.first_tri_or_child + left_node.num_tris;
            } else {
                end = right_node.first_tri_or_child + right_node.num_tris;
            }

            for (var i = first; i < end; i++) {
                let temp_hit_info = intersect_tri(ray, triangles[i]);
                if temp_hit_info.has_hit && temp_hit_info.distance < hit_info.distance {
                    hit_info = temp_hit_info;
                }
            }
        }

        let traverse_left = hit_left && left_node.num_tris == 0u;
        let traverse_right = hit_right && right_node.num_tris == 0u;

        if traverse_left || traverse_right {
            if traverse_left && traverse_right {
                let left_closer = dist_left < dist_right;
                node_idx = select(right_node.first_tri_or_child, left_node.first_tri_or_child, left_closer);
                stack[stack_ptr] = select(left_node.first_tri_or_child, right_node.first_tri_or_child, left_closer);
                stack_ptr++;
            } else {
                node_idx = select(right_node.first_tri_or_child, left_node.first_tri_or_child, traverse_left);
            }
        } else {
            if stack_ptr == 0u {
                break;
            }
            stack_ptr--;
            node_idx = stack[stack_ptr];
        }
    }

    return hit_info;
}

fn debug_bvh(ray: Ray, factor: f32) -> vec3<f32> {
    var hit_info = HitInfo();
    hit_info.distance = 1e30f;

    var stack = array<u32, 16u>();
    var stack_ptr: u32 = 0u;
    var node_idx: u32 = 1u; 

    var debug_value = 0.0f;
    loop {
        debug_value += 1.0f;

        let left_node = bvh_nodes[node_idx];
        let right_node = bvh_nodes[node_idx + 1u];

        let dist_left = intersect_node(ray, left_node, hit_info.distance);
        let dist_right = intersect_node(ray, right_node, hit_info.distance);

        let hit_left = dist_left < 1e30f;
        let hit_right = dist_right < 1e30f;

        let intersect_left = hit_left && left_node.num_tris > 0u;
        let intersect_right = hit_right && right_node.num_tris > 0u;

        if intersect_left || intersect_right {
            let first = select(right_node.first_tri_or_child, left_node.first_tri_or_child, intersect_left);
            var end: u32;
            if !intersect_right {
                end = left_node.first_tri_or_child + left_node.num_tris;
            } else {
                end = right_node.first_tri_or_child + right_node.num_tris;
            }
            debug_value += f32(end - first) * 1.1f;

            for (var i = first; i < end; i++) {
                let temp_hit_info = intersect_tri(ray, triangles[i]);
                if temp_hit_info.has_hit && temp_hit_info.distance < hit_info.distance {
                    hit_info = temp_hit_info;
                }
            }
        }

        let traverse_left = hit_left && left_node.num_tris == 0u;
        let traverse_right = hit_right && right_node.num_tris == 0u;

        if traverse_left || traverse_right {
            if traverse_left && traverse_right {
                let left_closer = dist_left < dist_right;
                node_idx = select(right_node.first_tri_or_child, left_node.first_tri_or_child, left_closer);
                stack[stack_ptr] = select(left_node.first_tri_or_child, right_node.first_tri_or_child, left_closer);
                stack_ptr++;
            } else {
                node_idx = select(right_node.first_tri_or_child, left_node.first_tri_or_child, traverse_left);
            }
        } else {
            if stack_ptr == 0u {
                break;
            }
            stack_ptr--;
            node_idx = stack[stack_ptr];
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

// https://www.reedbeta.com/blog/quick-and-easy-gpu-random-numbers-in-d3d11/
fn wang_hash(seed: ptr<function, u32>) -> u32 {
    *seed = (*seed ^ 61u) ^ (*seed >> 16u);
    *seed *= 9;
    *seed = *seed ^ (*seed >> 4u);
    *seed *= 0x27d4eb2d;
    *seed = *seed ^ (*seed >> 15u);
    return *seed;
}

fn rand_f32(input: ptr<function, u32>) -> f32 {
    return f32(wang_hash(input)) / f32(0xFFFFFFFF);
}

fn rand_point_on_tri(tri: Triangle, rng_seed: ptr<function, u32>) -> vec3<f32> {
    var u = rand_f32(rng_seed);
    var v = rand_f32(rng_seed);
    if u + v > 1.0f {
        u = 1.0f - u;
        v = 1.0f - v;
    }
    return tri.vertices[0].position + u * (tri.vertices[1].position - tri.vertices[0].position) + v * (tri.vertices[2].position - tri.vertices[0].position);
}

fn sample_texture(index: u32, uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(textures[index], textures_array_sampler, uv, 0.0f);
}

fn sample_ggx_vndf(ve: vec3<f32>, a_x: f32, a_y: f32, rng_seed: ptr<function, u32>) -> vec3<f32> {
    let u1 = rand_f32(rng_seed);
    let u2 = rand_f32(rng_seed);

    let Vh = normalize(vec3<f32>(a_x * ve.x, a_y * ve.y, ve.z));

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

    let Ne = normalize(vec3<f32>(a_x * Nh.x, a_y * Nh.y, max(0.0f, Nh.z)));
    return Ne;
}

fn sample_gtr_1(rgh: f32, rng_seed: ptr<function, u32>) -> vec3<f32> {
    let r_1 = rand_f32(rng_seed);
    let r_2 = rand_f32(rng_seed);

    let a = max(0.001f, rgh);
    let a_2 = a * a;

    let phi = r_1 * TWO_PI;

    let cos_theta = sqrt((1.0f - pow(a_2, 1.0f - r_2)) / (1.0f - a_2));
    let sin_theta = clamp(sqrt(1.0f - (cos_theta * cos_theta)), 0.0f, 1.0f);
    let sin_phi = sin(phi);
    let cos_phi = cos(phi);

    return vec3<f32>(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta);
}

fn cosine_sample_hemisphere(rng_seed: ptr<function, u32>) -> vec3<f32> {
    var dir: vec3<f32>;
    let r = sqrt(rand_f32(rng_seed));
    let phi = TWO_PI * rand_f32(rng_seed);
    dir.x = r * cos(phi);
    dir.y = r * sin(phi);
    dir.z = sqrt(max(0.0f, 1.0f - dir.x * dir.x - dir.y * dir.y));
    return dir;
}

fn uniform_sample_hemisphere(rng_seed: ptr<function, u32>) -> vec3<f32> {
    let r_1 = rand_f32(rng_seed);
    let r_2 = rand_f32(rng_seed);
    let r = sqrt(max(0.0f, 1.0f - r_1 * r_1));
    let phi = TWO_PI * r_2;
    return vec3<f32>(r * cos(phi), r * sin(phi), r_1);
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

fn tri_surface_area(triangle: Triangle) -> f32 {
    let e_1 = triangle.vertices[1].position - triangle.vertices[0].position;
    let e_2 = triangle.vertices[2].position - triangle.vertices[0].position;
    return 0.5f * length(cross(e_1, e_2));
}

fn luminance(c: vec3<f32>) -> f32 {
    return 0.212671f * c.x + 0.715160f * c.y + 0.072169f * c.z;
}
