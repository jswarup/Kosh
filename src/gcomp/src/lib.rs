#![cfg_attr( target_arch = "spirv", no_std)]
#![deny( warnings)]
#![allow( unexpected_cfgs)]

use	spirv_std::{ glam::UVec3, spirv };

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
