#![cfg_attr( target_arch = "spirv", no_std)]
#![deny( warnings)]
#![allow( unexpected_cfgs, non_snake_case, unused_imports)]

use	spirv_std::{
    glam::{ Vec2, Vec3, Vec4, UVec3 },
    num_traits::Float,
    spirv,
};

//---------------------------------------------------------------------------------------------------------------------------------
// Pure Mathematical Algorithms (#![no_std] portable across CPU, SPIR-V, and CUDA)
//---------------------------------------------------------------------------------------------------------------------------------

/// Wang hash — a fast, deterministic integer hash for GPU/CPU pseudo-random number generation.
#[inline( always)]
pub fn	wang_hash( mut seed: u32) -> u32
{
    seed = ( seed ^ 61) ^ ( seed >> 16);
    seed = seed.wrapping_mul( 9);
    seed = seed ^ ( seed >> 4);
    seed = seed.wrapping_mul( 0x27d4eb2d);
    seed = seed ^ ( seed >> 15);
    seed
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Maps a hash value to a float in [0.0, 1.0) by masking to 24 bits and dividing by 2^24.
#[inline( always)]
pub fn	hash_to_float( h: u32) -> f32
{
    ( h & 0x00FF_FFFF) as f32 / 16777216.0
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Computes the number of Collatz steps for an integer.
#[inline( always)]
pub fn	collatz( mut n: u32) -> Option< u32>
{
    let  	mut i = 0;
    if n == 0 {
        return None;
    }
    while n != 1 {
        n = if n.is_multiple_of( 2) {
            n / 2
        } else {
            // Overflow guard (3*n + 1 > 0xffff_ffff)
            if n >= 0x5555_5555 {
                return None;
            }
            3 * n + 1
        };
        i += 1;
    }
    Some( i)
}

//---------------------------------------------------------------------------------------------------------------------------------
// Unified Element-Wise Kernel Operations (Executed identically by CPU, Rust-GPU, and CUDA)
//---------------------------------------------------------------------------------------------------------------------------------

/// In-place element doubling.
#[inline( always)]
pub fn	double_elem( idx: usize, data: &mut [f32])
{
    if idx < data.len() {
        data[idx] *= 2.0;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Element-wise vector addition: out = a + b.
#[inline( always)]
pub fn	vector_add_elem( idx: usize, a: &[f32], b: &[f32], out: &mut [f32])
{
    if idx < out.len() && idx < a.len() && idx < b.len() {
        out[idx] = a[idx] + b[idx];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Element-wise Collatz sequence computation.
#[inline( always)]
pub fn	collatz_elem( idx: usize, input: &[u32], output: &mut [u32])
{
    if idx < output.len() && idx < input.len() {
        output[idx] = collatz( input[idx]).unwrap_or( u32::MAX);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Element-wise 3D point cloud generation in [-20, 20]³.
#[inline( always)]
pub fn	pointcloud_elem( idx: usize, output: &mut [f32])
{
    let  	base = idx * 4;
    if base + 3 < output.len() {
        let  	hx = wang_hash( idx as u32 * 3 + 0);
        let  	hy = wang_hash( idx as u32 * 3 + 1);
        let  	hz = wang_hash( idx as u32 * 3 + 2);

        let  	x = hash_to_float( hx) * 40.0 - 20.0;
        let  	y = hash_to_float( hy) * 40.0 - 20.0;
        let  	z = hash_to_float( hz) * 40.0 - 20.0;

        output[base + 0] = x;
        output[base + 1] = y;
        output[base + 2] = z;
        output[base + 3] = 1.0;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Camera uniform parameter block for dedicated vertex & fragment rendering pipelines.
#[repr( C)]
#[derive( Copy, Clone, Debug, Default, PartialEq)]
pub struct CameraUniforms
{
    pub _RotX:       f32,
    pub _RotY:       f32,
    pub _Zoom:       f32,
    pub _PanX:       f32,
    pub _PanY:       f32,
    pub _Fov:        f32,
    pub _Distance:   f32,
    pub _Width:      f32,
    pub _Height:     f32,
    pub _CenterX:    f32,
    pub _CenterY:    f32,
    pub _CenterZ:    f32,
    pub _ScaleNorm:  f32,
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Transforms a 3D point in world space to NDC coordinates and returns (clip_pos, point_size, depth_factor).
#[inline( always)]
pub fn	vertex_transform_pos( pos: Vec3, cam: &CameraUniforms) -> ( Vec4, f32, f32)
{
    let  	nx = ( pos.x - cam._CenterX) * cam._ScaleNorm;
    let  	ny = ( pos.y - cam._CenterY) * cam._ScaleNorm;
    let  	nz = ( pos.z - cam._CenterZ) * cam._ScaleNorm;

    let  	cosY = cam._RotY.cos();
    let  	sinY = cam._RotY.sin();
    let  	x1 = nx * cosY + nz * sinY;
    let  	z1 = -nx * sinY + nz * cosY;

    let  	cosX = cam._RotX.cos();
    let  	sinX = cam._RotX.sin();
    let  	y2 = ny * cosX - z1 * sinX;
    let  	z2 = ny * sinX + z1 * cosX;

    let  	denom = cam._Distance + z2;
    let  	w = if denom > 1e-4 { denom } else { 1e-4 };
    let  	scale = ( cam._Fov * cam._Zoom) / w;

    let  	projX = cam._Width / 2.0 + cam._PanX + x1 * scale;
    let  	projY = cam._Height / 2.0 + cam._PanY - y2 * scale;

    let  	ndcX = ( projX / cam._Width) * 2.0 - 1.0;
    let  	ndcY = ( projY / cam._Height) * 2.0 - 1.0;
    let  	ndcZ = z2 / 400.0;

    let  	depthFactor = 0.3f32.max( 1.0f32.min( ( 300.0 - z2) / 400.0));
    let  	ptSize = 6.0 + depthFactor * 8.0;

    ( Vec4::new( ndcX, ndcY, ndcZ, 1.0), ptSize, depthFactor)
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Evaluates point-sprite fragment shading: circular falloff, glowing white core, and depth alpha.
#[inline( always)]
pub fn	fragment_point_color( pointCoord: Vec2, baseColor: Vec4) -> Vec4
{
    let  	delta = pointCoord - Vec2::new( 0.5, 0.5);
    let  	distSq = delta.x * delta.x + delta.y * delta.y;
    if distSq > 0.25 {
        Vec4::ZERO
    } else {
        let  	dist = distSq.sqrt() * 2.0;
        let  	alpha = ( 1.0 - dist).max( 0.0) * baseColor.w;
        let  	core = if dist < 0.3 { 1.0 - dist / 0.3 } else { 0.0 };
        let  	r = baseColor.x + core * ( 1.0 - baseColor.x);
        let  	g = baseColor.y + core * ( 1.0 - baseColor.y);
        let  	b = baseColor.z + core * ( 1.0 - baseColor.z);
        Vec4::new( r, g, b, alpha)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Element-wise 3D point cloud camera transformation and perspective projection.
/// Reads 3D point (x, y, z) and camera uniform parameters, writes 6 projected values:
/// [screen_x, screen_y, radius, core_radius, alpha, depth_factor].
#[inline( always)]
pub fn	camera_transform_elem(
    idx: usize,
    in_points: &[f32],
    cam_params: &[f32],
    out_projected: &mut [f32],
)
{
    let  	inBase = idx * 3;
    let  	outBase = idx * 6;

    if inBase + 2 < in_points.len() && outBase + 5 < out_projected.len() && cam_params.len() >= 13 {
        let  	x = in_points[inBase + 0];
        let  	y = in_points[inBase + 1];
        let  	z = in_points[inBase + 2];

        let  	rotX = cam_params[0];
        let  	rotY = cam_params[1];
        let  	zoom = cam_params[2];
        let  	panX = cam_params[3];
        let  	panY = cam_params[4];
        let  	fov = cam_params[5];
        let  	distance = cam_params[6];
        let  	width = cam_params[7];
        let  	height = cam_params[8];
        let  	cx = cam_params[9];
        let  	cy = cam_params[10];
        let  	cz = cam_params[11];
        let  	scaleNorm = cam_params[12];

        let  	nx = ( x - cx) * scaleNorm;
        let  	ny = ( y - cy) * scaleNorm;
        let  	nz = ( z - cz) * scaleNorm;

        let  	cosY = rotY.cos();
        let  	sinY = rotY.sin();
        let  	x1 = nx * cosY + nz * sinY;
        let  	z1 = -nx * sinY + nz * cosY;

        let  	cosX = rotX.cos();
        let  	sinX = rotX.sin();
        let  	y2 = ny * cosX - z1 * sinX;
        let  	z2 = ny * sinX + z1 * cosX;

        let  	scale = ( fov * zoom) / ( distance + z2);

        let  	projX = width / 2.0 + panX + x1 * scale;
        let  	projY = height / 2.0 + panY - y2 * scale;

        let  	depthFactor = 0.3f32.max( 1.0f32.min( ( 300.0 - z2) / 400.0));
        let  	radius = 3.0 + depthFactor * 4.0;
        let  	coreRadius = 1.0 + depthFactor * 1.5;
        let  	alpha = 0.5 + depthFactor * 0.5;

        out_projected[outBase + 0] = projX;
        out_projected[outBase + 1] = projY;
        out_projected[outBase + 2] = radius;
        out_projected[outBase + 3] = coreRadius;
        out_projected[outBase + 4] = alpha;
        out_projected[outBase + 5] = depthFactor;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// Rust-GPU (SPIR-V) Entrypoints
//---------------------------------------------------------------------------------------------------------------------------------

#[spirv( compute( threads( 64)))]
pub fn	double_cs(
    #[spirv( global_invocation_id)] id: UVec3,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] data: &mut [f32],
)
{
    double_elem( id.x as usize, data);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[spirv( compute( threads( 64)))]
pub fn	vecadd_cs(
    #[spirv( global_invocation_id)] id: UVec3,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] a: &[f32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 1)] b: &[f32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 2)] result: &mut [f32],
)
{
    vector_add_elem( id.x as usize, a, b, result);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[spirv( compute( threads( 64)))]
pub fn	main_cs(
    #[spirv( global_invocation_id)] id: UVec3,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] input: &[u32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 1)] output: &mut [u32],
)
{
    collatz_elem( id.x as usize, input, output);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[spirv( compute( threads( 64)))]
pub fn	collatz_cs(
    #[spirv( global_invocation_id)] id: UVec3,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] input: &[u32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 1)] output: &mut [u32],
)
{
    collatz_elem( id.x as usize, input, output);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Generates pseudo-random 3D point positions in [-20, 20]³ using Wang hash PRNG.
/// Each thread writes one Vec4(x, y, z, 1.0) to the output storage buffer.
#[spirv( compute( threads( 64)))]
pub fn	pts_pointcloud_cs(
    #[spirv( global_invocation_id)] id: UVec3,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] output: &mut [spirv_std::glam::Vec4],
)
{
    let  	idx = id.x as usize;
    if idx >= output.len() {
        return;
    }

    let  	hx = wang_hash( idx as u32 * 3 + 0);
    let  	hy = wang_hash( idx as u32 * 3 + 1);
    let  	hz = wang_hash( idx as u32 * 3 + 2);

    let  	x = hash_to_float( hx) * 40.0 - 20.0;
    let  	y = hash_to_float( hy) * 40.0 - 20.0;
    let  	z = hash_to_float( hz) * 40.0 - 20.0;

    output[idx] = spirv_std::glam::Vec4::new( x, y, z, 1.0);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Transforms and projects 3D point cloud coordinates to screen space with camera uniforms.
#[spirv( compute( threads( 64)))]
pub fn	camera_transform_cs(
    #[spirv( global_invocation_id)] id: UVec3,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] in_points: &[f32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 1)] cam_params: &[f32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 2)] out_projected: &mut [f32],
)
{
    camera_transform_elem( id.x as usize, in_points, cam_params, out_projected);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Dedicated GPU Vertex Shader for hardware-accelerated 3D graphics scene display.
/// Transforms 3D object-space vertices to Normalized Device Coordinates (NDC)
/// and computes depth-scaled point sizes and color attributes.
#[spirv( vertex)]
pub fn	scene_vs(
    #[spirv( vertex_index)] vertIdx: u32,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] inPoints: &[Vec4],
    #[spirv( uniform, descriptor_set = 0, binding = 1)] camUniforms: &CameraUniforms,
    #[spirv( position)] outPosition: &mut Vec4,
    #[spirv( point_size)] outPointSize: &mut f32,
    #[spirv( location = 0)] outColor: &mut Vec4,
)
{
    let  	idx = vertIdx as usize;
    if idx < inPoints.len() {
        let  	pt = inPoints[idx];
        let  	pos = Vec3::new( pt.x, pt.y, pt.z);
        let  	( clipPos, ptSize, depthFactor) = vertex_transform_pos( pos, camUniforms);
        *outPosition = clipPos;
        *outPointSize = ptSize;
        *outColor = Vec4::new( 0.0, 0.95, 1.0, 0.5 + depthFactor * 0.5);
    } else {
        *outPosition = Vec4::ZERO;
        *outPointSize = 1.0;
        *outColor = Vec4::ZERO;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Dedicated GPU Fragment Shader for hardware-accelerated 3D graphics scene display.
/// Renders anti-aliased point sprites with depth-modulated glowing cores directly on hardware rasterizer.
#[spirv( fragment)]
pub fn	scene_fs(
    #[spirv( location = 0)] inColor: Vec4,
    #[spirv( point_coord)] pointCoord: Vec2,
    #[spirv( location = 0)] outFragColor: &mut Vec4,
)
{
    *outFragColor = fragment_point_color( pointCoord, inColor);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Tests whether a 3D point (with radius) is inside the 6 frustum planes.
/// Returns 1 if visible (inside all 6 planes), 0 if culled.
#[inline( always)]
pub fn	frustum_cull_elem(
    idx: usize,
    in_points: &[f32],
    frustum_planes: &[f32],
    out_visible: &mut [u32],
)
{
    let  	base = idx * 3;
    if base + 2 < in_points.len() && idx < out_visible.len() && frustum_planes.len() >= 24 {
        let  	x = in_points[base + 0];
        let  	y = in_points[base + 1];
        let  	z = in_points[base + 2];

        let  	mut visible = 1u32;
        let  	mut p = 0;
        while p < 6 {
            let  	pBase = p * 4;
            let  	a = frustum_planes[pBase + 0];
            let  	b = frustum_planes[pBase + 1];
            let  	c = frustum_planes[pBase + 2];
            let  	d = frustum_planes[pBase + 3];

            let  	dist = a * x + b * y + c * z + d;
            if dist < -0.5 {
                visible = 0;
                break;
            }
            p += 1;
        }

        out_visible[idx] = visible;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// GPU Compute shader for accelerated camera frustum culling.
#[spirv( compute( threads( 64)))]
pub fn	frustum_cull_cs(
    #[spirv( global_invocation_id)] id: UVec3,
    #[spirv( storage_buffer, descriptor_set = 0, binding = 0)] in_points: &[f32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 1)] frustum_planes: &[f32],
    #[spirv( storage_buffer, descriptor_set = 0, binding = 2)] out_visible: &mut [u32],
)
{
    frustum_cull_elem( id.x as usize, in_points, frustum_planes, out_visible);
}

//---------------------------------------------------------------------------------------------------------------------------------


