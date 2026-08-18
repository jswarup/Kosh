//-- swarm/engine.rs ----------------------------------------------------------------------------------------------------------------

use	crate::silo::{ Buff, ISliceExt, U32, U64 };
use	crate::swarm::cpu::{ CpuBuffer, CpuDevice, CpuKernel };
use	crate::swarm::cudaoxide::{ CudaOxideBuffer, CudaOxideDevice, CudaOxideKernel };
use	crate::swarm::ops::StandardOp;
use	crate::swarm::rustgpu::{ RustGpuBuffer, RustGpuDevice, RustGpuKernel };
use	crate::swarm::traits::{
    BackendKind, BufferUsage, IComputeBuffer, IComputeDevice, IComputeKernel,
    KernelSource, SwarmError, WorkgroupDim,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Unified enum-dispatched compute buffer eliminating dynamic virtual dispatch on hot paths.
pub enum SwarmBuffer
{
    Cpu( CpuBuffer),
    RustGpu( RustGpuBuffer),
    CudaOxide( CudaOxideBuffer),
}

impl IComputeBuffer for SwarmBuffer
{
    fn	Size( &self) -> U64
    {
        match self {
            SwarmBuffer::Cpu( b) => b.Size(),
            SwarmBuffer::RustGpu( b) => b.Size(),
            SwarmBuffer::CudaOxide( b) => b.Size(),
        }
    }

    fn	Label( &self) -> &str
    {
        match self {
            SwarmBuffer::Cpu( b) => b.Label(),
            SwarmBuffer::RustGpu( b) => b.Label(),
            SwarmBuffer::CudaOxide( b) => b.Label(),
        }
    }

    fn	Write( &mut self, data: &[u8]) -> Result< (), SwarmError>
    {
        match self {
            SwarmBuffer::Cpu( b) => b.Write( data),
            SwarmBuffer::RustGpu( b) => b.Write( data),
            SwarmBuffer::CudaOxide( b) => b.Write( data),
        }
    }

    fn	Read( &self) -> Result< Buff< u8>, SwarmError>
    {
        match self {
            SwarmBuffer::Cpu( b) => b.Read(),
            SwarmBuffer::RustGpu( b) => b.Read(),
            SwarmBuffer::CudaOxide( b) => b.Read(),
        }
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

/// Unified enum-dispatched compute kernel representation.
pub enum SwarmKernel
{
    Cpu( CpuKernel),
    RustGpu( RustGpuKernel),
    CudaOxide( CudaOxideKernel),
}

impl IComputeKernel for SwarmKernel
{
    fn	Name( &self) -> &str
    {
        match self {
            SwarmKernel::Cpu( k) => k.Name(),
            SwarmKernel::RustGpu( k) => k.Name(),
            SwarmKernel::CudaOxide( k) => k.Name(),
        }
    }

    fn	Backend( &self) -> BackendKind
    {
        match self {
            SwarmKernel::Cpu( k) => k.Backend(),
            SwarmKernel::RustGpu( k) => k.Backend(),
            SwarmKernel::CudaOxide( k) => k.Backend(),
        }
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Unified enum-dispatched compute device interface.
pub enum SwarmDevice
{
    Cpu( CpuDevice),
    RustGpu( RustGpuDevice),
    CudaOxide( CudaOxideDevice),
}

impl IComputeDevice for SwarmDevice
{
    fn	Backend( &self) -> BackendKind
    {
        match self {
            SwarmDevice::Cpu( d) => d.Backend(),
            SwarmDevice::RustGpu( d) => d.Backend(),
            SwarmDevice::CudaOxide( d) => d.Backend(),
        }
    }

    fn	CreateBuffer(
        &self,
        label: &str,
        size: U64,
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        match self {
            SwarmDevice::Cpu( d) => d.CreateBuffer( label, size, usage),
            SwarmDevice::RustGpu( d) => d.CreateBuffer( label, size, usage),
            SwarmDevice::CudaOxide( d) => d.CreateBuffer( label, size, usage),
        }
    }

    fn	CreateBufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        match self {
            SwarmDevice::Cpu( d) => d.CreateBufferInit( label, data, usage),
            SwarmDevice::RustGpu( d) => d.CreateBufferInit( label, data, usage),
            SwarmDevice::CudaOxide( d) => d.CreateBufferInit( label, data, usage),
        }
    }

    fn	CompileKernel(
        &self,
        label: &str,
        entryPoint: &str,
        source: KernelSource,
    ) -> Result< Box< dyn IComputeKernel>, SwarmError>
    {
        match self {
            SwarmDevice::Cpu( d) => d.CompileKernel( label, entryPoint, source),
            SwarmDevice::RustGpu( d) => d.CompileKernel( label, entryPoint, source),
            SwarmDevice::CudaOxide( d) => d.CompileKernel( label, entryPoint, source),
        }
    }

    fn	Dispatch(
        &self,
        kernel: &dyn IComputeKernel,
        buffers: &[&dyn IComputeBuffer],
        dim: WorkgroupDim,
    ) -> Result< (), SwarmError>
    {
        match self {
            SwarmDevice::Cpu( d) => d.Dispatch( kernel, buffers, dim),
            SwarmDevice::RustGpu( d) => d.Dispatch( kernel, buffers, dim),
            SwarmDevice::CudaOxide( d) => d.Dispatch( kernel, buffers, dim),
        }
    }

    fn	Synchronize( &self) -> Result< (), SwarmError>
    {
        match self {
            SwarmDevice::Cpu( d) => d.Synchronize(),
            SwarmDevice::RustGpu( d) => d.Synchronize(),
            SwarmDevice::CudaOxide( d) => d.Synchronize(),
        }
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Unified high-level compute engine providing hardware discovery, backend selection, and execution.
pub struct SwarmEngine
{
    _Device: SwarmDevice,
}

impl SwarmEngine
{
    /// Creates a compute engine bound to a specific backend.
    pub fn	New( backend: BackendKind) -> Result< Self, SwarmError>
    {
        let  	device = match backend {
            BackendKind::Cpu => SwarmDevice::Cpu( CpuDevice::New()),
            BackendKind::RustGpu => SwarmDevice::RustGpu( RustGpuDevice::Init()?),
            BackendKind::CudaOxide => SwarmDevice::CudaOxide( CudaOxideDevice::Init()?),
        };

        Ok( SwarmEngine { _Device: device })
    }

    /// Automatically discovers and selects the best available compute hardware.
    /// Priority order: CudaOxide -> RustGpu -> Cpu fallback.
    pub fn	Auto() -> Self
    {
        if let Ok( dev) = RustGpuDevice::Init() {
            return SwarmEngine {
                _Device: SwarmDevice::RustGpu( dev),
            };
        }

        if let Ok( dev) = CudaOxideDevice::Init() {
            return SwarmEngine {
                _Device: SwarmDevice::CudaOxide( dev),
            };
        }

        SwarmEngine {
            _Device: SwarmDevice::Cpu( CpuDevice::New()),
        }
    }

    pub fn	Backend( &self) -> BackendKind
    {
        self._Device.Backend()
    }

    pub fn	Device( &self) -> &SwarmDevice
    {
        &self._Device
    }

    /// Compiles a standard built-in compute operation for the active backend.
    pub fn	CompileOp( &self, op: StandardOp) -> Result< Box< dyn IComputeKernel>, SwarmError>
    {
        let  	backend = self._Device.Backend();
        let  	source = op.KernelSource( backend);
        self._Device.CompileKernel( op.Label(), op.EntryPoint( backend), source)
    }

    pub fn	CreateBufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        self._Device.CreateBufferInit( label, data, usage)
    }

    pub fn	CompileKernel(
        &self,
        label: &str,
        entryPoint: &str,
        source: KernelSource,
    ) -> Result< Box< dyn IComputeKernel>, SwarmError>
    {
        self._Device.CompileKernel( label, entryPoint, source)
    }

    /// Unified Vector Double runner across all backends.
    pub fn	RunDouble( &self, data: &[f32]) -> Result< Buff< f32>, SwarmError>
    {
        let  	kernel = self.CompileOp( StandardOp::Double)?;
        let  	buf = self._Device.CreateBufferInit( "double_data", data.CastSlice(), BufferUsage::STORAGE)?;
        let  	workgroups = ( ( data.len() as u32) + 63) / 64;
        self._Device.Dispatch( kernel.as_ref(), &[buf.as_ref()], WorkgroupDim::Linear( U32( workgroups)))?;
        let  	raw = buf.Read()?;
        let  	res: &[f32] = raw.CastSliceFrom();
        Ok( Buff::from( res))
    }

    /// Unified Vector Addition runner across all backends.
    pub fn	RunVectorAdd( &self, a: &[f32], b: &[f32]) -> Result< Buff< f32>, SwarmError>
    {
        if a.len() != b.len() {
            return Err( SwarmError::ExecutionError( "Input buffer lengths do not match".to_string()));
        }
        let  	sz = a.len();
        let  	zeroBuff = Buff::Create( sz, |_| 0.0f32);
        let  	kernel = self.CompileOp( StandardOp::VectorAdd)?;
        let  	bufA = self._Device.CreateBufferInit( "vecadd_a", a.CastSlice(), BufferUsage::STORAGE)?;
        let  	bufB = self._Device.CreateBufferInit( "vecadd_b", b.CastSlice(), BufferUsage::STORAGE)?;
        let  	bufOut = self._Device.CreateBufferInit( "vecadd_out", zeroBuff.CastSlice(), BufferUsage::STORAGE)?;
        let  	workgroups = ( ( sz as u32) + 63) / 64;
        self._Device.Dispatch( kernel.as_ref(), &[bufA.as_ref(), bufB.as_ref(), bufOut.as_ref()], WorkgroupDim::Linear( U32( workgroups)))?;
        let  	raw = bufOut.Read()?;
        let  	res: &[f32] = raw.CastSliceFrom();
        Ok( Buff::from( res))
    }

    /// Unified Collatz sequence runner across all backends.
    pub fn	RunCollatz( &self, input: &[u32]) -> Result< Buff< u32>, SwarmError>
    {
        let  	sz = input.len();
        let  	zeroBuff = Buff::Create( sz, |_| 0u32);
        let  	kernel = self.CompileOp( StandardOp::Collatz)?;
        let  	bufIn = self._Device.CreateBufferInit( "collatz_in", input.CastSlice(), BufferUsage::STORAGE)?;
        let  	bufOut = self._Device.CreateBufferInit( "collatz_out", zeroBuff.CastSlice(), BufferUsage::STORAGE)?;
        let  	workgroups = ( ( sz as u32) + 63) / 64;
        self._Device.Dispatch( kernel.as_ref(), &[bufIn.as_ref(), bufOut.as_ref()], WorkgroupDim::Linear( U32( workgroups)))?;
        let  	raw = bufOut.Read()?;
        let  	res: &[u32] = raw.CastSliceFrom();
        Ok( Buff::from( res))
    }

    /// Unified Point Cloud generator across all backends.
    pub fn	RunPointCloud(
        &self,
        numPoints: U32,
        spirvBytes: Option< &[u8]>,
    ) -> Result< Buff< [f32; 3]>, SwarmError>
    {
        let  	count = numPoints.AsUsize();
        let  	zeroFloats = Buff::Create( ( count * 4) as u32, |_| 0.0f32);
        let  	workgroups = ( numPoints.AsU32() + 63) / 64;

        let  	kernel = match ( self._Device.Backend(), spirvBytes) {
            ( BackendKind::RustGpu, Some( spv)) => {
                self._Device.CompileKernel( "pts_pointcloud_shader", "pts_pointcloud_cs", KernelSource::SpirV( spv))?
            }
            _ => self.CompileOp( StandardOp::PointCloud)?,
        };

        let  	bufOut = self._Device.CreateBufferInit( "pointcloud_out", zeroFloats.CastSlice(), BufferUsage::STORAGE)?;
        self._Device.Dispatch( kernel.as_ref(), &[bufOut.as_ref()], WorkgroupDim::Linear( U32( workgroups)))?;
        let  	raw = bufOut.Read()?;
        let  	rawFloats: &[f32] = raw.CastSliceFrom();

        let  	points = Buff::Create( numPoints, |i| {
            let  	base = i.AsUsize() * 4;
            if base + 2 < rawFloats.len() {
                [rawFloats[base], rawFloats[base + 1], rawFloats[base + 2]]
            } else {
                [0.0f32, 0.0, 0.0]
            }
        });

        Ok( points)
    }

    /// Unified Camera Projection and Transformation runner across all compute backends.
    /// Takes 3D points [x, y, z] and 13 camera uniform parameters, returns projected [f32; 6] for each point:
    /// [screen_x, screen_y, radius, core_radius, alpha, depth_factor].
    pub fn	RunCameraTransform(
        &self,
        points: &[[f32; 3]],
        camParams: &[f32; 13],
        spirvBytes: Option< &[u8]>,
    ) -> Result< Buff< [f32; 6]>, SwarmError>
    {
        let  	numPoints = points.len();
        if numPoints == 0 {
            return Ok( Buff::New());
        }

        let  	mut flatPoints = Buff::New();
        for pt in points {
            flatPoints.Push( pt[0]);
            flatPoints.Push( pt[1]);
            flatPoints.Push( pt[2]);
        }

        let  	zeroOutput = Buff::Create( ( numPoints * 6) as u32, |_| 0.0f32);
        let  	workgroups = ( ( numPoints as u32) + 63) / 64;

        let  	kernel = match ( self._Device.Backend(), spirvBytes) {
            ( BackendKind::RustGpu, Some( spv)) => {
                self._Device.CompileKernel( "camera_transform_shader", "camera_transform_cs", KernelSource::SpirV( spv))
                    .or_else( |_| self.CompileOp( StandardOp::CameraTransform))?
            }
            _ => self.CompileOp( StandardOp::CameraTransform)?,
        };

        let  	bufInPoints = self._Device.CreateBufferInit( "in_points", flatPoints.CastSlice(), BufferUsage::STORAGE)?;
        let  	bufCamParams = self._Device.CreateBufferInit( "cam_params", camParams.CastSlice(), BufferUsage::STORAGE)?;
        let  	bufOut = self._Device.CreateBufferInit( "out_projected", zeroOutput.CastSlice(), BufferUsage::STORAGE)?;

        self._Device.Dispatch(
            kernel.as_ref(),
            &[bufInPoints.as_ref(), bufCamParams.as_ref(), bufOut.as_ref()],
            WorkgroupDim::Linear( U32( workgroups)),
        )?;

        let  	raw = bufOut.Read()?;
        let  	rawFloats: &[f32] = raw.CastSliceFrom();

        let  	projected = Buff::Create( U32( numPoints as u32), |i| {
            let  	base = i.AsUsize() * 6;
            if base + 5 < rawFloats.len() {
                [
                    rawFloats[base + 0],
                    rawFloats[base + 1],
                    rawFloats[base + 2],
                    rawFloats[base + 3],
                    rawFloats[base + 4],
                    rawFloats[base + 5],
                ]
            } else {
                [0.0f32, 0.0, 0.0, 0.0, 0.0, 0.0]
            }
        });

        Ok( projected)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Multi-GPU heterogeneous compute cluster orchestrating sharded parallel dispatches across all hardware adapters.
pub struct SwarmCluster
{
    pub _Primary:    SwarmEngine,
    pub _Auxiliary:  Buff< SwarmEngine>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl SwarmCluster
{
    /// Automatically discovers all hardware compute adapters and initializes the cluster.
    pub fn	Auto() -> Self
    {
        let  	devices = RustGpuDevice::EnumerateDevices();
        if !devices.is_empty() {
            let  	primary = SwarmEngine {
                _Device: SwarmDevice::RustGpu( devices[0].clone()),
            };
            let  	mut aux = Buff::New();
            for i in 1..devices.len() {
                aux.Push( SwarmEngine {
                    _Device: SwarmDevice::RustGpu( devices[i].clone()),
                });
            }
            return SwarmCluster {
                _Primary:    primary,
                _Auxiliary:  aux,
            };
        }

        let  	fallback = SwarmEngine::Auto();
        SwarmCluster {
            _Primary:    fallback,
            _Auxiliary:  Buff::New(),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Total number of compute engines available in the cluster.
    pub fn	DeviceCount( &self) -> usize
    {
        1 + self._Auxiliary.len()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Returns the primary display/viewport compute engine.
    pub fn	Primary( &self) -> &SwarmEngine
    {
        &self._Primary
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Sharded parallel camera transformation across all GPU devices in the cluster.
    /// For massive point clouds, splits data into K contiguous chunks, dispatches to each GPU
    /// in parallel, and concatenates the resulting projected points.
    pub fn	RunCameraTransformSharded(
        &self,
        points: &[[f32; 3]],
        camParams: &[f32; 13],
        spirvBytes: Option< &[u8]>,
    ) -> Result< Buff< [f32; 6]>, SwarmError>
    {
        let  	totalPoints = points.len();
        if totalPoints == 0 {
            return Ok( Buff::New());
        }

        let  	numDevices = self.DeviceCount();
        if numDevices <= 1 || totalPoints < 4096 {
            return self._Primary.RunCameraTransform( points, camParams, spirvBytes);
        }

        let  	chunkSize = ( totalPoints + numDevices - 1) / numDevices;
        let  	mut chunks = Buff::New();
        for i in 0..numDevices {
            let  	start = i * chunkSize;
            if start < totalPoints {
                let  	end = ( start + chunkSize).min( totalPoints);
                chunks.Push( &points[start..end]);
            }
        }

        let  	mut results = Buff::New();
        for ( i, chunk) in chunks.iter().enumerate() {
            let  	engine = if i == 0 { &self._Primary } else { &self._Auxiliary[i - 1] };
            let  	proj = engine.RunCameraTransform( chunk, camParams, spirvBytes)?;
            results.Push( proj);
        }

        let  	mut totalProjected = Buff::New();
        for r in &results {
            for pt in r {
                totalProjected.Push( *pt);
            }
        }

        Ok( totalProjected)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

