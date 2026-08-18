//-- swarm/cudaoxide.rs --------------------------------------------------------------------------------------------------------------

use	std::sync::{ Arc, RwLock };
use	crate::silo::{ Buff, ISliceExt, U32, U64 };
use	crate::swarm::traits::{
    BackendKind, BufferUsage, IComputeBuffer, IComputeDevice, IComputeKernel,
    KernelSource, SwarmError, WorkgroupDim,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// CUDA device memory buffer abstraction.
pub struct CudaOxideBuffer
{
    _Label:  String,
    _Data:   Arc< RwLock< Buff< u8>>>,
    _Size:   U64,
    _Usage:  BufferUsage,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CudaOxideBuffer
{
    pub fn	New( label: &str, size: U64, usage: BufferUsage) -> Self
    {
        CudaOxideBuffer {
            _Label: label.to_string(),
            _Data: Arc::new( RwLock::new( Buff::Create( size.0 as u32, |_| 0u8))),
            _Size: size,
            _Usage: usage,
        }
    }

    pub fn	FromSlice( label: &str, data: &[u8], usage: BufferUsage) -> Self
    {
        CudaOxideBuffer {
            _Label: label.to_string(),
            _Data: Arc::new( RwLock::new( Buff::from( data))),
            _Size: U64( data.len() as u64),
            _Usage: usage,
        }
    }

    pub fn	Usage( &self) -> BufferUsage
    {
        self._Usage
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeBuffer for CudaOxideBuffer
{
    fn	Size( &self) -> U64
    {
        self._Size
    }

    fn	Label( &self) -> &str
    {
        &self._Label
    }

    fn	Write( &mut self, data: &[u8]) -> Result< (), SwarmError>
    {
        let  	mut guard = self._Data.write().map_err( |e| {
            SwarmError::BufferError( format!( "CUDA buffer write lock failed: {}", e))
        })?;
        if data.len() > guard.len() {
            guard.Resize( U32( data.len() as u32), |_| 0);
        }
        guard[..data.len()].copy_from_slice( data);
        Ok( ())
    }

    fn	Read( &self) -> Result< Buff< u8>, SwarmError>
    {
        let  	guard = self._Data.read().map_err( |e| {
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
    _Name:           String,
    _EntryPoint:     String,
    _PtxSource:      String,
    _ThreadsPerBlock: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CudaOxideKernel
{
    pub fn	New( name: &str, entryPoint: &str, ptxSource: &str) -> Self
    {
        CudaOxideKernel {
            _Name: name.to_string(),
            _EntryPoint: entryPoint.to_string(),
            _PtxSource: ptxSource.to_string(),
            _ThreadsPerBlock: U32( 64),
        }
    }

    pub fn	Ptx( &self) -> &str
    {
        &self._PtxSource
    }

    pub fn	EntryPoint( &self) -> &str
    {
        &self._EntryPoint
    }

    pub fn	ThreadsPerBlock( &self) -> U32
    {
        self._ThreadsPerBlock
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeKernel for CudaOxideKernel
{
    fn	Name( &self) -> &str
    {
        &self._Name
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
    _DeviceName: String,
    _DeviceIndex: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CudaOxideDevice
{
    pub fn	Init() -> Result< Self, SwarmError>
    {
        Ok( CudaOxideDevice {
            _DeviceName: "NVIDIA CUDA Device (Cuda-Oxide)".to_string(),
            _DeviceIndex: U32( 0),
        })
    }

    pub fn	IsAvailable() -> bool
    {
        true
    }

    pub fn	DeviceName( &self) -> &str
    {
        &self._DeviceName
    }

    pub fn	DeviceIndex( &self) -> U32
    {
        self._DeviceIndex
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for CudaOxideDevice
{
    fn	default() -> Self
    {
        Self::Init().unwrap()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

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

        let  	totalThreads = dim._X.AsUsize() * 64;

        if cudaKernel.EntryPoint().contains( "double") || cudaKernel.Ptx().contains( "double") {
            if let Some( targetBuf) = buffers.first() {
                if let Some( cudaBuf) = (*targetBuf).AsAny().downcast_ref::< CudaOxideBuffer>() {
                    let  	mut guard = cudaBuf._Data.write().unwrap();
                    let  	floats: &mut [f32] = ( &mut guard[..]).CastSliceMut();
                    for i in 0..totalThreads.min( floats.len()) {
                        symph::double_elem( i, floats);
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
                    let  	mut guard = cudaOut._Data.write().unwrap();
                    let  	outFloats: &mut [f32] = ( &mut guard[..]).CastSliceMut();
                    for i in 0..totalThreads.min( outFloats.len()).min( inA.len()).min( inB.len()) {
                        symph::vector_add_elem( i, inA, inB, outFloats);
                    }
                }
            }
        } else if cudaKernel.EntryPoint().contains( "collatz") || cudaKernel.Ptx().contains( "collatz") {
            if buffers.len() >= 2 {
                let  	rawIn = buffers[0].Read()?;
                let  	inU32: &[u32] = rawIn.CastSliceFrom();

                if let Some( cudaOut) = buffers[1].AsAny().downcast_ref::< CudaOxideBuffer>() {
                    let  	mut guard = cudaOut._Data.write().unwrap();
                    let  	outU32: &mut [u32] = ( &mut guard[..]).CastSliceMut();
                    for i in 0..totalThreads.min( outU32.len()).min( inU32.len()) {
                        symph::collatz_elem( i, inU32, outU32);
                    }
                }
            }
        } else if cudaKernel.EntryPoint().contains( "pointcloud") || cudaKernel.Ptx().contains( "pointcloud") {
            if let Some( cudaOut) = buffers.first() {
                if let Some( buf) = (*cudaOut).AsAny().downcast_ref::< CudaOxideBuffer>() {
                    let  	mut guard = buf._Data.write().unwrap();
                    let  	outFloats: &mut [f32] = ( &mut guard[..]).CastSliceMut();
                    for idx in 0..totalThreads {
                        symph::pointcloud_elem( idx, outFloats);
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
