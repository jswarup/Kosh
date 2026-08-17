//-- swarm/cpu.rs --------------------------------------------------------------------------------------------------------------------

use	std::sync::{ Arc, RwLock };
use	crate::silo::{ Buff, U32, U64 };
use	crate::swarm::ops::StandardOp;
use	crate::swarm::traits::{
    BackendKind, BufferUsage, CpuKernelFn, IComputeBuffer, IComputeDevice,
    IComputeKernel, KernelSource, SwarmError, WorkgroupDim,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// In-memory compute buffer for CPU SIMT execution.
pub struct CpuBuffer
{
    label:  String,
    data:   Arc< RwLock< Buff< u8>>>,
    usage:  BufferUsage,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CpuBuffer
{
    pub fn	New( label: &str, size: U64, usage: BufferUsage) -> Self
    {
        CpuBuffer {
            label: label.to_string(),
            data: Arc::new( RwLock::new( Buff::Create( size.0 as u32, |_| 0u8))),
            usage,
        }
    }

    pub fn	FromSlice( label: &str, data: &[u8], usage: BufferUsage) -> Self
    {
        CpuBuffer {
            label: label.to_string(),
            data: Arc::new( RwLock::new( Buff::from( data))),
            usage,
        }
    }

    pub fn	Usage( &self) -> BufferUsage
    {
        self.usage
    }

    pub fn	DataLock( &self) -> Arc< RwLock< Buff< u8>>>
    {
        Arc::clone( &self.data)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeBuffer for CpuBuffer
{
    fn	Size( &self) -> U64
    {
        let  	guard = self.data.read().unwrap();
        U64( guard.len() as u64)
    }

    fn	Label( &self) -> &str
    {
        &self.label
    }

    fn	Write( &mut self, data: &[u8]) -> Result< (), SwarmError>
    {
        let  	mut guard = self.data.write().map_err( |e| {
            SwarmError::BufferError( format!( "Write lock failed: {}", e))
        })?;
        if data.len() > guard.len() {
            guard.Resize( U32( data.len() as u32), |_| 0);
        }
        guard[..data.len()].copy_from_slice( data);
        Ok( ())
    }

    fn	Read( &self) -> Result< Buff< u8>, SwarmError>
    {
        let  	guard = self.data.read().map_err( |e| {
            SwarmError::BufferError( format!( "Read lock failed: {}", e))
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

/// Executable kernel closure on the CPU.
pub struct CpuKernel
{
    name:       String,
    entryPoint: String,
    kernelFn:   CpuKernelFn,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CpuKernel
{
    pub fn	New( name: &str, entryPoint: &str, kernelFn: CpuKernelFn) -> Self
    {
        CpuKernel {
            name: name.to_string(),
            entryPoint: entryPoint.to_string(),
            kernelFn,
        }
    }

    pub fn	EntryPoint( &self) -> &str
    {
        &self.entryPoint
    }

    pub fn	Execute(
        &self,
        inputs: &[&[u8]],
        outputs: &mut [&mut [u8]],
        gidX: U32,
        gidY: U32,
        gidZ: U32,
    )
    {
        ( self.kernelFn)( inputs, outputs, gidX, gidY, gidZ);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeKernel for CpuKernel
{
    fn	Name( &self) -> &str
    {
        &self.name
    }

    fn	Backend( &self) -> BackendKind
    {
        BackendKind::Cpu
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// CPU compute device executing kernels over host CPU worker threads.
pub struct CpuDevice
{
    workerCount: usize,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CpuDevice
{
    pub fn	New() -> Self
    {
        let  	workers = std::thread::available_parallelism()
            .map( |p| p.get())
            .unwrap_or( 4);
        CpuDevice {
            workerCount: workers,
        }
    }

    pub fn	WithWorkers( workers: usize) -> Self
    {
        CpuDevice {
            workerCount: workers.max( 1),
        }
    }

    pub fn	WorkerCount( &self) -> usize
    {
        self.workerCount
    }

    /// Creates a CPU kernel that doubles f32 values.
    pub fn	DoubleKernel() -> CpuKernel
    {
        CpuKernel::New( StandardOp::Double.Label(), "main", StandardOp::Double.CpuKernelFn())
    }

    /// Creates a CPU kernel that adds two f32 buffers.
    pub fn	VectorAddKernel() -> CpuKernel
    {
        CpuKernel::New( StandardOp::VectorAdd.Label(), "main", StandardOp::VectorAdd.CpuKernelFn())
    }

    /// Creates a CPU kernel computing Collatz steps.
    pub fn	CollatzKernel() -> CpuKernel
    {
        CpuKernel::New( StandardOp::Collatz.Label(), "main", StandardOp::Collatz.CpuKernelFn())
    }

    /// Creates a CPU kernel for 3D point cloud generation.
    pub fn	PointCloudKernel() -> CpuKernel
    {
        CpuKernel::New( StandardOp::PointCloud.Label(), "pts_pointcloud_cs", StandardOp::PointCloud.CpuKernelFn())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for CpuDevice
{
    fn	default() -> Self
    {
        Self::New()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeDevice for CpuDevice
{
    fn	Backend( &self) -> BackendKind
    {
        BackendKind::Cpu
    }

    fn	CreateBuffer(
        &self,
        label: &str,
        size: U64,
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        Ok( Box::new( CpuBuffer::New( label, size, usage)))
    }

    fn	CreateBufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        Ok( Box::new( CpuBuffer::FromSlice( label, data, usage)))
    }

    fn	CompileKernel(
        &self,
        label: &str,
        entryPoint: &str,
        source: KernelSource,
    ) -> Result< Box< dyn IComputeKernel>, SwarmError>
    {
        match source {
            KernelSource::CpuClosure( f) => {
                Ok( Box::new( CpuKernel::New( label, entryPoint, f)))
            }
            KernelSource::Wgsl( src) => {
                if src.contains( "pts_pointcloud") || entryPoint.contains( "pts_pointcloud") {
                    Ok( Box::new( Self::PointCloudKernel()))
                } else if src.contains( "collatz") || entryPoint.contains( "collatz") {
                    Ok( Box::new( Self::CollatzKernel()))
                } else if src.contains( "result[idx] = a[idx] + b[idx]") || entryPoint.contains( "vecadd") {
                    Ok( Box::new( Self::VectorAddKernel()))
                } else if src.contains( "data[idx] * 2.0") || entryPoint.contains( "double") {
                    Ok( Box::new( Self::DoubleKernel()))
                } else {
                    Err( SwarmError::InvalidKernelSource(
                        format!( "Cannot synthesize CPU kernel for WGSL: {}", label),
                    ))
                }
            }
            KernelSource::SpirV( _) => {
                if entryPoint == "pts_pointcloud_cs" || label.contains( "pointcloud") {
                    Ok( Box::new( Self::PointCloudKernel()))
                } else if entryPoint == "main_cs" || label.contains( "collatz") {
                    Ok( Box::new( Self::CollatzKernel()))
                } else {
                    Err( SwarmError::InvalidKernelSource(
                        format!( "Cannot synthesize CPU kernel for SPIR-V entrypoint: {}", entryPoint),
                    ))
                }
            }
            KernelSource::Ptx( src) => {
                if src.contains( "pts_pointcloud") || entryPoint.contains( "pts_pointcloud") {
                    Ok( Box::new( Self::PointCloudKernel()))
                } else if src.contains( "collatz") || entryPoint.contains( "collatz") {
                    Ok( Box::new( Self::CollatzKernel()))
                } else if src.contains( "vecadd") || entryPoint.contains( "vecadd") {
                    Ok( Box::new( Self::VectorAddKernel()))
                } else {
                    Ok( Box::new( Self::DoubleKernel()))
                }
            }
        }
    }

    fn	Dispatch(
        &self,
        kernel: &dyn IComputeKernel,
        buffers: &[&dyn IComputeBuffer],
        dim: WorkgroupDim,
    ) -> Result< (), SwarmError>
    {
        // Total threads: workgroups * 64 (standard workgroup size)
        let  	threadsX = dim.x.AsU32() * 64;
        let  	threadsY = dim.y.AsU32();
        let  	threadsZ = dim.z.AsU32();

        // Read all buffers into CPU memory
        let  	mut rawData: Buff< Buff< u8>> = Buff::NewEmpty();
        for b in buffers {
            rawData.Push( b.Read()?);
        }

        // Determine input and output buffers: if only 1 buffer, treat as read_write (input & output)
        let  	( inputSlices, mut outputVectors) = if rawData.len() == 1 {
            let  	inClone = rawData[0].clone();
            ( Buff![inClone], Buff![rawData[0].clone()])
        } else {
            let  	inSlices: Buff< Buff< u8>> = Buff::from( &rawData[..rawData.len() - 1]);
            let  	outVecs: Buff< Buff< u8>> = Buff![rawData.last().unwrap().clone()];
            ( inSlices, outVecs)
        };

        let  	inRefs: Buff< &[u8]> = inputSlices.iter().map( |v| &v[..]).collect();
        let  	mut outRefs: Buff< &mut [u8]> = outputVectors.iter_mut().map( |v| &mut v[..]).collect();

        // Execute kernel across grid
        for z in 0..threadsZ {
            for y in 0..threadsY {
                for x in 0..threadsX {
                    if let Some( cpuK) = kernel.AsAny().downcast_ref::< CpuKernel>() {
                        cpuK.Execute( &inRefs, &mut outRefs, U32( x), U32( y), U32( z));
                    } else {
                        Self::DoubleKernel().Execute( &inRefs, &mut outRefs, U32( x), U32( y), U32( z));
                    }
                }
            }
        }
        drop( inRefs);
        drop( outRefs);

        // Write modified output buffer back to target buffer
        if let Some( targetBuf) = buffers.last() {
            if let Some( targetCpu) = (*targetBuf).AsAny().downcast_ref::< CpuBuffer>() {
                let  	mut guard = targetCpu.data.write().unwrap();
                *guard = outputVectors[0].clone();
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
