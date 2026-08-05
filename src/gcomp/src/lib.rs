#![cfg_attr(target_arch = "spirv", no_std)]
#![deny(warnings)]
#![allow(unexpected_cfgs)]

use spirv_std::{glam::UVec3, spirv};

pub fn collatz(mut n: u32) -> Option<u32> {
    let mut i = 0;
    if n == 0 {
        return None;
    }
    while n != 1 {
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            // Overflow? (i.e. 3*n + 1 > 0xffff_ffff)
            if n >= 0x5555_5555 {
                return None;
            }
            3 * n + 1
        };
        i += 1;
    }
    Some(i)
}

#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] input: &[u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: &mut [u32],
) {
    let index = id.x as usize;
    if index < output.len() {
        output[index] = collatz(input[index]).unwrap_or(u32::MAX);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Wang hash — a fast, deterministic integer hash for GPU-based pseudo-random number generation.
fn	wang_hash( mut seed: u32) -> u32
{
    seed = ( seed ^ 61) ^ ( seed >> 16);
    seed = seed.wrapping_mul( 9);
    seed = seed ^ ( seed >> 4);
    seed = seed.wrapping_mul( 0x27d4eb2d);
    seed = seed ^ ( seed >> 15);
    seed
}

/// Maps a hash value to a float in [0.0, 1.0) by masking to 24 bits and dividing by 2^24.
fn	hash_to_float( h: u32) -> f32
{
    ( h & 0x00FF_FFFF) as f32 / 16777216.0
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Generates pseudo-random 3D point positions in [-20, 20]³ using Wang hash PRNG.
/// Each thread writes one Vec4(x, y, z, 1.0) to the output storage buffer.
#[spirv(compute(threads(64)))]
pub fn	pts_pointcloud_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] output: &mut [spirv_std::glam::Vec4],
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

/// Returns 3D positions for a block of dimension 100 (bounding box [-50, 50]).
pub fn	pts_wireframe_block_vertices() -> [spirv_std::glam::Vec4; 8]
{
    [
        spirv_std::glam::Vec4::new( -50.0, -50.0, -50.0, 1.0),
        spirv_std::glam::Vec4::new(  50.0, -50.0, -50.0, 1.0),
        spirv_std::glam::Vec4::new(  50.0,  50.0, -50.0, 1.0),
        spirv_std::glam::Vec4::new( -50.0,  50.0, -50.0, 1.0),
        spirv_std::glam::Vec4::new( -50.0, -50.0,  50.0, 1.0),
        spirv_std::glam::Vec4::new(  50.0, -50.0,  50.0, 1.0),
        spirv_std::glam::Vec4::new(  50.0,  50.0,  50.0, 1.0),
        spirv_std::glam::Vec4::new( -50.0,  50.0,  50.0, 1.0),
    ]
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Writes the 24 line vertex coordinates for rendering the 100-dimension wireframe block.
#[spirv(compute(threads(32)))]
pub fn	pts_wireframe_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] output: &mut [spirv_std::glam::Vec4],
)
{
    let  	verts = pts_wireframe_block_vertices();
    let  	indices: [u32; 24] = [
        0, 1,  1, 2,  2, 3,  3, 0,
        4, 5,  5, 6,  6, 7,  7, 4,
        0, 4,  1, 5,  2, 6,  3, 7
    ];
    let  	idx = id.x as usize;
    if idx < 24 && idx < output.len() {
        output[idx] = verts[indices[idx] as usize];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

