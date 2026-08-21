//-- swarm/mod.rs --------------------------------------------------------------------------------------------------------------------

pub mod traits;
pub mod ops;
pub mod cpu;
pub mod rustgpu;
pub mod cudaoxide;
pub mod engine;
pub mod gpusop;
pub mod scene;
pub mod viewport;

pub use	traits::{
    BackendKind, BufferUsage, CpuKernelFn, IComputeBuffer, IComputeDevice,
    IComputeKernel, KernelSource, SwarmError, WorkgroupDim,
};
pub use	ops::{ StandardOp, SwarmMath };
pub use	cpu::{ CpuBuffer, CpuDevice, CpuKernel };
pub use	rustgpu::{ RustGpuBuffer, RustGpuDevice, RustGpuKernel };
pub use	cudaoxide::{ CudaOxideBuffer, CudaOxideDevice, CudaOxideKernel };
pub use	engine::{ SwarmBuffer, SwarmDevice, SwarmEngine, SwarmCluster, SwarmKernel };
pub use	gpusop::IGpuOp;
pub use	scene::{ Camera, SceneGraph, SceneDisplayFrame };
pub use	viewport::{ ViewportRenderer, ViewportVertex, ViewportUniforms, GpuMesh };

#[cfg( test)]
mod _tests;

//---------------------------------------------------------------------------------------------------------------------------------
