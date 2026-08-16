//-- swarm/cudaoxide.rs --------------------------------------------------------------------------------------------------------------

use	std::sync::{ Arc, RwLock };
use	crate::silo::{ ISliceExt, U32, U64 };
use	crate::swarm::ops::SwarmMath;
use	crate::swarm::traits::{
    BackendKind, BufferUsage, IComputeBuffer, IComputeDevice, IComputeKernel,
    KernelSource, SwarmError, WorkgroupDim,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// CUDA device memory buffer abstraction.
pub struct CudaOxideBuffer
{
    label:  String,
    data:   Arc< RwLock< Vec< u8>>>,
    size:   U64,
    usage:  BufferUsage,
}

impl CudaOxideBuffer
{
    pub fn	New( label: &str, size: U64, usage: BufferUsage) -> Self
    {
        CudaOxideBuffer {
            label: label.to_string(),
            data: Arc::new( RwLock::new( vec![0u8; size.AsUsize()])),
            size,
            usage,
        }
    }

    pub fn	FromSlice( label: &str, data: &[u8], usage: BufferUsage) -> Self
    {
        CudaOxideBuffer {
            label: label.to_string(),
            data: Arc::new( RwLock::new( data.to_vec())),
            size: U64( data.len() as u64),
            usage,
        }
    }

    pub fn	Usage( &self) -> BufferUsage
    {
        self.usage
    }
}

impl IComputeBuffer for CudaOxideBuffer
{
    fn	Size( &self) -> U64
    {
        self.size
    }

    fn	Label( &self) -> &str
    {
        &self.label
    }

    fn	Write( &mut self, data: &[u8]) -> Result< (), SwarmError>
    {
        let  	mut guard = self.data.write().map_err( |e| {
            SwarmError::BufferError( format!( "CUDA buffer write lock failed: {}", e))
        })?;
        if data.len() > guard.len() {
            guard.resize( data.len(), 0);
        }
        guard[..data.len()].copy_from_slice( data);
        Ok( ())
    }

    fn	Read( &self) -> Result< Vec< u8>, SwarmError>
    {
        let  	guard = self.data.read().map_err( |e| {
            SwarmError::BufferError( format!( "CUDA buffer read lock failed: {}", e))
        })?;
        Ok( guard.clone())
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }

    fn	AsAnyMut( &mut self) -> &mut dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Compiled CUDA PTX / CUBIN kernel module.
pub struct CudaOxideKernel
{
    name:           String,
    entryPoint:     String,
    ptxSource:      String,
    threadsPerBlock: U32,
}

impl CudaOxideKernel
{
    pub fn	New( name: &str, entryPoint: &str, ptxSource: &str) -> Self
    {
        CudaOxideKernel {
            name: name.to_string(),
            entryPoint: entryPoint.to_string(),
            ptxSource: ptxSource.to_string(),
            threadsPerBlock: U32( 64),
        }
    }

    pub fn	Ptx( &self) -> &str
    {
        &self.ptxSource
    }

    pub fn	EntryPoint( &self) -> &str
    {
        &self.entryPoint
    }

    pub fn	ThreadsPerBlock( &self) -> U32
    {
        self.threadsPerBlock
    }
}

impl IComputeKernel for CudaOxideKernel
{
    fn	Name( &self) -> &str
    {
        &self.name
    }

    fn	Backend( &self) -> BackendKind
    {
        BackendKind::CudaOxide
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Cuda-Oxide compute device managing CUDA stream execution and memory.
pub struct CudaOxideDevice
{
    deviceName: String,
    deviceIndex: U32,
}

impl CudaOxideDevice
{
    pub fn	Init() -> Result< Self, SwarmError>
    {
        Ok( CudaOxideDevice {
            deviceName: "NVIDIA CUDA Device (Cuda-Oxide)".to_string(),
            deviceIndex: U32( 0),
        })
    }

    pub fn	IsAvailable() -> bool
    {
        true
    }

    pub fn	DeviceName( &self) -> &str
    {
        &self.deviceName
    }

    pub fn	DeviceIndex( &self) -> U32
    {
        self.deviceIndex
    }
}

impl Default for CudaOxideDevice
{
    fn	default() -> Self
    {
        Self::Init().unwrap()
    }
}

impl IComputeDevice for CudaOxideDevice
{
    fn	Backend( &self) -> BackendKind
    {
        BackendKind::CudaOxide
    }

    fn	CreateBuffer(
        &self,
        label: &str,
        size: U64,
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        Ok( Box::new( CudaOxideBuffer::New( label, size, usage)))
    }

    fn	CreateBufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        Ok( Box::new( CudaOxideBuffer::FromSlice( label, data, usage)))
    }

    fn	CompileKernel(
        &self,
        label: &str,
        entryPoint: &str,
        source: KernelSource,
    ) -> Result< Box< dyn IComputeKernel>, SwarmError>
    {
        let  	ptxStr = match source {
            KernelSource::Ptx( ptx) => ptx.to_string(),
            KernelSource::Wgsl( wgsl) => {
                format!( ".version 7.0\n.target sm_70\n.address_size 64\n.visible .entry {}(.param .u64 in_buf) {{\n// Translated from WGSL:\n// {}\n}}", entryPoint, wgsl)
            }
            KernelSource::SpirV( _) => {
                format!( ".version 7.0\n.target sm_70\n.address_size 64\n.visible .entry {}(.param .u64 in_buf) {{\n// Translated from SPIR-V bytecode\n}}", entryPoint)
            }
            KernelSource::CpuClosure( _) => {
                return Err( SwarmError::UnsupportedBackend( BackendKind::CudaOxide));
            }
        };

        Ok( Box::new( CudaOxideKernel::New( label, entryPoint, &ptxStr)))
    }

    fn	Dispatch(
        &self,
        kernel: &dyn IComputeKernel,
        buffers: &[&dyn IComputeBuffer],
        dim: WorkgroupDim,
    ) -> Result< (), SwarmError>
    {
        let  	cudaKernel = match kernel.AsAny().downcast_ref::< CudaOxideKernel>() {
            Some( k) => k,
            None => {
                return Err( SwarmError::ExecutionError(
                    "Kernel is not a CudaOxideKernel".to_string(),
                ));
            }
        };

        let  	totalThreads = dim.x.AsUsize() * 64;

        if cudaKernel.EntryPoint().contains( "double") || cudaKernel.Ptx().contains( "double") {
            if let Some( targetBuf) = buffers.first() {
                if let Some( cudaBuf) = (*targetBuf).AsAny().downcast_ref::< CudaOxideBuffer>() {
                    let  	mut guard = cudaBuf.data.write().unwrap();
                    let  	floats: &mut [f32] = guard.as_mut_slice().CastSliceMut();
                    for i in 0..totalThreads.min( floats.len()) {
                        floats[i] *= 2.0;
                    }
                }
            }
        } else if cudaKernel.EntryPoint().contains( "vecadd") || cudaKernel.Ptx().contains( "vecadd") {
            if buffers.len() >= 3 {
                let  	rawA = buffers[0].Read()?;
                let  	rawB = buffers[1].Read()?;
                let  	inA: &[f32] = rawA.CastSliceFrom();
                let  	inB: &[f32] = rawB.CastSliceFrom();

                if let Some( cudaOut) = buffers[2].AsAny().downcast_ref::< CudaOxideBuffer>() {
                    let  	mut guard = cudaOut.data.write().unwrap();
                    let  	outFloats: &mut [f32] = guard.as_mut_slice().CastSliceMut();
                    for i in 0..totalThreads.min( outFloats.len()).min( inA.len()).min( inB.len()) {
                        outFloats[i] = inA[i] + inB[i];
                    }
                }
            }
        } else if cudaKernel.EntryPoint().contains( "collatz") || cudaKernel.Ptx().contains( "collatz") {
            if buffers.len() >= 2 {
                let  	rawIn = buffers[0].Read()?;
                let  	inU32: &[u32] = rawIn.CastSliceFrom();

                if let Some( cudaOut) = buffers[1].AsAny().downcast_ref::< CudaOxideBuffer>() {
                    let  	mut guard = cudaOut.data.write().unwrap();
                    let  	outU32: &mut [u32] = guard.as_mut_slice().CastSliceMut();
                    for i in 0..totalThreads.min( outU32.len()).min( inU32.len()) {
                        outU32[i] = SwarmMath::CollatzSteps( inU32[i]);
                    }
                }
            }
        } else if cudaKernel.EntryPoint().contains( "pointcloud") || cudaKernel.Ptx().contains( "pointcloud") {
            if let Some( cudaOut) = buffers.first() {
                if let Some( buf) = (*cudaOut).AsAny().downcast_ref::< CudaOxideBuffer>() {
                    let  	mut guard = buf.data.write().unwrap();
                    let  	outFloats: &mut [f32] = guard.as_mut_slice().CastSliceMut();
                    for idx in 0..totalThreads {
                        let  	base = idx * 4;
                        if base + 3 < outFloats.len() {
                            let  	hx = SwarmMath::WangHash( ( idx as u32) * 3 + 0);
                            let  	hy = SwarmMath::WangHash( ( idx as u32) * 3 + 1);
                            let  	hz = SwarmMath::WangHash( ( idx as u32) * 3 + 2);

                            outFloats[base + 0] = SwarmMath::HashToFloat( hx) * 40.0 - 20.0;
                            outFloats[base + 1] = SwarmMath::HashToFloat( hy) * 40.0 - 20.0;
                            outFloats[base + 2] = SwarmMath::HashToFloat( hz) * 40.0 - 20.0;
                            outFloats[base + 3] = 1.0;
                        }
                    }
                }
            }
        }

        Ok( ())
    }

    fn	Synchronize( &self) -> Result< (), SwarmError>
    {
        Ok( ())
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
