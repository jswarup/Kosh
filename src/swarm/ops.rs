//-- swarm/ops.rs -------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::silo::ISliceExt;
use	crate::swarm::traits::{ BackendKind, CpuKernelFn, KernelSource };

//---------------------------------------------------------------------------------------------------------------------------------

/// Standard PRNG and numeric algorithms shared across compute implementations (backed by gcomp).
pub struct SwarmMath;

impl SwarmMath
{
    #[inline( always)]
    pub fn	WangHash( seed: u32) -> u32
    {
        gcomp::wang_hash( seed)
    }

    #[inline( always)]
    pub fn	HashToFloat( h: u32) -> f32
    {
        gcomp::hash_to_float( h)
    }

    #[inline( always)]
    pub fn	CollatzSteps( n: u32) -> u32
    {
        gcomp::collatz( n).unwrap_or( u32::MAX)
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
            ( StandardOp::Double, BackendKind::RustGpu) => "double_cs",
            ( StandardOp::VectorAdd, BackendKind::RustGpu) => "vecadd_cs",
            ( StandardOp::Collatz, BackendKind::RustGpu) => "collatz_cs",
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
                @compute @workgroup_size(64) fn double_cs(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let idx = gid.x;
                    if idx < arrayLength(&data) { data[idx] = data[idx] * 2.0; }
                }
            "#,
            StandardOp::VectorAdd => r#"
                @group(0) @binding(0) var<storage, read> a: array<f32>;
                @group(0) @binding(1) var<storage, read> b: array<f32>;
                @group(0) @binding(2) var<storage, read_write> result: array<f32>;
                @compute @workgroup_size(64) fn vecadd_cs(@builtin(global_invocation_id) gid: vec3<u32>) {
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
                @compute @workgroup_size(64) fn collatz_cs(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let idx = gid.x;
                    if idx < arrayLength(&output) { output[idx] = collatz_steps(input[idx]); }
                }
            "#,
            StandardOp::PointCloud => r#"
                @group(0) @binding(0) var<storage, read_write> points: array<vec4<f32>>;
                fn wang_hash(seed_in: u32) -> u32 {
                    var seed: u32 = seed_in;
                    seed = (seed ^ 61u) ^ (seed >> 16u);
                    seed = seed * 9u;
                    seed = seed ^ (seed >> 4u);
                    seed = seed * 0x27d4eb2du;
                    seed = seed ^ (seed >> 15u);
                    return seed;
                }
                fn hash_to_float(h: u32) -> f32 {
                    return f32(h & 0x00FFFFFFu) / 16777216.0;
                }
                @compute @workgroup_size(64) fn pts_pointcloud_cs(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let idx = gid.x;
                    if idx < arrayLength(&points) {
                        let hx = wang_hash(idx * 3u + 0u);
                        let hy = wang_hash(idx * 3u + 1u);
                        let hz = wang_hash(idx * 3u + 2u);
                        let x = hash_to_float(hx) * 40.0 - 20.0;
                        let y = hash_to_float(hy) * 40.0 - 20.0;
                        let z = hash_to_float(hz) * 40.0 - 20.0;
                        points[idx] = vec4<f32>(x, y, z, 1.0);
                    }
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
                gcomp::double_elem( gidX.AsUsize(), outBuf);
            }),
            StandardOp::VectorAdd => Arc::new( |inputs, outputs, gidX, _gidY, _gidZ| {
                if inputs.len() < 2 || outputs.is_empty() {
                    return;
                }
                let  	inA: &[f32] = inputs[0].CastSliceFrom();
                let  	inB: &[f32] = inputs[1].CastSliceFrom();
                let  	outBuf: &mut [f32] = outputs[0].CastSliceMut();
                gcomp::vector_add_elem( gidX.AsUsize(), inA, inB, outBuf);
            }),
            StandardOp::Collatz => Arc::new( |inputs, outputs, gidX, _gidY, _gidZ| {
                if inputs.is_empty() || outputs.is_empty() {
                    return;
                }
                let  	inData: &[u32] = inputs[0].CastSliceFrom();
                let  	outBuf: &mut [u32] = outputs[0].CastSliceMut();
                gcomp::collatz_elem( gidX.AsUsize(), inData, outBuf);
            }),
            StandardOp::PointCloud => Arc::new( |_inputs, outputs, gidX, _gidY, _gidZ| {
                if outputs.is_empty() {
                    return;
                }
                let  	outBuf: &mut [f32] = outputs[0].CastSliceMut();
                gcomp::pointcloud_elem( gidX.AsUsize(), outBuf);
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

/// Universal compute kernel definition macro that creates a shared SIMT function.
#[macro_export]
macro_rules! swarm_kernel {
    (
        $name:ident,
        |$idx:ident, $input:ident: $in_ty:ty, $output:ident: $out_ty:ty| $body:block
    ) => {
        #[inline( always)]
        pub fn $name( $idx: usize, $input: $in_ty, $output: $out_ty)
        {
            $body
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
