//-- vertshade.rs -----------------------------------------------------------------------------------------------------------------
#![allow( unexpected_cfgs, non_snake_case, unused_imports)]

use	spirv_std::{
    glam::{ Vec2, Vec3, Vec4, UVec3 },
    num_traits::Float,
    spirv,
};

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
