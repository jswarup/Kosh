//-- swarm/traits.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	std::sync::Arc;
use	crate::silo::{ Buff, U32, U64 };

//---------------------------------------------------------------------------------------------------------------------------------

/// Target hardware / runtime backend kind.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind
{
    Cpu,
    RustGpu,
    CudaOxide,
}

impl fmt::Display for BackendKind
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            BackendKind::Cpu => write!( f, "CPU"),
            BackendKind::RustGpu => write!( f, "Rust-GPU (WebGPU/SPIR-V)"),
            BackendKind::CudaOxide => write!( f, "Cuda-Oxide (CUDA/PTX)"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Buffer usage flags for host and device memory management.
#[derive( Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BufferUsage
{
    pub bits: u32,
}

impl BufferUsage
{
    pub const STORAGE:      BufferUsage = BufferUsage { bits: 1 << 0 };
    pub const UNIFORM:      BufferUsage = BufferUsage { bits: 1 << 1 };
    pub const READ_ONLY:    BufferUsage = BufferUsage { bits: 1 << 2 };
    pub const READ_WRITE:   BufferUsage = BufferUsage { bits: 1 << 3 };
    pub const COPY_SRC:     BufferUsage = BufferUsage { bits: 1 << 4 };
    pub const COPY_DST:     BufferUsage = BufferUsage { bits: 1 << 5 };

    pub fn	Contains( &self, other: BufferUsage) -> bool
    {
        ( self.bits & other.bits) == other.bits
    }

    pub fn	Or( self, other: BufferUsage) -> BufferUsage
    {
        BufferUsage { bits: self.bits | other.bits }
    }
}

impl std::ops::BitOr for BufferUsage
{
    type Output = BufferUsage;

    fn	bitor( self, rhs: Self) -> Self::Output
    {
        self.Or( rhs)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 3D workgroup / threadblock dispatch dimensions.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkgroupDim
{
    pub _X: U32,
    pub _Y: U32,
    pub _Z: U32,
}

impl WorkgroupDim
{
    pub fn	New< X: Into< U32>, Y: Into< U32>, Z: Into< U32>>( x: X, y: Y, z: Z) -> Self
    {
        WorkgroupDim { _X: x.into(), _Y: y.into(), _Z: z.into() }
    }

    pub fn	Linear< X: Into< U32>>( x: X) -> Self
    {
        WorkgroupDim { _X: x.into(), _Y: U32( 1), _Z: U32( 1) }
    }

    pub fn	Total( &self) -> U64
    {
        U64( ( self._X.AsUsize() * self._Y.AsUsize() * self._Z.AsUsize()) as u64)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Function signature for CPU SIMT kernel closures.
pub type CpuKernelFn = Arc< dyn Fn( &[&[u8]], &mut [&mut [u8]], U32, U32, U32) + Send + Sync>;

//---------------------------------------------------------------------------------------------------------------------------------

/// Unified compute kernel source representation.
#[derive( Clone)]
pub enum KernelSource< 'a>
{
    Wgsl( &'a str),
    SpirV( &'a [u8]),
    Ptx( &'a str),
    CpuClosure( CpuKernelFn),
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Error types occurring during compute operations.
#[derive( Clone, Debug, PartialEq, Eq)]
pub enum SwarmError
{
    DeviceUnavailable( String),
    CompilationError( String),
    BufferError( String),
    ExecutionError( String),
    UnsupportedBackend( BackendKind),
    InvalidKernelSource( String),
}

impl fmt::Display for SwarmError
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self {
            SwarmError::DeviceUnavailable( msg) => write!( f, "Device unavailable: {}", msg),
            SwarmError::CompilationError( msg) => write!( f, "Compilation error: {}", msg),
            SwarmError::BufferError( msg) => write!( f, "Buffer error: {}", msg),
            SwarmError::ExecutionError( msg) => write!( f, "Execution error: {}", msg),
            SwarmError::UnsupportedBackend( b) => write!( f, "Unsupported backend: {}", b),
            SwarmError::InvalidKernelSource( msg) => write!( f, "Invalid kernel source: {}", msg),
        }
    }
}

impl std::error::Error for SwarmError {}

//---------------------------------------------------------------------------------------------------------------------------------

/// Common interface for host/device compute buffers.
pub trait IComputeBuffer: Send + Sync + 'static
{
    fn	Size( &self) -> U64;

    fn	Label( &self) -> &str;

    fn	Write( &mut self, data: &[u8]) -> Result< (), SwarmError>;

    fn	Read( &self) -> Result< Buff< u8>, SwarmError>;

    fn	AsAny( &self) -> &dyn std::any::Any;

    fn	AsAnyMut( &mut self) -> &mut dyn std::any::Any;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Common interface for compiled compute kernels.
pub trait IComputeKernel: Send + Sync + 'static
{
    fn	Name( &self) -> &str;

    fn	Backend( &self) -> BackendKind;

    fn	AsAny( &self) -> &dyn std::any::Any;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Common interface for compute devices (CPU, Rust-GPU, Cuda-Oxide).
pub trait IComputeDevice: Send + Sync + 'static
{
    fn	Backend( &self) -> BackendKind;

    fn	CreateBuffer(
        &self,
        label: &str,
        size: U64,
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>;

    fn	CreateBufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>;

    fn	CompileKernel(
        &self,
        label: &str,
        entryPoint: &str,
        source: KernelSource,
    ) -> Result< Box< dyn IComputeKernel>, SwarmError>;

    fn	Dispatch(
        &self,
        kernel: &dyn IComputeKernel,
        buffers: &[&dyn IComputeBuffer],
        dim: WorkgroupDim,
    ) -> Result< (), SwarmError>;

    fn	Synchronize( &self) -> Result< (), SwarmError>;

    fn	AsAny( &self) -> &dyn std::any::Any;
}

//---------------------------------------------------------------------------------------------------------------------------------
