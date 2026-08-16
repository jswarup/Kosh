//-- swarm/ops.rs -------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::silo::{ ISliceExt, U32 };
use	crate::swarm::traits::{ BackendKind, CpuKernelFn, KernelSource };

//---------------------------------------------------------------------------------------------------------------------------------

/// Standard PRNG and numeric algorithms shared across compute implementations.
pub struct SwarmMath;

impl SwarmMath
{
    #[inline]
    pub fn	WangHash( mut seed: u32) -> u32
    {
        seed = ( seed ^ 61) ^ ( seed >> 16);
        seed = seed.wrapping_mul( 9);
        seed = seed ^ ( seed >> 4);
        seed = seed.wrapping_mul( 0x27d4eb2d);
        seed = seed ^ ( seed >> 15);
        seed
    }

    #[inline]
    pub fn	HashToFloat( h: u32) -> f32
    {
        ( h & 0x00FF_FFFF) as f32 / 16777216.0
    }

    #[inline]
    pub fn	CollatzSteps( mut n: u32) -> u32
    {
        if n == 0 {
            return u32::MAX;
        }
        let  	mut steps = 0u32;
        while n != 1 {
            if n % 2 == 0 {
                n /= 2;
            } else {
                if n >= 0x5555_5555 {
                    return u32::MAX;
                }
                n = 3 * n + 1;
            }
            steps += 1;
        }
        steps
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Standard compute operations supported out-of-the-box across all backends.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardOp
{
    Double,
    VectorAdd,
    Collatz,
    PointCloud,
}

impl StandardOp
{
    pub fn	Label( &self) -> &'static str
    {
        match self {
            StandardOp::Double => "double_kernel",
            StandardOp::VectorAdd => "vecadd_kernel",
            StandardOp::Collatz => "collatz_kernel",
            StandardOp::PointCloud => "pointcloud_kernel",
        }
    }

    pub fn	EntryPoint( &self, backend: BackendKind) -> &'static str
    {
        match ( self, backend) {
            ( StandardOp::PointCloud, BackendKind::RustGpu) => "pts_pointcloud_cs",
            ( StandardOp::Double, BackendKind::CudaOxide) => "double_kernel",
            ( StandardOp::VectorAdd, BackendKind::CudaOxide) => "vecadd_kernel",
            ( StandardOp::Collatz, BackendKind::CudaOxide) => "collatz_kernel",
            ( StandardOp::PointCloud, BackendKind::CudaOxide) => "pointcloud_kernel",
            _ => "main",
        }
    }

    pub fn	Wgsl( &self) -> &'static str
    {
        match self {
            StandardOp::Double => r#"
                @group(0) @binding(0) var<storage, read_write> data: array<f32>;
                @compute @workgroup_size(64) fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let idx = gid.x;
                    if idx < arrayLength(&data) { data[idx] = data[idx] * 2.0; }
                }
            "#,
            StandardOp::VectorAdd => r#"
                @group(0) @binding(0) var<storage, read> a: array<f32>;
                @group(0) @binding(1) var<storage, read> b: array<f32>;
                @group(0) @binding(2) var<storage, read_write> result: array<f32>;
                @compute @workgroup_size(64) fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let idx = gid.x;
                    if idx < arrayLength(&result) { result[idx] = a[idx] + b[idx]; }
                }
            "#,
            StandardOp::Collatz => r#"
                @group(0) @binding(0) var<storage, read> input: array<u32>;
                @group(0) @binding(1) var<storage, read_write> output: array<u32>;
                fn collatz_steps(n_in: u32) -> u32 {
                    var n: u32 = n_in;
                    var steps: u32 = 0u;
                    while n != 1u {
                        if (n % 2u) == 0u { n = n / 2u; } else { n = 3u * n + 1u; }
                        steps = steps + 1u;
                    }
                    return steps;
                }
                @compute @workgroup_size(64) fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let idx = gid.x;
                    if idx < arrayLength(&output) { output[idx] = collatz_steps(input[idx]); }
                }
            "#,
            StandardOp::PointCloud => r#"
                @group(0) @binding(0) var<storage, read_write> points: array<vec4<f32>>;
                @compute @workgroup_size(64) fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let idx = gid.x;
                }
            "#,
        }
    }

    pub fn	Ptx( &self) -> &'static str
    {
        match self {
            StandardOp::Double => ".version 7.0\n.target sm_70\n.entry double_kernel",
            StandardOp::VectorAdd => ".version 7.0\n.target sm_70\n.entry vecadd_kernel",
            StandardOp::Collatz => ".version 7.0\n.target sm_70\n.entry collatz_kernel",
            StandardOp::PointCloud => ".version 7.0\n.target sm_70\n.entry pointcloud_kernel",
        }
    }

    pub fn	CpuKernelFn( &self) -> CpuKernelFn
    {
        match self {
            StandardOp::Double => Arc::new( |_inputs, outputs, gidX, _gidY, _gidZ| {
                if outputs.is_empty() {
                    return;
                }
                let  	outBuf: &mut [f32] = outputs[0].CastSliceMut();
                let  	idx = gidX.AsUsize();
                if idx < outBuf.len() {
                    outBuf[idx] *= 2.0;
                }
            }),
            StandardOp::VectorAdd => Arc::new( |inputs, outputs, gidX, _gidY, _gidZ| {
                if inputs.len() < 2 || outputs.is_empty() {
                    return;
                }
                let  	inA: &[f32] = inputs[0].CastSliceFrom();
                let  	inB: &[f32] = inputs[1].CastSliceFrom();
                let  	outBuf: &mut [f32] = outputs[0].CastSliceMut();
                let  	idx = gidX.AsUsize();
                if idx < outBuf.len() && idx < inA.len() && idx < inB.len() {
                    outBuf[idx] = inA[idx] + inB[idx];
                }
            }),
            StandardOp::Collatz => Arc::new( |inputs, outputs, gidX, _gidY, _gidZ| {
                if inputs.is_empty() || outputs.is_empty() {
                    return;
                }
                let  	inData: &[u32] = inputs[0].CastSliceFrom();
                let  	outBuf: &mut [u32] = outputs[0].CastSliceMut();
                let  	idx = gidX.AsUsize();
                if idx < outBuf.len() && idx < inData.len() {
                    outBuf[idx] = SwarmMath::CollatzSteps( inData[idx]);
                }
            }),
            StandardOp::PointCloud => Arc::new( |_inputs, outputs, gidX, _gidY, _gidZ| {
                if outputs.is_empty() {
                    return;
                }
                let  	outBuf: &mut [f32] = outputs[0].CastSliceMut();
                let  	idx = gidX.AsUsize();
                let  	base = idx * 4;
                if base + 3 < outBuf.len() {
                    let  	hx = SwarmMath::WangHash( ( idx as u32) * 3 + 0);
                    let  	hy = SwarmMath::WangHash( ( idx as u32) * 3 + 1);
                    let  	hz = SwarmMath::WangHash( ( idx as u32) * 3 + 2);

                    outBuf[base + 0] = SwarmMath::HashToFloat( hx) * 40.0 - 20.0;
                    outBuf[base + 1] = SwarmMath::HashToFloat( hy) * 40.0 - 20.0;
                    outBuf[base + 2] = SwarmMath::HashToFloat( hz) * 40.0 - 20.0;
                    outBuf[base + 3] = 1.0;
                }
            }),
        }
    }

    pub fn	KernelSource( &self, backend: BackendKind) -> KernelSource< 'static>
    {
        match backend {
            BackendKind::Cpu => KernelSource::CpuClosure( self.CpuKernelFn()),
            BackendKind::RustGpu => KernelSource::Wgsl( self.Wgsl()),
            BackendKind::CudaOxide => KernelSource::Ptx( self.Ptx()),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
